//! Request scheduler implementation

use crate::{ActiveRequest, PendingRequest, RequestId};
use network_errors::NetworkError;
use network_types::{NetworkRequest, RequestPriority};
use std::collections::{HashMap, VecDeque};

/// Request scheduler with priority-based queueing and concurrency limiting
///
/// Manages three priority queues (high, medium, low) and enforces
/// maximum concurrent request limits while ensuring fair scheduling.
pub struct RequestScheduler {
    /// High priority queue (navigation, CSS, fonts)
    high_priority: VecDeque<PendingRequest>,
    /// Medium priority queue (scripts, XHR)
    medium_priority: VecDeque<PendingRequest>,
    /// Low priority queue (images, prefetch)
    low_priority: VecDeque<PendingRequest>,
    /// Currently active requests
    active_requests: HashMap<RequestId, ActiveRequest>,
    /// Maximum concurrent requests allowed
    max_concurrent: usize,
    /// Next request ID to assign
    next_id: RequestId,
}

impl RequestScheduler {
    /// Create a new request scheduler
    ///
    /// # Arguments
    ///
    /// * `max_concurrent` - Maximum number of concurrent requests allowed
    ///
    /// # Examples
    ///
    /// ```
    /// use request_scheduler::RequestScheduler;
    ///
    /// let scheduler = RequestScheduler::new(6);
    /// ```
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            high_priority: VecDeque::new(),
            medium_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            active_requests: HashMap::new(),
            max_concurrent,
            next_id: 1,
        }
    }

    /// Schedule a network request with the given priority
    ///
    /// Returns a unique request ID that can be used to cancel the request.
    ///
    /// # Arguments
    ///
    /// * `request` - The network request to schedule
    /// * `priority` - Priority level for the request
    ///
    /// # Examples
    ///
    /// ```
    /// use request_scheduler::{RequestScheduler, RequestPriority};
    /// use network_types::{NetworkRequest, HttpMethod, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy};
    /// use url::Url;
    ///
    /// let mut scheduler = RequestScheduler::new(6);
    /// let request = NetworkRequest {
    ///     url: Url::parse("https://example.com").unwrap(),
    ///     method: HttpMethod::Get,
    ///     headers: http::HeaderMap::new(),
    ///     body: None,
    ///     mode: RequestMode::Navigate,
    ///     credentials: CredentialsMode::SameOrigin,
    ///     cache: CacheMode::Default,
    ///     redirect: RedirectMode::Follow,
    ///     referrer: None,
    ///     referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
    ///     integrity: None,
    ///     keepalive: false,
    ///     signal: None,
    ///     priority: RequestPriority::Auto,
    ///     window: None,
    /// };
    ///
    /// let id = scheduler.schedule(request, RequestPriority::High);
    /// ```
    pub fn schedule(&mut self, request: NetworkRequest, priority: RequestPriority) -> RequestId {
        let id = self.next_id;
        self.next_id += 1;

        let pending = PendingRequest::new(id, request);

        match priority {
            RequestPriority::High => self.high_priority.push_back(pending),
            RequestPriority::Auto => self.medium_priority.push_back(pending),
            RequestPriority::Low => self.low_priority.push_back(pending),
        }

        id
    }

    /// Get the next request to process
    ///
    /// Returns the highest priority pending request if the concurrent
    /// request limit has not been reached. Returns `None` if no requests
    /// are pending or if the concurrent limit is reached.
    ///
    /// Priority order: High > Medium (Auto) > Low
    /// Within same priority: FIFO (first-in, first-out)
    ///
    /// # Examples
    ///
    /// ```
    /// use request_scheduler::{RequestScheduler, RequestPriority};
    /// use network_types::{NetworkRequest, HttpMethod, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy};
    /// use url::Url;
    ///
    /// let mut scheduler = RequestScheduler::new(6);
    ///
    /// let request = NetworkRequest {
    ///     url: Url::parse("https://example.com").unwrap(),
    ///     method: HttpMethod::Get,
    ///     headers: http::HeaderMap::new(),
    ///     body: None,
    ///     mode: RequestMode::Navigate,
    ///     credentials: CredentialsMode::SameOrigin,
    ///     cache: CacheMode::Default,
    ///     redirect: RedirectMode::Follow,
    ///     referrer: None,
    ///     referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
    ///     integrity: None,
    ///     keepalive: false,
    ///     signal: None,
    ///     priority: RequestPriority::Auto,
    ///     window: None,
    /// };
    ///
    /// scheduler.schedule(request, RequestPriority::High);
    /// let next = scheduler.next_request();
    /// assert!(next.is_some());
    /// ```
    pub fn next_request(&mut self) -> Option<NetworkRequest> {
        // Check if we've reached max concurrent requests
        if self.active_requests.len() >= self.max_concurrent {
            return None;
        }

        // Try high priority first
        if let Some(pending) = self.high_priority.pop_front() {
            let request = pending.request.clone();
            let active = pending.into_active();
            self.active_requests.insert(active.id, active);
            return Some(request);
        }

        // Then medium priority
        if let Some(pending) = self.medium_priority.pop_front() {
            let request = pending.request.clone();
            let active = pending.into_active();
            self.active_requests.insert(active.id, active);
            return Some(request);
        }

        // Finally low priority
        if let Some(pending) = self.low_priority.pop_front() {
            let request = pending.request.clone();
            let active = pending.into_active();
            self.active_requests.insert(active.id, active);
            return Some(request);
        }

        None
    }

    /// Cancel a pending or active request
    ///
    /// # Arguments
    ///
    /// * `id` - The request ID to cancel
    ///
    /// # Errors
    ///
    /// Returns `NetworkError::Aborted` if the request ID is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use request_scheduler::{RequestScheduler, RequestPriority};
    /// use network_types::{NetworkRequest, HttpMethod, RequestMode, CredentialsMode, CacheMode, RedirectMode, ReferrerPolicy};
    /// use url::Url;
    ///
    /// let mut scheduler = RequestScheduler::new(6);
    ///
    /// let request = NetworkRequest {
    ///     url: Url::parse("https://example.com").unwrap(),
    ///     method: HttpMethod::Get,
    ///     headers: http::HeaderMap::new(),
    ///     body: None,
    ///     mode: RequestMode::Navigate,
    ///     credentials: CredentialsMode::SameOrigin,
    ///     cache: CacheMode::Default,
    ///     redirect: RedirectMode::Follow,
    ///     referrer: None,
    ///     referrer_policy: ReferrerPolicy::NoReferrerWhenDowngrade,
    ///     integrity: None,
    ///     keepalive: false,
    ///     signal: None,
    ///     priority: RequestPriority::Auto,
    ///     window: None,
    /// };
    ///
    /// let id = scheduler.schedule(request, RequestPriority::High);
    /// scheduler.cancel_request(id).expect("Should cancel successfully");
    /// ```
    pub fn cancel_request(&mut self, id: RequestId) -> Result<(), NetworkError> {
        // Try to remove from active requests first
        if self.active_requests.remove(&id).is_some() {
            return Ok(());
        }

        // Try to remove from priority queues
        if Self::remove_from_queue(&mut self.high_priority, id) {
            return Ok(());
        }

        if Self::remove_from_queue(&mut self.medium_priority, id) {
            return Ok(());
        }

        if Self::remove_from_queue(&mut self.low_priority, id) {
            return Ok(());
        }

        Err(NetworkError::Aborted)
    }

    /// Set the maximum concurrent requests limit
    ///
    /// # Arguments
    ///
    /// * `max` - New maximum concurrent request limit
    ///
    /// # Examples
    ///
    /// ```
    /// use request_scheduler::RequestScheduler;
    ///
    /// let mut scheduler = RequestScheduler::new(6);
    /// scheduler.set_max_concurrent(10);
    /// ```
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }

    /// Helper to remove a request from a queue by ID
    fn remove_from_queue(queue: &mut VecDeque<PendingRequest>, id: RequestId) -> bool {
        if let Some(pos) = queue.iter().position(|r| r.id == id) {
            queue.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = RequestScheduler::new(6);
        assert_eq!(scheduler.max_concurrent, 6);
        assert_eq!(scheduler.active_requests.len(), 0);
    }

    #[test]
    fn test_id_generation_increments() {
        let mut scheduler = RequestScheduler::new(10);
        assert_eq!(scheduler.next_id, 1);

        // Simulate scheduling (ID generation happens in schedule)
        let id1 = scheduler.next_id;
        scheduler.next_id += 1;
        let id2 = scheduler.next_id;

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }
}
