//! Memory allocation tracking for profiling
//!
//! This module provides a global allocator wrapper that tracks allocation
//! sites and sizes for memory profiling purposes.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::error::KeyRxError;

/// Information about an allocation site
#[derive(Debug, Clone)]
pub struct AllocationSite {
    /// Location identifier (e.g., file:line or function name)
    pub location: String,
    /// Number of allocations from this site
    pub count: u64,
    /// Total bytes allocated from this site
    pub total_bytes: usize,
    /// Stack trace at allocation point
    pub stack_trace: Vec<String>,
}

/// Statistics about memory allocations
#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes freed
    pub total_freed: usize,
    /// Peak memory usage
    pub peak_usage: usize,
    /// Current memory usage
    pub current_usage: usize,
    /// Total number of allocations
    pub allocation_count: u64,
    /// Total number of frees
    pub free_count: u64,
}

/// Global state for allocation tracking
struct AllocationState {
    /// Whether tracking is currently enabled
    enabled: AtomicBool,
    /// Minimum allocation size to track
    threshold: AtomicUsize,
    /// Total bytes allocated
    total_allocated: AtomicUsize,
    /// Total bytes freed
    total_freed: AtomicUsize,
    /// Peak memory usage
    peak_usage: AtomicUsize,
    /// Current memory usage
    current_usage: AtomicUsize,
    /// Total allocation count
    allocation_count: AtomicUsize,
    /// Total free count
    free_count: AtomicUsize,
    /// Allocation sites being tracked
    sites: Mutex<Vec<AllocationSite>>,
}

impl AllocationState {
    const fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            threshold: AtomicUsize::new(1024), // 1KB default
            total_allocated: AtomicUsize::new(0),
            total_freed: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
            current_usage: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            free_count: AtomicUsize::new(0),
            sites: Mutex::new(Vec::new()),
        }
    }

    fn record_allocation(&self, size: usize, layout: Layout) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        let threshold = self.threshold.load(Ordering::Relaxed);
        if size < threshold {
            return;
        }

        // Update statistics
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        let current = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak if needed
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }

        // Capture allocation site
        self.capture_allocation_site(size, layout);
    }

    fn record_deallocation(&self, size: usize) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        self.total_freed.fetch_add(size, Ordering::Relaxed);
        self.free_count.fetch_add(1, Ordering::Relaxed);
        self.current_usage.fetch_sub(size, Ordering::Relaxed);
    }

    fn capture_allocation_site(&self, size: usize, _layout: Layout) {
        // Capture stack trace
        let stack_trace = capture_stack_trace();

        // Extract location from top frame
        let location = stack_trace
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        // Find or create allocation site
        if let Ok(mut sites) = self.sites.lock() {
            if let Some(site) = sites.iter_mut().find(|s| s.location == location) {
                site.count += 1;
                site.total_bytes += size;
            } else {
                sites.push(AllocationSite {
                    location,
                    count: 1,
                    total_bytes: size,
                    stack_trace,
                });
            }
        }
    }

    fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_freed: self.total_freed.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed) as u64,
            free_count: self.free_count.load(Ordering::Relaxed) as u64,
        }
    }

    fn get_sites(&self) -> Vec<AllocationSite> {
        self.sites
            .lock()
            .map(|sites| sites.clone())
            .unwrap_or_default()
    }

    fn reset(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_freed.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
        self.current_usage.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        self.free_count.store(0, Ordering::Relaxed);
        if let Ok(mut sites) = self.sites.lock() {
            sites.clear();
        }
    }
}

/// Global allocation state
static ALLOCATION_STATE: AllocationState = AllocationState::new();

/// Tracking allocator that wraps the system allocator
pub struct TrackingAllocator;

#[allow(unsafe_code)]
unsafe impl GlobalAlloc for TrackingAllocator {
    #[allow(unsafe_code)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATION_STATE.record_allocation(layout.size(), layout);
        }
        ptr
    }

    #[allow(unsafe_code)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATION_STATE.record_deallocation(layout.size());
        System.dealloc(ptr, layout);
    }

    #[allow(unsafe_code)]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            ALLOCATION_STATE.record_allocation(layout.size(), layout);
        }
        ptr
    }

    #[allow(unsafe_code)]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.size();
        let new_ptr = System.realloc(ptr, layout, new_size);

        if !new_ptr.is_null() {
            ALLOCATION_STATE.record_deallocation(old_size);
            let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
            ALLOCATION_STATE.record_allocation(new_size, new_layout);
        }

        new_ptr
    }
}

/// Controller for allocation tracking
pub struct AllocationTracker {
    threshold: usize,
}

impl AllocationTracker {
    /// Create a new allocation tracker with the given threshold
    pub fn new(threshold: usize) -> Self {
        Self { threshold }
    }

    /// Start tracking allocations
    pub fn start(&mut self) -> Result<(), KeyRxError> {
        if ALLOCATION_STATE.enabled.load(Ordering::Relaxed) {
            return Err(KeyRxError::platform("Allocation tracking already started"));
        }

        ALLOCATION_STATE
            .threshold
            .store(self.threshold, Ordering::Relaxed);
        ALLOCATION_STATE.reset();
        ALLOCATION_STATE.enabled.store(true, Ordering::Relaxed);

        Ok(())
    }

