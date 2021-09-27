use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

#[derive(Default)]
pub struct ServiceContext {
    sessions: Mutex<HashMap<String, Weak<Session>>>,
}

impl ServiceContext {
    pub fn new() -> Self {
        Self::default()
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

        // Create new session.
        let session = Arc::new(Session::new(self.clone(), session_id));
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
        let ctx = Arc::new(ServiceContext::new());
        assert_eq!(ctx.sessions.lock().unwrap().len(), 0);

        let session = ctx.get_session("abcd");
        assert_eq!(ctx.sessions.lock().unwrap().len(), 1);
        assert!(ctx.sessions.lock().unwrap().get("abcd").is_some());

        std::mem::drop(session);
        assert_eq!(ctx.sessions.lock().unwrap().len(), 0);
    }
}
