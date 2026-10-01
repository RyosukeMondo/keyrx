import React, { useRef, useState } from 'react';

interface IdGridItem {
  id: string;
  label: string;
  /** Lazy so the label follows the active locale. */
  ariaLabel: () => string;
}

interface IdGridProps {
  items: IdGridItem[];
  columns: number;
  onSelect: (id: string) => void;
  /** Accessible name of the whole grid. */
  label: string;
}

/**
 * A grid of buttons that is ONE Tab stop (roving tabindex): arrow keys move
 * within the grid, Home/End jump to the first/last item. Used for the 256-entry
 * modifier and lock pickers, which would otherwise add 256 Tab stops each.
 */
export const IdGrid: React.FC<IdGridProps> = ({
  items,
  columns,
  onSelect,
  label,
}) => {
  const [active, setActive] = useState(0);
  const ref = useRef<HTMLDivElement>(null);

  const focusIndex = (index: number) => {
    const next = Math.max(0, Math.min(items.length - 1, index));
    setActive(next);
    ref.current
      ?.querySelector<HTMLButtonElement>(`[data-index="${next}"]`)
      ?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent, index: number) => {
    const moves: Record<string, number> = {
      ArrowRight: index + 1,
      ArrowLeft: index - 1,
      ArrowDown: index + columns,
      ArrowUp: index - columns,
      Home: 0,
      End: items.length - 1,
    };
    if (e.key in moves) {
      e.preventDefault();
      focusIndex(moves[e.key]);
    }
  };

  return (
    <div
      ref={ref}
      role="group"
      aria-label={label}
      className="grid gap-2"
      style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }}
    >
      {items.map((item, index) => (
        <button
          key={item.id}
          type="button"
          data-index={index}
          tabIndex={index === active ? 0 : -1}
          onClick={() => onSelect(item.id)}
          onFocus={() => setActive(index)}
          onKeyDown={(e) => handleKeyDown(e, index)}
          className="px-2 py-1 bg-slate-700 hover:bg-primary-500 text-slate-300 hover:text-white rounded text-xs font-mono transition-colors"
          title={item.id}
          aria-label={item.ariaLabel()}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
};
