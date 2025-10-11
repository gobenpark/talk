//! In-memory session storage implementation
//!
//! This module provides a thread-safe, in-memory implementation of the SessionStore trait
//! using a HashMap protected by an async RwLock.

use crate::error::StorageError;
use crate::session::Session;
use crate::storage::SessionStore;
use crate::types::SessionId;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory session storage implementation
///
/// This implementation stores sessions in a HashMap protected by an async RwLock,
/// making it thread-safe for concurrent access. It's suitable for development,
/// testing, and single-instance deployments.
///
/// # Examples
///
/// ```
/// use talk::{InMemorySessionStore, Session};
/// use talk::storage::SessionStore;
/// use talk::types::AgentId;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = InMemorySessionStore::new();
///     let session = Session::new(AgentId::new());
///     let session_id = store.create(session).await?;
///
///     let retrieved = store.get(&session_id).await?;
///     assert!(retrieved.is_some());
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl InMemorySessionStore {
    /// Create a new in-memory session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the number of sessions currently stored
    ///
    /// This is useful for monitoring and testing purposes.
    pub async fn len(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> bool {
        self.sessions.read().await.is_empty()
    }

    /// Clear all sessions from the store
    ///
    /// This is primarily useful for testing purposes.
    pub async fn clear(&self) {
        self.sessions.write().await.clear();
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&self, session: Session) -> Result<SessionId, StorageError> {
        let id = session.id;
        let mut sessions = self.sessions.write().await;

        // Check if session already exists
        if sessions.contains_key(&id) {
            return Err(StorageError::AlreadyExists(format!(
                "Session with ID {} already exists",
                id
            )));
        }

        sessions.insert(id, session);
        Ok(id)
    }

    async fn get(&self, id: &SessionId) -> Result<Option<Session>, StorageError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn update(&self, id: &SessionId, session: Session) -> Result<(), StorageError> {
        let mut sessions = self.sessions.write().await;

        // Check if session exists before updating
        if !sessions.contains_key(id) {
            return Err(StorageError::NotFound(format!(
                "Session with ID {} not found",
                id
            )));
        }

        sessions.insert(*id, session);
        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> Result<(), StorageError> {
        let mut sessions = self.sessions.write().await;

        if sessions.remove(id).is_none() {
            return Err(StorageError::NotFound(format!(
                "Session with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn list(&self) -> Result<Vec<SessionId>, StorageError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.keys().copied().collect())
    }

    async fn exists(&self, id: &SessionId) -> Result<bool, StorageError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.contains_key(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AgentId;

    #[tokio::test]
    async fn test_create_session() {
        let store = InMemorySessionStore::new();
        let session = Session::new(AgentId::new());
        let session_id = session.id;

        let result = store.create(session).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), session_id);
    }

    #[tokio::test]
    async fn test_create_duplicate_session() {
        let store = InMemorySessionStore::new();
        let session = Session::new(AgentId::new());
        let session_clone = session.clone();

        store.create(session).await.unwrap();
        let result = store.create(session_clone).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageError::AlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_get_session() {
        let store = InMemorySessionStore::new();
        let session = Session::new(AgentId::new());
        let session_id = session.id;

        store.create(session.clone()).await.unwrap();

        let retrieved = store.get(&session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session_id);
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let store = InMemorySessionStore::new();
        let session_id = SessionId::new();

        let retrieved = store.get(&session_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_update_session() {
        let store = InMemorySessionStore::new();
        let mut session = Session::new(AgentId::new());
        let session_id = session.id;

        store.create(session.clone()).await.unwrap();

        session.complete();
        store.update(&session_id, session.clone()).await.unwrap();

        let retrieved = store.get(&session_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, crate::session::SessionStatus::Completed);
    }

    #[tokio::test]
    async fn test_update_nonexistent_session() {
        let store = InMemorySessionStore::new();
        let session = Session::new(AgentId::new());
        let session_id = session.id;

        let result = store.update(&session_id, session).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_session() {
        let store = InMemorySessionStore::new();
        let session = Session::new(AgentId::new());
        let session_id = session.id;

        store.create(session).await.unwrap();
        store.delete(&session_id).await.unwrap();

        let retrieved = store.get(&session_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_session() {
        let store = InMemorySessionStore::new();
        let session_id = SessionId::new();

        let result = store.delete(&session_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let store = InMemorySessionStore::new();
        let session1 = Session::new(AgentId::new());
        let session2 = Session::new(AgentId::new());
        let session3 = Session::new(AgentId::new());

        store.create(session1.clone()).await.unwrap();
        store.create(session2.clone()).await.unwrap();
        store.create(session3.clone()).await.unwrap();

        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&session1.id));
        assert!(list.contains(&session2.id));
        assert!(list.contains(&session3.id));
    }

    #[tokio::test]
    async fn test_list_empty_store() {
        let store = InMemorySessionStore::new();
        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 0);
    }

    #[tokio::test]
    async fn test_exists_session() {
        let store = InMemorySessionStore::new();
        let session = Session::new(AgentId::new());
        let session_id = session.id;

        store.create(session).await.unwrap();

        let exists = store.exists(&session_id).await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_exists_nonexistent_session() {
        let store = InMemorySessionStore::new();
        let session_id = SessionId::new();

        let exists = store.exists(&session_id).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_len_and_is_empty() {
        let store = InMemorySessionStore::new();
        assert!(store.is_empty().await);
        assert_eq!(store.len().await, 0);

        store.create(Session::new(AgentId::new())).await.unwrap();
        assert!(!store.is_empty().await);
        assert_eq!(store.len().await, 1);

        store.create(Session::new(AgentId::new())).await.unwrap();
        assert_eq!(store.len().await, 2);
    }

    #[tokio::test]
    async fn test_clear() {
        let store = InMemorySessionStore::new();
        store.create(Session::new(AgentId::new())).await.unwrap();
        store.create(Session::new(AgentId::new())).await.unwrap();

        assert_eq!(store.len().await, 2);

        store.clear().await;
        assert!(store.is_empty().await);
        assert_eq!(store.len().await, 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let store = InMemorySessionStore::new();
        let store_clone1 = store.clone();
        let store_clone2 = store.clone();

        // Create sessions concurrently
        let handle1 = tokio::spawn(async move {
            for _ in 0..10 {
                let session = Session::new(AgentId::new());
                store_clone1.create(session).await.unwrap();
            }
        });

        let handle2 = tokio::spawn(async move {
            for _ in 0..10 {
                let session = Session::new(AgentId::new());
                store_clone2.create(session).await.unwrap();
            }
        });

        handle1.await.unwrap();
        handle2.await.unwrap();

        assert_eq!(store.len().await, 20);
    }
}
