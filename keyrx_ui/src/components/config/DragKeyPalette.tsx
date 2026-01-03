import React, { useMemo } from 'react';
import { useDraggable } from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import { AssignableKey } from '@/types/config';
import { cn } from '@/utils/cn';

/**
 * DragKeyPalette component props
 */
export interface DragKeyPaletteProps {
  /** Callback when drag starts */
  onDragStart?: (key: AssignableKey) => void;
  /** Callback when drag ends */
  onDragEnd?: () => void;
  /** Filter by category (optional) */
  filterCategory?: string;
  /** Additional CSS classes */
  className?: string;
}

/**
 * Static list of assignable keys for the palette
 */
const ASSIGNABLE_KEYS: AssignableKey[] = [
  // Virtual Keys (VK_) - Letters
  { id: 'VK_A', category: 'vk', label: 'A', description: 'Virtual Key A' },
  { id: 'VK_B', category: 'vk', label: 'B', description: 'Virtual Key B' },
  { id: 'VK_C', category: 'vk', label: 'C', description: 'Virtual Key C' },
  { id: 'VK_D', category: 'vk', label: 'D', description: 'Virtual Key D' },
  { id: 'VK_E', category: 'vk', label: 'E', description: 'Virtual Key E' },
  { id: 'VK_F', category: 'vk', label: 'F', description: 'Virtual Key F' },
  { id: 'VK_G', category: 'vk', label: 'G', description: 'Virtual Key G' },
  { id: 'VK_H', category: 'vk', label: 'H', description: 'Virtual Key H' },
  { id: 'VK_I', category: 'vk', label: 'I', description: 'Virtual Key I' },
  { id: 'VK_J', category: 'vk', label: 'J', description: 'Virtual Key J' },
  { id: 'VK_K', category: 'vk', label: 'K', description: 'Virtual Key K' },
  { id: 'VK_L', category: 'vk', label: 'L', description: 'Virtual Key L' },
  { id: 'VK_M', category: 'vk', label: 'M', description: 'Virtual Key M' },
  { id: 'VK_N', category: 'vk', label: 'N', description: 'Virtual Key N' },
  { id: 'VK_O', category: 'vk', label: 'O', description: 'Virtual Key O' },
  { id: 'VK_P', category: 'vk', label: 'P', description: 'Virtual Key P' },
  { id: 'VK_Q', category: 'vk', label: 'Q', description: 'Virtual Key Q' },
  { id: 'VK_R', category: 'vk', label: 'R', description: 'Virtual Key R' },
  { id: 'VK_S', category: 'vk', label: 'S', description: 'Virtual Key S' },
  { id: 'VK_T', category: 'vk', label: 'T', description: 'Virtual Key T' },
  { id: 'VK_U', category: 'vk', label: 'U', description: 'Virtual Key U' },
  { id: 'VK_V', category: 'vk', label: 'V', description: 'Virtual Key V' },
  { id: 'VK_W', category: 'vk', label: 'W', description: 'Virtual Key W' },
  { id: 'VK_X', category: 'vk', label: 'X', description: 'Virtual Key X' },
  { id: 'VK_Y', category: 'vk', label: 'Y', description: 'Virtual Key Y' },
  { id: 'VK_Z', category: 'vk', label: 'Z', description: 'Virtual Key Z' },

  // Virtual Keys - Numbers
  { id: 'VK_0', category: 'vk', label: '0', description: 'Virtual Key 0' },
  { id: 'VK_1', category: 'vk', label: '1', description: 'Virtual Key 1' },
  { id: 'VK_2', category: 'vk', label: '2', description: 'Virtual Key 2' },
  { id: 'VK_3', category: 'vk', label: '3', description: 'Virtual Key 3' },
  { id: 'VK_4', category: 'vk', label: '4', description: 'Virtual Key 4' },
  { id: 'VK_5', category: 'vk', label: '5', description: 'Virtual Key 5' },
  { id: 'VK_6', category: 'vk', label: '6', description: 'Virtual Key 6' },
  { id: 'VK_7', category: 'vk', label: '7', description: 'Virtual Key 7' },
  { id: 'VK_8', category: 'vk', label: '8', description: 'Virtual Key 8' },
  { id: 'VK_9', category: 'vk', label: '9', description: 'Virtual Key 9' },

  // Virtual Keys - Special
  { id: 'VK_ESCAPE', category: 'vk', label: 'Esc', description: 'Escape key' },
  { id: 'VK_ENTER', category: 'vk', label: 'Enter', description: 'Enter/Return key' },
  { id: 'VK_SPACE', category: 'vk', label: 'Space', description: 'Space bar' },
  { id: 'VK_BACKSPACE', category: 'vk', label: 'Bksp', description: 'Backspace key' },
  { id: 'VK_TAB', category: 'vk', label: 'Tab', description: 'Tab key' },
  { id: 'VK_DELETE', category: 'vk', label: 'Del', description: 'Delete key' },
  { id: 'VK_HOME', category: 'vk', label: 'Home', description: 'Home key' },
  { id: 'VK_END', category: 'vk', label: 'End', description: 'End key' },
  { id: 'VK_PAGEUP', category: 'vk', label: 'PgUp', description: 'Page Up key' },
  { id: 'VK_PAGEDOWN', category: 'vk', label: 'PgDn', description: 'Page Down key' },

  // Virtual Keys - Arrows
  { id: 'VK_LEFT', category: 'vk', label: '←', description: 'Left arrow key' },
  { id: 'VK_RIGHT', category: 'vk', label: '→', description: 'Right arrow key' },
  { id: 'VK_UP', category: 'vk', label: '↑', description: 'Up arrow key' },
  { id: 'VK_DOWN', category: 'vk', label: '↓', description: 'Down arrow key' },

  // Modifiers (MD_)
  { id: 'MD_SHIFT', category: 'modifier', label: 'Shift', description: 'Shift modifier' },
  { id: 'MD_CTRL', category: 'modifier', label: 'Ctrl', description: 'Control modifier' },
  { id: 'MD_ALT', category: 'modifier', label: 'Alt', description: 'Alt modifier' },
  { id: 'MD_SUPER', category: 'modifier', label: 'Super', description: 'Super/Windows/Command key' },

  // Locks (LK_)
  { id: 'LK_CAPSLOCK', category: 'lock', label: 'CapsLock', description: 'Caps Lock toggle' },
  { id: 'LK_NUMLOCK', category: 'lock', label: 'NumLock', description: 'Num Lock toggle' },
  { id: 'LK_SCROLLLOCK', category: 'lock', label: 'ScrollLock', description: 'Scroll Lock toggle' },

  // Layers
  { id: 'layer_base', category: 'layer', label: 'Layer: Base', description: 'Switch to base layer' },
  { id: 'layer_nav', category: 'layer', label: 'Layer: Nav', description: 'Switch to navigation layer' },
  { id: 'layer_num', category: 'layer', label: 'Layer: Num', description: 'Switch to numeric layer' },
  { id: 'layer_fn', category: 'layer', label: 'Layer: Fn', description: 'Switch to function layer' },
];

