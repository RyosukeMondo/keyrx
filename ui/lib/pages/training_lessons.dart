// KeyRx training lesson definitions and data structures.
//
// Contains the lesson framework types and predefined lessons
// for keyboard remapping training.

import 'package:flutter/material.dart';
import '../services/engine_service.dart';

/// Result of an exercise validation attempt.
class ExerciseResult {
  const ExerciseResult({
    required this.success,
    this.message,
  });

  final bool success;
  final String? message;
}

/// An interactive exercise that validates user input.
class TrainingExercise {
  const TrainingExercise({
    required this.prompt,
    required this.validator,
    this.successMessage = 'Correct!',
    this.failureMessage,
  });

  /// The instruction shown to the user (e.g., "Press A to see B").
  final String prompt;

  /// Validates the exercise and returns success/failure with message.
  final ExerciseResult Function(EngineSnapshot snapshot) validator;

  /// Message shown on success.
  final String successMessage;

  /// Message shown on failure (if null, derived from validator result).
  final String? failureMessage;
}

/// A single step within a training lesson.
class TrainingStep {
  const TrainingStep({
    required this.instruction,
    required this.validator,
    this.hint,
    this.expectedOutput,
    this.exercise,
  });

  final String instruction;
  final bool Function(EngineSnapshot snapshot) validator;
  final String? hint;
  final String? expectedOutput;

  /// Optional interactive exercise for this step.
  final TrainingExercise? exercise;
}

/// A complete training lesson with multiple steps.
class TrainingLesson {
  const TrainingLesson({
    required this.id,
    required this.title,
    required this.description,
    required this.steps,
    required this.icon,
  });

  final String id;
  final String title;
  final String description;
  final List<TrainingStep> steps;
  final IconData icon;
}

/// Animated certificate dialog shown when all lessons are complete.
class CertificateDialog extends StatefulWidget {
  const CertificateDialog({super.key, required this.lessonCount, required this.onDismiss});
  final int lessonCount;
  final VoidCallback onDismiss;
  @override
  State<CertificateDialog> createState() => _CertificateDialogState();
}

class _CertificateDialogState extends State<CertificateDialog>
    with SingleTickerProviderStateMixin {
  late AnimationController _ctrl;
  late Animation<double> _scale, _fade;

  @override
  void initState() {
    super.initState();
    _ctrl = AnimationController(vsync: this, duration: const Duration(milliseconds: 800));
    _scale = CurvedAnimation(parent: _ctrl, curve: Curves.elasticOut);
    _fade = CurvedAnimation(parent: _ctrl, curve: const Interval(0.0, 0.5));
    _ctrl.forward();
  }

  @override
  void dispose() { _ctrl.dispose(); super.dispose(); }

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: 'Certificate of completion. You mastered ${widget.lessonCount} lessons.',
      child: Dialog(
        backgroundColor: Colors.transparent,
        child: FadeTransition(
          opacity: _fade,
          child: ScaleTransition(
            scale: _scale,
            child: Container(
              constraints: const BoxConstraints(maxWidth: 400),
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  colors: [Colors.amber.shade100, Colors.amber.shade50, Colors.white],
                ),
                borderRadius: BorderRadius.circular(20),
                border: Border.all(color: Colors.amber.shade400, width: 3),
                boxShadow: [BoxShadow(color: Colors.amber.withValues(alpha: 0.3), blurRadius: 20)],
              ),
              padding: const EdgeInsets.all(32),
              child: Column(mainAxisSize: MainAxisSize.min, children: [
                Icon(Icons.emoji_events, size: 72, color: Colors.amber.shade600),
                const SizedBox(height: 12),
                Text('CERTIFICATE', style: TextStyle(fontSize: 12, letterSpacing: 4, color: Colors.amber.shade800)),
                Text('of Completion', style: TextStyle(fontSize: 22, fontWeight: FontWeight.bold, color: Colors.amber.shade900)),
                const SizedBox(height: 16),
                Text('Congratulations!', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: Colors.grey.shade800)),
                const SizedBox(height: 8),
                Text('You mastered all ${widget.lessonCount} KeyRx lessons.', textAlign: TextAlign.center, style: TextStyle(color: Colors.grey.shade700)),
                const SizedBox(height: 16),
                Row(mainAxisAlignment: MainAxisAlignment.center, children: List.generate(5, (_) => Icon(Icons.star, color: Colors.amber.shade600))),
                const SizedBox(height: 20),
                FilledButton.icon(onPressed: widget.onDismiss, icon: const Icon(Icons.celebration), label: const Text('Continue'),
                  style: FilledButton.styleFrom(backgroundColor: Colors.amber.shade600)),
              ]),
            ),
          ),
        ),
      ),
    );
  }
}

