import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import {
  LoadingSkeleton,
  SkeletonCard,
  SkeletonTable,
  SkeletonProfile,
  SkeletonButton,
} from './LoadingSkeleton';

describe('LoadingSkeleton', () => {
  it('renders with default variant', () => {
    const { container } = render(<LoadingSkeleton />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toBeInTheDocument();
    expect(skeleton).toHaveClass('rounded-md');
    expect(skeleton).toHaveClass('animate-pulse');
  });

  it('renders text variant', () => {
    const { container } = render(<LoadingSkeleton variant="text" />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveClass('rounded', 'h-4');
  });

  it('renders circular variant', () => {
    const { container } = render(<LoadingSkeleton variant="circular" />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveClass('rounded-full');
  });

  it('renders rectangular variant', () => {
    const { container } = render(<LoadingSkeleton variant="rectangular" />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveClass('rounded-md');
  });

  it('renders card variant', () => {
    const { container } = render(<LoadingSkeleton variant="card" />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveClass('rounded-lg');
  });

  it('applies custom width and height', () => {
    const { container } = render(<LoadingSkeleton width="200px" height="50px" />);
    const skeleton = container.querySelector('[aria-busy="true"]') as HTMLElement;
    expect(skeleton?.style.width).toBe('200px');
    expect(skeleton?.style.height).toBe('50px');
  });

  it('applies numeric width and height', () => {
    const { container } = render(<LoadingSkeleton width={200} height={50} />);
    const skeleton = container.querySelector('[aria-busy="true"]') as HTMLElement;
    expect(skeleton?.style.width).toBe('200px');
    expect(skeleton?.style.height).toBe('50px');
  });

  it('applies custom className', () => {
    const { container } = render(<LoadingSkeleton className="my-custom-class" />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveClass('my-custom-class');
  });

  it('renders multiple skeletons when count > 1', () => {
    const { container } = render(<LoadingSkeleton count={3} />);
    const skeletons = container.querySelectorAll('[aria-busy="true"]');
    expect(skeletons).toHaveLength(3);
  });

  it('has accessibility attributes', () => {
    const { container } = render(<LoadingSkeleton />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveAttribute('aria-busy', 'true');
    expect(skeleton).toHaveAttribute('aria-live', 'polite');
    expect(skeleton).toHaveAttribute('aria-label', 'Loading content');
  });
});

describe('SkeletonCard', () => {
  it('renders card skeleton with multiple text lines', () => {
    const { container } = render(<SkeletonCard />);
    const skeletons = container.querySelectorAll('[aria-busy="true"]');
    expect(skeletons.length).toBeGreaterThan(1);
  });

  it('applies custom className to wrapper', () => {
    const { container } = render(<SkeletonCard className="my-card" />);
    const wrapper = container.querySelector('.my-card');
    expect(wrapper).toBeInTheDocument();
  });
});

describe('SkeletonTable', () => {
  it('renders default number of rows', () => {
    const { container } = render(<SkeletonTable />);
    const skeletons = container.querySelectorAll('[aria-busy="true"]');
    // Header + 5 default rows
    expect(skeletons).toHaveLength(6);
  });

  it('renders custom number of rows', () => {
    const { container } = render(<SkeletonTable rows={3} />);
    const skeletons = container.querySelectorAll('[aria-busy="true"]');
    // Header + 3 rows
    expect(skeletons).toHaveLength(4);
  });

  it('applies custom className to wrapper', () => {
    const { container } = render(<SkeletonTable className="my-table" />);
    const wrapper = container.querySelector('.my-table');
    expect(wrapper).toBeInTheDocument();
  });
});

describe('SkeletonProfile', () => {
  it('renders profile skeleton with avatar and text', () => {
    const { container } = render(<SkeletonProfile />);
    const skeletons = container.querySelectorAll('[aria-busy="true"]');
    // Avatar + 2 text lines
    expect(skeletons).toHaveLength(3);
  });

  it('applies custom className to wrapper', () => {
    const { container } = render(<SkeletonProfile className="my-profile" />);
    const wrapper = container.querySelector('.my-profile');
    expect(wrapper).toBeInTheDocument();
  });
});

describe('SkeletonButton', () => {
  it('renders button skeleton', () => {
    const { container } = render(<SkeletonButton />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<SkeletonButton className="my-button" />);
    const skeleton = container.querySelector('[aria-busy="true"]');
    expect(skeleton).toHaveClass('my-button');
  });
});