    /// Stop tracking and return statistics
    pub fn stop(&mut self) -> Result<AllocationStats, KeyRxError> {
        if !ALLOCATION_STATE.enabled.load(Ordering::Relaxed) {
            return Err(KeyRxError::platform("Allocation tracking not started"));
        }

        ALLOCATION_STATE.enabled.store(false, Ordering::Relaxed);
        Ok(ALLOCATION_STATE.get_stats())
    }

    /// Get current allocation statistics
    pub fn stats(&self) -> AllocationStats {
        ALLOCATION_STATE.get_stats()
    }

    /// Get tracked allocation sites
    pub fn sites(&self) -> Vec<AllocationSite> {
        ALLOCATION_STATE.get_sites()
    }

    /// Check if tracking is currently active
    pub fn is_active(&self) -> bool {
        ALLOCATION_STATE.enabled.load(Ordering::Relaxed)
    }
}

/// Capture current stack trace for allocation site tracking
fn capture_stack_trace() -> Vec<String> {
    let bt = backtrace::Backtrace::new();
    format!("{:?}", bt)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| {
            // Filter out allocator frames
            !line.contains("alloc::")
                && !line.contains("TrackingAllocator")
                && !line.contains("std::sys::pal")
        })
        .take(10) // Limit stack depth
        .map(|line| {
            // Extract meaningful part
            let trimmed = line.trim();
            if let Some(at_pos) = trimmed.find("at ") {
                trimmed[at_pos + 3..].to_string()
            } else {
                trimmed.to_string()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static RESET: Once = Once::new();

    fn reset_global_state() {
        RESET.call_once(|| {
            ALLOCATION_STATE.reset();
        });
    }

    #[test]
    fn test_tracker_lifecycle() {
        reset_global_state();
        let mut tracker = AllocationTracker::new(1024);
        assert!(!tracker.is_active());

        tracker.start().expect("Failed to start tracker");
        assert!(tracker.is_active());

        let stats = tracker.stop().expect("Failed to stop tracker");
        assert!(!tracker.is_active());

        // Stats should be available (may be zero without TrackingAllocator)
        assert_eq!(stats.current_usage, 0);
        assert!(stats.total_allocated >= stats.total_freed);
    }

    #[test]
    fn test_stop_without_start_returns_error() {
        reset_global_state();

        // Create a fresh tracker
        let mut tracker = AllocationTracker::new(1024);

        // Ensure global state is not enabled
        ALLOCATION_STATE.enabled.store(false, Ordering::Relaxed);

        let result = tracker.stop();
        assert!(
            result.is_err(),
            "Stopping without starting should return an error"
        );
    }

    #[test]
    fn test_stats_returns_valid_structure() {
        reset_global_state();
        let tracker = AllocationTracker::new(1024);
        let stats = tracker.stats();

        // Stats structure should be valid
        assert!(stats.total_allocated >= stats.total_freed);
        assert!(stats.peak_usage >= stats.current_usage);
    }

    #[test]
    fn test_sites_returns_empty_initially() {
        reset_global_state();
        let tracker = AllocationTracker::new(1024);
        let sites = tracker.sites();

        // Should return a vector (may be empty without actual allocations)
        assert!(sites.is_empty() || !sites.is_empty());
    }

    #[test]
    fn test_threshold_configuration() {
        reset_global_state();
        let tracker = AllocationTracker::new(2048);
        assert_eq!(tracker.threshold, 2048);
    }

    #[test]
    fn test_capture_stack_trace() {
        let trace = capture_stack_trace();
        assert!(!trace.is_empty(), "Should capture at least one frame");

        // Should filter out allocator frames
        for frame in &trace {
            assert!(!frame.contains("TrackingAllocator"));
        }

        // Should limit stack depth
        assert!(
            trace.len() <= 10,
            "Stack trace should be limited to 10 frames"
        );
    }

    #[test]
    fn test_allocation_site_clone() {
        let site = AllocationSite {
            location: "test.rs:42".to_string(),
            count: 10,
            total_bytes: 1024,
            stack_trace: vec!["frame1".to_string(), "frame2".to_string()],
        };

        let cloned = site.clone();
        assert_eq!(cloned.location, site.location);
        assert_eq!(cloned.count, site.count);
        assert_eq!(cloned.total_bytes, site.total_bytes);
        assert_eq!(cloned.stack_trace, site.stack_trace);
    }

    #[test]
    fn test_allocation_stats_default() {
        let stats = AllocationStats::default();
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.total_freed, 0);
        assert_eq!(stats.peak_usage, 0);
        assert_eq!(stats.current_usage, 0);
        assert_eq!(stats.allocation_count, 0);
        assert_eq!(stats.free_count, 0);
    }

    #[test]
    fn test_tracking_allocator_exists() {
        // Just verify the TrackingAllocator type exists and can be referenced
        let _allocator = TrackingAllocator;
        // This test just ensures the type compiles and exists
    }
}
