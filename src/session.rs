use super::*;
use std::sync::atomic::AtomicI64;
use std::time::Duration;
use tokio::sync::watch;
use tokio::sync::Mutex;
use tracing::Instrument;

pub struct Session {
    ctx: Arc<ServiceContext>,
    session_id: String,
    max_users: usize,
    session_state_tx: Mutex<watch::Sender<SessionState>>,
    session_state_rx: watch::Receiver<SessionState>,
    next_user_id: AtomicI64,
}

impl Session {
    #[::tracing::instrument(skip(ctx), name = "Session::new")]
    pub fn new(ctx: Arc<ServiceContext>, session_id: &str, max_users: usize) -> Self {
        tracing::info!(session_id, "new_session");
        let session_id = session_id.to_string();
        let (session_state_tx, session_state_rx) = watch::channel(SessionState::default());
        let session_state_tx = Mutex::new(session_state_tx);
        let next_user_id = AtomicI64::new(1);
        Self {
            ctx,
            session_id,
            max_users,
            session_state_tx,
            session_state_rx,
            next_user_id,
        }
    }

    #[::tracing::instrument(skip(self, conn), name = "Session::user", fields(session_id = self.session_id, user_id))]
    pub async fn join(&self, mut conn: Connection) -> Result<()> {
        // Get a unique user id for this session.
        let user_id = self
            .next_user_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();
        ::tracing::Span::current().record("user_id", &user_id);

        // Add user to session state.
        let add_user_result = self
            .update_state(|mut state| {
                if state.users.len() >= self.max_users {
                    Err(PlancError::MaxUsersExceeded.into())
                } else {
                    state.users.insert(user_id.clone(), UserState::default());
                    Result::Ok(state)
                }
            })
            .await;
        if let Err(err) = add_user_result {
            ::tracing::warn!("join_denied");
            conn.send(&ServerMessage::Error(format!(
                "Error joining session: {}",
                err
            )))
            .await?;
            return Ok(());
        }
        ::tracing::info!("joined");

        // Subscribe client to state updates.
        let mut sender = conn.sender();
        let mut session_state_rx = self.session_state_rx.clone();
        let cloned_user_id = user_id.clone();
        tokio::spawn(
            async move {
                let user_id = cloned_user_id;
                while session_state_rx.changed().await.is_ok() {
                    // Clone state for some ad-hoc modifications and checks.
                    let mut new_state = session_state_rx.borrow().to_owned();

                    // Get a reference to this user's state or terminate this task.
                    let user = if let Some(user) = new_state.users.get(&user_id) {
                        user
                    } else {
                        break;
                    };

                    // Check if this user was kicked and stop this task if that is the case.
                    if user.kicked {
                        if let Err(err) = sender
                            .send(&ServerMessage::Error(
                                "You have been kicked from the session".to_string(),
                            ))
                            .await
                        {
                            ::tracing::warn!(?err, "send_state_task/send_kick_message");
                        }
                        break;
                    }

                    // Mask kicked users.
                    new_state.users.retain(|_, user| !user.kicked);

                    // Mask points so they are not visible from the console.
                    if new_state
                        .users
                        .values()
                        .any(|user| !user.is_spectator && user.points.is_none())
                    {
                        new_state
                            .users
                            .iter_mut()
                            .filter(|&(item_user_id, _)| *item_user_id != user_id)
                            .for_each(|(_, other_user)| {
                                if other_user.points.is_some() {
                                    other_user.points = Some("-1".to_string());
                                }
                            });
                    }

                    // Send the modified state.
                    if let Err(err) = sender.send(&ServerMessage::State(new_state)).await {
                        ::tracing::warn!(?err, "send_state_task/send_state_message");
                        break;
                    }
                }
            }
            .instrument(::tracing::Span::current()),
        );

        // Send a keep-alive message every 5 seconds. This may be necessary to keep the websocket
        // connection alive when certain reverse proxies are used.
        let mut sender = conn.sender();
        tokio::spawn(
            async move {
                while sender.send(&ServerMessage::KeepAlive).await.is_ok() {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
            .instrument(::tracing::Span::current()),
        );

        // Listen to messages from the connection.
        if let Err(err) = self.handle_connection(conn, &user_id).await {
            ::tracing::warn!(?err, "handle_connection");
        }

        // Remove user from state.
        self.update_state(|mut state| {
            state.users.remove(&user_id);
            if state.admin.as_ref() == Some(&user_id) {
                state.admin = None;
            }
            Result::Ok(state)
        })
        .await?;

        ::tracing::info!("user_leaving");
        Ok(())
    }

    async fn handle_connection(&self, mut conn: Connection, user_id: &str) -> Result<()> {
        while let Some(msg) = conn.recv().await {
            // Terminate the connection for kicked users.
            let user_state = self.user_state(user_id).await?;
            if user_state.kicked {
                return Err(PlancError::UserKicked.into());
            }

            let result = match msg? {
                ClientMessage::NameChange(name) if name.len() <= 32 => {
                    self.update_state(|mut state| {
                        if state
                            .users
                            .values()
                            .all(|user| user.name.as_ref() != Some(&name))
                        {
                            state.users.get_mut(user_id).unwrap().name = Some(name.clone());
                            Ok(state)
                        } else {
                            Err(PlancError::DuplicateName.into())
                        }
                    })
                    .await
                }
                ClientMessage::SetPoints(points) if points.len() <= 8 => {
                    self.update_state(|mut state| {
                        let user_state = state.users.get_mut(user_id).unwrap();
                        if !user_state.is_spectator {
                            user_state.points = Some(points.clone());
                            Ok(state)
                        } else {
                            Err(PlancError::InvalidMessage.into())
                        }
                    })
                    .await
                }
                ClientMessage::ResetPoints => {
                    self.update_state(|mut state| {
                        if state.admin.as_deref() == Some(user_id) {
                            for user in state.users.values_mut() {
                                user.points = None;
                            }
                            Ok(state)
                        } else {
                            Err(PlancError::InsufficientPermissions.into())
                        }
                    })
                    .await
                }
                ClientMessage::Whoami => {
                    conn.send(&ServerMessage::Whoami(user_id.to_string())).await
                }
                ClientMessage::ClaimSession => {
                    self.update_state(|mut state| {
                        if state.admin.is_none() {
                            state.admin = Some(user_id.to_string());
                            ::tracing::info!("claiming_session");
                            Ok(state)
                        } else {
                            Err(PlancError::InsufficientPermissions.into())
                        }
                    })
                    .await
                }
                ClientMessage::KickUser(kickee_id) => {
                    self.update_state(|mut state| {
                        if state.admin.as_deref() == Some(user_id) {
                            if let Some(kickee) = state.users.get_mut(&kickee_id) {
                                kickee.kicked = true;
                                ::tracing::info!(kickee_id, "kicking_user");
                                Ok(state)
                            } else {
                                Err(PlancError::UnknownUserId.into())
                            }
                        } else {
                            Err(PlancError::InsufficientPermissions.into())
                        }
                    })
                    .await
                }
                ClientMessage::SetSpectator(is_spectator) => {
                    self.update_state(|mut state| {
                        let user_state = state.users.get_mut(user_id).unwrap();
                        user_state.is_spectator = is_spectator;
                        user_state.points = None;
                        Ok(state)
                    })
                    .await
                }
                _ => Err(PlancError::InvalidMessage.into()),
            };
            if let Err(err) = result {
                conn.send(&ServerMessage::Error(err.to_string())).await?;
                return Err(err);
            }
        }
        Ok(())
    }

    async fn update_state<F>(&self, mut func: F) -> Result<()>
    where
        F: FnMut(SessionState) -> Result<SessionState>,
    {
        let session_state_tx = self.session_state_tx.lock().await;
        let current_state = session_state_tx.borrow().clone();
        let new_state = func(current_state)?;
        session_state_tx.send(new_state).unwrap();
        Ok(())
    }

    async fn user_state(&self, user_id: &str) -> Result<UserState> {
        let session_state_tx = self.session_state_tx.lock().await;
        let current_state = session_state_tx.borrow();
        if let Some(user_state) = current_state.users.get(user_id) {
            Ok(user_state.clone())
        } else {
            Err(PlancError::UnknownUserId.into())
        }
    }
}

impl Drop for Session {
    #[tracing::instrument(skip(self), name = "Session::drop", fields(session_id = self.session_id))]
    fn drop(&mut self) {
        ::tracing::info!("Session::drop: Removing session \"{}\"", self.session_id);
        self.ctx.cleanup_session(&self.session_id);
    }
}
