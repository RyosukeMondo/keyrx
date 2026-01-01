import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Modal } from './Modal';

describe('Modal', () => {
  const mockOnClose = vi.fn();
  const defaultProps = {
    open: true,
    onClose: mockOnClose,
    title: 'Test Modal',
    children: <div>Modal content</div>,
  };

  beforeEach(() => {
    mockOnClose.mockClear();
    // Create a button to test focus return
    const button = document.createElement('button');
    button.id = 'trigger-button';
    document.body.appendChild(button);
    button.focus();
  });

  afterEach(() => {
    // Clean up
    const button = document.getElementById('trigger-button');
    if (button) {
      document.body.removeChild(button);
    }
  });

  it('renders modal when open is true', () => {
    render(<Modal {...defaultProps} />);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Test Modal')).toBeInTheDocument();
    expect(screen.getByText('Modal content')).toBeInTheDocument();
  });

  it('does not render when open is false', () => {
    render(<Modal {...defaultProps} open={false} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('displays the correct title', () => {
    render(<Modal {...defaultProps} title="Custom Title" />);
    expect(screen.getByText('Custom Title')).toBeInTheDocument();
  });

  it('renders children correctly', () => {
    render(
      <Modal {...defaultProps}>
        <p>Custom content</p>
        <button>Action</button>
      </Modal>
    );
    expect(screen.getByText('Custom content')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Action' })).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', async () => {
    const user = userEvent.setup();
    render(<Modal {...defaultProps} />);

    const closeButton = screen.getByRole('button', { name: 'Close modal' });
    await user.click(closeButton);

    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when Escape key is pressed', async () => {
    render(<Modal {...defaultProps} />);

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when backdrop is clicked', async () => {
    const user = userEvent.setup();
    render(<Modal {...defaultProps} />);

    // Click the backdrop (not the modal content)
    const backdrop = screen.getByRole('dialog').parentElement;
    if (backdrop) {
      await user.click(backdrop);
      expect(mockOnClose).toHaveBeenCalledTimes(1);
    }
  });

  it('does not call onClose when modal content is clicked', async () => {
    const user = userEvent.setup();
    render(<Modal {...defaultProps} />);

    const modalContent = screen.getByRole('dialog');
    await user.click(modalContent);

    expect(mockOnClose).not.toHaveBeenCalled();
  });

  it('has correct ARIA attributes', () => {
    render(<Modal {...defaultProps} />);
    const modal = screen.getByRole('dialog');

    expect(modal).toHaveAttribute('aria-modal', 'true');
    expect(modal).toHaveAttribute('aria-labelledby', 'modal-title');
  });

  it('focuses first focusable element when opened', async () => {
    render(
      <Modal {...defaultProps}>
        <button>First</button>
        <button>Second</button>
      </Modal>
    );

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();
    });
  });

  it('traps focus within modal (Tab forward)', async () => {
    const user = userEvent.setup();
    render(
      <Modal {...defaultProps}>
        <button>First</button>
        <button>Second</button>
      </Modal>
    );

    // Focus should start on first button
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();
    });

    // Tab to second button
    await user.tab();
    expect(screen.getByRole('button', { name: 'Second' })).toHaveFocus();

    // Tab to close button
    await user.tab();
    expect(screen.getByRole('button', { name: 'Close modal' })).toHaveFocus();

    // Tab should wrap to first button
    await user.tab();
    expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();
  });

  it('traps focus within modal (Shift+Tab backward)', async () => {
    const user = userEvent.setup();
    render(
      <Modal {...defaultProps}>
        <button>First</button>
        <button>Second</button>
      </Modal>
    );

    // Focus should start on first button
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();
    });

    // Shift+Tab should wrap to last focusable element (close button)
    await user.tab({ shift: true });
    expect(screen.getByRole('button', { name: 'Close modal' })).toHaveFocus();
  });

  it('returns focus to previous element when closed', async () => {
    const { rerender } = render(<Modal {...defaultProps} />);

    // Verify modal is open and has focus
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    // Close the modal
    rerender(<Modal {...defaultProps} open={false} />);

    // Focus should return to the trigger button
    await waitFor(() => {
      const triggerButton = document.getElementById('trigger-button');
      expect(triggerButton).toHaveFocus();
    });
  });

  it('prevents body scroll when open', () => {
    const { rerender } = render(<Modal {...defaultProps} />);
    expect(document.body.style.overflow).toBe('hidden');

    rerender(<Modal {...defaultProps} open={false} />);
    expect(document.body.style.overflow).toBe('');
  });

  it('restores body scroll on unmount', () => {
    const { unmount } = render(<Modal {...defaultProps} />);
    expect(document.body.style.overflow).toBe('hidden');

    unmount();
    expect(document.body.style.overflow).toBe('');
  });

  it('applies custom className', () => {
    render(<Modal {...defaultProps} className="custom-class" />);
    const modal = screen.getByRole('dialog');
    expect(modal).toHaveClass('custom-class');
  });

  it('renders with Portal', () => {
    render(<Modal {...defaultProps} />);

    // Modal should be rendered directly in body, not in the component tree
    const modal = screen.getByRole('dialog');
    expect(modal.parentElement?.parentElement).toBe(document.body);
  });

  it('does not close on Escape when already closed', () => {
    render(<Modal {...defaultProps} open={false} />);

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(mockOnClose).not.toHaveBeenCalled();
  });

  it('handles rapid open/close transitions', async () => {
    const { rerender } = render(<Modal {...defaultProps} open={false} />);

    // Rapidly toggle open state
    rerender(<Modal {...defaultProps} open={true} />);
    rerender(<Modal {...defaultProps} open={false} />);
    rerender(<Modal {...defaultProps} open={true} />);

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    expect(document.body.style.overflow).toBe('hidden');
  });
});
