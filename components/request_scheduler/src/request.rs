//! Request tracking types for pending and active requests

use crate::RequestId;
use network_types::NetworkRequest;

/// A pending request waiting to be scheduled
#[derive(Debug)]
pub struct PendingRequest {
    /// Unique request ID
    pub id: RequestId,
    /// The actual network request
    pub request: NetworkRequest,
}

/// An active request currently being processed
#[derive(Debug)]
pub struct ActiveRequest {
    /// Unique request ID
    pub id: RequestId,
    /// The actual network request
    pub request: NetworkRequest,
}

impl PendingRequest {
    /// Create a new pending request
    pub fn new(id: RequestId, request: NetworkRequest) -> Self {
        Self { id, request }
    }

    /// Convert to active request
    pub fn into_active(self) -> ActiveRequest {
        ActiveRequest {
            id: self.id,
            request: self.request,
        }
    }
}

impl ActiveRequest {
    /// Create a new active request
    pub fn new(id: RequestId, request: NetworkRequest) -> Self {
        Self { id, request }
    }
}
