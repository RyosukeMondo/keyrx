// Debugger widgets for state display and event timeline.
//
// Contains tag section, pending decisions card, timing card, event log, and timeline widgets.

import 'package:flutter/material.dart';

import '../services/engine_service.dart';
import 'debugger_meters.dart';

/// Builds a tag section card with chips and change tracking.
class TagSectionCard extends StatelessWidget {
  const TagSectionCard({
    super.key,
    required this.title,
    required this.items,
    this.previousItems,
    required this.pulseAnimation,
    required this.animationDuration,
  });

  final String title;
  final List<String> items;
  final Set<String>? previousItems;
  final Animation<double> pulseAnimation;
  final Duration animationDuration;

  @override
  Widget build(BuildContext context) {
    final previousSet = previousItems ?? <String>{};
    final currentSet = items.toSet();

    final chips = items.map((item) {
      final isNew = !previousSet.contains(item);

      return AnimatedContainer(
        duration: animationDuration,
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
        child: AnimatedScale(
          scale: isNew ? pulseAnimation.value : 1.0,
          duration: animationDuration,
          child: Chip(
            label: Text(item),
            visualDensity: VisualDensity.compact,
            backgroundColor: isNew
                ? Theme.of(context).colorScheme.primaryContainer
                : null,
            side: isNew
                ? BorderSide(
                    color: Theme.of(context).colorScheme.primary,
                    width: 2,
                  )
                : null,
          ),
        ),
      );
    }).toList();

    final removedCount = previousSet.difference(currentSet).length;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: Theme.of(context).textTheme.titleMedium,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const SizedBox(width: 8),
                AnimatedContainer(
                  duration: animationDuration,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: items.isEmpty
                        ? Colors.grey.withValues(alpha: 0.2)
                        : Theme.of(
                            context,
                          ).colorScheme.primary.withValues(alpha: 0.2),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${items.length}',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: items.isEmpty
                          ? Colors.grey
                          : Theme.of(context).colorScheme.primary,
                    ),
                  ),
                ),
              ],
            ),
            const Divider(),
            if (chips.isEmpty)
              AnimatedOpacity(
                opacity: 1.0,
                duration: animationDuration,
                child: Text(
                  'None',
                  style: TextStyle(
                    color: Theme.of(context).textTheme.bodySmall?.color,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              )
            else
              SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(children: chips),
              ),
            if (removedCount > 0)
              AnimatedOpacity(
                opacity: 0.6,
                duration: animationDuration,
                child: Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(
                    '$removedCount removed',
                    style: TextStyle(
                      fontSize: 11,
                      color: Colors.red.shade300,
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

/// Displays layout compositor state including priorities and shared modifiers.
class LayoutsCard extends StatelessWidget {
  const LayoutsCard({
    super.key,
    required this.layouts,
    required this.sharedModifiers,
    required this.animationDuration,
  });

  final List<LayoutState> layouts;
  final List<int> sharedModifiers;
  final Duration animationDuration;

  @override
  Widget build(BuildContext context) {
    final sorted = [...layouts]
      ..sort((a, b) => b.priority.compareTo(a.priority));

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    'Layouts',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                AnimatedContainer(
                  duration: animationDuration,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: sorted.isEmpty
                        ? Colors.grey.withValues(alpha: 0.2)
                        : Theme.of(
                            context,
                          ).colorScheme.primary.withValues(alpha: 0.16),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${sorted.length}',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: sorted.isEmpty
                          ? Colors.grey
                          : Theme.of(context).colorScheme.primary,
                    ),
                  ),
                ),
              ],
            ),
            const Divider(),
            if (sorted.isEmpty)
              Text(
                'No layouts reported',
                style: TextStyle(
                  color: Theme.of(context).textTheme.bodySmall?.color,
                  fontStyle: FontStyle.italic,
                ),
              )
            else
              Column(
                children: [
                  for (final layout in sorted) ...[
                    _LayoutRow(layout: layout),
                    const Divider(height: 12),
                  ],
                ],
              ),
            if (sharedModifiers.isNotEmpty) ...[
              const SizedBox(height: 8),
              Wrap(
                spacing: 6,
                runSpacing: 4,
                children: sharedModifiers
                    .map(
                      (id) => Chip(
                        label: Text('shared mod$id'),
                        visualDensity: VisualDensity.compact,
                        backgroundColor: Theme.of(context)
                            .colorScheme
                            .surfaceContainerHighest
                            .withValues(alpha: 0.5),
                      ),
                    )
                    .toList(),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _LayoutRow extends StatelessWidget {
  const _LayoutRow({required this.layout});

  final LayoutState layout;

  @override
  Widget build(BuildContext context) {
    final layerLabel = layout.activeLayers.isEmpty
        ? 'layers: —'
        : 'layers: ${layout.activeLayers.join(", ")}';
    final modifierChips = layout.modifiers
        .map(
          (id) =>
              Chip(label: Text('mod$id'), visualDensity: VisualDensity.compact),
        )
        .toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(
              layout.enabled ? Icons.check_circle : Icons.cancel,
              color: layout.enabled ? Colors.green : Colors.redAccent,
              size: 18,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    layout.name,
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  if (layout.tags.isNotEmpty)
                    Wrap(
                      spacing: 4,
                      runSpacing: 2,
                      children: layout.tags
                          .map(
                            (tag) => Chip(
                              label: Text(tag),
                              visualDensity: VisualDensity.compact,
                              backgroundColor: Theme.of(
                                context,
                              ).colorScheme.secondaryContainer,
                            ),
                          )
                          .toList(),
                    ),
                  if (layout.description?.isNotEmpty == true)
                    Text(
                      layout.description!,
                      style: Theme.of(context).textTheme.bodySmall,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                ],
              ),
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Chip(
                  label: Text('prio ${layout.priority}'),
                  visualDensity: VisualDensity.compact,
                  backgroundColor: Theme.of(
                    context,
                  ).colorScheme.primaryContainer.withValues(alpha: 0.6),
                  side: BorderSide.none,
                ),
                Text(layerLabel, style: Theme.of(context).textTheme.bodySmall),
              ],
            ),
          ],
        ),
        if (modifierChips.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Wrap(spacing: 6, runSpacing: 4, children: modifierChips),
          ),
      ],
    );
  }
}

