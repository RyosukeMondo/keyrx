import React, { useState, useMemo, useRef, useEffect, useCallback } from 'react';
import { Listbox, Transition } from '@headlessui/react';

interface DropdownOption {
  value: string;
  label: string;
}

interface DropdownProps {
  options: DropdownOption[];
  value: string;
  onChange: (value: string) => void;
  searchable?: boolean;
  'aria-label': string;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
}

export const Dropdown = React.memo<DropdownProps>(({
  options,
  value,
  onChange,
  searchable = false,
  'aria-label': ariaLabel,
  disabled = false,
  placeholder = 'Select an option',
  className = '',
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const searchInputRef = useRef<HTMLInputElement>(null);
  const openRef = useRef(false);

  // Filter options based on search term
  const filteredOptions = useMemo(() => {
    if (!searchable || !searchTerm) {
      return options;
    }

    const lowerSearchTerm = searchTerm.toLowerCase();
    return options.filter((option) =>
      option.label.toLowerCase().includes(lowerSearchTerm)
    );
  }, [options, searchTerm, searchable]);

  // Find selected option label
  const selectedOption = useMemo(
    () => options.find((opt) => opt.value === value),
    [options, value]
  );

  const handleOpenChange = useCallback((isOpen: boolean) => {
    openRef.current = isOpen;

    if (isOpen && searchable && searchInputRef.current) {
      // Focus search input when dropdown opens
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 0);
    } else if (!isOpen) {
      // Clear search when dropdown closes
      setSearchTerm('');
    }
  }, [searchable]);

  return (
    <Listbox
      value={value}
      onChange={onChange}
      disabled={disabled}
      as="div"
      className={`relative ${className}`}
    >
      {({ open }) => {
        // Track open state changes
        if (open !== openRef.current) {
          handleOpenChange(open);
        }

        return (
          <>
            <Listbox.Button
              aria-label={ariaLabel}
              className={`
                relative w-full rounded-md border border-slate-600
                bg-slate-700 py-3 px-4 text-left text-base
                text-slate-100 transition-all duration-150
                hover:border-slate-500 hover:bg-slate-600
                focus:outline focus:outline-2 focus:outline-primary-500
                focus:outline-offset-2
                disabled:cursor-not-allowed disabled:opacity-50
              `}
            >
              <span className="block truncate">
                {selectedOption?.label || placeholder}
              </span>
              <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                <svg
                  className={`h-5 w-5 text-slate-400 transition-transform duration-200 ${
                    open ? 'rotate-180' : ''
                  }`}
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M19 9l-7 7-7-7"
                  />
                </svg>
              </span>
            </Listbox.Button>

            <Transition
              show={open}
              enter="transition duration-150 ease-out"
              enterFrom="opacity-0 translate-y-[-10px]"
              enterTo="opacity-100 translate-y-0"
              leave="transition duration-100 ease-in"
              leaveFrom="opacity-100 translate-y-0"
              leaveTo="opacity-0 translate-y-[-10px]"
            >
              <Listbox.Options
                className={`
                  absolute z-dropdown mt-2 max-h-60 w-full overflow-auto
                  rounded-md border border-slate-600 bg-slate-800
                  py-1 shadow-lg focus:outline-none
                `}
              >
                {searchable && (
                  <div className="sticky top-0 z-10 border-b border-slate-600 bg-slate-800 p-2">
                    <input
                      ref={searchInputRef}
                      type="text"
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      placeholder="Search..."
                      className={`
                        w-full rounded border border-slate-600 bg-slate-700
                        px-3 py-2 text-sm text-slate-100
                        placeholder-slate-400
                        focus:border-primary-500 focus:outline-none
                      `}
                      onClick={(e) => e.stopPropagation()}
                    />
                  </div>
                )}

                {filteredOptions.length === 0 ? (
                  <div className="py-2 px-3 text-sm text-slate-400">
                    No options found
                  </div>
                ) : (
                  filteredOptions.map((option) => (
                    <Listbox.Option
                      key={option.value}
                      value={option.value}
                      className={({ active, selected }) =>
                        `
                          relative cursor-pointer select-none py-2 px-3
                          text-base transition-colors duration-100
                          ${active ? 'bg-primary-500 text-white' : 'text-slate-100'}
                          ${selected ? 'font-semibold' : 'font-normal'}
                        `
                      }
                    >
                      {({ selected }) => (
                        <>
                          <span className="block truncate">{option.label}</span>
                          {selected && (
                            <span className="absolute inset-y-0 right-0 flex items-center pr-3 text-primary-500">
                              <svg
                                className="h-5 w-5"
                                fill="currentColor"
                                viewBox="0 0 20 20"
                              >
                                <path
                                  fillRule="evenodd"
                                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                                  clipRule="evenodd"
                                />
                              </svg>
                            </span>
                          )}
                        </>
                      )}
                    </Listbox.Option>
                  ))
                )}
              </Listbox.Options>
            </Transition>
          </>
        );
      }}
    </Listbox>
  );
});

Dropdown.displayName = 'Dropdown';
