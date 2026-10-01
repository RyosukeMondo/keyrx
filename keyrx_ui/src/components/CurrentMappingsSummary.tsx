import React from 'react';
import { Pencil, Trash2, ChevronDown, ChevronRight } from 'lucide-react';
import { cn } from '@/utils/cn';
import type { KeyMapping } from '@/types';
import { friendlyKeyName } from '@/utils/keyNames';
import { KeyName } from '@/components/KeyName';
import { t, type MessageKey } from '@/i18n';

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
    label: 'type.simple' as MessageKey,
  },
  tap_hold: {
    bg: 'bg-red-500/20',
    border: 'border-red-500/50',
    text: 'text-red-400',
    badge: 'bg-red-500',
    label: 'type.tapHold' as MessageKey,
  },
  macro: {
    bg: 'bg-yellow-500/20',
    border: 'border-yellow-500/50',
    text: 'text-yellow-400',
    badge: 'bg-yellow-500',
    label: 'cm.macro' as MessageKey,
  },
  layer_switch: {
    bg: 'bg-purple-500/20',
    border: 'border-purple-500/50',
    text: 'text-purple-400',
    badge: 'bg-purple-500',
    label: 'cm.layer' as MessageKey,
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
    const grouped: Record<
      string,
      Array<{ keyCode: string; mapping: KeyMapping }>
    > = {
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
        return mapping.tapAction
          ? friendlyKeyName(mapping.tapAction)
          : t('cm.notSet');
      case 'tap_hold':
        return t('cm.tapHold', {
          tap: mapping.tapAction ? friendlyKeyName(mapping.tapAction) : '?',
          ms: mapping.threshold || 200,
          hold: mapping.holdAction ? friendlyKeyName(mapping.holdAction) : '?',
        });
      case 'macro':
        return mapping.macroSteps?.length
          ? t('cm.steps', { count: mapping.macroSteps.length })
          : t('cm.empty2');
      case 'layer_switch':
        return mapping.targetLayer || t('cm.notSet');
      default:
        return t('cm.unknown');
    }
  };

  if (totalMappings === 0) {
    return (
      <div className="p-4 bg-slate-800/30 border border-slate-700/50 rounded-lg">
        <p className="text-sm text-slate-400 text-center">{t('cm.empty')}</p>
      </div>
    );
  }

  return (
    <div className="bg-slate-800/30 border border-slate-700/50 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 border-b border-slate-700/50 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-slate-200">
          {t('cm.title')}
          <span className="ml-2 text-xs text-slate-400 font-normal">
            {t('cm.count', { count: totalMappings })}
          </span>
        </h3>
      </div>

      {/* Mapping Groups */}
      <div className="divide-y divide-slate-700/50">
        {(
          Object.keys(MAPPING_TYPE_STYLES) as Array<
            keyof typeof MAPPING_TYPE_STYLES
          >
        ).map((type) => {
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
                    {t(style.label)}
                  </span>
                  <span className="text-xs text-slate-400">({count})</span>
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
                        <KeyName
                          code={keyCode}
                          className="text-sm font-semibold text-slate-200 shrink-0"
                        />
                        <span className="text-slate-400">→</span>
                        <span className="text-sm text-slate-300 truncate">
                          {formatMappingDescription(mapping)}
                        </span>
                      </div>

                      <div className="flex items-center gap-1 shrink-0 ml-2">
                        <button
                          onClick={() => onEditMapping(keyCode)}
                          className="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-600/50 rounded transition-colors"
                          title={t('cm.edit')}
                          aria-label={t('cm.editFor', {
                            key: friendlyKeyName(keyCode),
                          })}
                        >
                          <Pencil className="w-3.5 h-3.5" />
                        </button>
                        <button
                          onClick={() => onClearMapping(keyCode)}
                          className="p-1.5 text-slate-400 hover:text-red-400 hover:bg-red-500/20 rounded transition-colors"
                          title={t('cm.remove')}
                          aria-label={t('cm.removeFor', {
                            key: friendlyKeyName(keyCode),
                          })}
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
