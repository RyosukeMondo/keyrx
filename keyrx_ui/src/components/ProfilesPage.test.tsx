/**
 * Component tests for ProfilesPage.
 *
 * Tests cover:
 * - Profile list rendering
 * - Loading and error states
 * - Create profile action
 * - Activate profile action
 * - Delete profile action
 * - Duplicate profile action
 * - Export profile action
 * - API integration and error handling
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react';
import { ProfilesPage, type Profile } from './ProfilesPage';

// Mock ProfileCard component
vi.mock('./ProfileCard', () => ({
  ProfileCard: ({ profile, onActivate, onDelete, onDuplicate, onExport, onRename }: any) => (
    <div data-testid={`profile-card-${profile.name}`}>
      <div>{profile.name}</div>
      <div>{profile.is_active ? 'Active' : 'Inactive'}</div>
      <button onClick={onActivate} data-testid={`activate-${profile.name}`}>
        Activate
      </button>
      <button onClick={onDelete} data-testid={`delete-${profile.name}`}>
        Delete
      </button>
      <button onClick={onDuplicate} data-testid={`duplicate-${profile.name}`}>
        Duplicate
      </button>
      <button onClick={onExport} data-testid={`export-${profile.name}`}>
        Export
      </button>
      <button onClick={onRename} data-testid={`rename-${profile.name}`}>
        Rename
      </button>
    </div>
  ),
}));

// Mock ProfileDialog component
vi.mock('./ProfileDialog', () => ({
  ProfileDialog: ({ mode, initialName, onClose, onSubmit }: any) => (
    <div data-testid="profile-dialog">
      <div>{mode === 'create' ? 'Create Profile' : 'Rename Profile'}</div>
      {initialName && <div>Initial: {initialName}</div>}
      <button
        onClick={() => onSubmit('test-profile', 'blank')}
        data-testid="dialog-submit"
      >
        Submit
      </button>
      <button onClick={onClose} data-testid="dialog-close">
        Close
      </button>
    </div>
  ),
}));

describe('ProfilesPage', () => {
  const mockProfiles: Profile[] = [
    {
      name: 'default',
      rhai_path: '/path/to/default.rhai',
      krx_path: '/path/to/default.krx',
      modified_at: 1234567890,
      layer_count: 2,
      is_active: true,
    },
    {
      name: 'gaming',
      rhai_path: '/path/to/gaming.rhai',
      krx_path: '/path/to/gaming.krx',
      modified_at: 1234567891,
      layer_count: 3,
      is_active: false,
    },
    {
      name: 'work',
      rhai_path: '/path/to/work.rhai',
      krx_path: '/path/to/work.krx',
      modified_at: 1234567892,
      layer_count: 1,
      is_active: false,
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    global.fetch = vi.fn();
    global.alert = vi.fn();
    global.confirm = vi.fn();
    global.prompt = vi.fn();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('rendering', () => {
    it('should show loading state initially', () => {
      vi.mocked(fetch).mockImplementation(
        () => new Promise(() => {}) // Never resolves
      );

      render(<ProfilesPage />);

      expect(screen.getByText('Loading profiles...')).toBeInTheDocument();
    });

    it('should render profile list when loaded', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      expect(screen.getByTestId('profile-card-default')).toBeInTheDocument();
      expect(screen.getByTestId('profile-card-gaming')).toBeInTheDocument();
      expect(screen.getByTestId('profile-card-work')).toBeInTheDocument();
    });

    it('should render header with New Profile button', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      expect(screen.getByRole('button', { name: '+ New Profile' })).toBeInTheDocument();
    });

    it('should show "no profiles" message when list is empty', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: [] }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('No profiles found.')).toBeInTheDocument();
      });

      expect(
        screen.getByRole('button', { name: 'Create your first profile' })
      ).toBeInTheDocument();
    });
  });

  describe('error handling', () => {
    it('should show error state when fetch fails', async () => {
      vi.mocked(fetch).mockRejectedValueOnce(new Error('Network error'));

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Error: Network error/)).toBeInTheDocument();
      });

      expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
    });

    it('should show error state when response is not ok', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: false,
        status: 500,
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Error: HTTP error! status: 500/)).toBeInTheDocument();
      });
    });

    it('should reload profiles when retry button is clicked', async () => {
      // First call fails
      vi.mocked(fetch).mockRejectedValueOnce(new Error('Network error'));

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText(/Error: Network error/)).toBeInTheDocument();
      });

      // Second call succeeds
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      const retryButton = screen.getByRole('button', { name: 'Retry' });
      fireEvent.click(retryButton);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      expect(screen.getByTestId('profile-card-default')).toBeInTheDocument();
    });
  });

  describe('create profile', () => {
    it('should show create dialog when New Profile button is clicked', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      const newProfileButton = screen.getByRole('button', { name: '+ New Profile' });
      fireEvent.click(newProfileButton);

      expect(screen.getByTestId('profile-dialog')).toBeInTheDocument();
      expect(screen.getByText('Create Profile')).toBeInTheDocument();
    });

    it('should close dialog when close button is clicked', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      const newProfileButton = screen.getByRole('button', { name: '+ New Profile' });
      fireEvent.click(newProfileButton);

      const closeButton = screen.getByTestId('dialog-close');
      fireEvent.click(closeButton);

      expect(screen.queryByTestId('profile-dialog')).not.toBeInTheDocument();
    });

    it('should create profile and reload list when form is submitted', async () => {
      // Initial load
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      const newProfileButton = screen.getByRole('button', { name: '+ New Profile' });
      fireEvent.click(newProfileButton);

      // Mock create request
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      } as Response);

      // Mock reload request
      const newProfile: Profile = {
        name: 'test-profile',
        rhai_path: '/path/to/test-profile.rhai',
        krx_path: '/path/to/test-profile.krx',
        modified_at: 1234567893,
        layer_count: 1,
        is_active: false,
      };
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: [...mockProfiles, newProfile] }),
      } as Response);

      const submitButton = screen.getByTestId('dialog-submit');
      await act(async () => {
        fireEvent.click(submitButton);
      });

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith('http://localhost:3030/api/profiles', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: 'test-profile', template: 'blank' }),
        });
      });

      // Dialog should close
      expect(screen.queryByTestId('profile-dialog')).not.toBeInTheDocument();

      // List should reload
      await waitFor(() => {
        expect(screen.getByTestId('profile-card-test-profile')).toBeInTheDocument();
      });
    });

    it('should show alert when create fails', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      const newProfileButton = screen.getByRole('button', { name: '+ New Profile' });
      fireEvent.click(newProfileButton);

      // Mock create request failure
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: false,
        json: async () => ({ error: 'Profile already exists' }),
      } as Response);

      const submitButton = screen.getByTestId('dialog-submit');
      await act(async () => {
        fireEvent.click(submitButton);
      });

      await waitFor(() => {
        expect(alert).toHaveBeenCalledWith('Profile already exists');
      });

      // Dialog should remain open
      expect(screen.getByTestId('profile-dialog')).toBeInTheDocument();
    });
  });

  describe('activate profile', () => {
    it('should activate profile when activate button is clicked', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock activate request
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      } as Response);

      // Mock reload request
      const updatedProfiles = mockProfiles.map((p) => ({
        ...p,
        is_active: p.name === 'gaming',
      }));
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: updatedProfiles }),
      } as Response);

      const activateButton = screen.getByTestId('activate-gaming');
      await act(async () => {
        fireEvent.click(activateButton);
      });

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith(
          'http://localhost:3030/api/profiles/gaming/activate',
          { method: 'POST' }
        );
      });

      // List should reload
      await waitFor(() => {
        expect(fetch).toHaveBeenCalledTimes(3); // initial + activate + reload
      });
    });

    it('should handle activate failure', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock activate request failure
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: false,
      } as Response);

      const activateButton = screen.getByTestId('activate-gaming');
      await act(async () => {
        fireEvent.click(activateButton);
      });

      await waitFor(() => {
        expect(alert).toHaveBeenCalledWith('Failed to activate profile');
      });
    });
  });

  describe('delete profile', () => {
    it('should delete profile when delete button is clicked and confirmed', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock confirm dialog
      vi.mocked(confirm).mockReturnValueOnce(true);

      // Mock delete request
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      } as Response);

      // Mock reload request
      const remainingProfiles = mockProfiles.filter((p) => p.name !== 'gaming');
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: remainingProfiles }),
      } as Response);

      const deleteButton = screen.getByTestId('delete-gaming');
      await act(async () => {
        fireEvent.click(deleteButton);
      });

      await waitFor(() => {
        expect(confirm).toHaveBeenCalledWith('Delete profile "gaming"?');
      });

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith(
          'http://localhost:3030/api/profiles/gaming',
          { method: 'DELETE' }
        );
      });

      // Profile should be removed from list
      await waitFor(() => {
        expect(screen.queryByTestId('profile-card-gaming')).not.toBeInTheDocument();
      });
    });

    it('should not delete profile when confirmation is cancelled', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock confirm dialog - user cancels
      vi.mocked(confirm).mockReturnValueOnce(false);

      const deleteButton = screen.getByTestId('delete-gaming');
      await act(async () => {
        fireEvent.click(deleteButton);
      });

      await waitFor(() => {
        expect(confirm).toHaveBeenCalledWith('Delete profile "gaming"?');
      });

      // Delete request should not be made
      expect(fetch).toHaveBeenCalledTimes(1); // Only initial load
    });

    it('should handle delete failure', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      vi.mocked(confirm).mockReturnValueOnce(true);

      // Mock delete request failure
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: false,
      } as Response);

      const deleteButton = screen.getByTestId('delete-gaming');
      await act(async () => {
        fireEvent.click(deleteButton);
      });

      await waitFor(() => {
        expect(alert).toHaveBeenCalledWith('Failed to delete profile');
      });
    });
  });

  describe('duplicate profile', () => {
    it('should duplicate profile when duplicate button is clicked', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock prompt dialog
      vi.mocked(prompt).mockReturnValueOnce('gaming-copy');

      // Mock duplicate request
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({}),
      } as Response);

      // Mock reload request
      const duplicatedProfile: Profile = {
        name: 'gaming-copy',
        rhai_path: '/path/to/gaming-copy.rhai',
        krx_path: '/path/to/gaming-copy.krx',
        modified_at: 1234567893,
        layer_count: 3,
        is_active: false,
      };
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: [...mockProfiles, duplicatedProfile] }),
      } as Response);

      const duplicateButton = screen.getByTestId('duplicate-gaming');
      await act(async () => {
        fireEvent.click(duplicateButton);
      });

      await waitFor(() => {
        expect(prompt).toHaveBeenCalledWith(
          'Enter name for duplicated profile:',
          'gaming-copy'
        );
      });

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith(
          'http://localhost:3030/api/profiles/gaming/duplicate',
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ dest: 'gaming-copy' }),
          }
        );
      });

      // Duplicated profile should appear in list
      await waitFor(() => {
        expect(screen.getByTestId('profile-card-gaming-copy')).toBeInTheDocument();
      });
    });

    it('should not duplicate profile when prompt is cancelled', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock prompt dialog - user cancels
      vi.mocked(prompt).mockReturnValueOnce(null);

      const duplicateButton = screen.getByTestId('duplicate-gaming');
      await act(async () => {
        fireEvent.click(duplicateButton);
      });

      // Duplicate request should not be made
      expect(fetch).toHaveBeenCalledTimes(1); // Only initial load
    });

    it('should handle duplicate failure', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      vi.mocked(prompt).mockReturnValueOnce('gaming-copy');

      // Mock duplicate request failure
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: false,
      } as Response);

      const duplicateButton = screen.getByTestId('duplicate-gaming');
      await act(async () => {
        fireEvent.click(duplicateButton);
      });

      await waitFor(() => {
        expect(alert).toHaveBeenCalledWith('Failed to duplicate profile');
      });
    });
  });

  describe('export profile', () => {
    it('should export profile when export button is clicked', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock export file fetch
      const mockBlob = new Blob(['config content'], { type: 'text/plain' });
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        blob: async () => mockBlob,
      } as Response);

      // Mock URL.createObjectURL and related DOM methods
      const mockUrl = 'blob:mock-url';
      global.URL.createObjectURL = vi.fn(() => mockUrl);
      global.URL.revokeObjectURL = vi.fn();

      const mockAppendChild = vi.fn();
      const mockRemoveChild = vi.fn();
      const mockClick = vi.fn();

      const originalCreateElement = document.createElement.bind(document);
      vi.spyOn(document, 'createElement').mockImplementation((tagName: string) => {
        const element = originalCreateElement(tagName);
        if (tagName === 'a') {
          element.click = mockClick;
        }
        return element;
      });

      vi.spyOn(document.body, 'appendChild').mockImplementation(mockAppendChild);
      vi.spyOn(document.body, 'removeChild').mockImplementation(mockRemoveChild);

      const exportButton = screen.getByTestId('export-gaming');
      await act(async () => {
        fireEvent.click(exportButton);
      });

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith('/path/to/gaming.rhai');
      });

      await waitFor(() => {
        expect(URL.createObjectURL).toHaveBeenCalledWith(mockBlob);
        expect(mockClick).toHaveBeenCalled();
        expect(URL.revokeObjectURL).toHaveBeenCalledWith(mockUrl);
      });
    });

    it('should handle export failure', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      // Mock export file fetch failure
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: false,
      } as Response);

      const exportButton = screen.getByTestId('export-gaming');
      await act(async () => {
        fireEvent.click(exportButton);
      });

      await waitFor(() => {
        expect(alert).toHaveBeenCalledWith('Failed to export profile');
      });
    });
  });

  describe('rename profile', () => {
    it('should show rename dialog when rename button is clicked', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      const renameButton = screen.getByTestId('rename-gaming');
      fireEvent.click(renameButton);

      expect(screen.getByTestId('profile-dialog')).toBeInTheDocument();
      expect(screen.getByText('Rename Profile')).toBeInTheDocument();
      expect(screen.getByText('Initial: gaming')).toBeInTheDocument();
    });

    it('should show not implemented alert when rename is submitted', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByText('Profiles')).toBeInTheDocument();
      });

      const renameButton = screen.getByTestId('rename-gaming');
      fireEvent.click(renameButton);

      const submitButton = screen.getByTestId('dialog-submit');
      fireEvent.click(submitButton);

      expect(alert).toHaveBeenCalledWith('Rename functionality not yet implemented in API');

      // Dialog should close
      expect(screen.queryByTestId('profile-dialog')).not.toBeInTheDocument();
    });
  });

  describe('API integration', () => {
    it('should call API with correct URL for profile list', async () => {
      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: mockProfiles }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith('http://localhost:3030/api/profiles');
      });
    });

    it('should handle profiles with special characters in names', async () => {
      const specialProfile: Profile = {
        name: 'my-profile_v2',
        rhai_path: '/path/to/my-profile_v2.rhai',
        krx_path: '/path/to/my-profile_v2.krx',
        modified_at: 1234567890,
        layer_count: 1,
        is_active: false,
      };

      vi.mocked(fetch).mockResolvedValueOnce({
        ok: true,
        json: async () => ({ profiles: [specialProfile] }),
      } as Response);

      render(<ProfilesPage />);

      await waitFor(() => {
        expect(screen.getByTestId('profile-card-my-profile_v2')).toBeInTheDocument();
      });

      vi.mocked(confirm).mockReturnValueOnce(true);
      vi.mocked(fetch).mockResolvedValueOnce({ ok: true } as Response);

      const deleteButton = screen.getByTestId('delete-my-profile_v2');
      await act(async () => {
        fireEvent.click(deleteButton);
      });

      await waitFor(() => {
        expect(fetch).toHaveBeenCalledWith(
          'http://localhost:3030/api/profiles/my-profile_v2',
          { method: 'DELETE' }
        );
      });
    });
  });
});
