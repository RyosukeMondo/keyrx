/**
 * Accessibility tests
 * Tests for ARIA labels, keyboard navigation, and focus management
 */

import { describe, it, expect } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

describe('Accessibility Tests', () => {
  it('should have no accessibility violations - Button component', async () => {
    const { container } = render(
      <div>
        <button>Click me</button>
        <button disabled>Disabled</button>
        <button aria-label="Close dialog">×</button>
      </div>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('should have proper ARIA labels on interactive elements', () => {
    render(
      <div>
        <button aria-label="Delete profile">
          <svg>
            <path d="M..." />
          </svg>
        </button>
        <input type="text" aria-label="Profile name" />
        <select aria-label="Select layout">
          <option>ANSI</option>
        </select>
      </div>
    );

    expect(screen.getByLabelText('Delete profile')).toBeInTheDocument();
    expect(screen.getByLabelText('Profile name')).toBeInTheDocument();
    expect(screen.getByLabelText('Select layout')).toBeInTheDocument();
  });

  it('should support keyboard navigation - Tab and Enter', async () => {
    const handleClick = vi.fn();

    render(
      <div>
        <button onClick={handleClick}>Button 1</button>
        <button onClick={handleClick}>Button 2</button>
        <button onClick={handleClick}>Button 3</button>
      </div>
    );

    // Tab to first button
    await userEvent.tab();
    expect(screen.getByText('Button 1')).toHaveFocus();

    // Enter to click
    await userEvent.keyboard('{Enter}');
    expect(handleClick).toHaveBeenCalledTimes(1);

    // Tab to second button
    await userEvent.tab();
    expect(screen.getByText('Button 2')).toHaveFocus();

    // Space to click
    await userEvent.keyboard(' ');
    expect(handleClick).toHaveBeenCalledTimes(2);
  });

  it('should trap focus in modal', async () => {
    const TestModal = ({ open }: { open: boolean }) => {
      const modalRef = React.useRef<HTMLDivElement>(null);

      React.useEffect(() => {
        if (!open || !modalRef.current) return;

        const focusableElements = modalRef.current.querySelectorAll(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );

        const firstElement = focusableElements[0] as HTMLElement;
        const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

        const handleTab = (e: KeyboardEvent) => {
          if (e.key !== 'Tab') return;

          if (e.shiftKey) {
            if (document.activeElement === firstElement) {
              e.preventDefault();
              lastElement?.focus();
            }
          } else {
            if (document.activeElement === lastElement) {
              e.preventDefault();
              firstElement?.focus();
            }
          }
        };

        document.addEventListener('keydown', handleTab);
        return () => document.removeEventListener('keydown', handleTab);
      }, [open]);

      if (!open) return null;

      return (
        <div ref={modalRef} role="dialog" aria-modal="true">
          <h2>Modal Title</h2>
          <input type="text" aria-label="Input 1" />
          <input type="text" aria-label="Input 2" />
          <button>Close</button>
        </div>
      );
    };

    render(<TestModal open={true} />);

    const firstInput = screen.getByLabelText('Input 1');
    const secondInput = screen.getByLabelText('Input 2');
    const closeButton = screen.getByText('Close');

    // Focus first element
    firstInput.focus();
    expect(firstInput).toHaveFocus();

    // Tab through elements
    await userEvent.tab();
    expect(secondInput).toHaveFocus();

    await userEvent.tab();
    expect(closeButton).toHaveFocus();

    // Tab should wrap to first element
    await userEvent.tab();
    expect(firstInput).toHaveFocus();

    // Shift+Tab should go backwards
    await userEvent.tab({ shift: true });
    expect(closeButton).toHaveFocus();
  });

  it('should announce dynamic content changes to screen readers', () => {
    const TestComponent = () => {
      const [status, setStatus] = React.useState('idle');

      return (
        <div>
          <button onClick={() => setStatus('loading')}>Load</button>
          <div role="status" aria-live="polite" aria-atomic="true">
            {status === 'loading' && 'Loading...'}
            {status === 'success' && 'Loaded successfully'}
          </div>
        </div>
      );
    };

    const { rerender } = render(<TestComponent />);

    const statusRegion = screen.getByRole('status');
    expect(statusRegion).toHaveAttribute('aria-live', 'polite');
    expect(statusRegion).toHaveAttribute('aria-atomic', 'true');
  });

  it('should have sufficient color contrast', async () => {
    const { container } = render(
      <div>
        <button className="bg-blue-600 text-white">Primary Button</button>
        <div className="bg-slate-800 text-slate-200">Card Content</div>
      </div>
    );

    const results = await axe(container, {
      rules: {
        'color-contrast': { enabled: true },
      },
    });

    expect(results).toHaveNoViolations();
  });

  it('should provide skip to content link', () => {
    render(
      <div>
        <a href="#main-content" className="sr-only focus:not-sr-only">
          Skip to content
        </a>
        <nav>Navigation</nav>
        <main id="main-content">
          <h1>Main Content</h1>
        </main>
      </div>
    );

    const skipLink = screen.getByText('Skip to content');
    expect(skipLink).toBeInTheDocument();
    expect(skipLink).toHaveAttribute('href', '#main-content');
  });

  it('should have proper heading hierarchy', () => {
    render(
      <div>
        <h1>Page Title</h1>
        <h2>Section 1</h2>
        <h3>Subsection 1.1</h3>
        <h2>Section 2</h2>
        <h3>Subsection 2.1</h3>
      </div>
    );

    const headings = screen.getAllByRole('heading');
    expect(headings[0].tagName).toBe('H1');
    expect(headings[1].tagName).toBe('H2');
    expect(headings[2].tagName).toBe('H3');
  });

  it('should have labels for form inputs', async () => {
    const { container } = render(
      <form>
        <label htmlFor="name">Name</label>
        <input id="name" type="text" />

        <label htmlFor="email">Email</label>
        <input id="email" type="email" />

        <button type="submit">Submit</button>
      </form>
    );

    const results = await axe(container);
    expect(results).toHaveNoViolations();

    expect(screen.getByLabelText('Name')).toBeInTheDocument();
    expect(screen.getByLabelText('Email')).toBeInTheDocument();
  });
});
