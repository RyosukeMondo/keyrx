import React, { useMemo, useState } from 'react';
import { Languages } from 'lucide-react';
import type { SVGKeyData } from '@/utils/kle-parser';
import { friendlyKeyName, normalizeKeyCode } from '@/utils/keyNames';
import { t } from '@/i18n';

/**
 * IME on/off choices for Japanese input.
 *
 * Linux Japanese IMEs (fcitx5-mozc, ibus-mozc) do not agree on which key turns
 * the IME on or off, so the panel offers both common styles instead of
 * guessing, and says which evdev keys each one emits:
 *
 *  - `toggle`: one key acts as 半角/全角. keyrx `Zenkaku` is `KeyCode::Grave`,
 *    emitted as evdev KEY_GRAVE (41) -- the key the kernel reports for the JIS
 *    半角/全角 scancode 0x29. It toggles the IME under a `jp` xkb layout.
 *  - `pair`: two keys, 変換 = IME on and 無変換 = IME off. keyrx `Henkan` /
 *    `Muhenkan` are emitted as evdev KEY_HENKAN (92) / KEY_MUHENKAN (94).
 *    Whether the IME treats them as on/off or as conversion keys depends on
 *    its keymap, which is why this is a choice and not the default.
 */
export type ImeStyle = 'toggle' | 'pair';

/** Pairs of [physical key, key it should act as], DSL `VK_*` names. */
export type ImeAssignments = Array<[string, string]>;

interface ImeKeysPanelProps {
  layoutKeys: SVGKeyData[];
  onApply: (assignments: ImeAssignments) => void;
  onCancel: () => void;
}

/** Sensible defaults on a JIS/ANSI board: 英数 (Caps Lock) toggles; Alt keys pair. */
const DEFAULTS = {
  toggle: 'VK_CapsLock',
  on: 'VK_RAlt',
  off: 'VK_LAlt',
};

export const ImeKeysPanel: React.FC<ImeKeysPanelProps> = ({
  layoutKeys,
  onApply,
  onCancel,
}) => {
  const [style, setStyle] = useState<ImeStyle>('toggle');
  const [toggleKey, setToggleKey] = useState(DEFAULTS.toggle);
  const [onKey, setOnKey] = useState(DEFAULTS.on);
  const [offKey, setOffKey] = useState(DEFAULTS.off);

  const options = useMemo(() => {
    const seen = new Set<string>();
    return layoutKeys
      .map((k) => normalizeKeyCode(k.code))
      .filter((code) => (seen.has(code) ? false : (seen.add(code), true)))
      .map((code) => ({ code, label: friendlyKeyName(code) }));
  }, [layoutKeys]);

  const valid =
    style === 'toggle' ? !!toggleKey : !!onKey && !!offKey && onKey !== offKey;

  const apply = () => {
    if (!valid) return;
    onApply(
      style === 'toggle'
        ? [[toggleKey, 'VK_Zenkaku']]
        : [
            [onKey, 'VK_Henkan'],
            [offKey, 'VK_Muhenkan'],
          ]
    );
  };

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
        {options.map((o) => (
          <option key={o.code} value={o.code}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );

  const radio = (value: ImeStyle, title: string, hint: string) => (
    <label className="flex cursor-pointer items-start gap-3 rounded-lg border border-slate-600 p-3 has-[:checked]:border-sky-400 has-[:checked]:bg-sky-400/10">
      <input
        type="radio"
        name="ime-style"
        value={value}
        checked={style === value}
        onChange={() => setStyle(value)}
        className="mt-1"
      />
      <span>
        <span className="block text-sm font-medium text-slate-100">
          {title}
        </span>
        <span className="block text-xs text-slate-400">{hint}</span>
      </span>
    </label>
  );

  return (
    <section
      aria-labelledby="ime-keys-title"
      className="rounded-xl border border-sky-400/30 bg-slate-800/80 p-4 md:p-5"
    >
      <div className="mb-3 flex items-center gap-2">
        <Languages className="h-5 w-5 text-sky-300" aria-hidden="true" />
        <h2 id="ime-keys-title" className="font-semibold text-slate-100">
          {t('ime.title')}
        </h2>
      </div>
      <p className="mb-4 text-sm text-slate-300">{t('ime.hint')}</p>

      <div className="mb-4 grid gap-3 md:grid-cols-2">
        {radio('toggle', t('ime.toggle'), t('ime.toggleHint'))}
        {radio('pair', t('ime.pair'), t('ime.pairHint'))}
      </div>

      <div className="grid gap-3 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
        {style === 'toggle' ? (
          select('ime-toggle-key', t('ime.toggleKey'), toggleKey, setToggleKey)
        ) : (
          <>
            {select('ime-on-key', t('ime.onKey'), onKey, setOnKey)}
            {select('ime-off-key', t('ime.offKey'), offKey, setOffKey)}
          </>
        )}
        <div className="flex gap-2 sm:col-start-3">
          <button
            type="button"
            disabled={!valid}
            onClick={apply}
            className="rounded-md bg-primary-500 px-4 py-2 text-sm font-semibold text-white hover:bg-primary-600 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t('ime.apply')}
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
