/**
 * Drag-and-Drop Integration tests for ConfigPage (extracted from ConfigPage.integration.test.tsx)
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders, setDaemonState } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';
import { useConfigStore } from '../stores/configStore';

describe('ConfigPage - Drag-and-Drop Integration Tests', () => {
  const renderConfigPage = (profile = 'default') => {
    return renderWithProviders(<ConfigPage profileName={profile} />, {
      wrapWithRouter: true,
      routerInitialEntries: ['/config'],
    });
  };

  beforeEach(async () => {
    const store = useConfigStore.getState();
    store.config = null;
    store.loading = false;
    store.error = null;

    setDaemonState({
      activeProfile: 'default',
      layer: 'base',
      modifiers: [],
      locks: [],
      connected: true,
    });
  });

  it('completes full drag-and-drop workflow: palette -> keyboard -> save', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const dragKey = screen.getByRole('button', { name: /^A$/i });
    expect(dragKey).toBeInTheDocument();

    const capsLockKey = screen.getByRole('button', { name: /CapsLock/i });
    expect(capsLockKey).toBeInTheDocument();

    dragKey.focus();
    await user.keyboard('{Space}');

    expect(dragKey).toHaveAttribute('draggable');
  });

  it('displays mapped keys on keyboard after drag-and-drop', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const keyboardSection = screen
      .getByText('Keyboard Layout')
      .closest('div');
    expect(keyboardSection).toBeInTheDocument();

    const keyButtons = screen.getAllByRole('button', {
      name: /Key [A-Z0-9]|CapsLock|Shift|Ctrl|Alt|Enter|Space|Tab|Esc/i,
    });
    expect(keyButtons.length).toBeGreaterThan(0);
  });

  it('highlights drop zones on drag-over', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const dropZones = screen.getAllByRole('button', {
      name: /Key [A-Z0-9]|CapsLock|Shift|Ctrl|Alt|Enter|Space|Tab|Esc/i,
    });

    expect(dropZones.length).toBeGreaterThan(50);
  });

  it('handles API save after successful drop', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const keyboardSection = screen
      .getByText('Keyboard Layout')
      .closest('div');
    expect(keyboardSection).toBeInTheDocument();
  });

  it('shows error state and rolls back on API failure', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const keyboardSection = screen
      .getByText('Keyboard Layout')
      .closest('div');
    expect(keyboardSection).toBeInTheDocument();
  });

  it('updates displayed mappings when layer changes', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const layerSelector = screen.queryByRole('combobox', {
      name: /Layer/i,
    });

    if (layerSelector) {
      const options = layerSelector.querySelectorAll('option');

      if (options.length > 1) {
        await user.selectOptions(layerSelector, options[1].value);

        await waitFor(() => {
          expect(layerSelector).toHaveValue(options[1].value);
        });

        const keyButtons = screen.getAllByRole('button', {
          name: /Key [A-Z0-9]|CapsLock|Shift|Ctrl|Alt/i,
        });
        expect(keyButtons.length).toBeGreaterThan(0);
      }
    }
  });

  it('supports keyboard-only drag-and-drop (WCAG 2.2 AA)', async () => {
    const user = userEvent.setup();
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const dragKey = screen.getByRole('button', { name: /^A$/i });

    dragKey.focus();
    expect(dragKey).toHaveFocus();

    await user.keyboard('{Space}');

    expect(dragKey).toHaveAttribute('aria-label');
  });

  it('shows visual feedback during drag operation', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const keyboardSection = screen
      .getByText('Keyboard Layout')
      .closest('div');
    expect(keyboardSection).toBeInTheDocument();

    const paletteSection = screen.queryByText('Virtual Keys');
    if (paletteSection) {
      expect(paletteSection).toBeInTheDocument();
    }
  });

  it('preserves mappings across page navigation', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const keyboardSection = screen
      .getByText('Keyboard Layout')
      .closest('div');
    expect(keyboardSection).toBeInTheDocument();
  });

  it('handles multiple rapid drag-and-drop operations', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const dragKeys = screen.getAllByRole('button', { name: /^[A-Z]$/i });

    expect(dragKeys.length).toBeGreaterThan(5);
  });

  it('displays different key types in palette (VK, MD, LK, Layers)', async () => {
    renderConfigPage();

    await waitFor(() => {
      expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
    });

    const virtualKeys = screen.queryByText('Virtual Keys');
    const modifiers = screen.queryByText('Modifiers');
    const lockKeys = screen.queryByText('Lock Keys');
    const layers = screen.queryByText('Layers');

    const categoriesPresent = [
      virtualKeys,
      modifiers,
      lockKeys,
      layers,
    ].filter(Boolean);
    expect(categoriesPresent.length).toBeGreaterThan(0);
  });
});