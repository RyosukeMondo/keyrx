/**
 * PageTransition Component
 *
 * Wraps page content with fade transition animations.
 * Used in AppShell to animate between route changes.
 */

import { motion } from 'framer-motion';
import { pageVariants, transitions, createAnimationProps } from '@/utils/animations';

interface PageTransitionProps {
  children: React.ReactNode;
  className?: string;
}

export const PageTransition: React.FC<PageTransitionProps> = ({
  children,
  className = '',
}) => {
  const animationProps = createAnimationProps(pageVariants, transitions.normal);

  return (
    <motion.div {...animationProps} className={className} aria-live="polite">
      {children}
    </motion.div>
  );
};
