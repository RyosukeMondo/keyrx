import React, { useState, useEffect } from 'react';
import { Modal } from '@/components/Modal';
import { Dropdown } from '@/components/Dropdown';
import { Input } from '@/components/Input';
import { Button } from '@/components/Button';
import type { KeyMapping } from '@/types/config';

/**
 * Props for the KeyMappingDialog component.
 */
export interface KeyMappingDialogProps {
  /** Whether the dialog is open */
  open: boolean;
  /** Callback when the dialog should close */
  onClose: () => void;
  /** Physical key code being configured (e.g., "CapsLock") */
  keyCode: string;
  /** Current mapping configuration (if editing existing) */
  currentMapping?: KeyMapping;
  /** Callback when save is clicked with the new mapping */
  onSave: (mapping: KeyMapping) => Promise<void>;
}

/**
 * Modal dialog for configuring individual key mappings.
 *
 * Provides a form interface for creating simple, tap_hold, macro, or layer_switch
 * key mappings. Validates input and emits the completed KeyMapping object.
 *
 * @example
 * ```tsx
 * <KeyMappingDialog
 *   open={isOpen}
 *   onClose={() => setIsOpen(false)}
 *   keyCode="CapsLock"
 *   currentMapping={existingMapping}
 *   onSave={async (mapping) => {
 *     await api.saveMapping(mapping);
 *     setIsOpen(false);
 *   }}
 * />
 * ```
 */
