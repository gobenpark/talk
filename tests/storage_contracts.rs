//! Integration test contracts for SessionStore trait
//!
//! These tests verify that all SessionStore implementations comply with the expected contract.

use talk::storage::SessionStore;
use talk::types::AgentId;
use talk::{InMemorySessionStore, Session};

/// Test the contract for SessionStore::create
///
/// This test verifies that:
/// - A new session can be created and returns the correct session ID
/// - Creating a duplicate session returns an error
#[tokio::test]
async fn test_session_store_create_contract() {
    let store = InMemorySessionStore::new();
    let session = Session::new(AgentId::new());
    let session_id = session.id;

    // Create session should succeed
    let result = store.create(session.clone()).await;
    assert!(
        result.is_ok(),
        "SessionStore::create should succeed for new session"
    );
    assert_eq!(
        result.unwrap(),
        session_id,
        "SessionStore::create should return the session ID"
    );

    // Creating duplicate session should fail
    let duplicate_result = store.create(session).await;
    assert!(
        duplicate_result.is_err(),
        "SessionStore::create should fail for duplicate session ID"
    );
}

/// Test the contract for SessionStore::get
///
/// This test verifies that:
/// - Getting an existing session returns Some(session)
/// - Getting a non-existent session returns None
/// - The returned session matches the original
#[tokio::test]
async fn test_session_store_get_contract() {
    let store = InMemorySessionStore::new();
    let session = Session::new(AgentId::new());
    let session_id = session.id;

    // Get non-existent session should return None
    let result = store.get(&session_id).await;
    assert!(result.is_ok(), "SessionStore::get should not return error");
    assert!(
        result.unwrap().is_none(),
        "SessionStore::get should return None for non-existent session"
    );

    // Create session
    store.create(session.clone()).await.unwrap();

    // Get existing session should return Some(session)
    let result = store.get(&session_id).await;
    assert!(result.is_ok(), "SessionStore::get should not return error");
    let retrieved = result.unwrap();
    assert!(
        retrieved.is_some(),
        "SessionStore::get should return Some for existing session"
    );
    assert_eq!(
        retrieved.unwrap().id,
        session_id,
        "SessionStore::get should return the correct session"
    );
}

/// Test the contract for SessionStore::update
///
/// This test verifies that:
/// - Updating an existing session succeeds
/// - Updating a non-existent session returns an error
/// - The updated session can be retrieved with changes
#[tokio::test]
async fn test_session_store_update_contract() {
    let store = InMemorySessionStore::new();
    let mut session = Session::new(AgentId::new());
    let session_id = session.id;

    // Update non-existent session should fail
    let result = store.update(&session_id, session.clone()).await;
    assert!(
        result.is_err(),
        "SessionStore::update should fail for non-existent session"
    );

    // Create session
    store.create(session.clone()).await.unwrap();

    // Update existing session should succeed
    session.complete();
    let result = store.update(&session_id, session.clone()).await;
    assert!(
        result.is_ok(),
        "SessionStore::update should succeed for existing session"
    );

    // Verify the session was updated
    let retrieved = store.get(&session_id).await.unwrap().unwrap();
    assert_eq!(
        retrieved.status,
        talk::SessionStatus::Completed,
        "SessionStore::update should persist changes"
    );
}

/// Test the contract for SessionStore::delete
///
/// This test verifies that:
/// - Deleting an existing session succeeds
/// - Deleting a non-existent session returns an error
/// - Deleted sessions cannot be retrieved
#[tokio::test]
async fn test_session_store_delete_contract() {
    let store = InMemorySessionStore::new();
    let session = Session::new(AgentId::new());
    let session_id = session.id;

    // Delete non-existent session should fail
    let result = store.delete(&session_id).await;
    assert!(
        result.is_err(),
        "SessionStore::delete should fail for non-existent session"
    );

    // Create session
    store.create(session).await.unwrap();

    // Delete existing session should succeed
    let result = store.delete(&session_id).await;
    assert!(
        result.is_ok(),
        "SessionStore::delete should succeed for existing session"
    );

    // Verify the session was deleted
    let retrieved = store.get(&session_id).await.unwrap();
    assert!(
        retrieved.is_none(),
        "SessionStore::delete should remove the session"
    );
}

