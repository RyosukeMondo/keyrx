import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { ConfigurationPage } from './ConfigurationPage';
import { MockStorageImpl } from '@/services/MockStorageImpl';
import type { ConfigStorage } from '@/services/ConfigStorage';

// Mock the child components
vi.mock('./ConfigEditor', () => ({
  ConfigEditor: ({ onSave, onValidationChange }: any) => (
    <div data-testid="config-editor">
      <button onClick={() => onSave('test config content')}>Save</button>
      <button onClick={() => onValidationChange(null)}>Clear Validation</button>
    </div>
  ),
}));

vi.mock('./ValidationStatusPanel', () => ({
  ValidationStatusPanel: () => <div data-testid="validation-panel">Validation Panel</div>,
}));

describe('ConfigurationPage', () => {
  let mockStorage: MockStorageImpl;

  beforeEach(() => {
    mockStorage = new MockStorageImpl();
    vi.clearAllMocks();
  });

  it('should render with all child components', () => {
    render(<ConfigurationPage />);

    expect(screen.getByText('Configuration Editor')).toBeInTheDocument();
    expect(screen.getByTestId('config-editor')).toBeInTheDocument();
    expect(screen.getByTestId('validation-panel')).toBeInTheDocument();
  });

  it('should render with initial config', () => {
    const initialConfig = 'initial config content';
    render(<ConfigurationPage initialConfig={initialConfig} />);

    expect(screen.getByTestId('config-editor')).toBeInTheDocument();
  });

  it('should use injected storage implementation', async () => {
    render(<ConfigurationPage storage={mockStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    await waitFor(() => {
      expect(mockStorage.size()).toBe(1);
    });

    const savedContent = await mockStorage.load('keyrx_config');
    expect(savedContent).toBe('test config content');
  });

  it('should display success message after successful save', async () => {
    render(<ConfigurationPage storage={mockStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    await waitFor(() => {
      expect(screen.getByText(/Configuration saved successfully/i)).toBeInTheDocument();
    });
  });

  it('should display error message when storage fails', async () => {
    // Create a storage mock that throws an error
    const errorStorage: ConfigStorage = {
      save: vi.fn().mockRejectedValue(new Error('Storage quota exceeded')),
      load: vi.fn().mockResolvedValue(null),
      delete: vi.fn().mockResolvedValue(undefined),
    };

    render(<ConfigurationPage storage={errorStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    await waitFor(() => {
      expect(screen.getByText(/Failed to save configuration/i)).toBeInTheDocument();
    });

    // Check that the specific error message is displayed
    await waitFor(() => {
      expect(screen.getByText(/Storage quota exceeded/i)).toBeInTheDocument();
    });
  });

  it('should handle storage unavailable error gracefully', async () => {
    const unavailableStorage: ConfigStorage = {
      save: vi.fn().mockRejectedValue(new Error('localStorage is not available')),
      load: vi.fn().mockResolvedValue(null),
      delete: vi.fn().mockResolvedValue(undefined),
    };

    render(<ConfigurationPage storage={unavailableStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    await waitFor(() => {
      expect(screen.getByText(/Failed to save configuration/i)).toBeInTheDocument();
      expect(screen.getByText(/localStorage is not available/i)).toBeInTheDocument();
    });
  });

  it('should show success message after save', async () => {
    render(<ConfigurationPage storage={mockStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    // Wait for the success message to appear
    await waitFor(() => {
      expect(screen.getByText(/Configuration saved successfully/i)).toBeInTheDocument();
    });
  });

  it('should show error message with details when storage fails', async () => {
    const errorStorage: ConfigStorage = {
      save: vi.fn().mockRejectedValue(new Error('Test error message')),
      load: vi.fn().mockResolvedValue(null),
      delete: vi.fn().mockResolvedValue(undefined),
    };

    render(<ConfigurationPage storage={errorStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    // Wait for the error message to appear
    await waitFor(() => {
      expect(screen.getByText(/Failed to save configuration/i)).toBeInTheDocument();
      expect(screen.getByText(/Test error message/i)).toBeInTheDocument();
    });
  });

  it('should work without storage prop (using default LocalStorageImpl)', () => {
    // This test verifies backward compatibility
    render(<ConfigurationPage />);

    expect(screen.getByTestId('config-editor')).toBeInTheDocument();
  });

  it('should handle validation changes', () => {
    render(<ConfigurationPage storage={mockStorage} />);

    const clearValidationButton = screen.getByText('Clear Validation');
    clearValidationButton.click();

    // Should not throw error
    expect(screen.getByTestId('config-editor')).toBeInTheDocument();
  });

  it('should display keyboard shortcuts help text', () => {
    render(<ConfigurationPage storage={mockStorage} />);

    expect(screen.getByText(/Keyboard Shortcuts:/i)).toBeInTheDocument();
    expect(screen.getByText(/F8 - Jump to next error/i)).toBeInTheDocument();
    expect(screen.getByText(/Ctrl\+S - Save/i)).toBeInTheDocument();
  });

  it('should save content with correct key name', async () => {
    render(<ConfigurationPage storage={mockStorage} />);

    const saveButton = screen.getByText('Save');
    saveButton.click();

    // Wait for save to complete (500ms simulated delay + processing time)
    await waitFor(
      async () => {
        const savedContent = await mockStorage.load('keyrx_config');
        expect(savedContent).toBe('test config content');
      },
      { timeout: 2000 }
    );

    // Verify the storage was called
    expect(mockStorage.size()).toBe(1);
  });
});
