// KeyRx keyboard remapping training screen.
//
// Provides step-by-step lessons for learning keyboard remapping concepts
// including layers, modifiers, tap-hold, and combos.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../config/config.dart';
import '../services/engine_service.dart';
import '../services/facade/keyrx_facade.dart';
import 'training_lessons.dart';

/// Keyboard remapping training screen with lesson framework.
class KeyrxTrainingScreen extends StatefulWidget {
  const KeyrxTrainingScreen({super.key});

  @override
  State<KeyrxTrainingScreen> createState() => _KeyrxTrainingScreenState();
}

class _KeyrxTrainingScreenState extends State<KeyrxTrainingScreen>
    with SingleTickerProviderStateMixin {

  EngineService? _engine;
  StreamSubscription<EngineSnapshot>? _stateSubscription;

  int _currentLessonIndex = 0;
  int _currentStepIndex = 0;
  bool _showHint = false;
  EngineSnapshot? _latestSnapshot;

  Map<String, int> _completedSteps = {};
  bool _progressLoaded = false;
  late final List<TrainingLesson> _lessons;

  // Exercise feedback state
  ExerciseResult? _lastExerciseResult;
  late AnimationController _feedbackAnimController;
  late Animation<double> _feedbackScaleAnim;
  bool _certificateShown = false;

  @override
  void initState() {
    super.initState();
    _lessons = buildTrainingLessons();
    _loadProgress();
    final facade = Provider.of<KeyrxFacade>(context, listen: false);
    _engine = facade.services.engineService;
    _subscribeToStateStream();
    _feedbackAnimController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 400),
    );
    _feedbackScaleAnim = CurvedAnimation(
      parent: _feedbackAnimController,
      curve: Curves.elasticOut,
    );
  }

  Future<void> _loadProgress() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = prefs.getStringList(StorageKeys.trainingProgressKey) ?? [];
    final progress = <String, int>{};
    for (final entry in stored) {
      final parts = entry.split(':');
      if (parts.length == 2) {
        progress[parts[0]] = int.tryParse(parts[1]) ?? 0;
      }
    }
    setState(() {
      _completedSteps = progress;
      _progressLoaded = true;
      _restorePositionFromProgress();
    });
  }

  void _restorePositionFromProgress() {
    for (var i = 0; i < _lessons.length; i++) {
      final lesson = _lessons[i];
      final completed = _completedSteps[lesson.id] ?? 0;
      if (completed < lesson.steps.length) {
        _currentLessonIndex = i;
        _currentStepIndex = completed;
        return;
      }
    }
    _currentLessonIndex = 0;
    _currentStepIndex = 0;
  }

  Future<void> _saveProgress() async {
    final prefs = await SharedPreferences.getInstance();
    final entries =
        _completedSteps.entries.map((e) => '${e.key}:${e.value}').toList();
    await prefs.setStringList(StorageKeys.trainingProgressKey, entries);
  }

  void _subscribeToStateStream() {
    _stateSubscription?.cancel();
    _stateSubscription = _engine?.stateStream.listen((snapshot) {
      if (!mounted) return;
      setState(() => _latestSnapshot = snapshot);
      _validateExercise(snapshot);
      _checkStepCompletion(snapshot);
    });
  }

  void _validateExercise(EngineSnapshot snapshot) {
    if (_currentLessonIndex >= _lessons.length) return;
    final lesson = _lessons[_currentLessonIndex];
    if (_currentStepIndex >= lesson.steps.length) return;

    final step = lesson.steps[_currentStepIndex];
    if (step.exercise == null) return;

    final result = step.exercise!.validator(snapshot);
    final previousSuccess = _lastExerciseResult?.success ?? false;

    setState(() => _lastExerciseResult = result);

    // Animate on state change
    if (result.success != previousSuccess) {
      _feedbackAnimController.forward(from: 0);
    }
  }

  void _checkStepCompletion(EngineSnapshot snapshot) {
    if (_currentLessonIndex >= _lessons.length) return;
    final lesson = _lessons[_currentLessonIndex];
    if (_currentStepIndex >= lesson.steps.length) return;
    if (lesson.steps[_currentStepIndex].validator(snapshot)) _advanceStep();
  }

  void _advanceStep() {
    final lesson = _lessons[_currentLessonIndex];
    setState(() {
      _showHint = false;
      _lastExerciseResult = null;
      if (_currentStepIndex < lesson.steps.length - 1) {
        _currentStepIndex++;
      } else {
        _completedSteps[lesson.id] = lesson.steps.length;
        _saveProgress();
        if (_currentLessonIndex < _lessons.length - 1) {
          _currentLessonIndex++;
          _currentStepIndex = 0;
        } else {
          // All lessons complete - show certificate
          _showCertificateDialog();
        }
      }
    });
  }

  void _showCertificateDialog() {
    if (_certificateShown) return;
    final allComplete = _lessons.every(
      (l) => (_completedSteps[l.id] ?? 0) >= l.steps.length,
    );
    if (!allComplete) return;

    _certificateShown = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      showDialog(
        context: context,
        barrierDismissible: false,
        builder: (ctx) => CertificateDialog(
          lessonCount: _lessons.length,
          onDismiss: () => Navigator.pop(ctx),
        ),
      );
    });
  }

  void _selectLesson(int index) {
    setState(() {
      _currentLessonIndex = index;
      final lesson = _lessons[index];
      final completed = _completedSteps[lesson.id] ?? 0;
      _currentStepIndex = completed < lesson.steps.length ? completed : 0;
      _showHint = false;
      _lastExerciseResult = null;
    });
  }

  @override
  void dispose() {
    _stateSubscription?.cancel();
    _feedbackAnimController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!_progressLoaded) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }
    return Scaffold(
      appBar: AppBar(
        title: const Text('KeyRx Training'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Reset Progress',
            onPressed: _confirmResetProgress,
          ),
        ],
      ),
      body: Column(
        children: [
          SizedBox(height: 120, child: _buildLessonCarousel()),
          const Divider(height: 1),
          Expanded(child: _buildStepContent()),
          if (_latestSnapshot != null) _buildStatePreview(),
        ],
      ),
    );
  }

  Widget _buildLessonCarousel() {
    return ListView.builder(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 12),
      itemCount: _lessons.length,
      itemBuilder: (context, index) {
        final lesson = _lessons[index];
        final isSelected = index == _currentLessonIndex;
        final completed = _completedSteps[lesson.id] ?? 0;
        final isComplete = completed >= lesson.steps.length;

        return Padding(
          padding: const EdgeInsets.symmetric(horizontal: 4),
          child: Material(
            elevation: isSelected ? 4 : 1,
            borderRadius: BorderRadius.circular(12),
            color: isSelected
                ? Theme.of(context).colorScheme.primaryContainer
                : isComplete
                    ? Colors.green.withValues(alpha: 0.1)
                    : Theme.of(context).cardColor,
            child: InkWell(
              onTap: () => _selectLesson(index),
              borderRadius: BorderRadius.circular(12),
              child: Container(
                width: 140,
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(lesson.icon, size: 20, color: isComplete ? Colors.green : null),
                        const Spacer(),
                        if (isComplete)
                          const Icon(Icons.check_circle, size: 16, color: Colors.green)
                        else
                          Text('$completed/${lesson.steps.length}',
                              style: Theme.of(context).textTheme.bodySmall),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Text(
                      lesson.title,
                      style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                          ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 4),
                    Expanded(
                      child: Text(lesson.description,
                          style: Theme.of(context).textTheme.bodySmall,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildStepContent() {
    final lesson = _lessons[_currentLessonIndex];
    final isLessonComplete = (_completedSteps[lesson.id] ?? 0) >= lesson.steps.length;
    if (isLessonComplete) return _buildLessonCompleteContent(lesson);

    final step = lesson.steps[_currentStepIndex];
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          LinearProgressIndicator(
            value: (_currentStepIndex + 1) / lesson.steps.length,
            backgroundColor: Colors.grey.withValues(alpha: 0.2),
          ),
          const SizedBox(height: 8),
          Text('Step ${_currentStepIndex + 1} of ${lesson.steps.length}',
              style: Theme.of(context).textTheme.bodySmall),
          const SizedBox(height: 24),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(20),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.assignment, color: Theme.of(context).colorScheme.primary),
                      const SizedBox(width: 12),
                      Text('Instruction', style: Theme.of(context).textTheme.titleMedium),
                    ],
                  ),
                  const Divider(),
                  const SizedBox(height: 8),
                  Text(step.instruction, style: Theme.of(context).textTheme.bodyLarge),
                  if (step.expectedOutput != null) ...[
                    const SizedBox(height: 12),
                    Row(
                      children: [
                        const Icon(Icons.output, size: 16, color: Colors.grey),
                        const SizedBox(width: 8),
                        Text('Expected: ${step.expectedOutput}',
                            style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.grey)),
                      ],
                    ),
                  ],
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          if (step.exercise != null) _buildExerciseFeedback(step.exercise!),
          const Spacer(),
          if (step.hint != null) _buildHintSection(step.hint!),
        ],
      ),
    );
  }

  Widget _buildExerciseFeedback(TrainingExercise exercise) {
    final result = _lastExerciseResult;
    final ok = result?.success ?? false;
    final color = result == null ? Colors.grey : ok ? Colors.green : Colors.red;
    final icon = result == null ? Icons.play_circle_outline : ok ? Icons.check_circle : Icons.cancel;
    final msg = ok ? exercise.successMessage : result?.message ?? exercise.failureMessage ?? 'Try again';

    return Semantics(
      label: ok ? 'Passed: ${exercise.successMessage}' : 'Exercise: ${exercise.prompt}',
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 300),
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: color.withValues(alpha: 0.5), width: 2),
        ),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            ScaleTransition(
              scale: _feedbackScaleAnim,
              child: Icon(icon, color: color, size: 28, semanticLabel: ok ? 'Success' : 'Pending'),
            ),
            const SizedBox(width: 12),
            Expanded(child: Text(exercise.prompt, style: Theme.of(context).textTheme.titleSmall?.copyWith(fontWeight: FontWeight.w600))),
          ]),
          if (result != null) ...[
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(color: color.withValues(alpha: 0.15), borderRadius: BorderRadius.circular(8)),
              child: Row(children: [
                Icon(ok ? Icons.thumb_up : Icons.info_outline, size: 16, color: color),
                const SizedBox(width: 8),
                Expanded(child: Text(msg, style: TextStyle(color: ok ? Colors.green.shade700 : Colors.red.shade700, fontWeight: FontWeight.w500))),
              ]),
            ),
          ],
        ]),
      ),
    );
  }

  Widget _buildHintSection(String hint) => _showHint
      ? Card(
          color: Colors.amber.withValues(alpha: 0.1),
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Row(children: [
              const Icon(Icons.lightbulb, color: Colors.amber),
              const SizedBox(width: 12),
              Expanded(child: Text(hint, style: Theme.of(context).textTheme.bodyMedium)),
            ]),
          ),
        )
      : Center(child: TextButton.icon(icon: const Icon(Icons.lightbulb_outline), label: const Text('Show Hint'), onPressed: () => setState(() => _showHint = true)));

  Widget _buildLessonCompleteContent(TrainingLesson lesson) {
    final allComplete = _lessons.every((l) => (_completedSteps[l.id] ?? 0) >= l.steps.length);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(allComplete ? Icons.emoji_events : Icons.check_circle,
                size: 64, color: allComplete ? Colors.amber : Colors.green),
            const SizedBox(height: 16),
            Text(allComplete ? 'Congratulations!' : '${lesson.title} Complete!',
                style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 8),
            Text(
              allComplete
                  ? 'You have mastered all KeyRx training lessons!'
                  : 'You have completed all steps in this lesson.',
              style: Theme.of(context).textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            if (!allComplete)
              FilledButton.icon(
                icon: const Icon(Icons.arrow_forward),
                label: const Text('Next Lesson'),
                onPressed: () {
                  if (_currentLessonIndex < _lessons.length - 1) _selectLesson(_currentLessonIndex + 1);
                },
              )
            else
              OutlinedButton.icon(
                icon: const Icon(Icons.replay),
                label: const Text('Practice Again'),
                onPressed: () => _selectLesson(0),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatePreview() {
    final s = _latestSnapshot!;
    final chips = <Widget>[];
    void add(String l, String v) => chips.addAll([_chip(l, v), const SizedBox(width: 8)]);
    if (s.lastEvent != null) add('Event', s.lastEvent!);
    if (s.activeLayers.isNotEmpty) add('Layers', s.activeLayers.join(', '));
    if (s.activeModifiers.isNotEmpty) add('Mods', s.activeModifiers.join(', '));
    if (s.heldKeys.isNotEmpty) add('Held', s.heldKeys.join(', '));
    if (s.pendingDecisions.isNotEmpty) chips.add(_chip('Pending', s.pendingDecisions.join(', ')));
    return Container(
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(children: [
        const Icon(Icons.visibility, size: 16),
        const SizedBox(width: 8),
        Expanded(child: SingleChildScrollView(scrollDirection: Axis.horizontal, child: Row(children: chips))),
      ]),
    );
  }

  Widget _chip(String l, String v) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer, borderRadius: BorderRadius.circular(12)),
        child: Text('$l: $v', style: Theme.of(context).textTheme.bodySmall?.copyWith(fontWeight: FontWeight.w500)),
      );

  void _confirmResetProgress() => showDialog(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Reset Progress?'),
          content: const Text('This will reset all lesson progress. Continue?'),
          actions: [
            TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Cancel')),
            FilledButton(onPressed: () { Navigator.pop(ctx); _resetProgress(); }, child: const Text('Reset')),
          ],
        ),
      );

  Future<void> _resetProgress() async {
    (await SharedPreferences.getInstance()).remove(StorageKeys.trainingProgressKey);
    setState(() {
      _completedSteps = {};
      _currentLessonIndex = 0;
      _currentStepIndex = 0;
      _showHint = false;
      _lastExerciseResult = null;
      _certificateShown = false;
    });
  }
}
