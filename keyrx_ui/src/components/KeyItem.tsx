import React from 'react';
import { cn } from '@/utils/cn';
import type { AssignableKey } from './assignableKeys';

interface KeyItemProps {
  keyItem: AssignableKey;
  onClick?: () => void;
}

/**
 * Individual key item within the palette
 */
export const KeyItem: React.FC<KeyItemProps> = ({ keyItem, onClick }) => {
  const ariaLabel = `${keyItem.label} key. ${
    keyItem.description || ''
  }. Click to select.`;

  return (
    <button
      onClick={onClick}
      className={cn(
        'px-3 py-2 text-sm font-medium rounded border transition-all duration-150',
        'bg-slate-700 border-slate-600 text-slate-100',
        'hover:bg-slate-600 hover:border-slate-500',
        'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
        'min-h-[44px] min-w-[44px]' // Touch-friendly minimum size
      )}
      aria-label={ariaLabel}
      aria-describedby={keyItem.description ? `${keyItem.id}-desc` : undefined}
      title={keyItem.description}
      type="button"
    >
      {keyItem.label}
      {keyItem.description && (
        <span id={`${keyItem.id}-desc`} className="sr-only">
          {keyItem.description}
        </span>
      )}
    </button>
  );
};

KeyItem.displayName = 'KeyItem';
