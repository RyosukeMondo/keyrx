import React from 'react';
import { History, Redo2, Undo2 } from 'lucide-react';
import { t } from '@/i18n';

interface HistoryControlsProps {
  canUndo: boolean;
  canRedo: boolean;
  canRevert: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onRevert: () => void;
}

const BUTTON =
  'rounded-lg border border-slate-700 bg-slate-800 p-2 text-slate-300 hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40';

/** Undo / redo of the working document and revert to the last saved version. */
export const HistoryControls: React.FC<HistoryControlsProps> = ({
  canUndo,
  canRedo,
  canRevert,
  onUndo,
  onRedo,
  onRevert,
}) => (
  <div className="flex items-center gap-1" role="group" aria-label="History">
    <button
      type="button"
      onClick={onUndo}
      disabled={!canUndo}
      aria-label={t('history.undo')}
      title={`${t('history.undo')} (Ctrl+Z)`}
      className={BUTTON}
    >
      <Undo2 className="h-4 w-4" aria-hidden="true" />
    </button>
    <button
      type="button"
      onClick={onRedo}
      disabled={!canRedo}
      aria-label={t('history.redo')}
      title={`${t('history.redo')} (Ctrl+Shift+Z)`}
      className={BUTTON}
    >
      <Redo2 className="h-4 w-4" aria-hidden="true" />
    </button>
    <button
      type="button"
      onClick={onRevert}
      disabled={!canRevert}
      aria-label={t('history.revert')}
      title={canRevert ? t('history.revert') : t('history.revertNone')}
      className={BUTTON}
    >
      <History className="h-4 w-4" aria-hidden="true" />
    </button>
  </div>
);
