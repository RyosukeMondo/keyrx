import type { KeyMapping } from '@/types';
import type { SVGKey } from '../SVGKeyboard';

export type MappingType =
  | 'simple'
  | 'modifier'
  | 'lock'
  | 'tap_hold'
  | 'layer_active';

export interface MappingConfig extends Partial<KeyMapping> {
  tapAction?: string;
  holdAction?: string;
  threshold?: number;
  modifierKey?: string;
  lockKey?: string;
  targetLayer?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: Record<string, string>;
}

export interface MappingConfigFormProps {
  mappingType: MappingType;
  currentConfig?: MappingConfig;
  onChange: (config: MappingConfig) => void;
  onValidate?: (config: MappingConfig) => ValidationResult;
  layoutKeys?: SVGKey[];
  enableKeyboardView?: boolean;
}
