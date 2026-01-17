import React, { useState, useRef, useEffect } from 'react';
import { Pencil } from 'lucide-react';

export interface InlineEditProps {
  value: string;
  onSave: (newValue: string) => void;
  className?: string;
  placeholder?: string;
  maxLength?: number;
  multiline?: boolean;
  disabled?: boolean;
  ariaLabel?: string;
}

/**
 * InlineEdit Component
 *
 * Displays text that becomes editable when clicked.
 * Shows a pencil icon on hover to indicate editability.
 * Auto-saves on blur with debounce.
 */
export const InlineEdit = React.memo<InlineEditProps>(
  ({
    value,
    onSave,
    className = '',
    placeholder = 'Click to edit',
    maxLength,
    multiline = false,
    disabled = false,
    ariaLabel,
  }) => {
    const [isEditing, setIsEditing] = useState(false);
    const [editValue, setEditValue] = useState(value);
    const [showPencil, setShowPencil] = useState(false);
    const inputRef = useRef<HTMLInputElement | HTMLTextAreaElement>(null);
    const saveTimeoutRef = useRef<NodeJS.Timeout>();

    // Update editValue when value prop changes
    useEffect(() => {
      setEditValue(value);
    }, [value]);

    // Focus input when entering edit mode
    useEffect(() => {
      if (isEditing && inputRef.current) {
        inputRef.current.focus();
        inputRef.current.select();
      }
    }, [isEditing]);

    const handleClick = () => {
      if (!disabled) {
        setIsEditing(true);
      }
    };

    const handleBlur = () => {
      // Debounce save by 500ms
      saveTimeoutRef.current = setTimeout(() => {
        const trimmedValue = editValue.trim();

        // Only save if value changed and is not empty
        if (trimmedValue && trimmedValue !== value) {
          onSave(trimmedValue);
        } else if (!trimmedValue) {
          // Revert to original value if empty
          setEditValue(value);
        }

        setIsEditing(false);
      }, 500);
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !multiline) {
        e.preventDefault();
        (e.target as HTMLElement).blur();
      } else if (e.key === 'Escape') {
        // Revert on escape
        setEditValue(value);
        setIsEditing(false);
        if (saveTimeoutRef.current) {
          clearTimeout(saveTimeoutRef.current);
        }
      }
    };

    // Cleanup timeout on unmount
    useEffect(() => {
      return () => {
        if (saveTimeoutRef.current) {
          clearTimeout(saveTimeoutRef.current);
        }
      };
    }, []);

    const displayValue = editValue || placeholder;
    const showPlaceholder = !editValue;

    if (isEditing) {
      const baseClassName = `${className} bg-slate-700 border border-blue-500 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-500`;
      const commonProps = {
        ref: inputRef,
        value: editValue,
        onChange: (
          e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
        ) => setEditValue(e.target.value),
        onBlur: handleBlur,
        onKeyDown: handleKeyDown,
        maxLength,
        'aria-label': ariaLabel,
      };

      if (multiline) {
        return (
          <textarea
            {...commonProps}
            rows={3}
            className={`${baseClassName} resize-none w-full`}
          />
        );
      }

      return (
        <input
          type="text"
          {...commonProps}
          className={`${baseClassName} w-full`}
        />
      );
    }

    return (
      <div
        className={`relative group ${
          disabled ? 'cursor-not-allowed opacity-60' : 'cursor-pointer'
        } pr-6`}
        onClick={handleClick}
        onMouseEnter={() => !disabled && setShowPencil(true)}
        onMouseLeave={() => setShowPencil(false)}
        role="button"
        tabIndex={disabled ? -1 : 0}
        onKeyDown={(e) => {
          if (!disabled && (e.key === 'Enter' || e.key === ' ')) {
            e.preventDefault();
            handleClick();
          }
        }}
        aria-label={ariaLabel || `Edit ${value || 'text'}`}
      >
        <span
          className={`${className} ${
            showPlaceholder ? 'text-slate-500 italic' : ''
          }`}
        >
          {displayValue}
        </span>
        {!disabled && (
          <Pencil
            size={14}
            className={`absolute right-0 top-1/2 -translate-y-1/2 text-slate-400 transition-opacity ${
              showPencil ? 'opacity-70' : 'opacity-0'
            }`}
            aria-hidden="true"
          />
        )}
      </div>
    );
  }
);

InlineEdit.displayName = 'InlineEdit';
