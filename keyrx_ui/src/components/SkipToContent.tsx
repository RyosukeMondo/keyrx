/**
 * SkipToContent Component
 *
 * Accessibility feature that allows keyboard users to skip
 * navigation and jump directly to main content.
 *
 * The link is visually hidden until focused, then appears
 * at the top of the page.
 */

import React from 'react';

export const SkipToContent: React.FC = () => {
  const handleClick = (event: React.MouseEvent<HTMLAnchorElement>) => {
    event.preventDefault();
    const main = document.querySelector('main');
    if (main) {
      main.focus();
      main.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <a
      href="#main-content"
      onClick={handleClick}
      className="
        absolute top-0 left-0 z-[9999]
        bg-primary-500 text-white
        px-4 py-3 rounded-md
        font-medium
        transform -translate-y-full
        focus:translate-y-0
        transition-transform duration-200
        focus:outline focus:outline-2 focus:outline-white focus:outline-offset-2
      "
      style={{
        // Position off-screen until focused
        clipPath: 'inset(50%)',
      }}
      onFocus={(e) => {
        e.currentTarget.style.clipPath = 'none';
      }}
      onBlur={(e) => {
        e.currentTarget.style.clipPath = 'inset(50%)';
      }}
    >
      Skip to main content
    </a>
  );
};