/// Builds the pending decisions card with categorized decisions.
class PendingDecisionsCard extends StatelessWidget {
  const PendingDecisionsCard({
    super.key,
    required this.pending,
    this.timing,
    required this.pulseAnimation,
    required this.animationDuration,
  });

  final List<String> pending;
  final EngineTiming? timing;
  final Animation<double> pulseAnimation;
  final Duration animationDuration;

  @override
  Widget build(BuildContext context) {
    final tapHoldDecisions = <String>[];
    final comboDecisions = <String>[];

    for (final decision in pending) {
      final lower = decision.toLowerCase();
      if (lower.contains('taphold') || lower.contains('tap-hold')) {
        tapHoldDecisions.add(decision);
      } else if (lower.contains('combo')) {
        comboDecisions.add(decision);
      } else {
        // Fallback: categorize by content heuristics
        if (lower.contains('hold') || lower.contains('tap')) {
          tapHoldDecisions.add(decision);
        } else {
          comboDecisions.add(decision);
        }
      }
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    'Pending Decisions',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                AnimatedContainer(
                  duration: animationDuration,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: pending.isEmpty
                        ? Colors.grey.withValues(alpha: 0.2)
                        : Colors.orange.withValues(alpha: 0.2),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${pending.length}',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: pending.isEmpty ? Colors.grey : Colors.orange,
                    ),
                  ),
                ),
              ],
            ),
            const Divider(),
            if (pending.isEmpty)
              Text(
                'None',
                style: TextStyle(
                  color: Theme.of(context).textTheme.bodySmall?.color,
                  fontStyle: FontStyle.italic,
                ),
              )
            else ...[
              // Tap-hold decisions with countdown
              for (final decision in tapHoldDecisions)
                PendingTapHoldWidget(
                  decision: decision,
                  timing: timing,
                  pulse: pulseAnimation,
                ),
              // Combo decisions with key highlights
              for (final decision in comboDecisions)
                PendingComboWidget(decision: decision, pulse: pulseAnimation),
            ],
          ],
        ),
      ),
    );
  }
}

