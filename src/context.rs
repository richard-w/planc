use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

pub struct ServiceContextConfig {
    pub max_sessions: usize,
    pub max_users: usize,
}

pub struct ServiceContext {
    config: ServiceContextConfig,
    sessions: Mutex<HashMap<String, Weak<Session>>>,
}

impl ServiceContext {
    pub fn new(config: ServiceContextConfig) -> Self {
        Self {
            config,
            sessions: Mutex::default(),
        }
    }

    /// Get a pointer to a session.
    ///
    /// If the session does not exist it will be created.
    pub fn get_session(self: &Arc<Self>, session_id: &str) -> Result<Arc<Session>> {
        let mut sessions = self.sessions.lock().unwrap();

        // Get session that already exists.
        if let Some(weak_session) = sessions.get(session_id) {
            if let Some(session) = weak_session.upgrade() {
                return Ok(session);
            } else {
                sessions.remove(session_id);
            }
        }

        // Check if maximum sessions would be exceeded.
        if sessions.len() >= self.config.max_sessions {
            return Err(PlancError::MaxSessionsExceeded.into());
        }

        // Create new session.
        let session = Arc::new(Session::new(
            self.clone(),
            session_id,
            self.config.max_users,
        ));
        sessions.insert(session_id.to_string(), Arc::downgrade(&session));
        Ok(session)
    }

    /// Cleanup weak references to a dropped session.
    pub fn cleanup_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap();

        // It is possible that a session with the same id was already recreated at this point so we
        // need to check that the reference in the session table is actually invalid.
        if let Some(pointer) = sessions.get(session_id) {
            if pointer.upgrade().is_none() {
                sessions.remove(session_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctx_refcounting_test() {
        let ctx = Arc::new(ServiceContext::new(ServiceContextConfig {
            max_sessions: 16,
            max_users: 8,
        }));
        assert_eq!(ctx.sessions.lock().unwrap().len(), 0);

        let session = ctx.get_session("abcd");
        assert_eq!(ctx.sessions.lock().unwrap().len(), 1);
        assert!(ctx.sessions.lock().unwrap().get("abcd").is_some());

        std::mem::drop(session);
        assert_eq!(ctx.sessions.lock().unwrap().len(), 0);
    }
}
