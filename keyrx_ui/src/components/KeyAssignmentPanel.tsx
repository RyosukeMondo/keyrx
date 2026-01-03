import React, { useState, useMemo } from 'react';
import { useDraggable } from '@dnd-kit/core';
import { cn } from '@/utils/cn';
import { Input } from './Input';

/**
 * Key category types for organizing key assignments
 */
type KeyCategory = 'virtual' | 'modifier' | 'lock' | 'layer' | 'macro';

/**
 * Represents a key that can be assigned in the visual editor
 */
export interface AssignableKey {
  /** Unique identifier for the key (e.g., "VK_A", "MD_CTRL") */
  id: string;
  /** Display label for the key (e.g., "A", "Ctrl") */
  label: string;
  /** Category this key belongs to */
  category: KeyCategory;
  /** Optional description for tooltip/help text */
  description?: string;
}

interface DraggableKeyItemProps {
  keyItem: AssignableKey;
}

/**
 * Individual draggable key item within the palette
 */
const DraggableKeyItem: React.FC<DraggableKeyItemProps> = ({ keyItem }) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: keyItem.id,
    data: keyItem,
  });

  // Build comprehensive aria-label with keyboard instructions
  const ariaLabel = isDragging
    ? `Dragging ${keyItem.label}. Use arrow keys to select target key, Space to drop, Escape to cancel.`
    : `${keyItem.label} key. ${keyItem.description || ''}. Press Space to grab and drag.`;

  return (
    <button
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      className={cn(
        'px-3 py-2 text-sm font-medium rounded border transition-all duration-150',
        'bg-slate-700 border-slate-600 text-slate-100',
        'hover:bg-slate-600 hover:border-slate-500',
        'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
        'min-h-[44px] min-w-[44px]', // Touch-friendly minimum size
        'cursor-grab active:cursor-grabbing',
        isDragging && 'opacity-50 cursor-grabbing'
      )}
      aria-label={ariaLabel}
      aria-grabbed={isDragging}
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

DraggableKeyItem.displayName = 'DraggableKeyItem';

export interface KeyAssignmentPanelProps {
  /** CSS class name for styling */
  className?: string;
}

/**
 * Categorized key palette component for visual configuration editor.
 * Provides draggable key sources organized by type (Virtual Keys, Modifiers, Locks, Layers, Macros).
 * Users can drag keys from this panel onto the keyboard visualizer to assign key mappings.
 */
export const KeyAssignmentPanel: React.FC<KeyAssignmentPanelProps> = ({ className = '' }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<KeyCategory | 'all'>('all');

  // Define available keys by category
  const allKeys = useMemo<AssignableKey[]>(() => {
    return [
      // Virtual Keys (VK_*) - Standard keyboard keys
      { id: 'VK_A', label: 'A', category: 'virtual', description: 'Letter A' },
      { id: 'VK_B', label: 'B', category: 'virtual', description: 'Letter B' },
      { id: 'VK_C', label: 'C', category: 'virtual', description: 'Letter C' },
      { id: 'VK_D', label: 'D', category: 'virtual', description: 'Letter D' },
      { id: 'VK_E', label: 'E', category: 'virtual', description: 'Letter E' },
      { id: 'VK_F', label: 'F', category: 'virtual', description: 'Letter F' },
      { id: 'VK_G', label: 'G', category: 'virtual', description: 'Letter G' },
      { id: 'VK_H', label: 'H', category: 'virtual', description: 'Letter H' },
      { id: 'VK_I', label: 'I', category: 'virtual', description: 'Letter I' },
      { id: 'VK_J', label: 'J', category: 'virtual', description: 'Letter J' },
      { id: 'VK_K', label: 'K', category: 'virtual', description: 'Letter K' },
      { id: 'VK_L', label: 'L', category: 'virtual', description: 'Letter L' },
      { id: 'VK_M', label: 'M', category: 'virtual', description: 'Letter M' },
      { id: 'VK_N', label: 'N', category: 'virtual', description: 'Letter N' },
      { id: 'VK_O', label: 'O', category: 'virtual', description: 'Letter O' },
      { id: 'VK_P', label: 'P', category: 'virtual', description: 'Letter P' },
      { id: 'VK_Q', label: 'Q', category: 'virtual', description: 'Letter Q' },
      { id: 'VK_R', label: 'R', category: 'virtual', description: 'Letter R' },
      { id: 'VK_S', label: 'S', category: 'virtual', description: 'Letter S' },
      { id: 'VK_T', label: 'T', category: 'virtual', description: 'Letter T' },
      { id: 'VK_U', label: 'U', category: 'virtual', description: 'Letter U' },
      { id: 'VK_V', label: 'V', category: 'virtual', description: 'Letter V' },
      { id: 'VK_W', label: 'W', category: 'virtual', description: 'Letter W' },
      { id: 'VK_X', label: 'X', category: 'virtual', description: 'Letter X' },
      { id: 'VK_Y', label: 'Y', category: 'virtual', description: 'Letter Y' },
      { id: 'VK_Z', label: 'Z', category: 'virtual', description: 'Letter Z' },
      { id: 'VK_1', label: '1', category: 'virtual', description: 'Number 1' },
      { id: 'VK_2', label: '2', category: 'virtual', description: 'Number 2' },
      { id: 'VK_3', label: '3', category: 'virtual', description: 'Number 3' },
      { id: 'VK_4', label: '4', category: 'virtual', description: 'Number 4' },
      { id: 'VK_5', label: '5', category: 'virtual', description: 'Number 5' },
      { id: 'VK_6', label: '6', category: 'virtual', description: 'Number 6' },
      { id: 'VK_7', label: '7', category: 'virtual', description: 'Number 7' },
      { id: 'VK_8', label: '8', category: 'virtual', description: 'Number 8' },
      { id: 'VK_9', label: '9', category: 'virtual', description: 'Number 9' },
      { id: 'VK_0', label: '0', category: 'virtual', description: 'Number 0' },
      { id: 'VK_ENTER', label: 'Enter', category: 'virtual', description: 'Enter key' },
      { id: 'VK_ESCAPE', label: 'Esc', category: 'virtual', description: 'Escape key' },
      { id: 'VK_BACKSPACE', label: 'Backspace', category: 'virtual', description: 'Backspace key' },
      { id: 'VK_TAB', label: 'Tab', category: 'virtual', description: 'Tab key' },
      { id: 'VK_SPACE', label: 'Space', category: 'virtual', description: 'Space bar' },
      { id: 'VK_F1', label: 'F1', category: 'virtual', description: 'Function key F1' },
      { id: 'VK_F2', label: 'F2', category: 'virtual', description: 'Function key F2' },
      { id: 'VK_F3', label: 'F3', category: 'virtual', description: 'Function key F3' },
      { id: 'VK_F4', label: 'F4', category: 'virtual', description: 'Function key F4' },
      { id: 'VK_F5', label: 'F5', category: 'virtual', description: 'Function key F5' },
      { id: 'VK_F6', label: 'F6', category: 'virtual', description: 'Function key F6' },
      { id: 'VK_F7', label: 'F7', category: 'virtual', description: 'Function key F7' },
      { id: 'VK_F8', label: 'F8', category: 'virtual', description: 'Function key F8' },
      { id: 'VK_F9', label: 'F9', category: 'virtual', description: 'Function key F9' },
      { id: 'VK_F10', label: 'F10', category: 'virtual', description: 'Function key F10' },
      { id: 'VK_F11', label: 'F11', category: 'virtual', description: 'Function key F11' },
      { id: 'VK_F12', label: 'F12', category: 'virtual', description: 'Function key F12' },
      { id: 'VK_UP', label: '↑', category: 'virtual', description: 'Arrow Up' },
      { id: 'VK_DOWN', label: '↓', category: 'virtual', description: 'Arrow Down' },
      { id: 'VK_LEFT', label: '←', category: 'virtual', description: 'Arrow Left' },
      { id: 'VK_RIGHT', label: '→', category: 'virtual', description: 'Arrow Right' },
      { id: 'VK_HOME', label: 'Home', category: 'virtual', description: 'Home key' },
      { id: 'VK_END', label: 'End', category: 'virtual', description: 'End key' },
      { id: 'VK_PAGEUP', label: 'PgUp', category: 'virtual', description: 'Page Up' },
      { id: 'VK_PAGEDOWN', label: 'PgDn', category: 'virtual', description: 'Page Down' },
      { id: 'VK_DELETE', label: 'Del', category: 'virtual', description: 'Delete key' },

      // Modifiers (MD_*)
      { id: 'MD_CTRL', label: 'Ctrl', category: 'modifier', description: 'Control modifier' },
      { id: 'MD_SHIFT', label: 'Shift', category: 'modifier', description: 'Shift modifier' },
      { id: 'MD_ALT', label: 'Alt', category: 'modifier', description: 'Alt modifier' },
      { id: 'MD_GUI', label: 'Super', category: 'modifier', description: 'Super/Windows/Command modifier' },
      { id: 'MD_RCTRL', label: 'RCtrl', category: 'modifier', description: 'Right Control modifier' },
      { id: 'MD_RSHIFT', label: 'RShift', category: 'modifier', description: 'Right Shift modifier' },
      { id: 'MD_RALT', label: 'RAlt', category: 'modifier', description: 'Right Alt modifier' },
      { id: 'MD_RGUI', label: 'RSuper', category: 'modifier', description: 'Right Super modifier' },

      // Locks (LK_*)
      { id: 'LK_CAPS', label: 'CapsLock', category: 'lock', description: 'Caps Lock toggle' },
      { id: 'LK_NUM', label: 'NumLock', category: 'lock', description: 'Num Lock toggle' },
      { id: 'LK_SCROLL', label: 'ScrollLock', category: 'lock', description: 'Scroll Lock toggle' },

      // Layers (common layer names)
      { id: 'LAYER_BASE', label: 'Base Layer', category: 'layer', description: 'Switch to base layer' },
      { id: 'LAYER_NAV', label: 'Nav Layer', category: 'layer', description: 'Switch to navigation layer' },
      { id: 'LAYER_NUM', label: 'Num Layer', category: 'layer', description: 'Switch to number layer' },
      { id: 'LAYER_FN', label: 'Fn Layer', category: 'layer', description: 'Switch to function layer' },
      { id: 'LAYER_GAMING', label: 'Gaming Layer', category: 'layer', description: 'Switch to gaming layer' },

      // Macros (example macros)
      { id: 'MACRO_COPY', label: 'Copy', category: 'macro', description: 'Copy macro (Ctrl+C)' },
      { id: 'MACRO_PASTE', label: 'Paste', category: 'macro', description: 'Paste macro (Ctrl+V)' },
      { id: 'MACRO_CUT', label: 'Cut', category: 'macro', description: 'Cut macro (Ctrl+X)' },
      { id: 'MACRO_UNDO', label: 'Undo', category: 'macro', description: 'Undo macro (Ctrl+Z)' },
      { id: 'MACRO_REDO', label: 'Redo', category: 'macro', description: 'Redo macro (Ctrl+Y)' },
    ];
  }, []);

  // Filter keys based on search query and selected category
  const filteredKeys = useMemo(() => {
    let keys = allKeys;

    // Filter by category
    if (selectedCategory !== 'all') {
      keys = keys.filter(key => key.category === selectedCategory);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      keys = keys.filter(
        key =>
          key.label.toLowerCase().includes(query) ||
          key.id.toLowerCase().includes(query) ||
          key.description?.toLowerCase().includes(query)
      );
    }

    return keys;
  }, [allKeys, searchQuery, selectedCategory]);

  // Group filtered keys by category
  const groupedKeys = useMemo(() => {
    const groups: Record<KeyCategory, AssignableKey[]> = {
      virtual: [],
      modifier: [],
      lock: [],
      layer: [],
      macro: [],
    };

    filteredKeys.forEach(key => {
      groups[key.category].push(key);
    });

    return groups;
  }, [filteredKeys]);

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
    <div className={cn('flex flex-col h-full bg-slate-800 border border-slate-700 rounded-md', className)} role="complementary" aria-label="Key assignment palette">
      {/* Header */}
      <div className="p-4 border-b border-slate-700">
        <h2 className="text-lg font-semibold text-slate-100 mb-2">Key Palette</h2>
        <p className="text-xs text-slate-400 mb-3" id="key-palette-instructions">
          Use mouse to drag or keyboard: Tab to focus a key, Space to grab, Arrow keys to navigate, Space to drop
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
          {categories.map(cat => (
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
              const categoryLabel = categories.find(c => c.value === category)?.label || category;
              return (
                <div key={category} className="mb-4">
                  <h3 className="text-sm font-semibold text-slate-400 mb-2 uppercase tracking-wide">
                    {categoryLabel}
                  </h3>
                  <div className="grid grid-cols-2 gap-2">
                    {keys.map(key => (
                      <DraggableKeyItem key={key.id} keyItem={key} />
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
                {filteredKeys.map(key => (
                  <DraggableKeyItem key={key.id} keyItem={key} />
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
