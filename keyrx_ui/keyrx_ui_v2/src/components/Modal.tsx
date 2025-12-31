import React, { useEffect, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  scaleVariants,
  backdropVariants,
  transitions,
  createAnimationProps,
} from '@/utils/animations';

export interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  className?: string;
}

export const Modal = React.memo<ModalProps>(({ open, onClose, title, children, className = '' }) => {
  const modalRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);

  // Store the element that had focus when modal opened
  useEffect(() => {
    if (open) {
      previousActiveElement.current = document.activeElement as HTMLElement;
    }
  }, [open]);

  // Handle Escape key
  const handleEscape = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape' && open) {
        onClose();
      }
    },
    [open, onClose]
  );

  // Add/remove escape key listener
  useEffect(() => {
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [handleEscape]);

  // Focus trap implementation
  useEffect(() => {
    if (!open || !modalRef.current) return;

    const modal = modalRef.current;
    const content = modal.querySelector('[data-modal-content]');
    const focusableElements = modal.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const firstFocusable = focusableElements[0];
    const lastFocusable = focusableElements[focusableElements.length - 1];

    // Focus first element in content if content has focusable elements
    if (content) {
      const contentFocusable = content.querySelector<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (contentFocusable) {
        contentFocusable.focus();
      } else {
        firstFocusable?.focus();
      }
    } else {
      firstFocusable?.focus();
    }

    const handleTab = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return;

      if (event.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstFocusable) {
          event.preventDefault();
          lastFocusable?.focus();
        }
      } else {
        // Tab
        if (document.activeElement === lastFocusable) {
          event.preventDefault();
          firstFocusable?.focus();
        }
      }
    };

    modal.addEventListener('keydown', handleTab as EventListener);
    return () => modal.removeEventListener('keydown', handleTab as EventListener);
  }, [open]);

  // Return focus when modal closes
  useEffect(() => {
    if (!open && previousActiveElement.current) {
      previousActiveElement.current.focus();
      previousActiveElement.current = null;
    }
  }, [open]);

  // Prevent body scroll when modal is open
  useEffect(() => {
    if (open) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [open]);

  const handleBackdropClick = (event: React.MouseEvent) => {
    if (event.target === event.currentTarget) {
      onClose();
    }
  };

  const backdropProps = createAnimationProps(backdropVariants, transitions.quick);
  const modalProps = createAnimationProps(scaleVariants, transitions.normal);

  const modalContent = (
    <AnimatePresence mode="wait">
      {open && (
        <motion.div
          {...backdropProps}
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          onClick={handleBackdropClick}
          role="presentation"
        >
          {/* Backdrop */}
          <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" aria-hidden="true" />

          {/* Modal */}
          <motion.div
            {...modalProps}
            ref={modalRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-title"
            className={`relative bg-slate-800 rounded-lg shadow-2xl border border-slate-700 w-full max-w-lg max-h-[90vh] overflow-hidden ${className}`}
          >
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
              <h2 id="modal-title" className="text-lg font-semibold text-slate-100">
                {title}
              </h2>
              <button
                onClick={onClose}
                className="text-slate-400 hover:text-slate-100 transition-colors p-1 rounded hover:bg-slate-700"
                aria-label="Close modal"
              >
                <svg
                  className="w-5 h-5"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Content */}
            <div className="px-6 py-4 overflow-y-auto max-h-[calc(90vh-8rem)]" data-modal-content>
              {children}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );

  return createPortal(modalContent, document.body);
});

Modal.displayName = 'Modal';
