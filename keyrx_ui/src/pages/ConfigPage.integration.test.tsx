/**
 * Integration tests for ConfigPage
 *
 * Exercises the CURRENT ConfigPage composition end-to-end (ProfileSidebar,
 * EditTab -> DeviceSelectionPanel/ConfigScopeTabs/GlobalKeyboardPanel/
 * DeviceKeyboardPanel/KeyConfigPanel, CodePanelContainer, SimulatorTab,
 * NotificationBanners, the save-confirmation ProfileDiffView modal) against
 * real child components, with HTTP mocked via MSW and the daemon WebSocket
 * mocked via jest-websocket-mock (see tests/helpers/websocket.ts).
 *
 * This intentionally does not re-test what a component's own *.test.tsx
 * already covers in isolation (e.g. KeyMappingDialog.test.tsx, CodePanelContainer.test.tsx,
 * KeyboardVisualizerContainer.test.tsx) -- it focuses on cross-component wiring
 * that only exists once ConfigPage assembles the pieces.
 */

import React from 'react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { screen, waitFor, within } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '../../tests/testUtils';
import {
  setupMockWebSocket,
  simulateConnected,
  waitForRpcRequest,
  sendRpcResponse,
} from '../../tests/helpers/websocket';
import { server } from '../test/mocks/server';
import ConfigPage from './ConfigPage';
import { useConfigStore } from '../stores/configStore';
import { env } from '../config/env';

// Mock WasmContext directly (same pattern as ConfigPage.test.tsx) so tests
// don't depend on the real WASM validation pipeline's timing.
vi.mock('../contexts/WasmContext', () => {
  const mockWasmContext = {
    isWasmReady: true,
    isLoading: false,
    error: null as Error | null,
    validateConfig: vi.fn().mockResolvedValue([]),
    runSimulation: vi.fn().mockResolvedValue(null),
  };

  return {
    useWasmContext: () => mockWasmContext,
    WasmProvider: ({ children }: { children: React.ReactNode }) => children,
  };
});

const DEFAULT_SOURCE = '// Config for profile\n';

/** Mock GET /api/profiles/:name/config -- there is no default handler for it. */
function mockProfileConfig(source = DEFAULT_SOURCE) {
  server.use(
    http.get('/api/profiles/:name/config', ({ params }) =>
      HttpResponse.json({ name: params.name as string, source })
    )
  );
}

function renderConfigPage(source = DEFAULT_SOURCE) {
  mockProfileConfig(source);
  return renderWithProviders(<ConfigPage />, { wrapWithRouter: true });
}

/** Waits for the workspace header (unambiguous h1, unlike the profile name
 * which also appears in the sidebar list) to confirm the page has settled
 * on the given profile. */
function waitForProfileHeading(name = 'default') {
  return screen.findByRole('heading', { name, level: 1 });
}

/** The GlobalKeyboardPanel/DeviceKeyboardPanel keyboard uses KC_-prefixed codes
 * (see src/data/layouts/ANSI_104.json), not the VK_-prefixed codes the old UI used.
 * Both tabs stay mounted (see ConfigPage's `hidden`-class tab panes), so the
 * read-only SimulatorTab keyboard is also in the DOM; its keys end in
 * "Simulator mode active." rather than "Click to configure.", which this
 * matches on to stay scoped to the editable (Map tab) keyboard(s). */
const CAPS_LOCK_KEY_RE = /^Key KC_CAPS\..*Click to configure\./;

beforeEach(async () => {
  useConfigStore.getState().reset();
  localStorage.clear();
  // The global setup.ts beforeEach starts a mock WebSocket server at the
  // helper's default WS_URL ('ws://localhost:3030/ws'), but ConfigPage's
  // useUnifiedApi() (no explicit URL) connects to env.wsUrl
  // ('ws://localhost:9867/ws-rpc' in this dev-mode test env, see
  // src/config/constants.ts DEFAULT_DAEMON_PORT/WS_RPC_PATH). Re-point the
  // mock server at the address the app actually dials so
  // simulateConnected()/waitForRpcRequest() work for ConfigPage.
  await setupMockWebSocket(env.wsUrl);
});

