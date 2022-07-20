use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize)]
pub struct SessionState {
    pub users: HashMap<String, UserState>,
    pub admin: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserState {
    pub name: Option<String>,
    pub points: Option<String>,
    pub is_spectator: bool,
    #[serde(skip)]
    pub kicked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tag", content = "content")]
pub enum ClientMessage {
    NameChange(String),
    SetPoints(String),
    ResetPoints,
    Whoami,
    ClaimSession,
    KickUser(String),
    SetSpectator(bool),
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag", content = "content")]
pub enum ServerMessage {
    State(SessionState),
    Whoami(String),
    Error(String),
    KeepAlive,
}