export const KeyMappingDialog = React.memo<KeyMappingDialogProps>(
  ({ open, onClose, keyCode, currentMapping, onSave }) => {
    const [mappingType, setMappingType] = useState<KeyMapping['type']>('simple');
    const [simpleAction, setSimpleAction] = useState('');
    const [tapAction, setTapAction] = useState('');
    const [holdAction, setHoldAction] = useState('');
    const [timeout, setTimeout] = useState('200');
    const [macroSequence, setMacroSequence] = useState('');
    const [layerName, setLayerName] = useState('');
    const [errors, setErrors] = useState<Record<string, string>>({});
    const [isSaving, setIsSaving] = useState(false);

    // Initialize form from currentMapping when dialog opens
    useEffect(() => {
      if (open && currentMapping) {
        setMappingType(currentMapping.type);

        if (currentMapping.type === 'simple' && currentMapping.simple) {
          setSimpleAction(currentMapping.simple);
        } else if (currentMapping.type === 'tap_hold' && currentMapping.tapHold) {
          setTapAction(currentMapping.tapHold.tap);
          setHoldAction(currentMapping.tapHold.hold);
          setTimeout(currentMapping.tapHold.timeoutMs.toString());
        } else if (currentMapping.type === 'macro' && currentMapping.macro) {
          setMacroSequence(currentMapping.macro.join(', '));
        } else if (currentMapping.type === 'layer_switch' && currentMapping.layer) {
          setLayerName(currentMapping.layer);
        }
      } else if (open && !currentMapping) {
        // Reset form for new mapping
        setMappingType('simple');
        setSimpleAction('');
        setTapAction('');
        setHoldAction('');
        setTimeout('200');
        setMacroSequence('');
        setLayerName('');
      }
      setErrors({});
    }, [open, currentMapping]);

    /**
     * Validates the form fields based on current mapping type.
     * @returns true if valid, false otherwise
     */
    const validateForm = (): boolean => {
      const newErrors: Record<string, string> = {};

      if (mappingType === 'simple') {
        if (!simpleAction.trim()) {
          newErrors.simpleAction = 'Simple action is required';
        }
      } else if (mappingType === 'tap_hold') {
        if (!tapAction.trim()) {
          newErrors.tapAction = 'Tap action is required';
        }
        if (!holdAction.trim()) {
          newErrors.holdAction = 'Hold action is required';
        }
        const timeoutNum = parseInt(timeout, 10);
        if (isNaN(timeoutNum) || timeoutNum < 100 || timeoutNum > 500) {
          newErrors.timeout = 'Timeout must be between 100-500ms';
        }
      } else if (mappingType === 'macro') {
        if (!macroSequence.trim()) {
          newErrors.macroSequence = 'Macro sequence is required';
        }
      } else if (mappingType === 'layer_switch') {
        if (!layerName.trim()) {
          newErrors.layerName = 'Layer name is required';
        }
      }

      setErrors(newErrors);
      return Object.keys(newErrors).length === 0;
    };

    /**
     * Handles the save button click.
     */
    const handleSave = async () => {
      if (!validateForm()) {
        return;
      }

      setIsSaving(true);
      try {
        const mapping: KeyMapping = {
          keyCode,
          type: mappingType,
        };

        // Add type-specific fields
        if (mappingType === 'simple') {
          mapping.simple = simpleAction.trim();
        } else if (mappingType === 'tap_hold') {
          mapping.tapHold = {
            tap: tapAction.trim(),
            hold: holdAction.trim(),
            timeoutMs: parseInt(timeout, 10),
          };
        } else if (mappingType === 'macro') {
          mapping.macro = macroSequence
            .split(',')
            .map((k) => k.trim())
            .filter((k) => k.length > 0);
        } else if (mappingType === 'layer_switch') {
          mapping.layer = layerName.trim();
        }

        await onSave(mapping);
        onClose();
      } catch (err) {
        setErrors({
          general: err instanceof Error ? err.message : 'Failed to save mapping',
        });
      } finally {
        setIsSaving(false);
      }
    };

    /**
     * Handles the cancel button click.
     */
    const handleCancel = () => {
      if (!isSaving) {
        onClose();
      }
    };

    // Mapping type options for dropdown
    const mappingTypeOptions = [
      { value: 'simple', label: 'Simple Mapping' },
      { value: 'tap_hold', label: 'Tap-Hold (Dual Function)' },
      { value: 'macro', label: 'Macro Sequence' },
      { value: 'layer_switch', label: 'Layer Switch' },
    ];

    return (
      <Modal open={open} onClose={handleCancel} title={`Configure ${keyCode}`}>
        <div className="space-y-4">
          {/* Mapping Type Selector */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Mapping Type
            </label>
            <div className="grid grid-cols-2 gap-2">
              {mappingTypeOptions.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => setMappingType(option.value as KeyMapping['type'])}
                  className={`
                    px-4 py-2 rounded-md text-sm font-medium transition-colors
                    ${
                      mappingType === option.value
                        ? 'bg-primary-500 text-white'
                        : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                    }
                    focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2
                  `}
                  aria-pressed={mappingType === option.value}
                  aria-label={`Select ${option.label}`}
                  disabled={isSaving}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          {/* Simple Mapping Form */}
          {mappingType === 'simple' && (
            <Input
              type="text"
              value={simpleAction}
              onChange={setSimpleAction}
              label="Action"
              placeholder="e.g., VK_A, VK_ENTER, MD_SHIFT"
              helpText="Enter the virtual key code to map to"
              error={errors.simpleAction}
              aria-label="Simple action"
              id="simple-action"
              disabled={isSaving}
            />
          )}

          {/* Tap-Hold Mapping Form */}
          {mappingType === 'tap_hold' && (
            <>
              <Input
                type="text"
                value={tapAction}
                onChange={setTapAction}
                label="Tap Action"
                placeholder="e.g., VK_ESCAPE"
                helpText="Action when key is quickly tapped"
                error={errors.tapAction}
                aria-label="Tap action"
                id="tap-action"
                disabled={isSaving}
              />
              <Input
                type="text"
                value={holdAction}
                onChange={setHoldAction}
                label="Hold Action"
                placeholder="e.g., MD_CTRL"
                helpText="Action when key is held down"
                error={errors.holdAction}
                aria-label="Hold action"
                id="hold-action"
                disabled={isSaving}
              />
              <div>
                <Input
                  type="number"
                  value={timeout}
                  onChange={setTimeout}
                  label="Timeout (ms)"
                  helpText="Time threshold between tap and hold (100-500ms)"
                  error={errors.timeout}
                  aria-label="Timeout in milliseconds"
                  id="timeout"
                  disabled={isSaving}
                />
                <div className="mt-2">
                  <input
                    type="range"
                    min="100"
                    max="500"
                    step="10"
                    value={timeout}
                    onChange={(e) => setTimeout(e.target.value)}
                    className="w-full"
                    aria-label="Timeout slider"
                    disabled={isSaving}
                  />
                  <div className="flex justify-between text-xs text-slate-400 mt-1">
                    <span>100ms</span>
                    <span>{timeout}ms</span>
                    <span>500ms</span>
                  </div>
                </div>
              </div>
            </>
          )}

          {/* Macro Mapping Form */}
          {mappingType === 'macro' && (
            <Input
              type="text"
              value={macroSequence}
              onChange={setMacroSequence}
              label="Macro Sequence"
              placeholder="e.g., VK_H, VK_E, VK_L, VK_L, VK_O"
              helpText="Enter key codes separated by commas"
              error={errors.macroSequence}
              aria-label="Macro sequence"
              id="macro-sequence"
              disabled={isSaving}
            />
          )}

          {/* Layer Switch Mapping Form */}
          {mappingType === 'layer_switch' && (
            <Input
              type="text"
              value={layerName}
              onChange={setLayerName}
              label="Layer Name"
              placeholder="e.g., nav, num, fn"
              helpText="Name of the layer to switch to"
              error={errors.layerName}
              aria-label="Layer name"
              id="layer-name"
              disabled={isSaving}
            />
          )}

          {/* General error message */}
          {errors.general && (
            <div className="p-3 bg-red-500/10 border border-red-500 rounded-md text-red-500 text-sm">
              {errors.general}
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-3 justify-end pt-4">
            <Button
              variant="secondary"
              onClick={handleCancel}
              aria-label="Cancel"
              disabled={isSaving}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleSave}
              aria-label="Save mapping"
              loading={isSaving}
              disabled={isSaving}
            >
              Save
            </Button>
          </div>
        </div>
      </Modal>
    );
  }
);

KeyMappingDialog.displayName = 'KeyMappingDialog';
