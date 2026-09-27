import React, { useState } from 'react';
import { cn } from '@/utils/cn';
import { Input } from './Input';
import { ALL_ASSIGNABLE_KEYS, type KeyCategory } from './assignableKeys';
import { KeyItem } from './KeyItem';
import { useKeyPaletteFilter } from './useKeyPaletteFilter';

export type { AssignableKey } from './assignableKeys';

export interface KeyAssignmentPanelProps {
  /** CSS class name for styling */
  className?: string;
}

/**
 * Categorized key palette component for visual configuration editor.
 * Provides draggable key sources organized by type (Virtual Keys, Modifiers, Locks, Layers, Macros).
 * Users can drag keys from this panel onto the keyboard visualizer to assign key mappings.
 */
export const KeyAssignmentPanel: React.FC<KeyAssignmentPanelProps> = ({
  className = '',
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<KeyCategory | 'all'>(
    'all'
  );

  const { filteredKeys, groupedKeys } = useKeyPaletteFilter(
    ALL_ASSIGNABLE_KEYS,
    searchQuery,
    selectedCategory
  );

  // Category metadata
  const categories: Array<{ value: KeyCategory | 'all'; label: string }> = [
    { value: 'all', label: 'All Keys' },
    { value: 'virtual', label: 'Virtual Keys' },
    { value: 'modifier', label: 'Modifiers' },
    { value: 'lock', label: 'Locks' },
    { value: 'layer', label: 'Layers' },
    { value: 'macro', label: 'Macros' },
  ];

  const handleSearchChange = (value: string) => {
    setSearchQuery(value);
  };

  const handleCategoryChange = (category: KeyCategory | 'all') => {
    setSelectedCategory(category);
  };

  return (
    <div
      className={cn(
        'flex flex-col h-full bg-slate-800 border border-slate-700 rounded-md',
        className
      )}
      role="complementary"
      aria-label="Key assignment palette"
    >
      {/* Header */}
      <div className="p-4 border-b border-slate-700">
        <h2 className="text-lg font-semibold text-slate-100 mb-2">
          Key Palette
        </h2>
        <p
          className="text-xs text-slate-400 mb-3"
          id="key-palette-instructions"
        >
          Click a key to select it for assignment
        </p>

        {/* Search input */}
        <Input
          type="text"
          value={searchQuery}
          onChange={handleSearchChange}
          placeholder="Search keys..."
          aria-label="Search keys"
          className="mb-3"
        />

        {/* Category filter tabs */}
        <div role="tablist" className="flex flex-wrap gap-2">
          {categories.map((cat) => (
            <button
              key={cat.value}
              role="tab"
              aria-selected={selectedCategory === cat.value}
              aria-controls={`panel-${cat.value}`}
              onClick={() => handleCategoryChange(cat.value)}
              className={cn(
                'px-3 py-1.5 text-sm font-medium rounded transition-colors duration-150',
                'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
                selectedCategory === cat.value
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              )}
              type="button"
            >
              {cat.label}
            </button>
          ))}
        </div>
      </div>

      {/* Key list */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {selectedCategory === 'all' ? (
          // Show all categories when "all" is selected
          <div id="panel-all" role="tabpanel">
            {Object.entries(groupedKeys).map(([category, keys]) => {
              if (keys.length === 0) return null;
              const categoryLabel =
                categories.find((c) => c.value === category)?.label || category;
              return (
                <div key={category} className="mb-4">
                  <h3 className="text-sm font-semibold text-slate-400 mb-2 uppercase tracking-wide">
                    {categoryLabel}
                  </h3>
                  <div className="grid grid-cols-2 gap-2">
                    {keys.map((key) => (
                      <KeyItem key={key.id} keyItem={key} />
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          // Show only selected category
          <div id={`panel-${selectedCategory}`} role="tabpanel">
            {filteredKeys.length === 0 ? (
              <p className="text-slate-400 text-center py-8">No keys found</p>
            ) : (
              <div className="grid grid-cols-2 gap-2">
                {filteredKeys.map((key) => (
                  <KeyItem key={key.id} keyItem={key} />
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Footer with count */}
      <div className="p-3 border-t border-slate-700 bg-slate-700/50">
        <p className="text-xs text-slate-400 text-center">
          {filteredKeys.length} {filteredKeys.length === 1 ? 'key' : 'keys'}
          {searchQuery && ` matching "${searchQuery}"`}
        </p>
      </div>
    </div>
  );
};

KeyAssignmentPanel.displayName = 'KeyAssignmentPanel';
