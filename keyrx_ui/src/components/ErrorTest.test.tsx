import { describe, it, expect, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ErrorTest } from './ErrorTest';

describe('ErrorTest', () => {
  // Suppress console.error for error boundary tests
  const originalError = console.error;
  beforeEach(() => {
    console.error = () => {};
  });

  afterEach(() => {
    console.error = originalError;
  });

  it('renders test page title', () => {
    renderWithProviders(<ErrorTest />);
    expect(screen.getByText('Error Handling Test')).toBeInTheDocument();
  });

  it('renders ErrorState demo section', () => {
    renderWithProviders(<ErrorTest />);
    expect(
      screen.getByText('ErrorState Component (API Errors)')
    ).toBeInTheDocument();
  });

  it('renders ErrorBoundary demo section', () => {
    renderWithProviders(<ErrorTest />);
    expect(
      screen.getByText('ErrorBoundary Component (React Errors)')
    ).toBeInTheDocument();
  });

  it('initially shows success state for API', () => {
    renderWithProviders(<ErrorTest />);
    expect(
      screen.getByText(
        'API request successful. Click below to simulate a failure.'
      )
    ).toBeInTheDocument();
  });

  it('shows ErrorState when API error button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ErrorTest />);

    const button = screen.getByLabelText('Simulate API error');
    await user.click(button);

    expect(screen.getByText('Failed to Load Data')).toBeInTheDocument();
    expect(
      screen.getByText(
        'Unable to fetch data from the server. Please check your connection and try again.'
      )
    ).toBeInTheDocument();
  });

  it('shows retry button in error state', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ErrorTest />);

    await user.click(screen.getByLabelText('Simulate API error'));

    expect(screen.getByLabelText('Retry Request')).toBeInTheDocument();
  });

  it('returns to success state when retry is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ErrorTest />);

    // Trigger error
    await user.click(screen.getByLabelText('Simulate API error'));
    expect(screen.getByText('Failed to Load Data')).toBeInTheDocument();

    // Click retry
    await user.click(screen.getByLabelText('Retry Request'));

    // Should show success state again
    expect(
      screen.getByText(
        'API request successful. Click below to simulate a failure.'
      )
    ).toBeInTheDocument();
  });

  it('renders warning about ErrorBoundary test', () => {
    renderWithProviders(<ErrorTest />);
    expect(screen.getByText(/Warning:/)).toBeInTheDocument();
    expect(
      screen.getByText(
        /Clicking this button will throw an error and trigger the ErrorBoundary/
      )
    ).toBeInTheDocument();
  });

  it('renders throw error button', () => {
    renderWithProviders(<ErrorTest />);
    expect(
      screen.getByLabelText('Throw error to test ErrorBoundary')
    ).toBeInTheDocument();
  });

  it('has proper structure and styling classes', () => {
    const { container } = renderWithProviders(<ErrorTest />);
    const wrapper = container.querySelector('.min-h-screen');
    expect(wrapper).toBeInTheDocument();
    expect(wrapper).toHaveClass('bg-slate-900');
  });
});
