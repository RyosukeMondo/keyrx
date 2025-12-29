import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Card } from './Card';

describe('Card', () => {
  it('renders children', () => {
    render(<Card>Test content</Card>);
    expect(screen.getByText('Test content')).toBeInTheDocument();
  });

  it('applies default variant styles', () => {
    const { container } = render(<Card>Content</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card.className).toContain('shadow-md');
  });

  it('applies elevated variant styles', () => {
    const { container } = render(<Card variant="elevated">Content</Card>);
    const card = container.firstChild as HTMLElement;
    expect(card.className).toContain('shadow-xl');
  });

  it('applies sm padding', () => {
    const { container } = render(<Card padding="sm">Content</Card>);
    const contentDiv = container.querySelector('.p-sm');
    expect(contentDiv).toBeInTheDocument();
  });

  it('applies md padding by default', () => {
    const { container } = render(<Card>Content</Card>);
    const contentDiv = container.querySelector('.p-md');
    expect(contentDiv).toBeInTheDocument();
  });

  it('applies lg padding', () => {
    const { container } = render(<Card padding="lg">Content</Card>);
    const contentDiv = container.querySelector('.p-lg');
    expect(contentDiv).toBeInTheDocument();
  });

  it('renders header slot', () => {
    render(
      <Card header={<div>Card Header</div>}>
        <div>Card Content</div>
      </Card>
    );
    expect(screen.getByText('Card Header')).toBeInTheDocument();
    expect(screen.getByText('Card Content')).toBeInTheDocument();
  });

  it('renders footer slot', () => {
    render(
      <Card footer={<div>Card Footer</div>}>
        <div>Card Content</div>
      </Card>
    );
    expect(screen.getByText('Card Footer')).toBeInTheDocument();
    expect(screen.getByText('Card Content')).toBeInTheDocument();
  });

  it('renders both header and footer', () => {
    render(
      <Card
        header={<div>Header</div>}
        footer={<div>Footer</div>}
      >
        <div>Content</div>
      </Card>
    );
    expect(screen.getByText('Header')).toBeInTheDocument();
    expect(screen.getByText('Content')).toBeInTheDocument();
    expect(screen.getByText('Footer')).toBeInTheDocument();
  });

  it('merges custom className', () => {
    const { container } = render(
      <Card className="custom-class">Content</Card>
    );
    const card = container.firstChild as HTMLElement;
    expect(card.className).toContain('custom-class');
    expect(card.className).toContain('bg-slate-800');
  });

  it('has proper styling structure', () => {
    const { container } = render(<Card>Content</Card>);
    const card = container.firstChild as HTMLElement;

    expect(card.className).toContain('bg-slate-800');
    expect(card.className).toContain('border');
    expect(card.className).toContain('border-slate-700');
    expect(card.className).toContain('rounded-md');
  });

  it('header has border and background', () => {
    const { container } = render(
      <Card header={<div>Header</div>}>Content</Card>
    );
    const header = container.querySelector('.border-b');
    expect(header).toBeInTheDocument();
    expect(header?.className).toContain('border-slate-700');
    expect(header?.className).toContain('bg-slate-700/50');
  });

  it('footer has border and background', () => {
    const { container } = render(
      <Card footer={<div>Footer</div>}>Content</Card>
    );
    const footer = container.querySelector('.border-t');
    expect(footer).toBeInTheDocument();
    expect(footer?.className).toContain('border-slate-700');
    expect(footer?.className).toContain('bg-slate-700/50');
  });
});
