import React, { useMemo, useState } from 'react';
import { ArrowLeftRight } from 'lucide-react';
import type { SVGKeyData } from '@/utils/kle-parser';
import { friendlyKeyName, normalizeKeyCode } from '@/utils/keyNames';
import { t } from '@/i18n';

interface SwapKeysPanelProps {
  layoutKeys: SVGKeyData[];
  /** Called with the two normalized (VK_*) key codes to exchange. */
  onSwap: (a: string, b: string) => void;
  onCancel: () => void;
}

/** The classic one-click fix: Caps Lock <-> Ctrl. */
const PRESET: [string, string] = ['VK_CapsLock', 'VK_LCtrl'];

/**
 * "Swap two keys": each key will act like the other. The common case
 * (Caps Lock <-> Ctrl) is one click; any other pair is two dropdowns.
 */
export const SwapKeysPanel: React.FC<SwapKeysPanelProps> = ({
  layoutKeys,
  onSwap,
  onCancel,
}) => {
  const [first, setFirst] = useState('');
  const [second, setSecond] = useState('');

  const options = useMemo(() => {
    const seen = new Set<string>();
    return layoutKeys
      .map((k) => normalizeKeyCode(k.code))
      .filter((code) => (seen.has(code) ? false : (seen.add(code), true)))
      .map((code) => ({ code, label: friendlyKeyName(code) }));
  }, [layoutKeys]);

  const canSwap = first !== '' && second !== '' && first !== second;

  const select = (
    id: string,
    label: string,
    value: string,
    onChange: (v: string) => void
  ) => (
    <div className="flex min-w-0 flex-col gap-1">
      <label htmlFor={id} className="text-xs font-medium text-slate-300">
        {label}
      </label>
      <select
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-slate-100"
      >
        <option value="">—</option>
        {options.map((o) => (
          <option key={o.code} value={o.code}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );

  return (
    <section
      aria-labelledby="swap-keys-title"
      className="rounded-xl border border-sky-400/30 bg-slate-800/80 p-4 md:p-5"
    >
      <div className="mb-3 flex items-center gap-2">
        <ArrowLeftRight className="h-5 w-5 text-sky-300" aria-hidden="true" />
        <h2 id="swap-keys-title" className="font-semibold text-slate-100">
          {t('swap.title')}
        </h2>
      </div>
      <p className="mb-4 text-sm text-slate-300">{t('swap.hint')}</p>

      <div className="mb-4">
        <button
          type="button"
          onClick={() => onSwap(...PRESET)}
          className="rounded-md bg-sky-600 px-4 py-2 text-sm font-semibold text-white hover:bg-sky-700"
        >
          {t('swap.preset')}
        </button>
      </div>

      <div className="grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
        {select('swap-first', t('swap.first'), first, setFirst)}
        {select('swap-second', t('swap.second'), second, setSecond)}
        <div className="flex gap-2">
          <button
            type="button"
            disabled={!canSwap}
            onClick={() => onSwap(first, second)}
            className="rounded-md bg-primary-500 px-4 py-2 text-sm font-semibold text-white hover:bg-primary-600 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t('swap.apply')}
          </button>
          <button
            type="button"
            onClick={onCancel}
            className="rounded-md bg-slate-700 px-4 py-2 text-sm font-medium text-slate-100 hover:bg-slate-600"
          >
            {t('swap.cancel')}
          </button>
        </div>
      </div>
    </section>
  );
};
