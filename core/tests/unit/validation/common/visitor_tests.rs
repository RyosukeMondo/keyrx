//! Unit tests for OperationVisitor trait and visit_all function.

use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{HoldAction, LayerAction};
use keyrx_core::scripting::{PendingOp, TimingUpdate};
use keyrx_core::validation::common::visitor::{visit_all, OperationVisitor};

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
