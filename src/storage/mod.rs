//! Session storage backends
//!
//! This module provides trait-based abstraction for session storage,
//! allowing different backend implementations (in-memory, Redis, PostgreSQL, etc.).

use crate::error::StorageError;
use crate::session::Session;
use crate::types::SessionId;
use async_trait::async_trait;

pub mod memory;

/// Trait for session storage backends
///
/// This trait defines the interface that all storage backends must implement
/// to be compatible with the Talk agent framework.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session in the store
    ///
    /// # Arguments
    ///
    /// * `session` - The session to create
    ///
    /// # Returns
    ///
    /// The session ID on success, or a storage error
    async fn create(&self, session: Session) -> Result<SessionId, StorageError>;

    /// Get a session by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The session ID to retrieve
    ///
    /// # Returns
    ///
    /// The session if found, None if not found, or a storage error
    async fn get(&self, id: &SessionId) -> Result<Option<Session>, StorageError>;

    /// Update an existing session
    ///
    /// # Arguments
    ///
    /// * `id` - The session ID to update
    /// * `session` - The updated session data
    ///
    /// # Returns
    ///
    /// Ok on success, or a storage error
    async fn update(&self, id: &SessionId, session: Session) -> Result<(), StorageError>;

    /// Delete a session by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The session ID to delete
    ///
    /// # Returns
    ///
    /// Ok on success, or a storage error
    async fn delete(&self, id: &SessionId) -> Result<(), StorageError>;

    /// List all session IDs in the store
    ///
    /// # Returns
    ///
    /// A vector of all session IDs, or a storage error
    async fn list(&self) -> Result<Vec<SessionId>, StorageError>;

    /// Check if a session exists
    ///
    /// # Arguments
    ///
    /// * `id` - The session ID to check
    ///
    /// # Returns
    ///
    /// true if the session exists, false otherwise, or a storage error
    async fn exists(&self, id: &SessionId) -> Result<bool, StorageError> {
        Ok(self.get(id).await?.is_some())
    }
}