/// Builds the timing configuration card.
class TimingCard extends StatelessWidget {
  const TimingCard({super.key, required this.timing});

  final EngineTiming timing;

  @override
  Widget build(BuildContext context) {
    final items = <String>[
      if (timing.tapTimeoutMs != null) 'Tap timeout: ${timing.tapTimeoutMs}ms',
      if (timing.comboTimeoutMs != null)
        'Combo timeout: ${timing.comboTimeoutMs}ms',
      if (timing.holdDelayMs != null) 'Hold delay: ${timing.holdDelayMs}ms',
      if (timing.eagerTap != null) 'Eager tap: ${timing.eagerTap}',
      if (timing.permissiveHold != null)
        'Permissive hold: ${timing.permissiveHold}',
      if (timing.retroTap != null) 'Retro tap: ${timing.retroTap}',
    ];

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Timing', style: Theme.of(context).textTheme.titleMedium),
            const Divider(),
            if (items.isEmpty)
              const Text('No timing settings reported')
            else
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: items
                    .map(
                      (item) => Padding(
                        padding: const EdgeInsets.symmetric(vertical: 2),
                        child: Text(item),
                      ),
                    )
                    .toList(),
              ),
          ],
        ),
      ),
    );
  }
}

/// Builds the event log list view.
class EventLogWidget extends StatelessWidget {
  const EventLogWidget({super.key, required this.events});

  final List<EngineSnapshot> events;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Event Log',
            style: Theme.of(context).textTheme.titleLarge,
          ),
        ),
        Expanded(
          child: events.isEmpty
              ? const Center(child: Text('Waiting for engine events...'))
              : ListView.builder(
                  itemCount: events.length,
                  itemBuilder: (context, index) {
                    final snap = events[index];
                    final ts = snap.timestamp.toLocal().toIso8601String();
                    return ListTile(
                      dense: true,
                      leading: Text('${index + 1}'),
                      title: Text(snap.lastEvent ?? 'Snapshot'),
                      subtitle: Text(ts),
                    );
                  },
                ),
        ),
      ],
    );
  }
}

/// Builds the latency timeline visualization.
class TimelineWidget extends StatelessWidget {
  const TimelineWidget({
    super.key,
    required this.events,
    required this.animationDuration,
  });

  final List<EngineSnapshot> events;
  final Duration animationDuration;

  @override
  Widget build(BuildContext context) {
    if (events.isEmpty) {
      return const SizedBox.shrink();
    }

    final latencies = events
        .where((e) => e.latencyUs != null)
        .map((e) => e.latencyUs!)
        .toList();
    final latestLatency = events.first.latencyUs;
    final avgLatency = latencies.isEmpty
        ? 0
        : latencies.reduce((a, b) => a + b) ~/ latencies.length;

    final bars = events.take(30).map((snap) {
      final value = snap.latencyUs?.toDouble() ?? 0;
      final widthNum = (value / (avgLatency == 0 ? 1 : avgLatency)).clamp(
        0.2,
        2,
      );
      final width = widthNum.toDouble();
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 2, horizontal: 12),
        child: Row(
          children: [
            AnimatedContainer(
              duration: animationDuration,
              height: 6,
              width: 120 * width,
              color: LatencyMeterCard.latencyColor(
                snap.latencyUs,
              ).withValues(alpha: 0.7),
            ),
            const SizedBox(width: 8),
            Text(snap.latencyUs != null ? '${snap.latencyUs}µs' : '—'),
          ],
        ),
      );
    }).toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              const Icon(Icons.timeline),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  'Latency Timeline (last ${bars.length})',
                  style: Theme.of(context).textTheme.titleMedium,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  'Avg: $avgLatency µs  Latest: ${latestLatency ?? 0}µs',
                  style: Theme.of(context).textTheme.bodySmall,
                  textAlign: TextAlign.right,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
        ...bars,
      ],
    );
  }
}
