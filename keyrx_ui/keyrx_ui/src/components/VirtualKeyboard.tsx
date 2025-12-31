/**
 * VirtualKeyboard Component
 *
 * Displays a visual keyboard layout with draggable keys for remapping.
 * Uses @dnd-kit for drag-and-drop functionality.
 */

import React, { useMemo } from 'react';
import { useDraggable, useDroppable } from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import { useConfigBuilderStore } from '../store/configBuilderStore';
import type { KeyInfo, KeyboardLayout } from '../types/configBuilder';
import './VirtualKeyboard.css';

/**
 * Standard QWERTY keyboard layout
 */
const QWERTY_LAYOUT: KeyboardLayout = {
  name: 'QWERTY',
  rows: [
    {
      keys: [
        { code: 'KEY_ESC', label: 'Esc', width: 1, isSpecial: true },
        { code: 'KEY_1', label: '1' },
        { code: 'KEY_2', label: '2' },
        { code: 'KEY_3', label: '3' },
        { code: 'KEY_4', label: '4' },
        { code: 'KEY_5', label: '5' },
        { code: 'KEY_6', label: '6' },
        { code: 'KEY_7', label: '7' },
        { code: 'KEY_8', label: '8' },
        { code: 'KEY_9', label: '9' },
        { code: 'KEY_0', label: '0' },
        { code: 'KEY_MINUS', label: '-' },
        { code: 'KEY_EQUAL', label: '=' },
        { code: 'KEY_BACKSPACE', label: 'Back', width: 2, isSpecial: true },
      ],
    },
    {
      keys: [
        { code: 'KEY_TAB', label: 'Tab', width: 1.5, isSpecial: true },
        { code: 'KEY_Q', label: 'Q' },
        { code: 'KEY_W', label: 'W' },
        { code: 'KEY_E', label: 'E' },
        { code: 'KEY_R', label: 'R' },
        { code: 'KEY_T', label: 'T' },
        { code: 'KEY_Y', label: 'Y' },
        { code: 'KEY_U', label: 'U' },
        { code: 'KEY_I', label: 'I' },
        { code: 'KEY_O', label: 'O' },
        { code: 'KEY_P', label: 'P' },
        { code: 'KEY_LEFTBRACE', label: '[' },
        { code: 'KEY_RIGHTBRACE', label: ']' },
        { code: 'KEY_BACKSLASH', label: '\\', width: 1.5 },
      ],
    },
    {
      keys: [
        { code: 'KEY_CAPSLOCK', label: 'Caps', width: 1.75, isSpecial: true },
        { code: 'KEY_A', label: 'A' },
        { code: 'KEY_S', label: 'S' },
        { code: 'KEY_D', label: 'D' },
        { code: 'KEY_F', label: 'F' },
        { code: 'KEY_G', label: 'G' },
        { code: 'KEY_H', label: 'H' },
        { code: 'KEY_J', label: 'J' },
        { code: 'KEY_K', label: 'K' },
        { code: 'KEY_L', label: 'L' },
        { code: 'KEY_SEMICOLON', label: ';' },
        { code: 'KEY_APOSTROPHE', label: "'" },
        { code: 'KEY_ENTER', label: 'Enter', width: 2.25, isSpecial: true },
      ],
    },
    {
      keys: [
        { code: 'KEY_LEFTSHIFT', label: 'Shift', width: 2.25, isModifier: true },
        { code: 'KEY_Z', label: 'Z' },
        { code: 'KEY_X', label: 'X' },
        { code: 'KEY_C', label: 'C' },
        { code: 'KEY_V', label: 'V' },
        { code: 'KEY_B', label: 'B' },
        { code: 'KEY_N', label: 'N' },
        { code: 'KEY_M', label: 'M' },
        { code: 'KEY_COMMA', label: ',' },
        { code: 'KEY_DOT', label: '.' },
        { code: 'KEY_SLASH', label: '/' },
        { code: 'KEY_RIGHTSHIFT', label: 'Shift', width: 2.75, isModifier: true },
      ],
    },
    {
      keys: [
        { code: 'KEY_LEFTCTRL', label: 'Ctrl', width: 1.5, isModifier: true },
        { code: 'KEY_LEFTMETA', label: 'Win', width: 1.5, isModifier: true },
        { code: 'KEY_LEFTALT', label: 'Alt', width: 1.5, isModifier: true },
        { code: 'KEY_SPACE', label: 'Space', width: 6 },
        { code: 'KEY_RIGHTALT', label: 'Alt', width: 1.5, isModifier: true },
        { code: 'KEY_RIGHTMETA', label: 'Win', width: 1.5, isModifier: true },
        { code: 'KEY_RIGHTCTRL', label: 'Ctrl', width: 1.5, isModifier: true },
      ],
    },
  ],
};

interface DraggableKeyProps {
  keyInfo: KeyInfo;
  isMapped: boolean;
  mappingTarget?: string;
}

/**
 * A single draggable key
 */
