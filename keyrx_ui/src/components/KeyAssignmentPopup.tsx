import React, { useState } from 'react';
import { Modal } from './Modal';
import { cn } from '@/utils/cn';
import { Input } from './Input';
import type { KeyMapping } from './KeyButton';
import type { AssignableKey } from './KeyAssignmentPanel';

type AssignmentType = 'key' | 'modifier' | 'lock' | 'layer' | 'macro' | 'tap-hold';

export interface KeyAssignmentPopupProps {
  open: boolean;
  onClose: () => void;
  keyCode: string;
  currentMapping?: KeyMapping;
  onSave: (mapping: KeyMapping) => void;
}

/**
 * Modal popup for detailed key assignment configuration.
 * Provides tabbed interface for different assignment types and tap-hold configuration.
 */
export const KeyAssignmentPopup: React.FC<KeyAssignmentPopupProps> = ({
  open,
  onClose,
  keyCode,
  currentMapping,
  onSave,
}) => {
  const [selectedType, setSelectedType] = useState<AssignmentType>('key');
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(currentMapping?.holdAction || '');
  const [holdThreshold, setHoldThreshold] = useState(
    currentMapping?.threshold?.toString() || '200'
  );
  const [targetLayer, setTargetLayer] = useState(currentMapping?.targetLayer || '');
  const [macroName, setMacroName] = useState('');

  const tabs: Array<{ value: AssignmentType; label: string }> = [
    { value: 'key', label: 'Key' },
    { value: 'modifier', label: 'Modifier' },
    { value: 'lock', label: 'Lock' },
    { value: 'layer', label: 'Layer' },
    { value: 'macro', label: 'Macro' },
    { value: 'tap-hold', label: 'Tap/Hold' },
  ];

  // Common key assignments
  const commonKeys: AssignableKey[] = [
    { id: 'VK_A', label: 'A', category: 'virtual' },
    { id: 'VK_B', label: 'B', category: 'virtual' },
    { id: 'VK_C', label: 'C', category: 'virtual' },
    { id: 'VK_ENTER', label: 'Enter', category: 'virtual' },
    { id: 'VK_ESCAPE', label: 'Esc', category: 'virtual' },
    { id: 'VK_BACKSPACE', label: 'Backspace', category: 'virtual' },
    { id: 'VK_TAB', label: 'Tab', category: 'virtual' },
    { id: 'VK_SPACE', label: 'Space', category: 'virtual' },
  ];

  const modifiers: AssignableKey[] = [
    { id: 'MD_CTRL', label: 'Ctrl', category: 'modifier' },
    { id: 'MD_SHIFT', label: 'Shift', category: 'modifier' },
    { id: 'MD_ALT', label: 'Alt', category: 'modifier' },
    { id: 'MD_GUI', label: 'Super', category: 'modifier' },
  ];

  const locks: AssignableKey[] = [
    { id: 'LK_CAPS', label: 'CapsLock', category: 'lock' },
    { id: 'LK_NUM', label: 'NumLock', category: 'lock' },
    { id: 'LK_SCROLL', label: 'ScrollLock', category: 'lock' },
  ];

  const layers: string[] = ['base', 'nav', 'num', 'fn', 'gaming'];

  const commonMacros: Array<{ id: string; label: string; description: string }> = [
    { id: 'MACRO_COPY', label: 'Copy', description: 'Ctrl+C' },
    { id: 'MACRO_PASTE', label: 'Paste', description: 'Ctrl+V' },
    { id: 'MACRO_CUT', label: 'Cut', description: 'Ctrl+X' },
    { id: 'MACRO_UNDO', label: 'Undo', description: 'Ctrl+Z' },
  ];

  const handleSave = () => {
    let mapping: KeyMapping;

    switch (selectedType) {
      case 'key':
        mapping = {
          type: 'simple',
          tapAction: tapAction || keyCode,
        };
        break;
      case 'modifier':
        mapping = {
          type: 'simple',
          tapAction: tapAction || 'MD_CTRL',
        };
        break;
      case 'lock':
        mapping = {
          type: 'simple',
          tapAction: tapAction || 'LK_CAPS',
        };
        break;
      case 'layer':
        mapping = {
          type: 'layer_switch',
          targetLayer: targetLayer || 'base',
        };
        break;
      case 'macro':
        mapping = {
          type: 'macro',
          tapAction: macroName || 'MACRO_COPY',
          macroSteps: [],
        };
        break;
      case 'tap-hold':
        mapping = {
          type: 'tap_hold',
          tapAction: tapAction || keyCode,
          holdAction: holdAction || 'MD_CTRL',
          threshold: parseInt(holdThreshold) || 200,
        };
        break;
      default:
        mapping = {
          type: 'simple',
          tapAction: keyCode,
        };
    }

    onSave(mapping);
    onClose();
  };

  const renderTabContent = () => {
    switch (selectedType) {
      case 'key':
        return (
          <div className="space-y-4">
            <p className="text-sm text-slate-400">
              Assign a simple key action to {keyCode}
            </p>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Target Key
              </label>
              <div className="grid grid-cols-4 gap-2">
                {commonKeys.map((key) => (
                  <button
                    key={key.id}
                    onClick={() => setTapAction(key.id)}
                    className={cn(
                      'px-3 py-2 text-sm font-medium rounded border transition-all',
                      tapAction === key.id
                        ? 'bg-primary-500 border-primary-400 text-white'
                        : 'bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600'
                    )}
                    type="button"
                  >
                    {key.label}
                  </button>
                ))}
              </div>
            </div>
            <Input
              label="Custom Key Code"
              value={tapAction}
              onChange={setTapAction}
              placeholder="e.g., VK_A"
            />
          </div>
        );

      case 'modifier':
        return (
          <div className="space-y-4">
            <p className="text-sm text-slate-400">
              Assign a modifier key to {keyCode}
            </p>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Modifier
              </label>
              <div className="grid grid-cols-2 gap-2">
                {modifiers.map((mod) => (
                  <button
                    key={mod.id}
                    onClick={() => setTapAction(mod.id)}
                    className={cn(
                      'px-3 py-2 text-sm font-medium rounded border transition-all',
                      tapAction === mod.id
                        ? 'bg-primary-500 border-primary-400 text-white'
                        : 'bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600'
                    )}
                    type="button"
                  >
                    {mod.label}
                  </button>
                ))}
              </div>
            </div>
          </div>
        );

      case 'lock':
        return (
          <div className="space-y-4">
            <p className="text-sm text-slate-400">
              Assign a lock key to {keyCode}
            </p>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Lock Type
              </label>
              <div className="grid grid-cols-3 gap-2">
                {locks.map((lock) => (
                  <button
                    key={lock.id}
                    onClick={() => setTapAction(lock.id)}
                    className={cn(
                      'px-3 py-2 text-sm font-medium rounded border transition-all',
                      tapAction === lock.id
                        ? 'bg-primary-500 border-primary-400 text-white'
                        : 'bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600'
                    )}
                    type="button"
                  >
                    {lock.label}
                  </button>
                ))}
              </div>
            </div>
          </div>
        );

      case 'layer':
        return (
          <div className="space-y-4">
            <p className="text-sm text-slate-400">
              Switch to a layer when {keyCode} is pressed
            </p>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Target Layer
              </label>
              <div className="grid grid-cols-3 gap-2">
                {layers.map((layer) => (
                  <button
                    key={layer}
                    onClick={() => setTargetLayer(layer)}
                    className={cn(
                      'px-3 py-2 text-sm font-medium rounded border transition-all',
                      targetLayer === layer
                        ? 'bg-primary-500 border-primary-400 text-white'
                        : 'bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600'
                    )}
                    type="button"
                  >
                    {layer}
                  </button>
                ))}
              </div>
            </div>
            <Input
              label="Custom Layer Name"
              value={targetLayer}
              onChange={setTargetLayer}
              placeholder="e.g., custom"
            />
          </div>
        );

      case 'macro':
        return (
          <div className="space-y-4">
            <p className="text-sm text-slate-400">
              Execute a macro when {keyCode} is pressed
            </p>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Common Macros
              </label>
              <div className="space-y-2">
                {commonMacros.map((macro) => (
                  <button
                    key={macro.id}
                    onClick={() => setMacroName(macro.id)}
                    className={cn(
                      'w-full px-4 py-3 text-left rounded border transition-all',
                      macroName === macro.id
                        ? 'bg-primary-500 border-primary-400 text-white'
                        : 'bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600'
                    )}
                    type="button"
                  >
                    <div className="font-medium">{macro.label}</div>
                    <div className="text-sm opacity-75">{macro.description}</div>
                  </button>
                ))}
              </div>
            </div>
            <Input
              label="Custom Macro ID"
              value={macroName}
              onChange={setMacroName}
              placeholder="e.g., MACRO_CUSTOM"
            />
          </div>
        );

      case 'tap-hold':
        return (
          <div className="space-y-4">
            <p className="text-sm text-slate-400">
              Configure tap/hold behavior for {keyCode}
            </p>
            <Input
              label="Tap Action"
              value={tapAction}
              onChange={setTapAction}
              placeholder="e.g., VK_A"
              helpText="Action when key is tapped quickly"
            />
            <Input
              label="Hold Action"
              value={holdAction}
              onChange={setHoldAction}
              placeholder="e.g., MD_CTRL"
              helpText="Action when key is held down"
            />
            <Input
              label="Hold Threshold (ms)"
              type="number"
              value={holdThreshold}
              onChange={setHoldThreshold}
              placeholder="200"
              helpText="Time in milliseconds before hold action triggers"
            />
            <div className="p-3 bg-slate-700/50 rounded border border-slate-600">
              <p className="text-sm text-slate-300">
                <strong>Tap:</strong> {tapAction || keyCode} &nbsp;|&nbsp;{' '}
                <strong>Hold ({holdThreshold}ms):</strong> {holdAction || 'None'}
              </p>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <Modal open={open} onClose={onClose} title={`Configure ${keyCode}`} className="max-w-2xl">
      <div className="space-y-4">
        {/* Tab Navigation */}
        <div role="tablist" className="flex flex-wrap gap-2 border-b border-slate-700 pb-3">
          {tabs.map((tab) => (
            <button
              key={tab.value}
              role="tab"
              aria-selected={selectedType === tab.value}
              aria-controls={`panel-${tab.value}`}
              onClick={() => setSelectedType(tab.value)}
              className={cn(
                'px-4 py-2 text-sm font-medium rounded-t transition-colors duration-150',
                'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2',
                selectedType === tab.value
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              )}
              type="button"
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Tab Content */}
        <div id={`panel-${selectedType}`} role="tabpanel" className="min-h-[300px]">
          {renderTabContent()}
        </div>

        {/* Action Buttons */}
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-slate-300 bg-slate-700 rounded hover:bg-slate-600 transition-colors"
            type="button"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 text-sm font-medium text-white bg-primary-500 rounded hover:bg-primary-600 transition-colors"
            type="button"
          >
            Save
          </button>
        </div>
      </div>
    </Modal>
  );
};

KeyAssignmentPopup.displayName = 'KeyAssignmentPopup';
