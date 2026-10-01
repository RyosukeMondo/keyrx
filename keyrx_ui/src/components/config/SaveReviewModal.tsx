import React, { Suspense, lazy, useMemo, useState } from 'react';
import { Modal } from '@/components/Modal';
import { diffMappings, type MappingChange } from '@/utils/mappingDiff';
import { t } from '@/i18n';

// The Rhai diff pulls in Monaco; only load it when "Details" is opened.
const ProfileDiffView = lazy(() =>
  import('./ProfileDiffView').then((m) => ({ default: m.ProfileDiffView }))
);

interface SaveReviewModalProps {
  open: boolean;
  original: string;
  modified: string;
  onCancel: () => void;
  onConfirm: () => void;
}

const GLYPH: Record<MappingChange['kind'], string> = {
  added: '+',
  changed: '~',
  removed: '−',
};

/** One human sentence for a change: "Caps Lock → Ctrl". */
export function describeChange(c: MappingChange): string {
  let text: string;
  if (c.kind === 'removed') {
    text = t('save.removed', { from: c.from });
  } else if (c.tapHold) {
    text = t('save.tapHold', {
      from: c.from,
      tap: c.tapHold.tap,
      hold: c.tapHold.hold,
    });
  } else {
    text = t('save.added', { from: c.from, to: c.to });
  }
  if (c.layer) text = t('save.layer', { text, layer: c.layer });
  if (c.device) text = t('save.device', { text, device: c.device });
  return text;
}

/**
 * Safe-save review: what will change, in plain language ("Caps Lock → Ctrl"),
 * with the raw Rhai diff one click away under "Details".
 */
export const SaveReviewModal: React.FC<SaveReviewModalProps> = ({
  open,
  original,
  modified,
  onCancel,
  onConfirm,
}) => {
  const changes = useMemo(
    () => (open ? diffMappings(original, modified) : null),
    [open, original, modified]
  );
  const [showDetails, setShowDetails] = useState(false);
  // If the summary cannot be computed, the diff is the only honest view.
  const detailsVisible = showDetails || changes === null;

  return (
    <Modal open={open} onClose={onCancel} title={t('save.title')} size="lg">
      <p className="mb-3 text-sm text-slate-300">{t('save.intro')}</p>

      {changes && changes.length > 0 ? (
        <ul className="mb-4 space-y-1.5" data-testid="save-review-changes">
          {changes.map((c, i) => (
            <li
              key={`${c.kind}-${c.device ?? '*'}-${c.layer ?? 'base'}-${c.from}-${i}`}
              className="flex items-start gap-3 rounded-md bg-slate-800 px-3 py-2 text-sm text-slate-100"
            >
              <span
                aria-hidden="true"
                className={`w-4 text-center font-mono ${
                  c.kind === 'removed'
                    ? 'text-rose-300'
                    : c.kind === 'added'
                      ? 'text-emerald-300'
                      : 'text-amber-300'
                }`}
              >
                {GLYPH[c.kind]}
              </span>
              <span>{describeChange(c)}</span>
            </li>
          ))}
        </ul>
      ) : (
        changes && (
          <p className="mb-4 text-sm text-slate-400">{t('save.none')}</p>
        )
      )}

      {changes !== null && (
        <button
          type="button"
          onClick={() => setShowDetails((v) => !v)}
          aria-expanded={showDetails}
          className="mb-3 text-sm font-medium text-primary-300 underline-offset-2 hover:underline"
        >
          {showDetails ? t('save.hideDetails') : t('save.details')}
        </button>
      )}

      {detailsVisible && (
        <Suspense
          fallback={<div className="h-24 animate-pulse rounded bg-slate-800" />}
        >
          <ProfileDiffView original={original} modified={modified} />
        </Suspense>
      )}

      <div className="mt-4 flex justify-end gap-3">
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md bg-slate-700 px-4 py-2 text-sm font-medium text-slate-100 transition-colors hover:bg-slate-600"
        >
          {t('save.cancel')}
        </button>
        <button
          type="button"
          onClick={onConfirm}
          className="rounded-md bg-primary-500 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-600"
        >
          {t('save.confirm')}
        </button>
      </div>
    </Modal>
  );
};
