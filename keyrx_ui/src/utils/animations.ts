/**
 * Common animation variants for Framer Motion
 * Follows design.md animation requirements:
 * - Use transform/opacity only (GPU-accelerated)
 * - Respect prefers-reduced-motion
 * - Durations: quick (150ms), normal (300ms), slow (500ms)
 */

import { Variants, Transition } from 'framer-motion';

/**
 * Check if user prefers reduced motion
 */
export const prefersReducedMotion = (): boolean => {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
};

/**
 * Transition presets
 */
export const transitions = {
  quick: { duration: 0.15, ease: 'easeOut' } as Transition,
  normal: { duration: 0.3, ease: 'easeInOut' } as Transition,
  slow: { duration: 0.5, ease: 'easeInOut' } as Transition,
  spring: { type: 'spring', stiffness: 300, damping: 30 } as Transition,
};

/**
 * Fade in/out animation
 */
export const fadeVariants: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
  exit: { opacity: 0 },
};

/**
 * Scale + fade animation (for modals)
 */
export const scaleVariants: Variants = {
  hidden: { opacity: 0, scale: 0.95 },
  visible: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0.95 },
};

/**
 * Slide from bottom (for mobile bottom sheets)
 */
export const slideUpVariants: Variants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 20 },
};

/**
 * Slide from top (for dropdowns)
 */
export const slideDownVariants: Variants = {
  hidden: { opacity: 0, y: -10 },
  visible: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -10 },
};

/**
 * Slide from left (for sidebars)
 */
export const slideLeftVariants: Variants = {
  hidden: { opacity: 0, x: -20 },
  visible: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
};

/**
 * Stagger children animation (for lists)
 */
export const staggerContainerVariants: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05,
    },
  },
};

export const staggerItemVariants: Variants = {
  hidden: { opacity: 0, y: 10 },
  visible: { opacity: 1, y: 0 },
};

/**
 * Page transition variants
 */
export const pageVariants: Variants = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -10 },
};

/**
 * Backdrop variants (for modals)
 */
export const backdropVariants: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
  exit: { opacity: 0 },
};

/**
 * Hover animation for buttons
 */
export const hoverScale = {
  scale: 1.02,
  transition: transitions.quick,
};

/**
 * Active/tap animation for buttons
 */
export const tapScale = {
  scale: 0.98,
  transition: transitions.quick,
};

/**
 * Create animation props that respect reduced motion
 */
export const createAnimationProps = (
  variants: Variants,
  transition: Transition = transitions.normal
) => {
  if (prefersReducedMotion()) {
    return {
      initial: false,
      animate: 'visible',
      exit: false,
      transition: { duration: 0 },
    };
  }

  return {
    variants,
    initial: 'hidden',
    animate: 'visible',
    exit: 'exit',
    transition,
  };
};
