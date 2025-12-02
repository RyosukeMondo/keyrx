// Re-export from debugger_page.dart for backward compatibility.
//
// The debugger has been refactored into modular structure:
// - debugger_page.dart: Main DebuggerPage widget and state management
// - debugger_widgets.dart: TagSectionCard, PendingDecisionsCard, TimingCard, EventLogWidget, TimelineWidget
// - debugger_meters.dart: LatencyMeterCard, PendingTapHoldWidget, PendingComboWidget

export 'debugger_page.dart';
export 'debugger_widgets.dart';
export 'debugger_meters.dart';
