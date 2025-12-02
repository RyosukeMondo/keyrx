// KeyRx training lesson definitions and data structures.
//
// Contains the lesson framework types and predefined lessons
// for keyboard remapping training.

import 'package:flutter/material.dart';
import '../services/engine_service.dart';

/// A single step within a training lesson.
class TrainingStep {
  const TrainingStep({
    required this.instruction,
    required this.validator,
    this.hint,
    this.expectedOutput,
  });

  final String instruction;
  final bool Function(EngineSnapshot snapshot) validator;
  final String? hint;
  final String? expectedOutput;
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
        ),
        TrainingStep(
          instruction: 'Press the key you configured as a remap source',
          hint: 'Check your script for remap() calls',
          validator: (snapshot) => snapshot.lastEvent != null,
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
        ),
        TrainingStep(
          instruction: 'While holding the layer key, press another key',
          hint: 'Keys behave differently on each layer',
          validator: (snapshot) =>
              snapshot.activeLayers.isNotEmpty && snapshot.lastEvent != null,
        ),
        TrainingStep(
          instruction: 'Release the layer key to deactivate the layer',
          hint: 'Releasing returns to the base layer',
          validator: (snapshot) => snapshot.activeLayers.isEmpty,
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
        ),
        TrainingStep(
          instruction: 'Hold Ctrl to add another modifier',
          hint: 'Multiple modifiers can be active simultaneously',
          validator: (snapshot) =>
              snapshot.activeModifiers.any((m) => m.toLowerCase().contains('ctrl')),
        ),
        TrainingStep(
          instruction: 'Release all modifiers',
          hint: 'Check that no modifiers remain active',
          validator: (snapshot) => snapshot.activeModifiers.isEmpty,
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
        ),
        TrainingStep(
          instruction: 'Hold the tap-hold key until the hold action triggers',
          hint: 'Watch the pending decisions for the countdown',
          expectedOutput: 'Hold action fires',
          validator: (snapshot) =>
              snapshot.pendingDecisions.any((p) => p.toLowerCase().contains('hold')),
        ),
        TrainingStep(
          instruction: 'Release the key after the hold triggers',
          hint: 'The hold behavior should now be active',
          validator: (snapshot) =>
              snapshot.pendingDecisions.isEmpty && snapshot.heldKeys.isEmpty,
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
        ),
        TrainingStep(
          instruction: 'While holding, press the second combo key to complete',
          hint: 'The combo should trigger when both keys are pressed',
          validator: (snapshot) =>
              snapshot.pendingDecisions.any((p) => p.toLowerCase().contains('combo')) ||
              snapshot.heldKeys.length >= 2,
        ),
        TrainingStep(
          instruction: 'Release all keys to complete the combo',
          hint: 'The combo action should have been triggered',
          validator: (snapshot) =>
              snapshot.heldKeys.isEmpty && snapshot.pendingDecisions.isEmpty,
        ),
      ],
    ),
  ];
}
