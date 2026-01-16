/**
 * ConfigurationLayout - Responsive layout container for configuration page
 *
 * This component provides the main layout structure for the keyboard configuration
 * page. It uses CSS Grid to create a responsive layout with areas for:
 * - Device selection panel
 * - Tab navigation
 * - Keyboard visualization with layer switcher
 * - Legend
 * - Key configuration panel
 *
 * The layout is responsive and adapts to different screen sizes, stacking
 * vertically on mobile and using a multi-column layout on larger screens.
 *
 * @module ConfigurationLayout
 */

import React, { ReactNode } from 'react';

interface ConfigurationLayoutProps {
  /** Name of the current profile (for context) */
  profileName: string;
  /** Child components to render in layout areas */
  children: ReactNode;
}

/**
 * Responsive grid layout for configuration page
 *
 * This component doesn't manage any state - it purely handles layout.
 * All interactive functionality is managed by parent component and children.
 *
 * @example
 * ```tsx
 * <ConfigurationLayout profileName="Default">
 *   <DevicePanel />
 *   <KeyboardArea />
 *   <ConfigPanel />
 * </ConfigurationLayout>
 * ```
 */
export const ConfigurationLayout: React.FC<ConfigurationLayoutProps> = ({
  children,
}) => {
  return <div className="flex flex-col gap-4">{children}</div>;
};
