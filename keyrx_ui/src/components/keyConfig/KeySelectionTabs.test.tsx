import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeySelectionTabs, type KeySelectionTab } from './KeySelectionTabs';
import type { SVGKey } from '../SVGKeyboard';

// Mock SVGKeyboard component
vi.mock('../SVGKeyboard', () => ({
  SVGKeyboard: ({
    onKeyClick,
    keys,
  }: {
    onKeyClick: (key: string) => void;
    keys: SVGKey[];
  }) => (
    <div data-testid="svg-keyboard">
      <button onClick={() => onKeyClick('A')}>Mock Key A</button>
      <div>Keys count: {keys.length}</div>
    </div>
  ),
}));

describe('KeySelectionTabs', () => {
  const mockOnTabChange = vi.fn();
  const mockOnKeySelect = vi.fn();
  const mockLayoutKeys: SVGKey[] = [
    { id: 'A', x: 0, y: 0, width: 1, height: 1, label: 'A' },
    { id: 'B', x: 1, y: 0, width: 1, height: 1, label: 'B' },
  ];

  afterEach(() => {
    mockOnTabChange.mockClear();
    mockOnKeySelect.mockClear();
  });

  describe('Tab Rendering', () => {
    it('renders all available tabs', () => {
      const availableTabs: KeySelectionTab[] = ['keyboard', 'modifier', 'lock'];
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={availableTabs}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText('Keyboard')).toBeInTheDocument();
      expect(screen.getByText('Modifier')).toBeInTheDocument();
      expect(screen.getByText('Lock')).toBeInTheDocument();
    });

    it('renders only specified tabs', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText('Keyboard')).toBeInTheDocument();
      expect(screen.getByText('Modifier')).toBeInTheDocument();
      expect(screen.queryByText('Lock')).not.toBeInTheDocument();
    });

    it('highlights the active tab', () => {
      render(
        <KeySelectionTabs
          activeTab="modifier"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier', 'lock']}
          onKeySelect={mockOnKeySelect}
        />
      );

      const modifierTab = screen.getByRole('tab', { selected: true });
      expect(modifierTab).toHaveTextContent('Modifier');
      expect(modifierTab).toHaveClass('text-primary-400');
    });

    it('renders layer tab when available', () => {
      render(
        <KeySelectionTabs
          activeTab="layer"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'layer']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByText('Layer')).toBeInTheDocument();
    });
  });

  describe('Tab Switching', () => {
    it('calls onTabChange when a tab is clicked', async () => {
      const user = userEvent.setup();
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier', 'lock']}
          onKeySelect={mockOnKeySelect}
        />
      );

      await user.click(screen.getByText('Modifier'));
      expect(mockOnTabChange).toHaveBeenCalledWith('modifier');

      await user.click(screen.getByText('Lock'));
      expect(mockOnTabChange).toHaveBeenCalledWith('lock');

      expect(mockOnTabChange).toHaveBeenCalledTimes(2);
    });

    it('does not prevent clicking the already active tab', async () => {
      const user = userEvent.setup();
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier']}
          onKeySelect={mockOnKeySelect}
        />
      );

      await user.click(screen.getByText('Keyboard'));
      expect(mockOnTabChange).toHaveBeenCalledWith('keyboard');
    });
  });

  describe('Keyboard Tab Content', () => {
    it('renders SVGKeyboard when keyboard tab is active and layoutKeys provided', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard']}
          onKeySelect={mockOnKeySelect}
          layoutKeys={mockLayoutKeys}
        />
      );

      expect(screen.getByTestId('svg-keyboard')).toBeInTheDocument();
      expect(screen.getByText('Keys count: 2')).toBeInTheDocument();
    });

    it('shows message when keyboard tab is active but no layoutKeys', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard']}
          onKeySelect={mockOnKeySelect}
          layoutKeys={[]}
        />
      );

      expect(
        screen.getByText('No keyboard layout available')
      ).toBeInTheDocument();
    });

    it('calls onKeySelect when a key is clicked in SVGKeyboard', async () => {
      const user = userEvent.setup();
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard']}
          onKeySelect={mockOnKeySelect}
          layoutKeys={mockLayoutKeys}
        />
      );

      await user.click(screen.getByText('Mock Key A'));
      expect(mockOnKeySelect).toHaveBeenCalledWith('A');
    });
  });

  describe('Modifier Tab Content', () => {
    it('renders modifier grid when modifier tab is active', () => {
      render(
        <KeySelectionTabs
          activeTab="modifier"
          onTabChange={mockOnTabChange}
          availableTabs={['modifier']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(
        screen.getByText('Select a custom modifier (MD_00 to MD_FF)')
      ).toBeInTheDocument();
      // Should have 256 buttons (00 to FF)
      const buttons = screen.getAllByRole('button').filter((btn) => {
        const label = btn.getAttribute('aria-label');
        return label?.startsWith('Modifier ');
      });
      expect(buttons).toHaveLength(256);
    });

    it('calls onKeySelect with correct MD_ id when modifier is clicked', async () => {
      const user = userEvent.setup();
      render(
        <KeySelectionTabs
          activeTab="modifier"
          onTabChange={mockOnTabChange}
          availableTabs={['modifier']}
          onKeySelect={mockOnKeySelect}
        />
      );

      const md00Button = screen.getByLabelText('Modifier 00');
      await user.click(md00Button);
      expect(mockOnKeySelect).toHaveBeenCalledWith('MD_00');

      const mdFFButton = screen.getByLabelText('Modifier FF');
      await user.click(mdFFButton);
      expect(mockOnKeySelect).toHaveBeenCalledWith('MD_FF');
    });
  });

  describe('Lock Tab Content', () => {
    it('renders lock grid when lock tab is active', () => {
      render(
        <KeySelectionTabs
          activeTab="lock"
          onTabChange={mockOnTabChange}
          availableTabs={['lock']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(
        screen.getByText('Select a lock state (LK_00 to LK_FF)')
      ).toBeInTheDocument();
      // Should have 256 buttons (00 to FF)
      const buttons = screen.getAllByRole('button').filter((btn) => {
        const label = btn.getAttribute('aria-label');
        return label?.startsWith('Lock ');
      });
      expect(buttons).toHaveLength(256);
    });

    it('displays special labels for known locks', () => {
      render(
        <KeySelectionTabs
          activeTab="lock"
          onTabChange={mockOnTabChange}
          availableTabs={['lock']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByLabelText('Lock CapsLock')).toBeInTheDocument();
      expect(screen.getByLabelText('Lock NumLock')).toBeInTheDocument();
      expect(screen.getByLabelText('Lock ScrollLock')).toBeInTheDocument();
    });

    it('calls onKeySelect with correct LK_ id when lock is clicked', async () => {
      const user = userEvent.setup();
      render(
        <KeySelectionTabs
          activeTab="lock"
          onTabChange={mockOnTabChange}
          availableTabs={['lock']}
          onKeySelect={mockOnKeySelect}
        />
      );

      const lk00Button = screen.getByLabelText('Lock CapsLock');
      await user.click(lk00Button);
      expect(mockOnKeySelect).toHaveBeenCalledWith('LK_00');

      const lk01Button = screen.getByLabelText('Lock NumLock');
      await user.click(lk01Button);
      expect(mockOnKeySelect).toHaveBeenCalledWith('LK_01');
    });
  });

  describe('Layer Tab Content', () => {
    it('renders layer grid when layer tab is active', () => {
      render(
        <KeySelectionTabs
          activeTab="layer"
          onTabChange={mockOnTabChange}
          availableTabs={['layer']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(
        screen.getByText('Select a layer (typically 0-15)')
      ).toBeInTheDocument();
      // Should have 16 buttons (0 to 15)
      for (let i = 0; i < 16; i++) {
        expect(screen.getByLabelText(`Layer ${i}`)).toBeInTheDocument();
      }
    });

    it('calls onKeySelect with correct layer_N id when layer is clicked', async () => {
      const user = userEvent.setup();
      render(
        <KeySelectionTabs
          activeTab="layer"
          onTabChange={mockOnTabChange}
          availableTabs={['layer']}
          onKeySelect={mockOnKeySelect}
        />
      );

      const layer0 = screen.getByLabelText('Layer 0');
      await user.click(layer0);
      expect(mockOnKeySelect).toHaveBeenCalledWith('layer_0');

      const layer15 = screen.getByLabelText('Layer 15');
      await user.click(layer15);
      expect(mockOnKeySelect).toHaveBeenCalledWith('layer_15');
    });
  });

  describe('Accessibility', () => {
    it('has tablist role on tab container', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByRole('tablist')).toBeInTheDocument();
    });

    it('marks selected tab with aria-selected', () => {
      render(
        <KeySelectionTabs
          activeTab="modifier"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier', 'lock']}
          onKeySelect={mockOnKeySelect}
        />
      );

      const selectedTab = screen.getByRole('tab', { selected: true });
      expect(selectedTab).toHaveTextContent('Modifier');

      const unselectedTabs = screen.getAllByRole('tab', { selected: false });
      expect(unselectedTabs).toHaveLength(2);
    });

    it('has tabpanel role on content areas', () => {
      render(
        <KeySelectionTabs
          activeTab="modifier"
          onTabChange={mockOnTabChange}
          availableTabs={['modifier']}
          onKeySelect={mockOnKeySelect}
        />
      );

      expect(screen.getByRole('tabpanel')).toBeInTheDocument();
    });

    it('associates tabs with panels using aria-controls', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier']}
          onKeySelect={mockOnKeySelect}
          layoutKeys={mockLayoutKeys}
        />
      );

      const keyboardTab = screen.getByRole('tab', { name: /keyboard/i });
      expect(keyboardTab).toHaveAttribute('aria-controls', 'keyboard-panel');

      const panel = screen.getByRole('tabpanel');
      expect(panel).toHaveAttribute('id', 'keyboard-panel');
    });
  });

  describe('Edge Cases', () => {
    it('handles single tab', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard']}
          onKeySelect={mockOnKeySelect}
        />
      );

      const tabs = screen.getAllByRole('tab');
      expect(tabs).toHaveLength(1);
    });

    it('handles empty availableTabs array', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={[]}
          onKeySelect={mockOnKeySelect}
        />
      );

      const tabs = screen.queryAllByRole('tab');
      expect(tabs).toHaveLength(0);
    });

    it('renders only active tab content', () => {
      render(
        <KeySelectionTabs
          activeTab="keyboard"
          onTabChange={mockOnTabChange}
          availableTabs={['keyboard', 'modifier', 'lock']}
          onKeySelect={mockOnKeySelect}
          layoutKeys={mockLayoutKeys}
        />
      );

      // Keyboard content should be visible
      expect(screen.getByTestId('svg-keyboard')).toBeInTheDocument();

      // Modifier and lock content should NOT be rendered
      expect(
        screen.queryByText('Select a custom modifier (MD_00 to MD_FF)')
      ).not.toBeInTheDocument();
      expect(
        screen.queryByText('Select a lock state (LK_00 to LK_FF)')
      ).not.toBeInTheDocument();
    });
  });
});
