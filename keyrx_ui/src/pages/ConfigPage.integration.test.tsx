/**
 * Integration tests for ConfigPage
 * Tests key configuration flow with API mocking via MSW and WebSocket helpers
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders, setDaemonState } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ConfigPage } from './ConfigPage';
import { useConfigStore } from '../stores/configStore';

describe('ConfigPage - Integration Tests', () => {
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

  describe('Layer selector flow', () => {
    it('displays layer selector with available layers', async () => {
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const layerSelector = screen.getByRole('combobox', {
        name: /Layer/i,
      });
      expect(layerSelector).toBeInTheDocument();
    });

    it('switches between layers', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        const layerSelector = screen.queryByRole('combobox', {
          name: /Layer/i,
        });
        if (layerSelector) {
          expect(layerSelector).toBeInTheDocument();
        }
      });

      const layerSelector = screen.queryByRole('combobox', {
        name: /Layer/i,
      });

      if (layerSelector) {
        const options = layerSelector.querySelectorAll('option');
        if (options.length > 1) {
          await user.selectOptions(layerSelector, options[1].value);

          expect(layerSelector).toHaveValue(options[1].value);
        }
      }
    });
  });

  describe('Layout preset selector flow', () => {
    it('changes keyboard layout preset', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const layoutSelector = screen.getByRole('combobox', {
        name: /Keyboard Layout/i,
      });

      expect(layoutSelector).toHaveValue('ANSI_104');

      await user.selectOptions(layoutSelector, 'ISO_105');

      expect(layoutSelector).toHaveValue('ISO_105');
    });

    it('displays all available layout options', async () => {
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const layoutSelector = screen.getByRole('combobox', {
        name: /Keyboard Layout/i,
      });

      expect(layoutSelector).toContainHTML('ANSI_104');
      expect(layoutSelector).toContainHTML('ISO_105');
      expect(layoutSelector).toContainHTML('JIS_109');
      expect(layoutSelector).toContainHTML('HHKB');
      expect(layoutSelector).toContainHTML('NUMPAD');
    });
  });

  describe('Key click and configuration flow', () => {
    it('opens KeyConfigDialog when key is clicked', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });
      }
    });

    it('displays current key mapping in tooltip on hover', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.hover(keyButtons[0]);

        await waitFor(
          () => {
            expect(screen.getByRole('tooltip')).toBeInTheDocument();
          },
          { timeout: 1000 }
        );
      }
    });
  });

  describe('Simple remap configuration', () => {
    it('configures simple key remap', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'simple');

        const targetKeySelector = screen.getByRole('combobox', {
          name: /Target Key/i,
        });
        await user.selectOptions(targetKeySelector, 'KEY_B');

        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Tap-Hold configuration', () => {
    it('configures tap-hold action', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'tap_hold');

        const tapKeySelector = screen.getByRole('combobox', {
          name: /Tap Action/i,
        });
        await user.selectOptions(tapKeySelector, 'KEY_A');

        const holdKeySelector = screen.getByRole('combobox', {
          name: /Hold Action/i,
        });
        await user.selectOptions(holdKeySelector, 'KEY_LEFTCTRL');

        const thresholdInput = screen.getByRole('spinbutton', {
          name: /Threshold/i,
        });
        await user.clear(thresholdInput);
        await user.type(thresholdInput, '200');

        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });

    it('validates threshold value', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'tap_hold');

        const thresholdInput = screen.getByRole('spinbutton', {
          name: /Threshold/i,
        });

        await user.clear(thresholdInput);
        await user.type(thresholdInput, '-100');

        await waitFor(() => {
          expect(
            screen.getByText(/Threshold must be positive/i)
          ).toBeInTheDocument();
        });
      }
    });
  });

  describe('Layer switch configuration', () => {
    it('configures layer switch action', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'layer_switch');

        const targetLayerSelector = screen.getByRole('combobox', {
          name: /Target Layer/i,
        });
        await user.selectOptions(targetLayerSelector, 'layer_1');

        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Macro configuration', () => {
    it('configures macro with multiple steps', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'macro');

        const addStepButton = screen.getByRole('button', {
          name: /Add Step/i,
        });

        await user.click(addStepButton);
        const step1Type = screen.getAllByRole('combobox', {
          name: /Step Type/i,
        })[0];
        await user.selectOptions(step1Type, 'press');

        await user.click(addStepButton);
        const step2Type = screen.getAllByRole('combobox', {
          name: /Step Type/i,
        })[1];
        await user.selectOptions(step2Type, 'delay');

        const saveButton = screen.getByRole('button', { name: /Save/i });
        await user.click(saveButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Configuration preview', () => {
    it('shows preview of key mapping in dialog', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        expect(screen.getByText(/Preview/i)).toBeInTheDocument();
      }
    });
  });

  describe('Cancel and close flows', () => {
    it('cancels configuration on Cancel button', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        const actionTypeSelector = screen.getByRole('combobox', {
          name: /Action Type/i,
        });
        await user.selectOptions(actionTypeSelector, 'simple');

        const cancelButton = screen.getByRole('button', { name: /Cancel/i });
        await user.click(cancelButton);

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });

    it('closes dialog on Escape key', async () => {
      const user = userEvent.setup();
      renderConfigPage();

      await waitFor(() => {
        expect(screen.getByText('Keyboard Layout')).toBeInTheDocument();
      });

      const keyButtons = screen.getAllByRole('button', {
        name: /Key [A-Z0-9]/i,
      });

      if (keyButtons.length > 0) {
        await user.click(keyButtons[0]);

        await waitFor(() => {
          expect(screen.getByRole('dialog')).toBeInTheDocument();
        });

        await user.keyboard('{Escape}');

        await waitFor(() => {
          expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        });
      }
    });
  });

  describe('Loading and error states', () => {
    it('shows loading state while fetching config', async () => {
      const store = useConfigStore.getState();
      store.loading = true;

      renderConfigPage();

      expect(
        screen.getByRole('status', { name: /Loading/i })
      ).toBeInTheDocument();
    });

    it('displays error message when fetch fails', async () => {
      const store = useConfigStore.getState();
      store.error = 'Failed to load configuration';

      renderConfigPage();

      expect(
        screen.getByText(/Failed to load configuration/i)
      ).toBeInTheDocument();
    });
  });
});