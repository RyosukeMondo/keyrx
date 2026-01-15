import React from 'react';
import { Pencil, Trash2, ChevronDown, ChevronRight } from 'lucide-react';
import { cn } from '@/utils/cn';
import type { KeyMapping } from '@/types';

interface CurrentMappingsSummaryProps {
  keyMappings: Map<string, KeyMapping>;
  onEditMapping: (keyCode: string) => void;
  onClearMapping: (keyCode: string) => void;
}

const MAPPING_TYPE_STYLES = {
  simple: {
    bg: 'bg-green-500/20',
    border: 'border-green-500/50',
    text: 'text-green-400',
    badge: 'bg-green-500',
    label: 'Simple',
  },
  tap_hold: {
    bg: 'bg-red-500/20',
    border: 'border-red-500/50',
    text: 'text-red-400',
    badge: 'bg-red-500',
    label: 'Tap/Hold',
  },
  macro: {
    bg: 'bg-yellow-500/20',
    border: 'border-yellow-500/50',
    text: 'text-yellow-400',
    badge: 'bg-yellow-500',
    label: 'Macro',
  },
  layer_switch: {
    bg: 'bg-purple-500/20',
    border: 'border-purple-500/50',
    text: 'text-purple-400',
    badge: 'bg-purple-500',
    label: 'Layer',
  },
} as const;

/**
 * Displays a summary of current key mappings with edit/delete actions.
 * Groups mappings by type and shows physical key -> action mappings.
 */
export function CurrentMappingsSummary({
  keyMappings,
  onEditMapping,
  onClearMapping,
}: CurrentMappingsSummaryProps) {
  const [expandedTypes, setExpandedTypes] = React.useState<Set<string>>(
    new Set(['simple', 'tap_hold', 'macro', 'layer_switch'])
  );

  // Group mappings by type
  const mappingsByType = React.useMemo(() => {
    const grouped: Record<string, Array<{ keyCode: string; mapping: KeyMapping }>> = {
      simple: [],
      tap_hold: [],
      macro: [],
      layer_switch: [],
    };

    keyMappings.forEach((mapping, keyCode) => {
      if (grouped[mapping.type]) {
        grouped[mapping.type].push({ keyCode, mapping });
      }
    });

    return grouped;
  }, [keyMappings]);

  const totalMappings = keyMappings.size;

  const toggleExpanded = (type: string) => {
    setExpandedTypes((prev) => {
      const next = new Set(prev);
      if (next.has(type)) {
        next.delete(type);
      } else {
        next.add(type);
      }
      return next;
    });
  };

  const formatMappingDescription = (mapping: KeyMapping): string => {
    switch (mapping.type) {
      case 'simple':
        return mapping.tapAction || '(not set)';
      case 'tap_hold':
        return `Tap: ${mapping.tapAction || '?'}, Hold (${mapping.threshold || 200}ms): ${mapping.holdAction || '?'}`;
      case 'macro':
        return mapping.macroSteps?.length
          ? `${mapping.macroSteps.length} step(s)`
          : '(empty)';
      case 'layer_switch':
        return mapping.targetLayer || '(not set)';
      default:
        return 'Unknown';
    }
  };

  if (totalMappings === 0) {
    return (
      <div className="p-4 bg-slate-800/30 border border-slate-700/50 rounded-lg">
        <p className="text-sm text-slate-500 text-center">
          No key mappings configured. Click a key on the keyboard to add a mapping.
        </p>
      </div>
    );
  }

  return (
    <div className="bg-slate-800/30 border border-slate-700/50 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 border-b border-slate-700/50 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-slate-200">
          Current Mappings
          <span className="ml-2 text-xs text-slate-400 font-normal">
            ({totalMappings} {totalMappings === 1 ? 'mapping' : 'mappings'})
          </span>
        </h3>
      </div>

      {/* Mapping Groups */}
      <div className="divide-y divide-slate-700/50">
        {(Object.keys(MAPPING_TYPE_STYLES) as Array<keyof typeof MAPPING_TYPE_STYLES>).map((type) => {
          const style = MAPPING_TYPE_STYLES[type];
          const mappings = mappingsByType[type];
          const isExpanded = expandedTypes.has(type);
          const count = mappings.length;

          if (count === 0) return null;

          return (
            <div key={type}>
              {/* Type Header */}
              <button
                onClick={() => toggleExpanded(type)}
                className="w-full px-4 py-2 flex items-center justify-between hover:bg-slate-700/30 transition-colors"
              >
                <div className="flex items-center gap-2">
                  {isExpanded ? (
                    <ChevronDown className="w-4 h-4 text-slate-400" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-slate-400" />
                  )}
                  <div className={cn('w-3 h-3 rounded', style.badge)} />
                  <span className={cn('text-sm font-medium', style.text)}>
                    {style.label}
                  </span>
                  <span className="text-xs text-slate-500">({count})</span>
                </div>
              </button>

              {/* Mapping List */}
              {isExpanded && (
                <div className="px-4 pb-2 space-y-1">
                  {mappings.map(({ keyCode, mapping }) => (
                    <div
                      key={keyCode}
                      className={cn(
                        'flex items-center justify-between px-3 py-2 rounded-md border',
                        style.bg,
                        style.border
                      )}
                    >
                      <div className="flex items-center gap-3 min-w-0 flex-1">
                        <span className="font-mono text-sm font-semibold text-slate-200 shrink-0">
                          {keyCode}
                        </span>
                        <span className="text-slate-500">→</span>
                        <span className="text-sm text-slate-300 truncate">
                          {formatMappingDescription(mapping)}
                        </span>
                      </div>

                      <div className="flex items-center gap-1 shrink-0 ml-2">
                        <button
                          onClick={() => onEditMapping(keyCode)}
                          className="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-600/50 rounded transition-colors"
                          title="Edit mapping"
                          aria-label={`Edit mapping for ${keyCode}`}
                        >
                          <Pencil className="w-3.5 h-3.5" />
                        </button>
                        <button
                          onClick={() => onClearMapping(keyCode)}
                          className="p-1.5 text-slate-400 hover:text-red-400 hover:bg-red-500/20 rounded transition-colors"
                          title="Remove mapping"
                          aria-label={`Remove mapping for ${keyCode}`}
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
