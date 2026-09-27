import React from 'react';
import { X, Radio } from 'lucide-react';
import { KeyPalette } from '../KeyPalette';
import { SVGKeyboard, type SVGKey } from '../SVGKeyboard';
import { KeyboardViewToggle } from './KeyboardViewToggle';

export interface SimpleMappingFormProps {
  enableKeyboardView: boolean;
  useKeyboard: boolean;
  onUseKeyboardChange: (useKeyboard: boolean) => void;
  tapAction: string;
  onTapActionChange: (value: string) => void;
  isListening: boolean;
  listeningFor: 'tap' | 'hold' | null;
  onStartListening: (target: 'tap' | 'hold') => void;
  layoutKeys: SVGKey[];
  error?: string;
}

/** Render simple mapping form */
export function SimpleMappingForm({
  enableKeyboardView,
  useKeyboard,
  onUseKeyboardChange,
  tapAction,
  onTapActionChange,
  isListening,
  listeningFor,
  onStartListening,
  layoutKeys,
  error,
}: SimpleMappingFormProps) {
  return (
    <div>
      {/* View toggle */}
      {enableKeyboardView && (
        <KeyboardViewToggle
          useKeyboard={useKeyboard}
          onChange={onUseKeyboardChange}
        />
      )}

      {/* Selected key display */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {tapAction && (
            <>
              <div className="px-4 py-2 bg-primary-500/20 border border-primary-500 rounded-lg">
                <span className="text-xl font-bold text-primary-300 font-mono">
                  {tapAction}
                </span>
              </div>
              <button
                onClick={() => onTapActionChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
        <button
          onClick={() => onStartListening('tap')}
          disabled={isListening}
          className={`px-3 py-1.5 rounded text-xs font-medium flex items-center gap-1.5 ${
            isListening && listeningFor === 'tap'
              ? 'bg-green-500 text-white animate-pulse'
              : 'bg-slate-600 text-slate-300 hover:bg-slate-500'
          }`}
          title="Press any key to capture"
        >
          <Radio className="w-4 h-4" />
          {isListening && listeningFor === 'tap'
            ? 'Listening...'
            : 'Listen for Key'}
        </button>
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Click a key below or use Listen button
      </p>

      {/* Key selection */}
      {useKeyboard && layoutKeys.length > 0 ? (
        <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
          <SVGKeyboard
            keys={layoutKeys}
            keyMappings={new Map()}
            onKeyClick={onTapActionChange}
            className="w-full"
          />
        </div>
      ) : (
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => onTapActionChange(key.id)}
            selectedKey={
              tapAction
                ? { id: tapAction, label: tapAction, category: 'basic' }
                : null
            }
          />
        </div>
      )}

      {error && <p className="text-xs text-red-400 mt-2">{error}</p>}
    </div>
  );
}

export interface ModifierMappingFormProps {
  enableKeyboardView: boolean;
  useKeyboard: boolean;
  onUseKeyboardChange: (useKeyboard: boolean) => void;
  modifierKey: string;
  onModifierKeyChange: (value: string) => void;
  layoutKeys: SVGKey[];
  error?: string;
}

/** Render modifier mapping form */
export function ModifierMappingForm({
  enableKeyboardView,
  useKeyboard,
  onUseKeyboardChange,
  modifierKey,
  onModifierKeyChange,
  layoutKeys,
  error,
}: ModifierMappingFormProps) {
  return (
    <div>
      {enableKeyboardView && (
        <KeyboardViewToggle
          useKeyboard={useKeyboard}
          onChange={onUseKeyboardChange}
        />
      )}

      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {modifierKey && (
            <>
              <div className="px-4 py-2 bg-cyan-500/20 border border-cyan-500 rounded-lg">
                <span className="text-xl font-bold text-cyan-300 font-mono">
                  {modifierKey}
                </span>
              </div>
              <button
                onClick={() => onModifierKeyChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Select a modifier key (Ctrl, Shift, Alt, etc.)
      </p>

      {useKeyboard && layoutKeys.length > 0 ? (
        <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
          <SVGKeyboard
            keys={layoutKeys}
            keyMappings={new Map()}
            onKeyClick={onModifierKeyChange}
            className="w-full"
          />
        </div>
      ) : (
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => onModifierKeyChange(key.id)}
            selectedKey={
              modifierKey
                ? {
                    id: modifierKey,
                    label: modifierKey,
                    category: 'modifiers',
                  }
                : null
            }
          />
        </div>
      )}

      {error && <p className="text-xs text-red-400 mt-2">{error}</p>}
    </div>
  );
}

export interface LockMappingFormProps {
  enableKeyboardView: boolean;
  useKeyboard: boolean;
  onUseKeyboardChange: (useKeyboard: boolean) => void;
  lockKey: string;
  onLockKeyChange: (value: string) => void;
  layoutKeys: SVGKey[];
  error?: string;
}

/** Render lock mapping form */
export function LockMappingForm({
  enableKeyboardView,
  useKeyboard,
  onUseKeyboardChange,
  lockKey,
  onLockKeyChange,
  layoutKeys,
  error,
}: LockMappingFormProps) {
  return (
    <div>
      {enableKeyboardView && (
        <KeyboardViewToggle
          useKeyboard={useKeyboard}
          onChange={onUseKeyboardChange}
        />
      )}

      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {lockKey && (
            <>
              <div className="px-4 py-2 bg-purple-500/20 border border-purple-500 rounded-lg">
                <span className="text-xl font-bold text-purple-300 font-mono">
                  {lockKey}
                </span>
              </div>
              <button
                onClick={() => onLockKeyChange('')}
                className="p-1 text-slate-400 hover:text-red-400"
                title="Clear selection"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
      </div>
      <p className="text-xs text-slate-400 mb-2">
        Select a lock key (CapsLock, NumLock, etc.)
      </p>

      {useKeyboard && layoutKeys.length > 0 ? (
        <div className="border border-slate-600 rounded-lg overflow-auto max-h-96 bg-slate-900">
          <SVGKeyboard
            keys={layoutKeys}
            keyMappings={new Map()}
            onKeyClick={onLockKeyChange}
            className="w-full"
          />
        </div>
      ) : (
        <div className="border border-slate-600 rounded-lg overflow-y-auto max-h-72">
          <KeyPalette
            compact
            onKeySelect={(key) => onLockKeyChange(key.id)}
            selectedKey={
              lockKey
                ? { id: lockKey, label: lockKey, category: 'special' }
                : null
            }
          />
        </div>
      )}

      {error && <p className="text-xs text-red-400 mt-2">{error}</p>}
    </div>
  );
}
