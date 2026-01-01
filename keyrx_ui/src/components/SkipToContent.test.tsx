import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SkipToContent } from './SkipToContent';

describe('SkipToContent', () => {
  beforeEach(() => {
    // Create a main element for testing
    const main = document.createElement('main');
    main.id = 'main-content';
    main.setAttribute('tabindex', '-1');
    document.body.appendChild(main);

    return () => {
      document.body.removeChild(main);
    };
  });

  it('renders skip link', () => {
    render(<SkipToContent />);
    expect(screen.getByText('Skip to main content')).toBeInTheDocument();
  });

  it('has correct href attribute', () => {
    render(<SkipToContent />);
    const link = screen.getByText('Skip to main content');
    expect(link).toHaveAttribute('href', '#main-content');
  });

  it('focuses main element when clicked', async () => {
    const user = userEvent.setup();
    render(<SkipToContent />);

    const link = screen.getByText('Skip to main content');
    const main = document.querySelector('main');

    await user.click(link);

    // Check that main element was focused
    expect(document.activeElement).toBe(main);
  });

  it('prevents default link behavior', async () => {
    const user = userEvent.setup();
    render(<SkipToContent />);

    const link = screen.getByText('Skip to main content');
    const clickEvent = new MouseEvent('click', { bubbles: true, cancelable: true });
    const preventDefaultSpy = vi.spyOn(clickEvent, 'preventDefault');

    link.dispatchEvent(clickEvent);

    expect(preventDefaultSpy).toHaveBeenCalled();
  });

  it('changes clip-path on focus', () => {
    render(<SkipToContent />);
    const link = screen.getByText('Skip to main content') as HTMLAnchorElement;

    // Initially has clip-path
    expect(link.style.clipPath).toBe('inset(50%)');

    // Focus the link
    link.focus();

    // Should remove clip-path
    expect(link.style.clipPath).toBe('none');
  });

  it('restores clip-path on blur', () => {
    render(<SkipToContent />);
    const link = screen.getByText('Skip to main content') as HTMLAnchorElement;

    // Focus the link
    link.focus();
    expect(link.style.clipPath).toBe('none');

    // Blur the link
    link.blur();
    expect(link.style.clipPath).toBe('inset(50%)');
  });

  it('has high z-index for visibility', () => {
    render(<SkipToContent />);
    const link = screen.getByText('Skip to main content');
    expect(link).toHaveClass('z-[9999]');
  });

  it('has accessibility styles', () => {
    render(<SkipToContent />);
    const link = screen.getByText('Skip to main content');
    expect(link).toHaveClass(
      'focus:outline',
      'focus:outline-2',
      'focus:outline-white',
      'focus:outline-offset-2'
    );
  });

  it('scrolls main element into view', async () => {
    const user = userEvent.setup();
    const scrollIntoViewMock = vi.fn();
    const main = document.querySelector('main');

    if (main) {
      main.scrollIntoView = scrollIntoViewMock;
    }

    render(<SkipToContent />);

    const link = screen.getByText('Skip to main content');
    await user.click(link);

    expect(scrollIntoViewMock).toHaveBeenCalledWith({ behavior: 'smooth' });
  });

  it('handles missing main element gracefully', async () => {
    const user = userEvent.setup();
    const main = document.querySelector('main');
    if (main) {
      document.body.removeChild(main);
    }

    render(<SkipToContent />);

    const link = screen.getByText('Skip to main content');

    // Should not throw error
    expect(async () => {
      await user.click(link);
    }).not.toThrow();
  });
});
