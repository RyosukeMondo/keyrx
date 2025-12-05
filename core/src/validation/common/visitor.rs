//! Operation visitor pattern for validation detectors.
//!
//! Provides a reusable abstraction for traversing `PendingOp` collections,
//! allowing detectors to focus on detection logic rather than iteration.

use crate::scripting::PendingOp;

/// A visitor that can inspect operations during traversal.
///
/// Detectors implement this trait to process operations in a pass.
/// The visitor pattern enables:
/// - Separation of traversal from detection logic
/// - Reusable iteration code across detectors
/// - Flexible filtering and early termination
pub trait OperationVisitor {
    /// Visit a single operation at the given index.
    ///
    /// # Arguments
    /// * `index` - Position of the operation in the list (for location tracking)
    /// * `operation` - The operation to inspect
    ///
    /// # Returns
    /// * `true` to continue traversal
    /// * `false` to stop early (optimization for detectors that found what they need)
    fn visit(&mut self, index: usize, operation: &PendingOp) -> bool;

    /// Called before starting traversal (optional hook).
    ///
    /// Useful for initialization or pre-processing.
    fn begin(&mut self) {}

    /// Called after completing traversal (optional hook).
    ///
    /// Useful for final aggregation or post-processing.
    fn end(&mut self) {}
}

/// Traverse all operations and invoke the visitor for each.
///
/// This is the main entry point for using the visitor pattern.
/// It handles:
/// - Iteration over the operation slice
/// - Calling begin/end hooks
/// - Early termination support
///
/// # Examples
///
/// ```ignore
/// struct MyVisitor {
///     count: usize,
/// }
///
/// impl OperationVisitor for MyVisitor {
///     fn visit(&mut self, index: usize, operation: &PendingOp) -> bool {
///         self.count += 1;
///         true // continue
///     }
/// }
///
/// let mut visitor = MyVisitor { count: 0 };
/// visit_all(&operations, &mut visitor);
/// println!("Visited {} operations", visitor.count);
/// ```
pub fn visit_all<V: OperationVisitor>(operations: &[PendingOp], visitor: &mut V) {
    visitor.begin();

    for (index, operation) in operations.iter().enumerate() {
        let should_continue = visitor.visit(index, operation);
        if !should_continue {
            break;
        }
    }

    visitor.end();
}