const DraggableKey: React.FC<DraggableKeyProps> = ({
  keyInfo,
  isMapped,
  mappingTarget,
}) => {
  const { attributes, listeners, setNodeRef, transform, isDragging } =
    useDraggable({
      id: keyInfo.code,
      data: {
        type: 'key',
        keyCode: keyInfo.code,
      },
    });

  const style = {
    transform: CSS.Translate.toString(transform),
    width: `${(keyInfo.width || 1) * 60}px`,
  };

  const className = [
    'virtual-key',
    keyInfo.isModifier && 'modifier',
    keyInfo.isSpecial && 'special',
    isMapped && 'mapped',
    isDragging && 'dragging',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={className}
      {...listeners}
      {...attributes}
    >
      <span className="key-label">{keyInfo.label}</span>
      {isMapped && mappingTarget && (
        <span className="mapping-indicator" title={`Mapped to ${mappingTarget}`}>
          → {mappingTarget}
        </span>
      )}
    </div>
  );
};

interface DroppableKeyProps {
  keyInfo: KeyInfo;
  isMapped: boolean;
  mappingTarget?: string;
  onDrop: (sourceKey: string, targetKey: string) => void;
}

/**
 * A key that can accept drops for creating mappings
 */
const DroppableKey: React.FC<DroppableKeyProps> = ({
  keyInfo,
  isMapped,
  mappingTarget,
  onDrop,
}) => {
  const { setNodeRef, isOver } = useDroppable({
    id: `drop-${keyInfo.code}`,
    data: {
      type: 'key',
      keyCode: keyInfo.code,
    },
  });

  const handleDrop = (sourceKey: string) => {
    if (sourceKey !== keyInfo.code) {
      onDrop(sourceKey, keyInfo.code);
    }
  };

  const style = {
    width: `${(keyInfo.width || 1) * 60}px`,
  };

  const className = [
    'virtual-key',
    'droppable',
    keyInfo.isModifier && 'modifier',
    keyInfo.isSpecial && 'special',
    isMapped && 'mapped',
    isOver && 'drop-target',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={className}
    >
      <span className="key-label">{keyInfo.label}</span>
      {isMapped && mappingTarget && (
        <span className="mapping-indicator" title={`Mapped to ${mappingTarget}`}>
          → {mappingTarget}
        </span>
      )}
    </div>
  );
};

interface VirtualKeyboardProps {
  /**
   * Keyboard layout to display
   */
  layout?: KeyboardLayout;
  /**
   * Whether to enable drag-and-drop for source keys
   */
  draggable?: boolean;
  /**
   * Whether to enable drop targets for creating mappings
   */
  droppable?: boolean;
}

/**
 * VirtualKeyboard Component
 *
 * Displays a visual keyboard with drag-and-drop support for key remapping.
 */
export const VirtualKeyboard: React.FC<VirtualKeyboardProps> = ({
  layout = QWERTY_LAYOUT,
  draggable = true,
  droppable = false,
}) => {
  const { layers, currentLayerId, addMapping } = useConfigBuilderStore();

  // Get current layer
  const currentLayer = useMemo(
    () => layers.find((l) => l.id === currentLayerId),
    [layers, currentLayerId]
  );

  // Build mapping lookup for highlighting
  const mappingLookup = useMemo(() => {
    const lookup = new Map<string, string>();
    if (currentLayer) {
      for (const mapping of currentLayer.mappings) {
        lookup.set(mapping.sourceKey, mapping.targetKey);
      }
    }
    return lookup;
  }, [currentLayer]);

  const handleDrop = (sourceKey: string, targetKey: string) => {
    if (currentLayerId) {
      addMapping(currentLayerId, sourceKey, targetKey, 'simple');
    }
  };

  return (
    <div className="virtual-keyboard">
      <div className="keyboard-header">
        <h3>{layout.name} Layout</h3>
        {currentLayer && (
          <span className="layer-name">Layer: {currentLayer.name}</span>
        )}
      </div>
      <div className="keyboard-rows">
        {layout.rows.map((row, rowIndex) => (
          <div
            key={rowIndex}
            className="keyboard-row"
            style={{
              marginLeft: `${(row.marginLeft || 0) * 60}px`,
              marginTop: `${(row.offsetY || 0) * 60}px`,
            }}
          >
            {row.keys.map((keyInfo) => {
              const isMapped = mappingLookup.has(keyInfo.code);
              const mappingTarget = mappingLookup.get(keyInfo.code);

              if (droppable) {
                return (
                  <DroppableKey
                    key={keyInfo.code}
                    keyInfo={keyInfo}
                    isMapped={isMapped}
                    mappingTarget={mappingTarget}
                    onDrop={handleDrop}
                  />
                );
              }

              if (draggable) {
                return (
                  <DraggableKey
                    key={keyInfo.code}
                    keyInfo={keyInfo}
                    isMapped={isMapped}
                    mappingTarget={mappingTarget}
                  />
                );
              }

              // Static key (no interaction)
              return (
                <div
                  key={keyInfo.code}
                  className="virtual-key"
                  style={{ width: `${(keyInfo.width || 1) * 60}px` }}
                >
                  <span className="key-label">{keyInfo.label}</span>
                  {isMapped && mappingTarget && (
                    <span
                      className="mapping-indicator"
                      title={`Mapped to ${mappingTarget}`}
                    >
                      → {mappingTarget}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
};
