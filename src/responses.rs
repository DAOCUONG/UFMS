//! [Response] / [Responses] constructors.
//!
//! The `responses` topic is the fleet-control → mobile-robot direction. Each
//! [`Response`] answers a [`crate::state::ZoneRequest`] or
//! [`crate::state::EdgeRequest`].

use prost_types::Timestamp;

use crate::vda5050::v3::{GrantType, Header, Response, Responses};

/// Ergonomic helpers for [`Responses`].
pub trait ResponsesExt {
    /// Look up the response for a given `request_id`, if present.
    fn lookup(&self, request_id: &str) -> Option<&Response>;
}

impl ResponsesExt for Responses {
    fn lookup(&self, request_id: &str) -> Option<&Response> {
        self.responses.iter().find(|r| r.request_id == request_id)
    }
}

impl Response {
    /// Build a `Response` granting a request, with an optional lease
    /// expiration timestamp.
    pub fn granted(
        request_id: impl Into<String>,
        lease_expiry: Option<Timestamp>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            grant_type: GrantType::GrantGranted as i32,
            lease_expiry,
        }
    }

    /// Build a `Response` queueing a request.
    pub fn queued(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            grant_type: GrantType::GrantQueued as i32,
            lease_expiry: None,
        }
    }

    /// Build a `Response` revoking a previously granted request.
    pub fn revoked(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            grant_type: GrantType::GrantRevoked as i32,
            lease_expiry: None,
        }
    }

    /// Build a `Response` rejecting a request.
    pub fn rejected(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            grant_type: GrantType::GrantRejected as i32,
            lease_expiry: None,
        }
    }

    /// True when a `lease_expiry` is set. Per spec, leases only make sense
    /// for granted requests — the helper exists so callers can avoid
    /// checking grant_type and lease_expiry together.
    pub fn has_lease(&self) -> bool {
        self.lease_expiry.is_some()
    }
}

/// Fluent builder for [`Responses`].
pub struct ResponseBuilder(Responses);

impl ResponseBuilder {
    pub fn new(header: Header) -> Self {
        Self(Responses {
            header: Some(header),
            responses: Vec::new(),
        })
    }

    pub fn response(mut self, r: Response) -> Self {
        self.0.responses.push(r);
        self
    }

    pub fn responses(mut self, responses: Vec<Response>) -> Self {
        self.0.responses = responses;
        self
    }

    pub fn build(self) -> Responses {
        self.0
    }
}
