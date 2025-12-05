//! Output queue with resource enforcement.
//!
//! Provides a bounded queue for output actions that integrates with
//! [`ResourceEnforcer`] queue limits. When the queue is full, the oldest
//! action is dropped and a warning is logged.

use crate::engine::{OutputAction, ResourceEnforcer, ResourceLimitError};
use crate::errors::KeyrxError;
use crate::traits::InputSource;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::warn;

/// Bounded queue for output actions.
#[derive(Debug)]
pub struct OutputQueue {
    actions: VecDeque<OutputAction>,
    enforcer: Arc<ResourceEnforcer>,
}

impl OutputQueue {
    /// Create a new queue with the provided resource enforcer.
    pub fn new(enforcer: Arc<ResourceEnforcer>) -> Self {
        Self {
            actions: VecDeque::new(),
            enforcer,
        }
    }

    /// Replace the enforcer and clear any pending actions, resetting counters.
    pub fn replace_enforcer(&mut self, enforcer: Arc<ResourceEnforcer>) {
        self.reset_counters();
        self.actions.clear();
        self.enforcer = enforcer;
    }

    /// Current number of queued actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Add an action to the queue, dropping the oldest on overflow.
    pub fn enqueue(&mut self, action: OutputAction) {
        if let Err(error) = self.enforcer.increment_queue() {
            self.drop_oldest(error);
            if let Err(error) = self.enforcer.increment_queue() {
                warn!(
                    target: "resource_enforcer",
                    %error,
                    action = ?action,
                    "Output queue saturated; dropping incoming action"
                );
                return;
            }
        }

        self.actions.push_back(action);
    }

    /// Flush queued actions to the input source.
    pub async fn flush<I: InputSource>(&mut self, input: &mut I) -> Result<(), KeyrxError> {
        while let Some(action) = self.actions.pop_front() {
            let result = input.send_output(action).await;
            self.enforcer.decrement_queue();
            result?;
        }
        Ok(())
    }

    fn drop_oldest(&mut self, error: ResourceLimitError) {
        if let Some(dropped) = self.actions.pop_front() {
            self.enforcer.decrement_queue();
            warn!(
                target: "resource_enforcer",
                %error,
                action = ?dropped,
                queue_depth = self.actions.len(),
                queue_limit = self.enforcer.snapshot().queue_limit,
                "Output queue full; dropping oldest action"
            );
        } else {
            warn!(
                target: "resource_enforcer",
                %error,
                "Output queue limit exceeded with no buffered actions; dropping incoming action"
            );
        }
    }

    fn reset_counters(&mut self) {
        if self.actions.is_empty() {
            return;
        }

        for _ in 0..self.actions.len() {
            self.enforcer.decrement_queue();
        }
    }
}

impl Drop for OutputQueue {
    fn drop(&mut self) {
        self.reset_counters();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::KeyCode;
    use crate::engine::ResourceLimits;
    use crate::mocks::MockInput;
    use std::time::Duration;

    #[tokio::test]
    async fn drops_oldest_when_limit_reached() {
        let enforcer = Arc::new(ResourceEnforcer::new(ResourceLimits::new(
            Duration::from_millis(1),
            1024,
            2,
        )));
        let mut queue = OutputQueue::new(enforcer.clone());

        queue.enqueue(OutputAction::PassThrough);
        queue.enqueue(OutputAction::Block);
        queue.enqueue(OutputAction::KeyDown(KeyCode::A));

        assert_eq!(queue.len(), 2);
        assert_eq!(enforcer.snapshot().queue_depth, 2);

        let mut input = MockInput::new();
        queue.flush(&mut input).await.unwrap();

        assert_eq!(
            input.output_log(),
            &[OutputAction::Block, OutputAction::KeyDown(KeyCode::A)]
        );
        assert_eq!(enforcer.snapshot().queue_depth, 0);
    }

    #[tokio::test]
    async fn flushes_and_updates_counters() {
        let enforcer = Arc::new(ResourceEnforcer::new(ResourceLimits::new(
            Duration::from_millis(1),
            1024,
            5,
        )));
        let mut queue = OutputQueue::new(enforcer.clone());

        queue.enqueue(OutputAction::KeyUp(KeyCode::B));
        queue.enqueue(OutputAction::PassThrough);

        assert_eq!(queue.len(), 2);
        assert_eq!(enforcer.snapshot().queue_depth, 2);

        let mut input = MockInput::new();
        queue.flush(&mut input).await.unwrap();

        assert!(queue.is_empty());
        assert_eq!(input.output_log().len(), 2);
        assert_eq!(enforcer.snapshot().queue_depth, 0);
    }
}
