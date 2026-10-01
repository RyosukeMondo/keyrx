import React from 'react';
import { keyNameParts } from '@/utils/keyNames';

interface KeyNameProps {
  /** Layout code, `VK_*` or DSL name of the key. */
  code: string;
  className?: string;
}

/**
 * A key's human name; in Japanese the English/DSL name follows in smaller
 * text (`無変換 Muhenkan`) so people can still match it to the docs and DSL.
 */
export const KeyName: React.FC<KeyNameProps> = ({ code, className }) => {
  const { primary, secondary } = keyNameParts(code);
  return (
    <span className={className}>
      {primary}
      {secondary && (
        <small className="ml-1 text-[0.75em] font-normal text-slate-300">
          {secondary}
        </small>
      )}
    </span>
  );
};
