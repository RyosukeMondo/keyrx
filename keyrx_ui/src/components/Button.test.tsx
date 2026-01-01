import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders with children', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button">
        Click me
      </Button>
    );
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(
      <Button onClick={handleClick} aria-label="Test button">
        Click me
      </Button>
    );
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('does not call onClick when disabled', () => {
    const handleClick = vi.fn();
    render(
      <Button onClick={handleClick} aria-label="Test button" disabled>
        Click me
      </Button>
    );
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('does not call onClick when loading', () => {
    const handleClick = vi.fn();
    render(
      <Button onClick={handleClick} aria-label="Test button" loading>
        Click me
      </Button>
    );
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('renders loading spinner when loading', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" loading>
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.querySelector('svg')).toBeInTheDocument();
  });

  it('applies primary variant classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" variant="primary">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('bg-primary-500');
  });

  it('applies secondary variant classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" variant="secondary">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('border-primary-500');
  });

  it('applies danger variant classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" variant="danger">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('bg-red-500');
  });

  it('applies ghost variant classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" variant="ghost">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('bg-transparent');
  });

  it('applies small size classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" size="sm">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('py-2');
    expect(button.className).toContain('px-3');
  });

  it('applies medium size classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" size="md">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('py-3');
    expect(button.className).toContain('px-4');
  });

  it('applies large size classes', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" size="lg">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('py-4');
    expect(button.className).toContain('px-6');
  });

  it('sets aria-label attribute', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Custom label">
        Click me
      </Button>
    );
    expect(screen.getByLabelText('Custom label')).toBeInTheDocument();
  });

  it('sets aria-disabled when disabled', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" disabled>
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-disabled', 'true');
  });

  it('sets aria-busy when loading', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" loading>
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-busy', 'true');
  });

  it('accepts custom className', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" className="custom-class">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button.className).toContain('custom-class');
  });

  it('applies button type attribute', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button" type="submit">
        Submit
      </Button>
    );
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('type', 'submit');
  });

  it('creates ripple element on click', () => {
    render(
      <Button onClick={vi.fn()} aria-label="Test button">
        Click me
      </Button>
    );
    const button = screen.getByRole('button');
    fireEvent.click(button);

    const ripple = button.querySelector('.ripple');
    expect(ripple).toBeInTheDocument();
  });
});
