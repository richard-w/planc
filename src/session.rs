use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use tokio::sync::watch;
use tokio::sync::Mutex;

pub struct Session {
    ctx: Arc<ServiceContext>,
    session_id: String,
    max_users: usize,
    session_state_tx: Mutex<watch::Sender<SessionState>>,
    session_state_rx: watch::Receiver<SessionState>,
    next_user_id: AtomicI64,
}

impl Session {
    pub fn new(ctx: Arc<ServiceContext>, session_id: &str, max_users: usize) -> Self {
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

    pub async fn join(&self, mut conn: Connection) -> Result<()> {
        // Get a unique user id for this session.
        let user_id = self
            .next_user_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();

        // Add user to session state.
        let add_user_result = self.update_state(|mut state| {
            if state.users.len() >= self.max_users {
                Err(Error::MaxUsersExceeded.into())
            } else {
                state.users.insert(user_id.clone(), UserState::default());
                Result::Ok(state)
            }
        })
        .await;
        if let Err(err) = add_user_result {
            conn.send(&ServerMessage::Error(format!("Error joining session: {}", err))).await?;
            return Ok(());
        }

        // Subscribe client to state updates.
        let mut sender = conn.sender();
        let mut session_state_rx = self.session_state_rx.clone();
        tokio::spawn(async move {
            while session_state_rx.changed().await.is_ok() {
                let mut new_state = session_state_rx.borrow().to_owned();
                if new_state.users.values().any(|user| user.points.is_none()) {
                    // Mask points so they are not visible from the console
                    new_state.users.values_mut().for_each(|user| {
                        if user.points.is_some() {
                            user.points = Some("-1".to_string());
                        }
                    });
                }
                if let Err(err) = sender.send(&ServerMessage::State(new_state)).await {
                    log::error!("Sending session state failed: {:?}", err);
                    break;
                }
            }
        });

        // Listen to messages from the connection.
        if let Err(err) = self.handle_connection(conn, &user_id).await {
            log::error!("Error handling connection: {:?}", err);
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

        Ok(())
    }

    async fn handle_connection(&self, mut conn: Connection, user_id: &str) -> Result<()> {
        while let Some(msg) = conn.recv().await {
            let result = match msg? {
                ClientMessage::NameChange(name) if name.len() <= 32 => {
                    self.update_state(|mut state| {
                        if state.users.values().all(|user| user.name.as_ref() != Some(&name)) {
                            state.users.get_mut(user_id).unwrap().name = Some(name.clone());
                            Ok(state)
                        } else {
                            Err(Error::DuplicateName.into())
                        }
                    })
                    .await
                }
                ClientMessage::SetPoints(points) if points.len() <= 8 => {
                    self.update_state(|mut state| {
                        state.users.get_mut(user_id).unwrap().points = Some(points.clone());
                        Result::Ok(state)
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
                            Err(Error::InsufficientPermissions.into())
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
                            Ok(state)
                        } else {
                            Err(Error::InsufficientPermissions.into())
                        }
                    })
                    .await
                }
                _ => Err(Error::InvalidMessage.into()),
            };
            if let Err(err) = result {
                conn.send(&ServerMessage::Error(err.to_string())).await?;
                return Err(err);
            }
        }
        Ok(())
    }

    async fn update_state<F, E>(&self, mut func: F) -> std::result::Result<(), E>
    where
        F: FnMut(SessionState) -> std::result::Result<SessionState, E>,
    {
        let session_state_tx = self.session_state_tx.lock().await;
        let current_state = session_state_tx.borrow().clone();
        let new_state = func(current_state)?;
        session_state_tx.send(new_state).unwrap();
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.ctx.cleanup_session(&self.session_id);
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SessionState {
    pub users: HashMap<String, UserState>,
    pub admin: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UserState {
    pub name: Option<String>,
    pub points: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum ClientMessage {
    NameChange(String),
    SetPoints(String),
    ResetPoints,
    Whoami,
    ClaimSession,
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "content")]
pub enum ServerMessage {
    State(SessionState),
    Whoami(String),
    Error(String),
}
