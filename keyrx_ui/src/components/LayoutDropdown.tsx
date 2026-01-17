import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  Listbox,
  ListboxButton,
  ListboxOptions,
  ListboxOption,
  Transition,
} from '@headlessui/react';
import {
  useFloating,
  offset,
  flip,
  shift,
  autoUpdate,
  FloatingPortal,
} from '@floating-ui/react';
import { LayoutPreview } from './LayoutPreview';
import {
  type LayoutType,
  type LayoutDropdownOption,
  LAYOUT_OPTIONS,
} from '../contexts/LayoutPreviewContext';

interface LayoutDropdownProps {
  options?: LayoutDropdownOption[];
  value: string;
  onChange: (value: string) => void;
  'aria-label': string;
  disabled?: boolean;
  className?: string;
  showPreviewOnHover?: boolean;
  compact?: boolean;
}

export const LayoutDropdown = React.memo<LayoutDropdownProps>(
  ({
    options = LAYOUT_OPTIONS,
    value,
    onChange,
    'aria-label': ariaLabel,
    disabled = false,
    className = '',
    showPreviewOnHover = true,
    compact = false,
  }) => {
    const [hoveredOption, setHoveredOption] = useState<LayoutType | null>(null);
    const [isPreviewVisible, setIsPreviewVisible] = useState(false);
    const hoverTimeoutRef = useRef<NodeJS.Timeout>();
    const leaveTimeoutRef = useRef<NodeJS.Timeout>();
    const optionRefs = useRef<Map<string, HTMLDivElement>>(new Map());

    // Floating UI for preview positioning
    const { x, y, refs, strategy } = useFloating({
      placement: 'right-start',
      middleware: [
        offset(12),
        flip({ fallbackPlacements: ['left-start', 'bottom', 'top'] }),
        shift({ padding: 8 }),
      ],
      whileElementsMounted: autoUpdate,
    });

    const selectedOption = options.find((opt) => opt.value === value);

    const handleOptionHover = useCallback(
      (optionValue: LayoutType) => {
        if (!showPreviewOnHover) return;

        clearTimeout(leaveTimeoutRef.current);
        clearTimeout(hoverTimeoutRef.current);

        hoverTimeoutRef.current = setTimeout(() => {
          setHoveredOption(optionValue);
          setIsPreviewVisible(true);

          const optionEl = optionRefs.current.get(optionValue);
          if (optionEl) {
            refs.setReference(optionEl);
          }
        }, 100);
      },
      [showPreviewOnHover, refs]
    );

    const handleOptionLeave = useCallback(() => {
      clearTimeout(hoverTimeoutRef.current);

      leaveTimeoutRef.current = setTimeout(() => {
        setIsPreviewVisible(false);
        setHoveredOption(null);
      }, 150);
    }, []);

    const handlePreviewEnter = useCallback(() => {
      clearTimeout(leaveTimeoutRef.current);
    }, []);

    const handlePreviewLeave = useCallback(() => {
      leaveTimeoutRef.current = setTimeout(() => {
        setIsPreviewVisible(false);
        setHoveredOption(null);
      }, 100);
    }, []);

    useEffect(() => {
      return () => {
        clearTimeout(hoverTimeoutRef.current);
        clearTimeout(leaveTimeoutRef.current);
      };
    }, []);

    const setOptionRef = useCallback(
      (value: string) => (el: HTMLDivElement | null) => {
        if (el) {
          optionRefs.current.set(value, el);
        } else {
          optionRefs.current.delete(value);
        }
      },
      []
    );

    return (
      <>
        <Listbox value={value} onChange={onChange} disabled={disabled}>
          {({ open }) => (
            <div className={`relative ${className}`}>
              <ListboxButton
                aria-label={ariaLabel}
                className={`
                relative rounded-md border border-slate-600
                bg-slate-700 text-left
                text-slate-100 transition-all duration-150
                hover:border-slate-500 hover:bg-slate-600
                focus:outline focus:outline-2 focus:outline-primary-500
                focus:outline-offset-2
                disabled:cursor-not-allowed disabled:opacity-50
                ${
                  compact
                    ? 'py-1.5 px-3 text-sm min-w-[140px]'
                    : 'w-full py-3 px-4 text-base'
                }
              `}
              >
                <span className="block truncate">
                  {selectedOption?.label || 'Select layout'}
                </span>
                <span
                  className={`pointer-events-none absolute inset-y-0 right-0 flex items-center ${
                    compact ? 'pr-2' : 'pr-3'
                  }`}
                >
                  <svg
                    className={`text-slate-400 transition-transform duration-200 ${
                      open ? 'rotate-180' : ''
                    } ${compact ? 'h-4 w-4' : 'h-5 w-5'}`}
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
              </ListboxButton>

              <Transition
                show={open}
                enter="transition duration-150 ease-out"
                enterFrom="opacity-0 scale-95"
                enterTo="opacity-100 scale-100"
                leave="transition duration-100 ease-in"
                leaveFrom="opacity-100 scale-100"
                leaveTo="opacity-0 scale-95"
                afterLeave={() => {
                  setIsPreviewVisible(false);
                  setHoveredOption(null);
                }}
              >
                <ListboxOptions
                  anchor="bottom start"
                  className={`
                  z-dropdown max-h-60 overflow-auto
                  rounded-md border border-slate-600 bg-slate-800
                  py-1 shadow-lg focus:outline-none
                  [--anchor-gap:8px]
                  ${compact ? 'min-w-[140px]' : 'w-[var(--button-width)]'}
                `}
                >
                  {options.map((option) => (
                    <ListboxOption
                      key={option.value}
                      value={option.value}
                      className={({ focus, selected }) =>
                        `
                        relative cursor-pointer select-none transition-colors duration-100
                        ${
                          compact
                            ? 'py-1.5 px-3 text-sm'
                            : 'py-2 px-3 text-base'
                        }
                        ${
                          focus ? 'bg-primary-500 text-white' : 'text-slate-100'
                        }
                        ${selected ? 'font-semibold' : 'font-normal'}
                      `
                      }
                    >
                      {({ selected }) => (
                        <div
                          ref={setOptionRef(option.value)}
                          onMouseEnter={() => handleOptionHover(option.value)}
                          onMouseLeave={handleOptionLeave}
                        >
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
                        </div>
                      )}
                    </ListboxOption>
                  ))}
                </ListboxOptions>
              </Transition>
            </div>
          )}
        </Listbox>

        {/* Hover preview - rendered in portal to escape overflow:hidden */}
        {showPreviewOnHover && isPreviewVisible && hoveredOption && (
          <FloatingPortal>
            <div
              ref={refs.setFloating} // eslint-disable-line react-hooks/refs
              className="z-tooltip"
              style={{
                position: strategy,
                top: y ?? 0,
                left: x ?? 0,
              }}
              onMouseEnter={handlePreviewEnter}
              onMouseLeave={handlePreviewLeave}
            >
              <LayoutPreview
                layout={hoveredOption}
                scale={0.35}
                showLabels={true}
              />
            </div>
          </FloatingPortal>
        )}
      </>
    );
  }
);

LayoutDropdown.displayName = 'LayoutDropdown';