/**
 * DraggableKeyItem component
 *
 * Individual draggable key button with @dnd-kit integration
 */
const DraggableKeyItem: React.FC<{
  keyItem: AssignableKey;
  onDragStart?: (key: AssignableKey) => void;
  onDragEnd?: () => void;
}> = ({ keyItem, onDragStart, onDragEnd }) => {
  const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
    id: keyItem.id,
    data: { key: keyItem },
  });

  const style = {
    transform: CSS.Translate.toString(transform),
  };

  // Handle drag start/end callbacks
  const wasDraggingRef = React.useRef(false);
  React.useEffect(() => {
    if (isDragging && !wasDraggingRef.current && onDragStart) {
      wasDraggingRef.current = true;
      onDragStart(keyItem);
    }
    if (!isDragging && wasDraggingRef.current && onDragEnd) {
      wasDraggingRef.current = false;
      onDragEnd();
    }
  }, [isDragging, keyItem, onDragStart, onDragEnd]);

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...attributes}
      className={cn(
        'p-3 bg-slate-700 rounded cursor-grab active:cursor-grabbing',
        'hover:bg-slate-600 transition-colors duration-150',
        'flex items-center justify-center text-center',
        'min-h-[44px] min-w-[44px]', // Ensure ≥44px touch targets (WCAG)
        'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
        'text-slate-100 font-medium text-sm',
        isDragging && 'opacity-50'
      )}
      role="button"
      aria-label={`Drag ${keyItem.description}`}
      aria-describedby={`${keyItem.id}-description`}
      tabIndex={0}
      title={keyItem.description}
    >
      <span>{keyItem.label}</span>
      <span id={`${keyItem.id}-description`} className="sr-only">
        {keyItem.description}
      </span>
    </div>
  );
};

