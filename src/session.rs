use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use tokio::sync::watch;
use tokio::sync::Mutex;

pub struct Session {
    ctx: Arc<ServiceContext>,
    session_id: String,
    session_state_tx: Mutex<watch::Sender<SessionState>>,
    session_state_rx: watch::Receiver<SessionState>,
    next_user_id: AtomicI64,
}

impl Session {
    pub fn new(ctx: Arc<ServiceContext>, session_id: &str) -> Self {
        let session_id = session_id.to_string();
        let (session_state_tx, session_state_rx) = watch::channel(SessionState::default());
        let session_state_tx = Mutex::new(session_state_tx);
        let next_user_id = AtomicI64::new(1);
        Self {
            ctx,
            session_id,
            session_state_tx,
            session_state_rx,
            next_user_id,
        }
    }

    pub async fn join(&self, conn: Connection) -> Result<()> {
        // Get a unique user id for this session.
        let user_id = self
            .next_user_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string();

        // Add user to session state.
        self.update_state(|mut state| {
            state.users.insert(user_id.clone(), UserState::default());
            Some(state)
        })
        .await;

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
                            user.points = Some(-1);
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
            Some(state)
        })
        .await;

        Ok(())
    }

    async fn handle_connection(&self, mut conn: Connection, user_id: &str) -> Result<()> {
        while let Some(msg) = conn.recv().await {
            match msg? {
                ClientMessage::NameChange(name) => {
                    self.update_state(|mut state| {
                        state.users.get_mut(user_id).unwrap().name = Some(name.clone());
                        Some(state)
                    })
                    .await;
                }
                ClientMessage::SetPoints(points) => {
                    self.update_state(|mut state| {
                        state.users.get_mut(user_id).unwrap().points = Some(points);
                        Some(state)
                    })
                    .await;
                }
                ClientMessage::ResetPoints => {
                    self.update_state(|mut state| {
                        if state.admin.as_deref() == Some(user_id) {
                            for user in state.users.values_mut() {
                                user.points = None;
                            }
                            Some(state)
                        } else {
                            None
                        }
                    })
                    .await;
                }
                ClientMessage::Whoami => {
                    conn.send(&ServerMessage::Whoami(user_id.to_string())).await?;
                }
                ClientMessage::ClaimSession => {
                    self.update_state(|mut state| {
                        if state.admin.is_none() {
                            state.admin = Some(user_id.to_string());
                            Some(state)
                        } else {
                            None
                        }
                    })
                    .await;
                }
            }
        }
        Ok(())
    }

    async fn update_state<F>(&self, mut func: F)
    where
        F: FnMut(SessionState) -> Option<SessionState>,
    {
        let session_state_tx = self.session_state_tx.lock().await;
        let current_state = session_state_tx.borrow().clone();
        if let Some(new_state) = func(current_state) {
            session_state_tx.send(new_state).unwrap();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.ctx.cleanup_session(&self.session_id);
    }
}

#[derive(Debug, Clone, Default, Serialize)]
struct SessionState {
    pub users: HashMap<String, UserState>,
    pub admin: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UserState {
    pub name: Option<String>,
    pub points: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum ClientMessage {
    NameChange(String),
    SetPoints(i32),
    ResetPoints,
    Whoami,
    ClaimSession,
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "content")]
enum ServerMessage {
    State(SessionState),
    Whoami(String),
}
