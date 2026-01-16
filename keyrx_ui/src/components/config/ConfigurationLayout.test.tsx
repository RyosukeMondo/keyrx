import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../../tests/testUtils';
import { ConfigurationLayout } from './ConfigurationLayout';

describe('ConfigurationLayout', () => {
  const defaultProps = {
    profileName: 'Default',
  };

  it('renders children correctly', () => {
    renderWithProviders(
      <ConfigurationLayout {...defaultProps}>
        <div data-testid="child-1">Child 1</div>
        <div data-testid="child-2">Child 2</div>
      </ConfigurationLayout>
    );

    expect(screen.getByTestId('child-1')).toBeInTheDocument();
    expect(screen.getByTestId('child-2')).toBeInTheDocument();
  });

  it('renders with flex column gap layout', () => {
    const { container } = renderWithProviders(
      <ConfigurationLayout {...defaultProps}>
        <div>Content</div>
      </ConfigurationLayout>
    );

    const layoutDiv = container.querySelector('.flex.flex-col.gap-4');
    expect(layoutDiv).toBeInTheDocument();
  });

  it('renders multiple children in order', () => {
    renderWithProviders(
      <ConfigurationLayout {...defaultProps}>
        <div data-testid="first">First</div>
        <div data-testid="second">Second</div>
        <div data-testid="third">Third</div>
      </ConfigurationLayout>
    );

    const children = screen.getAllByTestId(/first|second|third/);
    expect(children).toHaveLength(3);
    expect(children[0]).toHaveTextContent('First');
    expect(children[1]).toHaveTextContent('Second');
    expect(children[2]).toHaveTextContent('Third');
  });

  it('renders with empty children', () => {
    const { container } = renderWithProviders(
      <ConfigurationLayout {...defaultProps}>{null}</ConfigurationLayout>
    );

    const layoutDiv = container.querySelector('.flex.flex-col.gap-4');
    expect(layoutDiv).toBeInTheDocument();
    expect(layoutDiv?.children).toHaveLength(0);
  });

  it('handles complex children structures', () => {
    renderWithProviders(
      <ConfigurationLayout {...defaultProps}>
        <div data-testid="complex">
          <button>Button</button>
          <input type="text" />
        </div>
      </ConfigurationLayout>
    );

    const complexChild = screen.getByTestId('complex');
    expect(complexChild).toBeInTheDocument();
    expect(screen.getByRole('button')).toBeInTheDocument();
    expect(screen.getByRole('textbox')).toBeInTheDocument();
  });

  it('accepts different profile names', () => {
    const { rerender } = renderWithProviders(
      <ConfigurationLayout profileName="Test1">
        <div>Content</div>
      </ConfigurationLayout>
    );

    expect(screen.getByText('Content')).toBeInTheDocument();

    rerender(
      <ConfigurationLayout profileName="Test2">
        <div>Content</div>
      </ConfigurationLayout>
    );

    expect(screen.getByText('Content')).toBeInTheDocument();
  });
});
