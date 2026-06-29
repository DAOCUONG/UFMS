//! [Action] constructors and [ActionParameter] builders.
//!
//! The VDA 5050 spec encodes several rules only as comments:
//! - `instantActions` always use [`BlockingType::BlockingNone`].
//! - The `retriable` flag only applies to `order` and `zone` actions.
//! - `action_id` should be unique per topic.
//!
//! [`Action::for_instant`] forces `BlockingNone` (and refuses to set
//! `retriable`); [`Action::for_order`] / [`Action::for_zone`] default
//! `retriable` to `false`.

use prost_types::value::Kind;
use prost_types::{ListValue, Struct, Value};

use crate::error::{ActionError, ValidationError};
use crate::vda5050::v3::{Action, ActionParameter, BlockingType};

// ---------------------------------------------------------------------------
// Action constructors
// ---------------------------------------------------------------------------

impl Action {
    /// Build an `Action` for the `instantActions` topic. Per spec, the
    /// `blocking_type` is forced to [`BlockingType::BlockingNone`] and
    /// `retriable` is not allowed (it would carry no meaning on instant
    /// actions).
    pub fn for_instant(action_type: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            action_type: action_type.into(),
            action_id: action_id.into(),
            action_descriptor: String::new(),
            blocking_type: BlockingType::BlockingNone as i32,
            action_parameters: Vec::new(),
            retriable: None,
        }
    }

    /// Build an `Action` for an `order` (node or edge). `retriable` defaults
    /// to `Some(false)` per spec.
    pub fn for_order(action_type: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            action_type: action_type.into(),
            action_id: action_id.into(),
            action_descriptor: String::new(),
            blocking_type: BlockingType::BlockingSoft as i32,
            action_parameters: Vec::new(),
            retriable: Some(false),
        }
    }

    /// Build an `Action` for a `zoneSet.zones[].*_actions[]` list. `retriable`
    /// defaults to `Some(false)`.
    pub fn for_zone(action_type: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            action_type: action_type.into(),
            action_id: action_id.into(),
            action_descriptor: String::new(),
            blocking_type: BlockingType::BlockingSoft as i32,
            action_parameters: Vec::new(),
            retriable: Some(false),
        }
    }

    /// Chainable setter for `action_descriptor`.
    pub fn with_descriptor(mut self, descriptor: impl Into<String>) -> Self {
        self.action_descriptor = descriptor.into();
        self
    }

    /// Chainable setter for `blocking_type`.
    pub fn with_blocking_type(mut self, bt: BlockingType) -> Self {
        self.blocking_type = bt as i32;
        self
    }

    /// Chainable setter adding one parameter.
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.action_parameters.push(ActionParameter {
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Chainable setter replacing all parameters at once.
    pub fn with_parameters(mut self, params: Vec<ActionParameter>) -> Self {
        self.action_parameters = params;
        self
    }

    /// Chainable setter for `retriable`.
    pub fn with_retriable(mut self, retriable: bool) -> Self {
        self.retriable = Some(retriable);
        self
    }

    /// Validate the `Action`: `action_type` and `action_id` are non-empty.
    pub fn validate(&self) -> Result<(), ActionError> {
        if self.action_type.is_empty() {
            return Err(ActionError::MissingActionType);
        }
        if self.action_id.is_empty() {
            return Err(ActionError::MissingActionId);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ActionParameter typed builders
// ---------------------------------------------------------------------------

/// Typed builders for `ActionParameter.value`, which is a
/// `prost_types::Value` (a oneof-wrapper around `Kind`).
pub trait ActionParameterExt: Sized {
    /// String-typed parameter.
    fn string(key: impl Into<String>, value: impl Into<String>) -> ActionParameter;
    /// Boolean-typed parameter.
    fn bool(key: impl Into<String>, value: bool) -> ActionParameter;
    /// Floating-point parameter.
    fn number(key: impl Into<String>, value: f64) -> ActionParameter;
    /// Integer parameter (encoded as a `Kind::NumberValue` for maximum
    /// portability).
    fn integer(key: impl Into<String>, value: i64) -> ActionParameter;
    /// Array-typed parameter.
    fn array(key: impl Into<String>, value: Vec<Value>) -> ActionParameter;
    /// Object-typed parameter.
    fn object(key: impl Into<String>, value: Vec<(String, Value)>) -> ActionParameter;
    /// `null` typed parameter.
    fn null(key: impl Into<String>) -> ActionParameter;
}

fn wrap(kind: Kind) -> Value {
    Value { kind: Some(kind) }
}

impl ActionParameterExt for ActionParameter {
    fn string(key: impl Into<String>, value: impl Into<String>) -> ActionParameter {
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::StringValue(value.into()))),
        }
    }

    fn bool(key: impl Into<String>, value: bool) -> ActionParameter {
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::BoolValue(value))),
        }
    }

    fn number(key: impl Into<String>, value: f64) -> ActionParameter {
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::NumberValue(value))),
        }
    }

    fn integer(key: impl Into<String>, value: i64) -> ActionParameter {
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::NumberValue(value as f64))),
        }
    }

    fn array(key: impl Into<String>, value: Vec<Value>) -> ActionParameter {
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::ListValue(ListValue { values: value }))),
        }
    }

    fn object(key: impl Into<String>, value: Vec<(String, Value)>) -> ActionParameter {
        let mut fields = std::collections::BTreeMap::new();
        for (k, v) in value {
            fields.insert(k, v);
        }
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::StructValue(Struct { fields }))),
        }
    }

    fn null(key: impl Into<String>) -> ActionParameter {
        ActionParameter {
            key: key.into(),
            value: Some(wrap(Kind::NullValue(0))),
        }
    }
}

