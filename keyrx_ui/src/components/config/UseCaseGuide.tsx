import React, { useState } from 'react';
import {
  ArrowLeftRight,
  ArrowRight,
  ChevronDown,
  Code2,
  Keyboard,
  SlidersHorizontal,
  Sparkles,
} from 'lucide-react';
import { t, type MessageKey } from '@/i18n';

interface UseCaseGuideProps {
  hasDevice: boolean;
  onStartSwap: () => void;
  onStartSimple: () => void;
  onStartCommandPad: () => void;
  onStartAdvanced: () => void;
}

type PathId = 'swap' | 'simple' | 'command-pad' | 'advanced';

const paths: ReadonlyArray<{
  id: PathId;
  icon: typeof Keyboard;
  /** i18n prefix: `${prefix}.eyebrow|title|description|outcome` */
  prefix: 'guide.swap' | 'guide.simple' | 'guide.pad' | 'guide.advanced';
  accent: 'emerald' | 'amber' | 'violet' | 'sky';
}> = [
  { id: 'swap', icon: ArrowLeftRight, prefix: 'guide.swap', accent: 'sky' },
  { id: 'simple', icon: Keyboard, prefix: 'guide.simple', accent: 'emerald' },
  { id: 'command-pad', icon: Sparkles, prefix: 'guide.pad', accent: 'amber' },
  { id: 'advanced', icon: Code2, prefix: 'guide.advanced', accent: 'violet' },
];

const msg = (prefix: string, field: string) =>
  t(`${prefix}.${field}` as MessageKey);

const accentClasses = {
  sky: 'bg-sky-400/10 text-sky-300 ring-sky-400/20',
  emerald: 'bg-emerald-400/10 text-emerald-300 ring-emerald-400/20',
  amber: 'bg-amber-400/10 text-amber-300 ring-amber-400/20',
  violet: 'bg-violet-400/10 text-violet-300 ring-violet-400/20',
};

/** Goal-led entry points that keep KeyRx approachable without hiding power tools. */
export const UseCaseGuide: React.FC<UseCaseGuideProps> = ({
  hasDevice,
  onStartSwap,
  onStartSimple,
  onStartCommandPad,
  onStartAdvanced,
}) => {
  const [expanded, setExpanded] = useState(true);

  const handlePath = (id: PathId) => {
    if (id === 'swap') onStartSwap();
    if (id === 'simple') onStartSimple();
    if (id === 'command-pad') onStartCommandPad();
    if (id === 'advanced') onStartAdvanced();
  };

  return (
    <section
      className="overflow-hidden rounded-xl border border-slate-700/80 bg-slate-800/70 shadow-lg shadow-black/10"
      aria-labelledby="use-case-guide-title"
    >
      <button
        type="button"
        onClick={() => setExpanded((value) => !value)}
        className="flex w-full items-center justify-between gap-4 px-4 py-3 text-left hover:bg-slate-700/30 focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-400 md:px-5"
        aria-expanded={expanded}
      >
        <div className="flex min-w-0 items-center gap-3">
          <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-primary-500/15 text-primary-300 ring-1 ring-primary-400/20">
            <SlidersHorizontal className="h-4 w-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <h2
              id="use-case-guide-title"
              className="font-semibold text-slate-100"
            >
              {t('guide.title')}
            </h2>
            <p className="truncate text-xs text-slate-400">
              {t('guide.subtitle')}
            </p>
          </div>
        </div>
        <ChevronDown
          className={`h-5 w-5 shrink-0 text-slate-400 transition-transform ${expanded ? 'rotate-180' : ''}`}
          aria-hidden="true"
        />
      </button>

      {expanded && (
        <div className="grid gap-3 border-t border-slate-700/70 p-3 md:grid-cols-2 lg:grid-cols-4 md:p-4">
          {paths.map((path) => {
            const Icon = path.icon;
            const unavailable = path.id === 'command-pad' && !hasDevice;
            return (
              <button
                key={path.id}
                type="button"
                onClick={() => handlePath(path.id)}
                disabled={unavailable}
                className="group flex min-h-40 flex-col rounded-lg border border-slate-700 bg-slate-900/55 p-4 text-left transition hover:-translate-y-0.5 hover:border-slate-500 hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-55 disabled:hover:translate-y-0 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400"
              >
                <div className="mb-4 flex items-start justify-between gap-3">
                  <span
                    className={`flex h-9 w-9 items-center justify-center rounded-lg ring-1 ${accentClasses[path.accent]}`}
                  >
                    <Icon className="h-4 w-4" aria-hidden="true" />
                  </span>
                  <span className="text-[11px] font-semibold uppercase tracking-[0.14em] text-slate-400">
                    {msg(path.prefix, 'eyebrow')}
                  </span>
                </div>
                <span className="font-semibold text-slate-100">
                  {msg(path.prefix, 'title')}
                </span>
                <span className="mt-1 text-sm leading-5 text-slate-400">
                  {msg(path.prefix, 'description')}
                </span>
                <span className="mt-auto flex items-center justify-between gap-2 pt-4 text-xs text-slate-400">
                  {unavailable
                    ? t('guide.connectFirst')
                    : msg(path.prefix, 'outcome')}
                  {!unavailable && (
                    <ArrowRight
                      className="h-4 w-4 text-slate-400 transition-transform group-hover:translate-x-1"
                      aria-hidden="true"
                    />
                  )}
                </span>
              </button>
            );
          })}
        </div>
      )}
    </section>
  );
};