/// Test the contract for SessionStore::list
///
/// This test verifies that:
/// - Listing an empty store returns an empty vec
/// - Listing a store with sessions returns all session IDs
/// - The list includes all created sessions
#[tokio::test]
async fn test_session_store_list_contract() {
    let store = InMemorySessionStore::new();

    // List empty store should return empty vec
    let result = store.list().await;
    assert!(result.is_ok(), "SessionStore::list should not return error");
    assert_eq!(
        result.unwrap().len(),
        0,
        "SessionStore::list should return empty vec for empty store"
    );

    // Create multiple sessions
    let session1 = Session::new(AgentId::new());
    let session2 = Session::new(AgentId::new());
    let session3 = Session::new(AgentId::new());
    let id1 = session1.id;
    let id2 = session2.id;
    let id3 = session3.id;

    store.create(session1).await.unwrap();
    store.create(session2).await.unwrap();
    store.create(session3).await.unwrap();

    // List should return all session IDs
    let result = store.list().await;
    assert!(result.is_ok(), "SessionStore::list should not return error");
    let list = result.unwrap();
    assert_eq!(
        list.len(),
        3,
        "SessionStore::list should return all sessions"
    );
    assert!(list.contains(&id1), "SessionStore::list should include id1");
    assert!(list.contains(&id2), "SessionStore::list should include id2");
    assert!(list.contains(&id3), "SessionStore::list should include id3");
}

/// Test the contract for SessionStore::exists
///
/// This test verifies that:
/// - Checking existence of non-existent session returns false
/// - Checking existence of existing session returns true
/// - Default implementation based on get() works correctly
#[tokio::test]
async fn test_session_store_exists_contract() {
    let store = InMemorySessionStore::new();
    let session = Session::new(AgentId::new());
    let session_id = session.id;

    // Exists for non-existent session should return false
    let result = store.exists(&session_id).await;
    assert!(
        result.is_ok(),
        "SessionStore::exists should not return error"
    );
    assert_eq!(
        result.unwrap(),
        false,
        "SessionStore::exists should return false for non-existent session"
    );

    // Create session
    store.create(session).await.unwrap();

    // Exists for existing session should return true
    let result = store.exists(&session_id).await;
    assert!(
        result.is_ok(),
        "SessionStore::exists should not return error"
    );
    assert_eq!(
        result.unwrap(),
        true,
        "SessionStore::exists should return true for existing session"
    );
}

/// Test concurrent operations on SessionStore
///
/// This test verifies that:
/// - Multiple concurrent creates work correctly
/// - Multiple concurrent reads work correctly
/// - The store maintains consistency under concurrent access
#[tokio::test]
async fn test_session_store_concurrent_operations_contract() {
    let store = InMemorySessionStore::new();

    // Create sessions concurrently
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let store_clone = store.clone();
            tokio::spawn(async move {
                let session = Session::new(AgentId::new());
                store_clone.create(session).await
            })
        })
        .collect();

    // Wait for all creates to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(
            result.is_ok(),
            "Concurrent SessionStore::create operations should succeed"
        );
    }

    // Verify all sessions were created
    let list = store.list().await.unwrap();
    assert_eq!(list.len(), 20, "All concurrent creates should succeed");

    // Read sessions concurrently
    let read_handles: Vec<_> = list
        .iter()
        .map(|id| {
            let store_clone = store.clone();
            let id_clone = *id;
            tokio::spawn(async move { store_clone.get(&id_clone).await })
        })
        .collect();

    // Wait for all reads to complete
    for handle in read_handles {
        let result = handle.await.unwrap();
        assert!(
            result.is_ok(),
            "Concurrent SessionStore::get operations should succeed"
        );
        assert!(
            result.unwrap().is_some(),
            "Concurrent reads should retrieve existing sessions"
        );
    }
}

/// Test that SessionStore implementations are thread-safe (Send + Sync)
///
/// This is a compile-time test that verifies the store can be shared across threads.
#[tokio::test]
async fn test_session_store_thread_safety_contract() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<InMemorySessionStore>();
}
