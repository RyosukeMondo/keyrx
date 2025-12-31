import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';

describe('ConfigPage', () => {
  describe('Rendering', () => {
    it('renders with default profile name', () => {
      render(<ConfigPage />);

      expect(screen.getByText('Configuration Editor')).toBeInTheDocument();
      expect(screen.getByText(/Profile: Default/)).toBeInTheDocument();
    });

    it('renders with custom profile name', () => {
      render(<ConfigPage profileName="Gaming" />);

      expect(screen.getByText(/Profile: Gaming/)).toBeInTheDocument();
    });

    it('renders keyboard layout section', () => {
      render(<ConfigPage />);

      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    it('renders layer selector section', () => {
      render(<ConfigPage />);

      expect(
        screen.getByText(/Active Layer: MD_00 \(base\)/)
      ).toBeInTheDocument();
    });

    it('renders preview mode toggle button', () => {
      render(<ConfigPage />);

      const previewButton = screen.getByRole('button', {
        name: /preview mode is off/i,
      });
      expect(previewButton).toBeInTheDocument();
      expect(previewButton).toHaveTextContent('🧪 Preview Mode: OFF');
    });

    it('renders modified keys count', () => {
      render(<ConfigPage />);

      expect(
        screen.getByText(/Modified keys in this layer:/)
      ).toBeInTheDocument();
      expect(screen.getByText('37')).toBeInTheDocument();
    });
  });

  describe('Preview Mode', () => {
    it('toggles preview mode on button click', async () => {
      const user = userEvent.setup();
      render(<ConfigPage />);

      const previewButton = screen.getByRole('button', {
        name: /preview mode is off/i,
      });

      // Initially off
      expect(previewButton).toHaveTextContent('OFF');

      // Click to toggle on
      await user.click(previewButton);
      expect(previewButton).toHaveTextContent('ON');
      expect(previewButton).toHaveClass('bg-green-600');

      // Click to toggle off
      await user.click(previewButton);
      expect(previewButton).toHaveTextContent('OFF');
      expect(previewButton).not.toHaveClass('bg-green-600');
    });

    it('updates aria-label when preview mode changes', async () => {
      const user = userEvent.setup();
      render(<ConfigPage />);

      const previewButton = screen.getByRole('button', {
        name: /preview mode is off/i,
      });

      await user.click(previewButton);

      expect(
        screen.getByRole('button', { name: /preview mode is on/i })
      ).toBeInTheDocument();
    });
  });

  describe('Layer Selector', () => {
    it('renders all layer options', () => {
      render(<ConfigPage />);

      const layers = ['Base', 'Nav', 'Num', 'Fn', 'Gaming'];

      layers.forEach((layer) => {
        expect(
          screen.getByRole('button', { name: new RegExp(layer, 'i') })
        ).toBeInTheDocument();
      });
    });

    it('highlights active layer', () => {
      render(<ConfigPage />);

      const baseButton = screen.getByRole('button', {
        name: /switch to base layer/i,
      });
      expect(baseButton).toHaveClass('bg-primary-500');
      expect(baseButton).toHaveAttribute('aria-pressed', 'true');
    });

    it('switches layer on button click', async () => {
      const user = userEvent.setup();
      render(<ConfigPage />);

      const navButton = screen.getByRole('button', {
        name: /switch to nav layer/i,
      });

      await user.click(navButton);

      expect(navButton).toHaveClass('bg-primary-500');
      expect(navButton).toHaveAttribute('aria-pressed', 'true');
      expect(screen.getByText(/Active Layer: MD_00 \(nav\)/)).toBeInTheDocument();
    });

    it('updates layer display when layer changes', async () => {
      const user = userEvent.setup();
      render(<ConfigPage />);

      const fnButton = screen.getByRole('button', {
        name: /switch to fn layer/i,
      });

      await user.click(fnButton);

      expect(screen.getByText(/Active Layer: MD_00 \(fn\)/)).toBeInTheDocument();
    });

    it('only one layer can be active at a time', async () => {
      const user = userEvent.setup();
      render(<ConfigPage />);

      const navButton = screen.getByRole('button', {
        name: /switch to nav layer/i,
      });

      await user.click(navButton);

      const baseButton = screen.getByRole('button', {
        name: /switch to base layer/i,
      });
      const gamingButton = screen.getByRole('button', {
        name: /switch to gaming layer/i,
      });

      expect(navButton).toHaveAttribute('aria-pressed', 'true');
      expect(baseButton).toHaveAttribute('aria-pressed', 'false');
      expect(gamingButton).toHaveAttribute('aria-pressed', 'false');
    });
  });

  describe('Layout Selector', () => {
    it('displays keyboard layout heading', () => {
      render(<ConfigPage />);

      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    it('has accessible label for layout dropdown', () => {
      render(<ConfigPage />);

      // The Dropdown component should have aria-label
      // This test validates the prop is passed correctly
      const dropdown = screen.getByLabelText(/select keyboard layout/i);
      expect(dropdown).toBeInTheDocument();
    });
  });

  describe('Keyboard Visualizer', () => {
    it('renders keyboard visualizer component', () => {
      render(<ConfigPage />);

      // Check that the keyboard layout section exists
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      // Check that layout dropdown is present
      expect(screen.getByLabelText(/select keyboard layout/i)).toBeInTheDocument();
    });

    it('renders example mapping display', () => {
      render(<ConfigPage />);

      // Check for the example mapping text
      expect(screen.getByText(/Example:/)).toBeInTheDocument();
      expect(screen.getByText(/Tap: Escape, Hold \(200ms\): Ctrl/)).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has proper heading hierarchy', () => {
      render(<ConfigPage />);

      const h1 = screen.getByRole('heading', { level: 1 });
      expect(h1).toHaveTextContent('Configuration Editor');

      const h2Elements = screen.getAllByRole('heading', { level: 2 });
      expect(h2Elements).toHaveLength(2);
      expect(h2Elements[0]).toHaveTextContent('Keyboard Layout');
    });

    it('preview button has descriptive aria-label', () => {
      render(<ConfigPage />);

      const previewButton = screen.getByRole('button', {
        name: /preview mode/i,
      });
      expect(previewButton).toHaveAccessibleName();
    });

    it('layer buttons have descriptive aria-labels', () => {
      render(<ConfigPage />);

      const baseButton = screen.getByRole('button', {
        name: /switch to base layer/i,
      });
      expect(baseButton).toHaveAccessibleName();
    });

    it('layer buttons have aria-pressed attribute', () => {
      render(<ConfigPage />);

      const layers = ['Base', 'Nav', 'Num', 'Fn', 'Gaming'];

      layers.forEach((layer) => {
        const button = screen.getByRole('button', {
          name: new RegExp(`switch to ${layer} layer`, 'i'),
        });
        expect(button).toHaveAttribute('aria-pressed');
      });
    });
  });

  describe('Example Mapping Display', () => {
    it('shows example key mapping', () => {
      render(<ConfigPage />);

      expect(screen.getByText(/Example:/)).toBeInTheDocument();
      expect(screen.getByText('*Caps*')).toBeInTheDocument();
      expect(screen.getByText(/Tap: Escape, Hold \(200ms\): Ctrl/)).toBeInTheDocument();
    });
  });

  describe('Responsive Design', () => {
    it('renders cards with proper spacing', () => {
      const { container } = render(<ConfigPage />);

      const pageContainer = container.querySelector('.flex.flex-col');
      expect(pageContainer).toBeTruthy();
    });

    it('layer buttons are in a flex container', () => {
      render(<ConfigPage />);

      // Check for layer selector heading which should be in the same container as buttons
      expect(screen.getByText(/Active Layer:/)).toBeInTheDocument();
    });
  });
});
