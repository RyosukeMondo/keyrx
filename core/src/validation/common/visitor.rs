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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::keycodes::KeyCode;
    use crate::scripting::PendingOp;

    struct CountingVisitor {
        remap_count: usize,
        block_count: usize,
        total_count: usize,
        visit_order: Vec<usize>,
        begin_called: bool,
        end_called: bool,
    }

    impl OperationVisitor for CountingVisitor {
        fn visit(&mut self, index: usize, operation: &PendingOp) -> bool {
            self.total_count += 1;
            self.visit_order.push(index);

            match operation {
                PendingOp::Remap { .. } => self.remap_count += 1,
                PendingOp::Block { .. } => self.block_count += 1,
                _ => {}
            }

            true // continue
        }

        fn begin(&mut self) {
            self.begin_called = true;
        }

        fn end(&mut self) {
            self.end_called = true;
        }
    }

    struct EarlyTerminationVisitor {
        stop_at_index: usize,
        visited_indices: Vec<usize>,
    }

    impl OperationVisitor for EarlyTerminationVisitor {
        fn visit(&mut self, index: usize, _operation: &PendingOp) -> bool {
            self.visited_indices.push(index);
            index < self.stop_at_index
        }
    }

    #[test]
    fn test_visit_all_operations() {
        let operations = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::C },
            PendingOp::Remap {
                from: KeyCode::D,
                to: KeyCode::E,
            },
        ];

        let mut visitor = CountingVisitor {
            remap_count: 0,
            block_count: 0,
            total_count: 0,
            visit_order: Vec::new(),
            begin_called: false,
            end_called: false,
        };

        visit_all(&operations, &mut visitor);

        assert_eq!(visitor.total_count, 3);
        assert_eq!(visitor.remap_count, 2);
        assert_eq!(visitor.block_count, 1);
        assert_eq!(visitor.visit_order, vec![0, 1, 2]);
        assert!(visitor.begin_called);
        assert!(visitor.end_called);
    }

    #[test]
    fn test_empty_operations() {
        let operations: Vec<PendingOp> = vec![];

        let mut visitor = CountingVisitor {
            remap_count: 0,
            block_count: 0,
            total_count: 0,
            visit_order: Vec::new(),
            begin_called: false,
            end_called: false,
        };

        visit_all(&operations, &mut visitor);

        assert_eq!(visitor.total_count, 0);
        assert_eq!(visitor.visit_order, Vec::<usize>::new());
        assert!(visitor.begin_called);
        assert!(visitor.end_called);
    }

    #[test]
    fn test_early_termination() {
        let operations = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::D,
            },
            PendingOp::Remap {
                from: KeyCode::E,
                to: KeyCode::F,
            },
            PendingOp::Remap {
                from: KeyCode::G,
                to: KeyCode::H,
            },
        ];

        let mut visitor = EarlyTerminationVisitor {
            stop_at_index: 2,
            visited_indices: Vec::new(),
        };

        visit_all(&operations, &mut visitor);

        // Should visit indices 0, 1, 2, then stop
        assert_eq!(visitor.visited_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_visitor_with_all_operation_types() {
        use crate::engine::{HoldAction, LayerAction};
        use crate::scripting::TimingUpdate;

        let operations = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::C },
            PendingOp::Pass { key: KeyCode::D },
            PendingOp::TapHold {
                key: KeyCode::E,
                tap: KeyCode::F,
                hold: HoldAction::Key(KeyCode::G),
            },
            PendingOp::Combo {
                keys: vec![KeyCode::H, KeyCode::I],
                action: LayerAction::LayerPush(1),
            },
            PendingOp::LayerDefine {
                name: "layer1".to_string(),
                transparent: false,
            },
            PendingOp::LayerPush {
                name: "layer1".to_string(),
            },
            PendingOp::LayerToggle {
                name: "layer1".to_string(),
            },
            PendingOp::LayerPop,
            PendingOp::DefineModifier {
                name: "ctrl".to_string(),
                id: 1,
            },
            PendingOp::ModifierActivate {
                name: "ctrl".to_string(),
                id: 1,
            },
            PendingOp::ModifierDeactivate {
                name: "ctrl".to_string(),
                id: 1,
            },
            PendingOp::ModifierOneShot {
                name: "shift".to_string(),
                id: 2,
            },
            PendingOp::SetTiming(TimingUpdate::TapTimeout(250)),
        ];

        let mut visitor = CountingVisitor {
            remap_count: 0,
            block_count: 0,
            total_count: 0,
            visit_order: Vec::new(),
            begin_called: false,
            end_called: false,
        };

        visit_all(&operations, &mut visitor);

        assert_eq!(visitor.total_count, 14);
        assert_eq!(visitor.remap_count, 1);
        assert_eq!(visitor.block_count, 1);
        assert_eq!(visitor.visit_order.len(), 14);
        assert!(visitor.begin_called);
        assert!(visitor.end_called);
    }
}
