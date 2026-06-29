//! [Connection] lifecycle constructors.
//!
//! The VDA 5050 `connection` topic is the MQTT last-will topic. The spec
//! mandates specific lifecycle transitions (see `connection.proto` header
//! comment). These constructors encode them so callers don't have to set
//! `connection_state` from a raw `i32`.

use crate::vda5050::v3::{Connection, ConnectionState, Header};

/// Extension trait for ergonomic [`Connection`] construction.
pub trait ConnectionExt {
    /// Build a `Connection` declaring the mobile robot is online.
    fn online(header: Header) -> Self;
    /// Build a `Connection` declaring a coordinated shutdown.
    fn offline(header: Header) -> Self;
    /// Build a `Connection` declaring a hibernating state (connected but
    /// not publishing).
    fn hibernating(header: Header) -> Self;
    /// Build a `Connection` marking the (typically MQTT-last-will) broken
    /// state.
    fn broken(header: Header) -> Self;

    /// True when the connection is in any terminal-disconnect state.
    fn is_disconnected(&self) -> bool;
}

impl ConnectionExt for Connection {
    fn online(header: Header) -> Self {
        Self {
            header: Some(header),
            connection_state: ConnectionState::ConnectionOnline as i32,
        }
    }

    fn offline(header: Header) -> Self {
        Self {
            header: Some(header),
            connection_state: ConnectionState::ConnectionOffline as i32,
        }
    }

    fn hibernating(header: Header) -> Self {
        Self {
            header: Some(header),
            connection_state: ConnectionState::ConnectionHibernating as i32,
        }
    }

    fn broken(header: Header) -> Self {
        Self {
            header: Some(header),
            connection_state: ConnectionState::ConnectionBroken as i32,
        }
    }

    fn is_disconnected(&self) -> bool {
        matches!(
            self.connection_state,
            x if x == ConnectionState::ConnectionOffline as i32
                || x == ConnectionState::ConnectionBroken as i32
        )
    }
}