/**
 * DragKeyPalette Component
 *
 * Displays a palette of draggable virtual keys, modifiers, locks, and layers.
 * Users can drag keys from this palette onto the keyboard visualizer to create
 * key mappings.
 *
 * Supports:
 * - Drag-and-drop with @dnd-kit/core
 * - Category filtering (vk, modifier, lock, layer, macro)
 * - Keyboard accessibility (Tab to focus, Space to grab, Arrow keys to navigate)
 * - Touch-friendly targets (≥44px)
 * - WCAG 2.2 Level AA compliance
 *
 * Requirements: Requirement 4 - QMK-Style Drag-and-Drop Configuration Editor
 *
 * @example
 * ```tsx
 * <DragKeyPalette
 *   onDragStart={(key) => console.log('Started dragging', key.id)}
 *   onDragEnd={() => console.log('Drag ended')}
 *   filterCategory="vk"
 * />
 * ```
 */
export const DragKeyPalette: React.FC<DragKeyPaletteProps> = ({
  onDragStart,
  onDragEnd,
  filterCategory,
  className,
}) => {
  // Filter keys by category if specified
  const filteredKeys = useMemo(() => {
    if (!filterCategory) return ASSIGNABLE_KEYS;
    return ASSIGNABLE_KEYS.filter((key) => key.category === filterCategory);
  }, [filterCategory]);

  // Group keys by category for organized display
  const groupedKeys = useMemo(() => {
    const groups: Record<string, AssignableKey[]> = {
      vk: [],
      modifier: [],
      lock: [],
      layer: [],
      macro: [],
    };

    filteredKeys.forEach((key) => {
      groups[key.category].push(key);
    });

    return groups;
  }, [filteredKeys]);

  return (
    <div className={cn('space-y-6', className)}>
      {/* Virtual Keys */}
      {groupedKeys.vk.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold text-slate-200 mb-3">Virtual Keys</h3>
          <div className="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 gap-2">
            {groupedKeys.vk.map((key) => (
              <DraggableKeyItem
                key={key.id}
                keyItem={key}
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
              />
            ))}
          </div>
        </div>
      )}

      {/* Modifiers */}
      {groupedKeys.modifier.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold text-slate-200 mb-3">Modifiers</h3>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            {groupedKeys.modifier.map((key) => (
              <DraggableKeyItem
                key={key.id}
                keyItem={key}
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
              />
            ))}
          </div>
        </div>
      )}

      {/* Locks */}
      {groupedKeys.lock.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold text-slate-200 mb-3">Lock Keys</h3>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
            {groupedKeys.lock.map((key) => (
              <DraggableKeyItem
                key={key.id}
                keyItem={key}
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
              />
            ))}
          </div>
        </div>
      )}

      {/* Layers */}
      {groupedKeys.layer.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold text-slate-200 mb-3">Layers</h3>
          <div className="grid grid-cols-2 gap-2">
            {groupedKeys.layer.map((key) => (
              <DraggableKeyItem
                key={key.id}
                keyItem={key}
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
              />
            ))}
          </div>
        </div>
      )}

      {/* Empty state */}
      {filteredKeys.length === 0 && (
        <div className="text-center py-8 text-slate-400">
          No keys available for the selected category.
        </div>
      )}
    </div>
  );
};