describe('ConfigPage - Integration Tests', () => {
  describe('Page composition', () => {
    it('renders the profile sidebar from the API, the workflow tabs, and the visual editor', async () => {
      renderConfigPage();

      // ProfileSidebar loads real profiles from GET /api/profiles
      await waitFor(() => {
        expect(
          screen.getByRole('button', { name: 'Select profile default' })
        ).toBeInTheDocument();
        expect(
          screen.getByRole('button', { name: 'Select profile gaming' })
        ).toBeInTheDocument();
      });

      // Workspace header: profile name + daemon connection badge + tabs
      expect(
        screen.getByRole('heading', { name: 'default', level: 1 })
      ).toBeInTheDocument();
      expect(screen.getByText('Daemon offline')).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /1\. Map/ })).toHaveAttribute(
        'aria-selected',
        'true'
      );
      expect(screen.getByRole('tab', { name: /2\. Test/ })).toHaveAttribute(
        'aria-selected',
        'false'
      );

      // EditTab composition: use-case guide, device selection, global keyboard
      expect(
        screen.getByText('What do you want this keyboard to do?')
      ).toBeInTheDocument();
      expect(screen.getByTestId('device-selector')).toBeInTheDocument();
      expect(
        await screen.findByRole('button', { name: CAPS_LOCK_KEY_RE })
      ).toBeInTheDocument();
    });

    it('shows connected state once the daemon WebSocket handshake completes', async () => {
      renderConfigPage();
      await waitForProfileHeading();

      expect(screen.getByText('Daemon offline')).toBeInTheDocument();

      await simulateConnected();

      await waitFor(() => {
        expect(screen.getByText('Daemon connected')).toBeInTheDocument();
      });
    });

    it('switches between the Map and Test workflow tabs', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();

      const mapTab = screen.getByRole('tab', { name: /1\. Map/ });
      const testTab = screen.getByRole('tab', { name: /2\. Test/ });
      expect(mapTab).toHaveAttribute('aria-selected', 'true');

      await user.click(testTab);

      expect(testTab).toHaveAttribute('aria-selected', 'true');
      expect(mapTab).toHaveAttribute('aria-selected', 'false');
      // SimulatorTab content is now the active pane
      expect(
        screen.getByRole('heading', { name: 'Interactive Keyboard' })
      ).toBeInTheDocument();

      await user.click(mapTab);
      expect(mapTab).toHaveAttribute('aria-selected', 'true');
    });

    it('toggles the code panel open and closed', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();

      expect(
        screen.queryByTestId('code-panel-container')
      ).not.toBeInTheDocument();

      await user.click(screen.getByRole('button', { name: /Show Code/i }));

      await waitFor(() => {
        expect(screen.getByTestId('code-panel-container')).toBeInTheDocument();
      });
      expect(screen.getByText(/Code - default/)).toBeInTheDocument();

      // Exact, case-sensitive match: the workspace-header toggle is
      // "Hide Code", while CodePanelContainer's own collapse button inside
      // the panel is separately labelled "Hide code editor".
      await user.click(screen.getByRole('button', { name: 'Hide Code' }));
      await waitFor(() => {
        expect(
          screen.queryByTestId('code-panel-container')
        ).not.toBeInTheDocument();
      });
    });
  });

  describe('Device scope (global vs. device-specific)', () => {
    it('lists real devices from the API and lets the user select one for device-specific editing', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();

      const deviceSelector = screen.getByTestId('device-selector');
      expect(
        within(deviceSelector).getByText('Test Keyboard 1')
      ).toBeInTheDocument();
      expect(
        within(deviceSelector).getByText('Test Keyboard 2')
      ).toBeInTheDocument();

      // Global keyboard panel is visible by default (globalSelected: true)
      expect(
        screen.getByRole('heading', { name: 'Global Keys' })
      ).toBeInTheDocument();

      // Selecting a device (while still global) reveals the scope tabs
      await user.click(
        screen.getByRole('checkbox', { name: 'Select device Test Keyboard 1' })
      );

      const scopeTabs = await screen.findByRole('tablist', {
        name: 'Keyboard configuration scope',
      });
      expect(
        within(scopeTabs).getByRole('tab', { name: 'Device Keys' })
      ).toBeInTheDocument();

      // Device keyboard panel now renders with a device selector for the pane
      expect(screen.getByLabelText('Select device to configure')).toHaveValue(
        'device-1'
      );
    });

    it('warns when neither global nor any device is selected', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();

      await user.click(screen.getByTestId('global-checkbox'));

      expect(
        await screen.findByText('No devices selected')
      ).toBeInTheDocument();
      expect(
        screen.queryByRole('heading', { name: 'Global Keys' })
      ).not.toBeInTheDocument();
    });
  });

  describe('Visual key mapping editor', () => {
    it('configures, previews, and clears a key mapping directly on the keyboard', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();

      const capsKey = await screen.findByRole('button', {
        name: CAPS_LOCK_KEY_RE,
      });
      await user.click(capsKey);

      // KeyConfigPanel now shows the inline (non-modal) editor. The physical
      // key it's editing is keyed by the *normalized* VK_ code (see
      // SVGKeyboard.tsx normalizeKeyCode: KC_CAPS -> VK_CapsLock), which is
      // what onKeyClick actually receives -- the visible "KC_CAPS" label on
      // the key itself is only the raw layout-JSON code used for display.
      expect(
        screen.queryByText('Click a key on the keyboard above to configure it')
      ).not.toBeInTheDocument();
      expect(await screen.findByText('VK_CapsLock')).toBeInTheDocument();

      // Pick a target via the Lock tab (unambiguous vs. the keyboard-shaped
      // "keyboard" tab, which would render a second, identically-labelled
      // set of "Key KC_*" buttons).
      await user.click(screen.getByRole('tab', { name: /Select lock state/i }));
      await user.click(screen.getByRole('button', { name: 'Lock CapsLock' }));

      expect(
        screen.getByText('Press VK_CapsLock → Output LK_00')
      ).toBeInTheDocument();

      await user.click(screen.getByRole('button', { name: 'Save Mapping' }));

      // Mapping is now reflected in the shared configStore, which
      // re-renders both the keyboard visualizer and the mappings summary.
      await waitFor(() => {
        expect(
          screen.getByText('Current Mappings (1 mappings)')
        ).toBeInTheDocument();
      });
      expect(
        await screen.findByRole('button', {
          name: /^Key KC_CAPS\. KC_CAPS → LK_00\./,
        })
      ).toBeInTheDocument();
      expect(screen.getByText(/Unsaved/i)).toBeInTheDocument();

      // Re-select the mapped key and clear it
      await user.click(
        screen.getByRole('button', { name: /^Key KC_CAPS\. KC_CAPS → LK_00\./ })
      );
      await user.click(screen.getByRole('button', { name: 'Clear Mapping' }));

      await waitFor(() => {
        expect(
          screen.getByText('Current Mappings (0 mappings)')
        ).toBeInTheDocument();
      });
      expect(
        await screen.findByRole('button', { name: CAPS_LOCK_KEY_RE })
      ).toBeInTheDocument();
    });

    it('deselects the active key on Escape', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();

      const capsKey = await screen.findByRole('button', {
        name: CAPS_LOCK_KEY_RE,
      });
      await user.click(capsKey);
      // See the normalization note above: the panel is keyed by VK_CapsLock.
      expect(await screen.findByText('VK_CapsLock')).toBeInTheDocument();

      await user.keyboard('{Escape}');

      expect(
        screen.getByText('Click a key on the keyboard above to configure it')
      ).toBeInTheDocument();
    });
  });

  describe('Save flow', () => {
    it('opens a diff review before saving, sends set_profile_config over the daemon RPC, and returns to Saved', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();
      await simulateConnected();

      // Make an edit via the visual editor so the code differs from the server source
      const capsKey = await screen.findByRole('button', {
        name: CAPS_LOCK_KEY_RE,
      });
      await user.click(capsKey);
      await user.click(screen.getByRole('tab', { name: /Select lock state/i }));
      await user.click(screen.getByRole('button', { name: 'Lock CapsLock' }));
      await user.click(screen.getByRole('button', { name: 'Save Mapping' }));
      await screen.findByText(/Unsaved/i);

      const topSaveButton = screen.getByRole('button', { name: /^Save$/ });
      expect(topSaveButton).toBeEnabled();
      await user.click(topSaveButton);

      const dialog = await screen.findByRole('dialog', {
        name: 'Review Changes',
      });
      expect(
        within(dialog).getByRole('button', { name: 'Confirm Save' })
      ).toBeInTheDocument();

      await user.click(
        within(dialog).getByRole('button', { name: 'Confirm Save' })
      );

      const request = await waitForRpcRequest('set_profile_config');
      expect(request.params).toMatchObject({ name: 'default' });
      sendRpcResponse(request.id, null);

      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
      await waitFor(() => {
        expect(screen.getByText('Saved')).toBeInTheDocument();
      });
    });

    it('saves directly without a diff review when there are no pending code changes', async () => {
      const user = userEvent.setup();
      renderConfigPage();
      await waitForProfileHeading();
      await simulateConnected();

      const topSaveButton = await screen.findByRole('button', {
        name: /^Save$/,
      });
      await user.click(topSaveButton);

      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

      const request = await waitForRpcRequest('set_profile_config');
      sendRpcResponse(request.id, null);

      await waitFor(() => {
        expect(screen.getByText('Saved')).toBeInTheDocument();
      });
    });
  });

  describe('Notifications', () => {
    it("prompts to create the profile when the selected profile doesn't exist, and creates it", async () => {
      const user = userEvent.setup();

      // Empty backend: no profiles at all. ConfigPage falls back to
      // selectedProfileName 'default', which doesn't exist yet.
      //
      // Note: ProfileSidebar *also* auto-creates a 'default' profile the
      // first time it sees an empty list (see ProfileSidebar.tsx
      // autoCreateDefault) -- if that succeeded here it would immediately
      // resolve the "doesn't exist" state before this test could observe
      // the banner, since ConfigPage's own "auto-select first profile"
      // effect would then also kick in. So the first POST /api/profiles
      // is made to fail (simulating that automatic attempt failing, e.g.
      // a transient daemon error, which ProfileSidebar just toasts and
      // gives up on), leaving the manual "Create Profile" banner button
      // as the user's real recourse -- the second POST (from clicking it)
      // succeeds.
      const profiles: Array<Record<string, unknown>> = [];
      let createAttempts = 0;

      server.use(
        http.get('/api/profiles', () => HttpResponse.json({ profiles })),
        http.get('/api/profiles/active', () =>
          HttpResponse.json({ active_profile: null })
        ),
        http.get('/api/profiles/:name/config', () =>
          HttpResponse.json(
            { error: 'Profile not found', errorCode: 'PROFILE_NOT_FOUND' },
            { status: 404 }
          )
        ),
        http.post('/api/profiles', async ({ request }) => {
          createAttempts += 1;
          if (createAttempts === 1) {
            return HttpResponse.json(
              { error: 'Daemon unavailable', errorCode: 'DAEMON_ERROR' },
              { status: 503 }
            );
          }
          const body = (await request.json()) as { name: string };
          const profile = {
            name: body.name,
            rhaiPath: `/p/${body.name}.rhai`,
            krxPath: `/p/${body.name}.krx`,
            isActive: false,
            createdAt: new Date().toISOString(),
            modifiedAt: new Date().toISOString(),
            layerCount: 1,
            deviceCount: 0,
            keyCount: 0,
          };
          profiles.push(profile);
          return HttpResponse.json(profile);
        })
      );

      renderWithProviders(<ConfigPage />, { wrapWithRouter: true });
      await simulateConnected();

      // The 404 profile-config fetch retries once (react-query retry: 1,
      // retryDelay: 1000ms per useGetProfileConfig) before NotificationBanners'
      // isLoading gate clears, so this needs more than the 1s findBy default.
      const createButton = await screen.findByRole(
        'button',
        { name: 'Create Profile "default"' },
        { timeout: 3000 }
      );
      expect(
        screen.getByText('Profile "default" does not exist.')
      ).toBeInTheDocument();

      await user.click(createButton);

      await waitFor(() => {
        expect(
          screen.queryByText('Profile "default" does not exist.')
        ).not.toBeInTheDocument();
      });
      expect(createAttempts).toBe(2);
    });
  });
});