/// Factory to build the training lessons.
List<TrainingLesson> buildTrainingLessons() {
  return [
    TrainingLesson(
      id: 'remap',
      title: 'Basic Remapping',
      description: 'Learn to remap one key to another',
      icon: Icons.swap_horiz,
      steps: [
        TrainingStep(
          instruction: 'Press the CapsLock key to see it remapped to Escape',
          hint: 'CapsLock is often remapped to Escape for easier Vim usage',
          expectedOutput: 'Escape key event',
          validator: (snapshot) =>
              snapshot.lastEvent?.toLowerCase().contains('escape') ?? false,
          exercise: TrainingExercise(
            prompt: 'Press CapsLock to produce Escape',
            successMessage: 'CapsLock successfully remapped to Escape!',
            failureMessage: 'Expected Escape output, but got something else',
            validator: (snapshot) {
              if (snapshot.lastEvent == null) {
                return const ExerciseResult(success: false, message: 'No key event detected');
              }
              final isEscape = snapshot.lastEvent!.toLowerCase().contains('escape');
              return ExerciseResult(
                success: isEscape,
                message: isEscape ? null : 'Got "${snapshot.lastEvent}" instead of Escape',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Press the key you configured as a remap source',
          hint: 'Check your script for remap() calls',
          validator: (snapshot) => snapshot.lastEvent != null,
          exercise: TrainingExercise(
            prompt: 'Press any remapped key to see the output',
            successMessage: 'Key remapping detected!',
            validator: (snapshot) => ExerciseResult(
              success: snapshot.lastEvent != null,
              message: snapshot.lastEvent != null
                  ? 'Output: ${snapshot.lastEvent}'
                  : 'Press a key to see the remap',
            ),
          ),
        ),
      ],
    ),
    TrainingLesson(
      id: 'layer',
      title: 'Layers',
      description: 'Activate different key layouts',
      icon: Icons.layers,
      steps: [
        TrainingStep(
          instruction: 'Hold your layer key to activate a layer',
          hint: 'Layers let you access different key mappings while held',
          expectedOutput: 'Layer becomes active',
          validator: (snapshot) => snapshot.activeLayers.isNotEmpty,
          exercise: TrainingExercise(
            prompt: 'Hold your layer key to activate it',
            successMessage: 'Layer activated!',
            failureMessage: 'No layer is active yet',
            validator: (snapshot) {
              final hasLayers = snapshot.activeLayers.isNotEmpty;
              return ExerciseResult(
                success: hasLayers,
                message: hasLayers
                    ? 'Active layers: ${snapshot.activeLayers.join(", ")}'
                    : 'Hold your layer key to activate',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'While holding the layer key, press another key',
          hint: 'Keys behave differently on each layer',
          validator: (snapshot) =>
              snapshot.activeLayers.isNotEmpty && snapshot.lastEvent != null,
          exercise: TrainingExercise(
            prompt: 'With layer active, press a key to see layer-specific output',
            successMessage: 'Key on layer triggered!',
            validator: (snapshot) {
              final hasLayer = snapshot.activeLayers.isNotEmpty;
              final hasEvent = snapshot.lastEvent != null;
              if (!hasLayer) {
                return const ExerciseResult(success: false, message: 'Hold your layer key first');
              }
              return ExerciseResult(
                success: hasEvent,
                message: hasEvent ? 'Output: ${snapshot.lastEvent}' : 'Press a key while on the layer',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Release the layer key to deactivate the layer',
          hint: 'Releasing returns to the base layer',
          validator: (snapshot) => snapshot.activeLayers.isEmpty,
          exercise: TrainingExercise(
            prompt: 'Release all keys to return to base layer',
            successMessage: 'Back to base layer!',
            validator: (snapshot) => ExerciseResult(
              success: snapshot.activeLayers.isEmpty,
              message: snapshot.activeLayers.isEmpty
                  ? null
                  : 'Layers still active: ${snapshot.activeLayers.join(", ")}',
            ),
          ),
        ),
      ],
    ),
    TrainingLesson(
      id: 'modifier',
      title: 'Modifiers',
      description: 'Track modifier key states',
      icon: Icons.keyboard_alt,
      steps: [
        TrainingStep(
          instruction: 'Hold Shift to see it as an active modifier',
          hint: 'Modifiers are tracked separately from regular keys',
          expectedOutput: 'Shift appears in active modifiers',
          validator: (snapshot) => snapshot.activeModifiers
              .any((m) => m.toLowerCase().contains('shift')),
          exercise: TrainingExercise(
            prompt: 'Press and hold the Shift key',
            successMessage: 'Shift modifier detected!',
            failureMessage: 'Shift not detected as active modifier',
            validator: (snapshot) {
              final hasShift = snapshot.activeModifiers
                  .any((m) => m.toLowerCase().contains('shift'));
              return ExerciseResult(
                success: hasShift,
                message: hasShift
                    ? 'Active modifiers: ${snapshot.activeModifiers.join(", ")}'
                    : 'Hold Shift key',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Hold Ctrl to add another modifier',
          hint: 'Multiple modifiers can be active simultaneously',
          validator: (snapshot) =>
              snapshot.activeModifiers.any((m) => m.toLowerCase().contains('ctrl')),
          exercise: TrainingExercise(
            prompt: 'Press and hold the Ctrl key',
            successMessage: 'Ctrl modifier detected!',
            validator: (snapshot) {
              final hasCtrl = snapshot.activeModifiers
                  .any((m) => m.toLowerCase().contains('ctrl'));
              return ExerciseResult(
                success: hasCtrl,
                message: hasCtrl
                    ? 'Modifiers: ${snapshot.activeModifiers.join(", ")}'
                    : 'Hold Ctrl key',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Release all modifiers',
          hint: 'Check that no modifiers remain active',
          validator: (snapshot) => snapshot.activeModifiers.isEmpty,
          exercise: TrainingExercise(
            prompt: 'Release all modifier keys',
            successMessage: 'All modifiers released!',
            validator: (snapshot) => ExerciseResult(
              success: snapshot.activeModifiers.isEmpty,
              message: snapshot.activeModifiers.isEmpty
                  ? null
                  : 'Still active: ${snapshot.activeModifiers.join(", ")}',
            ),
          ),
        ),
      ],
    ),
    TrainingLesson(
      id: 'taphold',
      title: 'Tap-Hold',
      description: 'One key, two behaviors',
      icon: Icons.touch_app,
      steps: [
        TrainingStep(
          instruction: 'Quickly tap your tap-hold key to trigger the tap action',
          hint: 'A tap-hold key sends one action on tap, another on hold',
          expectedOutput: 'Tap action fires',
          validator: (snapshot) =>
              snapshot.lastEvent != null && snapshot.pendingDecisions.isEmpty,
          exercise: TrainingExercise(
            prompt: 'Quick tap (release before timeout) your tap-hold key',
            successMessage: 'Tap action triggered!',
            validator: (snapshot) {
              final hasPending = snapshot.pendingDecisions.isNotEmpty;
              final hasEvent = snapshot.lastEvent != null;
              if (hasPending) {
                return const ExerciseResult(
                  success: false,
                  message: 'Still waiting for decision - release quickly for tap',
                );
              }
              return ExerciseResult(
                success: hasEvent,
                message: hasEvent ? 'Tap output: ${snapshot.lastEvent}' : 'Tap your tap-hold key quickly',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Hold the tap-hold key until the hold action triggers',
          hint: 'Watch the pending decisions for the countdown',
          expectedOutput: 'Hold action fires',
          validator: (snapshot) =>
              snapshot.pendingDecisions.any((p) => p.toLowerCase().contains('hold')),
          exercise: TrainingExercise(
            prompt: 'Hold your tap-hold key past the timeout threshold',
            successMessage: 'Hold action detected!',
            failureMessage: 'Hold longer until the hold action triggers',
            validator: (snapshot) {
              final hasHold = snapshot.pendingDecisions
                  .any((p) => p.toLowerCase().contains('hold'));
              return ExerciseResult(
                success: hasHold,
                message: hasHold
                    ? 'Pending: ${snapshot.pendingDecisions.join(", ")}'
                    : 'Keep holding to trigger hold action',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Release the key after the hold triggers',
          hint: 'The hold behavior should now be active',
          validator: (snapshot) =>
              snapshot.pendingDecisions.isEmpty && snapshot.heldKeys.isEmpty,
          exercise: TrainingExercise(
            prompt: 'Release the tap-hold key after hold triggers',
            successMessage: 'Hold action completed!',
            validator: (snapshot) {
              final allClear = snapshot.pendingDecisions.isEmpty && snapshot.heldKeys.isEmpty;
              return ExerciseResult(
                success: allClear,
                message: allClear ? null : 'Release all keys',
              );
            },
          ),
        ),
      ],
    ),
    TrainingLesson(
      id: 'combo',
      title: 'Combos',
      description: 'Trigger actions with key combinations',
      icon: Icons.group_work,
      steps: [
        TrainingStep(
          instruction: 'Press your first combo key and hold it',
          hint: 'Combos require pressing multiple keys together',
          expectedOutput: 'First key held',
          validator: (snapshot) => snapshot.heldKeys.isNotEmpty,
          exercise: TrainingExercise(
            prompt: 'Press and hold the first key of your combo',
            successMessage: 'First key held!',
            validator: (snapshot) => ExerciseResult(
              success: snapshot.heldKeys.isNotEmpty,
              message: snapshot.heldKeys.isNotEmpty
                  ? 'Held: ${snapshot.heldKeys.join(", ")}'
                  : 'Press and hold a combo key',
            ),
          ),
        ),
        TrainingStep(
          instruction: 'While holding, press the second combo key to complete',
          hint: 'The combo should trigger when both keys are pressed',
          validator: (snapshot) =>
              snapshot.pendingDecisions.any((p) => p.toLowerCase().contains('combo')) ||
              snapshot.heldKeys.length >= 2,
          exercise: TrainingExercise(
            prompt: 'While holding first key, press the second combo key',
            successMessage: 'Combo triggered!',
            validator: (snapshot) {
              final hasCombo = snapshot.pendingDecisions
                  .any((p) => p.toLowerCase().contains('combo'));
              final hasTwoKeys = snapshot.heldKeys.length >= 2;
              final success = hasCombo || hasTwoKeys;
              return ExerciseResult(
                success: success,
                message: success
                    ? hasCombo
                        ? 'Combo: ${snapshot.pendingDecisions.join(", ")}'
                        : 'Keys: ${snapshot.heldKeys.join(" + ")}'
                    : 'Hold first key, then press second',
              );
            },
          ),
        ),
        TrainingStep(
          instruction: 'Release all keys to complete the combo',
          hint: 'The combo action should have been triggered',
          validator: (snapshot) =>
              snapshot.heldKeys.isEmpty && snapshot.pendingDecisions.isEmpty,
          exercise: TrainingExercise(
            prompt: 'Release all keys to finalize the combo',
            successMessage: 'Combo completed!',
            validator: (snapshot) {
              final allClear = snapshot.heldKeys.isEmpty && snapshot.pendingDecisions.isEmpty;
              return ExerciseResult(
                success: allClear,
                message: allClear ? null : 'Release all keys',
              );
            },
          ),
        ),
      ],
    ),
  ];
}
