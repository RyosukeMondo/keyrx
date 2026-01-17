import React, { useState } from 'react';

interface InputProps {
  type?: 'text' | 'number';
  value: string;
  onChange: (value: string) => void;
  error?: string;
  disabled?: boolean;
  'aria-label'?: string;
  label?: string;
  helpText?: string;
  maxLength?: number;
  placeholder?: string;
  className?: string;
  id?: string;
  name?: string;
}

export const Input = React.memo<InputProps>(
  ({
    type = 'text',
    value,
    onChange,
    error,
    disabled = false,
    'aria-label': ariaLabel,
    label,
    helpText,
    maxLength,
    placeholder,
    className = '',
    id,
    name,
  }) => {
    const [isFocused, setIsFocused] = useState(false);

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (!disabled) {
        onChange(e.target.value);
      }
    };

    const handleFocus = () => {
      setIsFocused(true);
    };

    const handleBlur = () => {
      setIsFocused(false);
    };

    // Base classes for all inputs
    const baseClasses = [
      'w-full',
      'px-3',
      'py-3',
      'rounded-md',
      'bg-slate-800',
      'text-slate-100',
      'border-2',
      'transition-all',
      'duration-150',
      'text-base',
      'font-sans',
    ].join(' ');

    // Border color classes based on state
    const borderClasses = error
      ? 'border-red-500'
      : isFocused
        ? 'border-primary-500'
        : 'border-slate-700 hover:border-slate-600';

    // Focus outline
    const focusClasses =
      'focus:outline focus:outline-2 focus:outline-primary-500 focus:outline-offset-2';

    // Disabled state
    const disabledClasses = disabled
      ? 'opacity-50 cursor-not-allowed bg-slate-900'
      : '';

    // Placeholder color
    const placeholderClasses = 'placeholder:text-slate-500';

    const inputClasses = [
      baseClasses,
      borderClasses,
      focusClasses,
      disabledClasses,
      placeholderClasses,
      className,
    ]
      .filter(Boolean)
      .join(' ');

    // Character count (current / max)
    const characterCount = maxLength ? value.length : null;
    const isNearLimit = maxLength && value.length >= maxLength * 0.9;

    return (
      <div className="w-full">
        {label && (
          <label
            htmlFor={id}
            className="block text-sm font-medium text-slate-300 mb-2"
          >
            {label}
          </label>
        )}
        <input
          type={type}
          value={value}
          onChange={handleChange}
          onFocus={handleFocus}
          onBlur={handleBlur}
          disabled={disabled}
          maxLength={maxLength}
          placeholder={placeholder}
          aria-label={ariaLabel || label}
          aria-invalid={!!error}
          aria-describedby={
            error ? `${id}-error` : helpText ? `${id}-help` : undefined
          }
          id={id}
          name={name}
          className={inputClasses}
        />

        {helpText && !error && (
          <div id={`${id}-help`} className="mt-2 text-sm text-slate-400">
            {helpText}
          </div>
        )}

        {/* Character counter */}
        {maxLength && (
          <div className="flex justify-end mt-1">
            <span
              className={`text-xs ${
                isNearLimit ? 'text-amber-500' : 'text-slate-400'
              }`}
            >
              {characterCount} / {maxLength}
            </span>
          </div>
        )}

        {/* Error message */}
        {error && (
          <div
            id={`${id}-error`}
            role="alert"
            aria-live="assertive"
            className="mt-2 text-sm text-red-500"
          >
            {error}
          </div>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
