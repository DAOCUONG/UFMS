//! [InstantActions] builder with spec-validation.
//!
//! Per `instant_actions.proto` (file-level comment): every action in an
//! `InstantActions` message **must** have
//! `blocking_type = BlockingType::None`. [`InstantActionsBuilder::validate`]
//! rejects any deviation.

use crate::error::ValidationError;
use crate::vda5050::v3::{Action, BlockingType, Header, InstantActions};

/// Fluent builder for [`InstantActions`].
pub struct InstantActionsBuilder {
    inner: InstantActions,
}

impl InstantActionsBuilder {
    /// Start a new builder with the given `header`.
    pub fn new(header: Header) -> Self {
        Self {
            inner: InstantActions {
                header: Some(header),
                actions: Vec::new(),
            },
        }
    }

    /// Add one [`Action`] (typically constructed with [`Action::for_instant`]
    /// or via [`ActionBuilder`]).
    pub fn add_action(mut self, action: Action) -> Self {
        self.inner.actions.push(action);
        self
    }

    /// Add many actions at once.
    pub fn add_actions(mut self, actions: impl IntoIterator<Item = Action>) -> Self {
        self.inner.actions.extend(actions);
        self
    }

    /// Replace the action list.
    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.inner.actions = actions;
        self
    }

    /// Borrow the in-progress message (useful for `.try_build()`-free
    /// inspection).
    pub fn as_message(&self) -> &InstantActions {
        &self.inner
    }

    /// Consume the builder and return the [`InstantActions`] message.
    /// Returns [`ValidationError::InstantActionBlocking`] if any action has
    /// a non-`BlockingNone` `blocking_type`.
    pub fn try_build(self) -> Result<InstantActions, ValidationError> {
        self.validate()?;
        Ok(self.inner)
    }

    /// Consume the builder and return the [`InstantActions`] message without
    /// validation (use only if you have already validated externally).
    pub fn build(self) -> InstantActions {
        self.inner
    }

    /// Validate without consuming. Every action must have
    /// `blocking_type == BlockingNone`.
    pub fn validate(&self) -> Result<(), ValidationError> {
        for a in &self.inner.actions {
            if a.blocking_type != BlockingType::BlockingNone as i32 {
                return Err(ValidationError::InstantActionBlocking {
                    action_id: a.action_id.clone(),
                });
            }
        }
        Ok(())
    }
}

impl From<InstantActionsBuilder> for InstantActions {
    fn from(b: InstantActionsBuilder) -> Self {
        b.build()
    }
}
