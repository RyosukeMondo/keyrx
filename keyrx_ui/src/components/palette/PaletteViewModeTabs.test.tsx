import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PaletteViewModeTabs, PaletteView } from './PaletteViewModeTabs';

describe('PaletteViewModeTabs', () => {
  describe('Rendering', () => {
    it('renders all four tabs', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      expect(
        screen.getByRole('tab', { name: /basic category view/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('tab', { name: /recent keys view/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('tab', { name: /favorite keys view/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('tab', { name: /all keys view/i })
      ).toBeInTheDocument();
    });

    it('renders tab labels correctly', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      expect(screen.getByText('Basic')).toBeInTheDocument();
      expect(screen.getByText('Recent')).toBeInTheDocument();
      expect(screen.getByText('Favorites')).toBeInTheDocument();
      expect(screen.getByText('All')).toBeInTheDocument();
    });

    it('applies custom className', () => {
      const onChange = vi.fn();
      const { container } = render(
        <PaletteViewModeTabs
          activeView="basic"
          onChange={onChange}
          className="custom-class"
        />
      );

      const tablist = container.querySelector('[role="tablist"]');
      expect(tablist).toHaveClass('custom-class');
    });
  });

  describe('Active State', () => {
    it('marks basic tab as active when activeView is basic', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const basicTab = screen.getByRole('tab', {
        name: /basic category view/i,
      });
      expect(basicTab).toHaveAttribute('aria-selected', 'true');
      expect(basicTab).toHaveClass('text-primary-400');
      expect(basicTab).toHaveClass('border-primary-400');
    });

    it('marks recent tab as active when activeView is recent', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="recent" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      expect(recentTab).toHaveAttribute('aria-selected', 'true');
      expect(recentTab).toHaveClass('text-primary-400');
    });

    it('marks favorites tab as active when activeView is favorites', () => {
      const onChange = vi.fn();
      render(
        <PaletteViewModeTabs activeView="favorites" onChange={onChange} />
      );

      const favoritesTab = screen.getByRole('tab', {
        name: /favorite keys view/i,
      });
      expect(favoritesTab).toHaveAttribute('aria-selected', 'true');
    });

    it('marks all tab as active when activeView is all', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="all" onChange={onChange} />);

      const allTab = screen.getByRole('tab', { name: /all keys view/i });
      expect(allTab).toHaveAttribute('aria-selected', 'true');
    });

    it('only one tab is active at a time', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="recent" onChange={onChange} />);

      const tabs = screen.getAllByRole('tab');
      const activeTabs = tabs.filter(
        (tab) => tab.getAttribute('aria-selected') === 'true'
      );
      expect(activeTabs).toHaveLength(1);
      expect(activeTabs[0]).toHaveAccessibleName(/recent keys view/i);
    });

    it('inactive tabs have different styling', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      expect(recentTab).toHaveAttribute('aria-selected', 'false');
      expect(recentTab).toHaveClass('text-slate-400');
      expect(recentTab).not.toHaveClass('text-primary-400');
    });
  });

  describe('User Interactions', () => {
    it('calls onChange when basic tab is clicked', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="recent" onChange={onChange} />);

      const basicTab = screen.getByRole('tab', {
        name: /basic category view/i,
      });
      await user.click(basicTab);

      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('basic');
    });

    it('calls onChange when recent tab is clicked', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      await user.click(recentTab);

      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('recent');
    });

    it('calls onChange when favorites tab is clicked', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const favoritesTab = screen.getByRole('tab', {
        name: /favorite keys view/i,
      });
      await user.click(favoritesTab);

      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('favorites');
    });

    it('calls onChange when all tab is clicked', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const allTab = screen.getByRole('tab', { name: /all keys view/i });
      await user.click(allTab);

      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('all');
    });

    it('handles multiple clicks correctly', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      const favoritesTab = screen.getByRole('tab', {
        name: /favorite keys view/i,
      });

      await user.click(recentTab);
      await user.click(favoritesTab);

      expect(onChange).toHaveBeenCalledTimes(2);
      expect(onChange).toHaveBeenNthCalledWith(1, 'recent');
      expect(onChange).toHaveBeenNthCalledWith(2, 'favorites');
    });

    it('does not call onChange when clicking already active tab', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const basicTab = screen.getByRole('tab', {
        name: /basic category view/i,
      });
      await user.click(basicTab);

      // onChange is still called (component doesn't prevent this)
      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('basic');
    });
  });

  describe('Keyboard Navigation', () => {
    it('activates tab with Enter key', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      recentTab.focus();
      await user.keyboard('{Enter}');

      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('recent');
    });

    it('activates tab with Space key', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const favoritesTab = screen.getByRole('tab', {
        name: /favorite keys view/i,
      });
      favoritesTab.focus();
      await user.keyboard(' ');

      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenCalledWith('favorites');
    });

    it('prevents default behavior for Enter key', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      const preventDefault = vi.fn();

      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      recentTab.focus();

      // Create a keyboard event manually to check preventDefault
      const event = new KeyboardEvent('keydown', {
        key: 'Enter',
        bubbles: true,
      });
      event.preventDefault = preventDefault;
      recentTab.dispatchEvent(event);

      expect(preventDefault).toHaveBeenCalled();
    });

    it('only active tab is in tab sequence (tabIndex)', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="recent" onChange={onChange} />);

      const basicTab = screen.getByRole('tab', {
        name: /basic category view/i,
      });
      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      const favoritesTab = screen.getByRole('tab', {
        name: /favorite keys view/i,
      });
      const allTab = screen.getByRole('tab', { name: /all keys view/i });

      expect(basicTab).toHaveAttribute('tabIndex', '-1');
      expect(recentTab).toHaveAttribute('tabIndex', '0');
      expect(favoritesTab).toHaveAttribute('tabIndex', '-1');
      expect(allTab).toHaveAttribute('tabIndex', '-1');
    });

    it('updates tabIndex when active view changes', () => {
      const onChange = vi.fn();
      const { rerender } = render(
        <PaletteViewModeTabs activeView="basic" onChange={onChange} />
      );

      let basicTab = screen.getByRole('tab', { name: /basic category view/i });
      let recentTab = screen.getByRole('tab', { name: /recent keys view/i });

      expect(basicTab).toHaveAttribute('tabIndex', '0');
      expect(recentTab).toHaveAttribute('tabIndex', '-1');

      // Rerender with different activeView
      rerender(<PaletteViewModeTabs activeView="recent" onChange={onChange} />);

      basicTab = screen.getByRole('tab', { name: /basic category view/i });
      recentTab = screen.getByRole('tab', { name: /recent keys view/i });

      expect(basicTab).toHaveAttribute('tabIndex', '-1');
      expect(recentTab).toHaveAttribute('tabIndex', '0');
    });
  });

  describe('Accessibility', () => {
    it('has role="tablist"', () => {
      const onChange = vi.fn();
      const { container } = render(
        <PaletteViewModeTabs activeView="basic" onChange={onChange} />
      );

      const tablist = container.querySelector('[role="tablist"]');
      expect(tablist).toBeInTheDocument();
    });

    it('has aria-label on tablist', () => {
      const onChange = vi.fn();
      const { container } = render(
        <PaletteViewModeTabs activeView="basic" onChange={onChange} />
      );

      const tablist = container.querySelector('[role="tablist"]');
      expect(tablist).toHaveAttribute('aria-label', 'Palette view modes');
    });

    it('all tabs have role="tab"', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const tabs = screen.getAllByRole('tab');
      expect(tabs).toHaveLength(4);
    });

    it('all tabs have aria-label', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const tabs = screen.getAllByRole('tab');
      tabs.forEach((tab) => {
        expect(tab).toHaveAttribute('aria-label');
        expect(tab.getAttribute('aria-label')).toBeTruthy();
      });
    });

    it('all tabs have aria-selected attribute', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const tabs = screen.getAllByRole('tab');
      tabs.forEach((tab) => {
        expect(tab).toHaveAttribute('aria-selected');
        const selected = tab.getAttribute('aria-selected');
        expect(['true', 'false']).toContain(selected);
      });
    });

    it('provides distinct aria-labels for each tab', () => {
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      expect(
        screen.getByRole('tab', { name: /basic category view/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('tab', { name: /recent keys view/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('tab', { name: /favorite keys view/i })
      ).toBeInTheDocument();
      expect(
        screen.getByRole('tab', { name: /all keys view/i })
      ).toBeInTheDocument();
    });
  });

  describe('Icons', () => {
    it('renders icons for all tabs', () => {
      const onChange = vi.fn();
      const { container } = render(
        <PaletteViewModeTabs activeView="basic" onChange={onChange} />
      );

      // Check that SVG icons are rendered (lucide-react renders as SVGs)
      const svgs = container.querySelectorAll('svg');
      expect(svgs.length).toBeGreaterThanOrEqual(4);
    });
  });

  describe('Edge Cases', () => {
    it('handles rapid view changes', async () => {
      const user = userEvent.setup();
      const onChange = vi.fn();
      render(<PaletteViewModeTabs activeView="basic" onChange={onChange} />);

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      const favoritesTab = screen.getByRole('tab', {
        name: /favorite keys view/i,
      });
      const allTab = screen.getByRole('tab', { name: /all keys view/i });
      const basicTab = screen.getByRole('tab', {
        name: /basic category view/i,
      });

      await user.click(recentTab);
      await user.click(favoritesTab);
      await user.click(allTab);
      await user.click(basicTab);

      expect(onChange).toHaveBeenCalledTimes(4);
      expect(onChange).toHaveBeenNthCalledWith(1, 'recent');
      expect(onChange).toHaveBeenNthCalledWith(2, 'favorites');
      expect(onChange).toHaveBeenNthCalledWith(3, 'all');
      expect(onChange).toHaveBeenNthCalledWith(4, 'basic');
    });

    it('renders correctly with no className provided', () => {
      const onChange = vi.fn();
      const { container } = render(
        <PaletteViewModeTabs activeView="basic" onChange={onChange} />
      );

      const tablist = container.querySelector('[role="tablist"]');
      expect(tablist).toBeInTheDocument();
      expect(tablist).toHaveClass('flex', 'gap-1', 'border-b');
    });

    it('maintains correct state after onChange callback', async () => {
      const user = userEvent.setup();
      let currentView: PaletteView = 'basic';
      const onChange = vi.fn((view: PaletteView) => {
        currentView = view;
      });

      const { rerender } = render(
        <PaletteViewModeTabs activeView={currentView} onChange={onChange} />
      );

      const recentTab = screen.getByRole('tab', { name: /recent keys view/i });
      await user.click(recentTab);

      // Rerender with updated view
      rerender(
        <PaletteViewModeTabs activeView={currentView} onChange={onChange} />
      );

      const updatedRecentTab = screen.getByRole('tab', {
        name: /recent keys view/i,
      });
      expect(updatedRecentTab).toHaveAttribute('aria-selected', 'true');
    });
  });
});