// ---------------------------------------------------------------------------
// Builder for the verbose `Order::Node::actions[i]` / `Order::Edge::actions[i]`
// case where you want chainable configuration.
// ---------------------------------------------------------------------------

/// Fluent builder for [`Action`]. Use [`Action::for_instant`], [`Action::for_order`],
/// or [`Action::for_zone`] to start with the right defaults.
pub struct ActionBuilder {
    inner: Action,
}

impl ActionBuilder {
    /// Borrow the underlying [`Action`].
    pub fn as_action(&self) -> &Action {
        &self.inner
    }

    /// Mutably borrow the underlying [`Action`] for direct field tweaks.
    pub fn as_action_mut(&mut self) -> &mut Action {
        &mut self.inner
    }

    /// Build the underlying [`Action`]. Validate with [`Action::validate`].
    pub fn build(self) -> Action {
        self.inner
    }

    /// Set `action_descriptor`.
    pub fn descriptor(mut self, descriptor: impl Into<String>) -> Self {
        self.inner.action_descriptor = descriptor.into();
        self
    }

    /// Set `blocking_type`.
    pub fn blocking_type(mut self, bt: BlockingType) -> Self {
        self.inner.blocking_type = bt as i32;
        self
    }

    /// Add one parameter.
    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.inner.action_parameters.push(ActionParameter {
            key: key.into(),
            value: Some(value.into()),
        });
        self
    }

    /// Replace all parameters at once.
    pub fn parameters(mut self, params: Vec<ActionParameter>) -> Self {
        self.inner.action_parameters = params;
        self
    }

    /// Set `retriable` flag.
    pub fn retriable(mut self, r: bool) -> Self {
        self.inner.retriable = Some(r);
        self
    }

    /// Convert any leftover error to a [`ValidationError`].
    pub fn try_build(self) -> Result<Action, ValidationError> {
        let a = self.inner;
        a.validate()?;
        Ok(a)
    }
}

impl From<ActionBuilder> for Action {
    fn from(b: ActionBuilder) -> Self {
        b.build()
    }
}
