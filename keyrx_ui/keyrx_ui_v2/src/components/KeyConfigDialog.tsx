import React, { useState, useEffect } from 'react';
import { Modal } from './Modal';
import { Button } from './Button';
import { Dropdown } from './Dropdown';
import { Input } from './Input';

interface KeyMapping {
  type: 'simple' | 'tap_hold' | 'macro' | 'layer_switch';
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
  macroSteps?: MacroStep[];
  targetLayer?: string;
}

interface MacroStep {
  type: 'press' | 'release' | 'delay';
  key?: string;
  duration?: number;
}

interface KeyConfigDialogProps {
  isOpen: boolean;
  onClose: () => void;
  keyCode: string;
  currentMapping?: KeyMapping;
  onSave: (keyCode: string, mapping: KeyMapping) => Promise<void>;
  availableKeys: string[];
  availableLayers: string[];
}

const ACTION_TYPES = [
  { value: 'tap_hold', label: 'Tap-Hold' },
  { value: 'simple', label: 'Simple Remap' },
  { value: 'macro', label: 'Macro' },
  { value: 'layer_switch', label: 'Layer Switch' },
];

export const KeyConfigDialog: React.FC<KeyConfigDialogProps> = ({
  isOpen,
  onClose,
  keyCode,
  currentMapping,
  onSave,
  availableKeys,
  availableLayers,
}) => {
  const [actionType, setActionType] = useState<KeyMapping['type']>(
    currentMapping?.type || 'tap_hold'
  );
  const [tapAction, setTapAction] = useState(currentMapping?.tapAction || '');
  const [holdAction, setHoldAction] = useState(
    currentMapping?.holdAction || ''
  );
  const [threshold, setThreshold] = useState(
    currentMapping?.threshold || 200
  );
  const [targetLayer, setTargetLayer] = useState(
    currentMapping?.targetLayer || ''
  );
  const [macroSteps, setMacroSteps] = useState<MacroStep[]>(
    currentMapping?.macroSteps || []
  );
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (currentMapping) {
      setActionType(currentMapping.type);
      setTapAction(currentMapping.tapAction || '');
      setHoldAction(currentMapping.holdAction || '');
      setThreshold(currentMapping.threshold || 200);
      setTargetLayer(currentMapping.targetLayer || '');
      setMacroSteps(currentMapping.macroSteps || []);
    } else {
      setActionType('tap_hold');
      setTapAction('');
      setHoldAction('');
      setThreshold(200);
      setTargetLayer('');
      setMacroSteps([]);
    }
  }, [currentMapping, isOpen]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const mapping: KeyMapping = {
        type: actionType,
      };

      if (actionType === 'simple') {
        mapping.tapAction = tapAction;
      } else if (actionType === 'tap_hold') {
        mapping.tapAction = tapAction;
        mapping.holdAction = holdAction;
        mapping.threshold = threshold;
      } else if (actionType === 'macro') {
        mapping.macroSteps = macroSteps;
      } else if (actionType === 'layer_switch') {
        mapping.targetLayer = targetLayer;
      }

      await onSave(keyCode, mapping);
      onClose();
    } finally {
      setIsSaving(false);
    }
  };

  const getPreviewText = (): string => {
    switch (actionType) {
      case 'simple':
        return tapAction
          ? `${keyCode} → ${tapAction}`
          : 'Select an output key';
      case 'tap_hold':
        if (!tapAction && !holdAction) {
          return 'Configure tap and hold actions';
        }
        return `Quick tap: ${keyCode} → ${tapAction || '?'}\nHold ${threshold}ms: ${keyCode} → ${holdAction || '?'}`;
      case 'macro':
        return macroSteps.length > 0
          ? `Macro: ${macroSteps.length} step(s)`
          : 'Add macro steps';
      case 'layer_switch':
        return targetLayer
          ? `Switch to layer: ${targetLayer}`
          : 'Select target layer';
      default:
        return '';
    }
  };

  const addMacroStep = () => {
    setMacroSteps([...macroSteps, { type: 'press', key: '' }]);
  };

  const removeMacroStep = (index: number) => {
    setMacroSteps(macroSteps.filter((_, i) => i !== index));
  };

  const updateMacroStep = (
    index: number,
    field: keyof MacroStep,
    value: string | number
  ) => {
    const updated = [...macroSteps];
    updated[index] = { ...updated[index], [field]: value };
    setMacroSteps(updated);
  };

  return (
    <Modal
      open={isOpen}
      onClose={onClose}
      title={`Configure Key: ${keyCode}`}
    >
      <div className="space-y-6">
        {/* Action Type Selector */}
        <div>
          <label className="block text-sm font-medium text-slate-100 mb-3">
            Action Type
          </label>
          <div className="grid grid-cols-2 gap-3">
            {ACTION_TYPES.map((type) => (
              <button
                key={type.value}
                onClick={() => setActionType(type.value as KeyMapping['type'])}
                className={`px-4 py-3 rounded-md text-sm font-medium transition-colors ${
                  actionType === type.value
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }`}
                aria-label={`Select ${type.label} action type`}
              >
                {type.label}
              </button>
            ))}
          </div>
        </div>

        {/* Simple Remap Form */}
        {actionType === 'simple' && (
          <div className="bg-slate-800 rounded-lg p-4">
            <label className="block text-sm font-medium text-slate-100 mb-2">
              Output Key
            </label>
            <Dropdown
              options={availableKeys.map((key) => ({
                value: key,
                label: key,
              }))}
              value={tapAction}
              onChange={setTapAction}
              searchable
              aria-label="Select output key for simple remap"
            />
          </div>
        )}

        {/* Tap-Hold Form */}
        {actionType === 'tap_hold' && (
          <div className="space-y-4">
            <div className="bg-slate-800 rounded-lg p-4">
              <label className="block text-sm font-medium text-slate-100 mb-2">
                Tap Action (quick press)
              </label>
              <Dropdown
                options={availableKeys.map((key) => ({
                  value: key,
                  label: key,
                }))}
                value={tapAction}
                onChange={setTapAction}
                searchable
                aria-label="Select tap action key"
              />
            </div>

            <div className="bg-slate-800 rounded-lg p-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-100 mb-2">
                  Hold Action (threshold exceeded)
                </label>
                <Dropdown
                  options={availableKeys.map((key) => ({
                    value: key,
                    label: key,
                  }))}
                  value={holdAction}
                  onChange={setHoldAction}
                  searchable
                  aria-label="Select hold action key"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-100 mb-2">
                  Threshold: {threshold} ms
                </label>
                <input
                  type="range"
                  min="10"
                  max="2000"
                  step="10"
                  value={threshold}
                  onChange={(e) => setThreshold(Number(e.target.value))}
                  className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer slider"
                  aria-label="Set hold threshold in milliseconds"
                />
                <div className="flex justify-between text-xs text-slate-400 mt-1">
                  <span>10ms</span>
                  <span>2000ms</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Macro Form */}
        {actionType === 'macro' && (
          <div className="bg-slate-800 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <label className="text-sm font-medium text-slate-100">
                Macro Steps
              </label>
              <Button
                variant="secondary"
                size="sm"
                onClick={addMacroStep}
                aria-label="Add macro step"
              >
                + Add Step
              </Button>
            </div>

            <div className="space-y-2 max-h-64 overflow-y-auto">
              {macroSteps.map((step, index) => (
                <div
                  key={index}
                  className="flex items-center gap-2 bg-slate-700 p-3 rounded"
                >
                  <select
                    value={step.type}
                    onChange={(e) =>
                      updateMacroStep(
                        index,
                        'type',
                        e.target.value as MacroStep['type']
                      )
                    }
                    className="bg-slate-600 text-slate-100 rounded px-2 py-1 text-sm"
                    aria-label={`Macro step ${index + 1} type`}
                  >
                    <option value="press">Press</option>
                    <option value="release">Release</option>
                    <option value="delay">Delay</option>
                  </select>

                  {step.type !== 'delay' ? (
                    <Dropdown
                      options={availableKeys.map((key) => ({
                        value: key,
                        label: key,
                      }))}
                      value={step.key || ''}
                      onChange={(value) => updateMacroStep(index, 'key', value)}
                      searchable
                      aria-label={`Macro step ${index + 1} key`}
                    />
                  ) : (
                    <Input
                      type="number"
                      value={String(step.duration || 0)}
                      onChange={(e) =>
                        updateMacroStep(index, 'duration', Number(e.target.value))
                      }
                      aria-label={`Macro step ${index + 1} delay duration`}
                    />
                  )}

                  <Button
                    variant="danger"
                    size="sm"
                    onClick={() => removeMacroStep(index)}
                    aria-label={`Remove macro step ${index + 1}`}
                  >
                    ×
                  </Button>
                </div>
              ))}

              {macroSteps.length === 0 && (
                <p className="text-sm text-slate-400 text-center py-4">
                  No macro steps defined. Click &quot;Add Step&quot; to begin.
                </p>
              )}
            </div>
          </div>
        )}

        {/* Layer Switch Form */}
        {actionType === 'layer_switch' && (
          <div className="bg-slate-800 rounded-lg p-4">
            <label className="block text-sm font-medium text-slate-100 mb-2">
              Target Layer
            </label>
            <Dropdown
              options={availableLayers.map((layer) => ({
                value: layer,
                label: layer,
              }))}
              value={targetLayer}
              onChange={setTargetLayer}
              searchable
              aria-label="Select target layer"
            />
          </div>
        )}

        {/* Preview Panel */}
        <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
          <label className="block text-sm font-medium text-slate-100 mb-2">
            Preview
          </label>
          <pre className="text-sm text-slate-300 whitespace-pre-wrap font-mono">
            {getPreviewText()}
          </pre>
        </div>

        {/* Action Buttons */}
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
          <Button
            variant="ghost"
            onClick={onClose}
            disabled={isSaving}
            aria-label="Cancel key configuration"
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleSave}
            loading={isSaving}
            disabled={isSaving}
            aria-label="Save key configuration"
          >
            Save Configuration
          </Button>
        </div>
      </div>
    </Modal>
  );
};
