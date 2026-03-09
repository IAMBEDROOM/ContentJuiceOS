use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

use log::warn;
use tokio::sync::Mutex;

use super::types::QueueStats;
use crate::platform::error::PlatformError;
use crate::types::Platform;

/// Type alias for a boxed async operation that can be deferred and retried.
pub type DeferredOperation =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<(), PlatformError>> + Send>> + Send>;

/// A deferred action that can be retried when a platform comes back online.
pub struct QueuedAction {
    pub id: String,
    pub description: String,
    pub created_at: Instant,
    pub operation: DeferredOperation,
}

/// In-memory queue of deferred actions per platform.
///
/// When a platform is Down, fire-and-forget operations are enqueued here
/// instead of failing immediately. They are drained when the platform
/// recovers (health probe succeeds).
pub struct ActionQueue {
    queues: Mutex<HashMap<Platform, VecDeque<QueuedAction>>>,
}

impl ActionQueue {
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        for &platform in Platform::all() {
            queues.insert(platform, VecDeque::new());
        }
        Self {
            queues: Mutex::new(queues),
        }
    }

    /// Enqueue a deferred action for later execution.
    pub async fn enqueue(&self, platform: Platform, action: QueuedAction) {
        let mut queues = self.queues.lock().await;
        let queue = queues.entry(platform).or_default();
        log::info!(
            "Queued action for {}: {} (id: {})",
            platform,
            action.description,
            action.id
        );
        queue.push_back(action);
    }

    /// Drain non-stale actions from the queue, discarding stale ones.
    /// Returns the actions that are still valid for execution.
    pub async fn drain(&self, platform: Platform, stale_threshold_secs: u64) -> Vec<QueuedAction> {
        let mut queues = self.queues.lock().await;
        let queue = match queues.get_mut(&platform) {
            Some(q) => q,
            None => return Vec::new(),
        };

        let mut valid = Vec::new();
        while let Some(action) = queue.pop_front() {
            if action.created_at.elapsed().as_secs() > stale_threshold_secs {
                warn!(
                    "Discarding stale queued action for {}: {} (age: {}s, threshold: {}s)",
                    platform,
                    action.description,
                    action.created_at.elapsed().as_secs(),
                    stale_threshold_secs,
                );
            } else {
                valid.push(action);
            }
        }
        valid
    }

    /// Get queue statistics for a platform.
    pub async fn stats(&self, platform: Platform) -> QueueStats {
        let queues = self.queues.lock().await;
        let queue = queues.get(&platform);

        match queue {
            Some(q) if !q.is_empty() => {
                let oldest_age = q.front().map(|a| a.created_at.elapsed().as_secs_f64());
                QueueStats {
                    platform: platform.as_str().to_string(),
                    pending_count: q.len(),
                    oldest_age_secs: oldest_age,
                }
            }
            _ => QueueStats {
                platform: platform.as_str().to_string(),
                pending_count: 0,
                oldest_age_secs: None,
            },
        }
    }

    /// Get the count of pending actions for a platform.
    pub async fn count(&self, platform: Platform) -> usize {
        let queues = self.queues.lock().await;
        queues.get(&platform).map(|q| q.len()).unwrap_or(0)
    }

    /// Discard all queued actions for a platform.
    #[allow(dead_code)]
    pub async fn clear(&self, platform: Platform) {
        let mut queues = self.queues.lock().await;
        if let Some(queue) = queues.get_mut(&platform) {
            queue.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(description: &str) -> QueuedAction {
        QueuedAction {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            created_at: Instant::now(),
            operation: Box::new(|| Box::pin(async { Ok(()) })),
        }
    }

    #[tokio::test]
    async fn enqueue_and_stats() {
        let queue = ActionQueue::new();

        let stats = queue.stats(Platform::Twitch).await;
        assert_eq!(stats.pending_count, 0);
        assert!(stats.oldest_age_secs.is_none());

        queue
            .enqueue(Platform::Twitch, make_action("test action"))
            .await;

        let stats = queue.stats(Platform::Twitch).await;
        assert_eq!(stats.pending_count, 1);
        assert!(stats.oldest_age_secs.is_some());
    }

    #[tokio::test]
    async fn drain_returns_valid_actions() {
        let queue = ActionQueue::new();

        queue
            .enqueue(Platform::Twitch, make_action("action 1"))
            .await;
        queue
            .enqueue(Platform::Twitch, make_action("action 2"))
            .await;

        let actions = queue.drain(Platform::Twitch, 300).await;
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].description, "action 1");
        assert_eq!(actions[1].description, "action 2");

        // Queue should be empty after drain
        let stats = queue.stats(Platform::Twitch).await;
        assert_eq!(stats.pending_count, 0);
    }

    #[tokio::test]
    async fn drain_discards_stale_actions() {
        let queue = ActionQueue::new();

        // Create an action with an old timestamp
        let stale_action = QueuedAction {
            id: uuid::Uuid::new_v4().to_string(),
            description: "stale action".to_string(),
            created_at: Instant::now() - std::time::Duration::from_secs(600),
            operation: Box::new(|| Box::pin(async { Ok(()) })),
        };

        queue.enqueue(Platform::Twitch, stale_action).await;
        queue
            .enqueue(Platform::Twitch, make_action("fresh action"))
            .await;

        let actions = queue.drain(Platform::Twitch, 300).await;
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].description, "fresh action");
    }

    #[tokio::test]
    async fn count_tracks_queue_size() {
        let queue = ActionQueue::new();
        assert_eq!(queue.count(Platform::Kick).await, 0);

        queue.enqueue(Platform::Kick, make_action("action")).await;
        assert_eq!(queue.count(Platform::Kick).await, 1);
    }

    #[tokio::test]
    async fn clear_removes_all() {
        let queue = ActionQueue::new();
        queue.enqueue(Platform::YouTube, make_action("a")).await;
        queue.enqueue(Platform::YouTube, make_action("b")).await;

        queue.clear(Platform::YouTube).await;
        assert_eq!(queue.count(Platform::YouTube).await, 0);
    }

    #[tokio::test]
    async fn platforms_are_independent() {
        let queue = ActionQueue::new();

        queue
            .enqueue(Platform::Twitch, make_action("twitch action"))
            .await;
        queue
            .enqueue(Platform::YouTube, make_action("youtube action"))
            .await;

        assert_eq!(queue.count(Platform::Twitch).await, 1);
        assert_eq!(queue.count(Platform::YouTube).await, 1);
        assert_eq!(queue.count(Platform::Kick).await, 0);
    }
}
