import React from 'react';
import { Keyboard, ListOrdered } from 'lucide-react';

export interface KeyboardViewToggleProps {
  useKeyboard: boolean;
  onChange: (useKeyboard: boolean) => void;
}

/**
 * Shared "Keyboard / List" view toggle header used by the single-key
 * mapping forms (simple, modifier, lock).
 */
export function KeyboardViewToggle({
  useKeyboard,
  onChange,
}: KeyboardViewToggleProps) {
  return (
    <div className="flex items-center justify-between mb-3">
      <label className="text-sm font-medium text-slate-300">Select Key</label>
      <div className="flex gap-2">
        <button
          onClick={() => onChange(true)}
          className={`px-3 py-1.5 rounded-md text-xs font-medium ${
            useKeyboard
              ? 'bg-primary-500 text-white'
              : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
          }`}
        >
          <Keyboard className="w-4 h-4 inline-block mr-1" />
          Keyboard
        </button>
        <button
          onClick={() => onChange(false)}
          className={`px-3 py-1.5 rounded-md text-xs font-medium ${
            !useKeyboard
              ? 'bg-primary-500 text-white'
              : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
          }`}
        >
          <ListOrdered className="w-4 h-4 inline-block mr-1" />
          List
        </button>
      </div>
    </div>
  );
}
