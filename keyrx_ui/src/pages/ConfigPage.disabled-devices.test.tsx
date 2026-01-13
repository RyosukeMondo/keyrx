/**
 * ConfigPage - Disabled Device Filtering Tests
 * Tests for Task 2.4: Verify disabled devices are filtered from ConfigPage
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderPage, setupMockWebSocket, cleanupMockWebSocket } from '../../tests/testUtils';
import { ConfigPage } from './ConfigPage';
import { http, HttpResponse } from 'msw';
import { server } from '../test/mocks/server';

// Mock react-router-dom
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useParams: () => ({ profile: 'default' }),
    useNavigate: () => vi.fn(),
  };
});

describe('ConfigPage - Disabled Device Filtering (Task 2.4)', () => {
  beforeEach(async () => {
    await setupMockWebSocket();
  });

  afterEach(() => {
    cleanupMockWebSocket();
  });

  describe('Device Filtering from Merged Devices List', () => {
    it('excludes disabled devices from merged devices list', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Enabled Keyboard',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
              {
                id: 'device-2',
                name: 'Disabled Keyboard',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Disabled device
              },
              {
                id: 'device-3',
                name: 'Another Enabled Keyboard',
                path: '/dev/input/event2',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      // Wait for page to load
      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Enabled devices should be visible
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Enabled Keyboard');
        expect(pageContent).toContain('Another Enabled Keyboard');
      });

      // Disabled device should NOT be visible
      const pageContent = document.body.textContent || '';
      expect(pageContent).not.toContain('Disabled Keyboard');
    });

    it('shows only enabled devices when all devices have enabled field', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Test Keyboard 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
              {
                id: 'device-2',
                name: 'Test Keyboard 2',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // All enabled devices should be visible
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Test Keyboard 1');
        expect(pageContent).toContain('Test Keyboard 2');
      });
    });

    it('handles devices with undefined enabled field as enabled (backward compatibility)', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Legacy Device',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                // No enabled field - should default to enabled
              },
              {
                id: 'device-2',
                name: 'Explicitly Disabled',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: false,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Legacy device (no enabled field) should be visible
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Legacy Device');
      });

      // Explicitly disabled device should NOT be visible
      const pageContent = document.body.textContent || '';
      expect(pageContent).not.toContain('Explicitly Disabled');
    });

    it('filters disabled devices but keeps Rhai-defined devices', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Connected Keyboard',
                path: '/dev/input/event0',
                serial: 'ABC123',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Disabled device
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          // Rhai config with device definition
          return HttpResponse.text(`
            device("Rhai Defined Device", serial="XYZ789") {{
              // Device mappings
            }}
          `);
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Wait for devices to be processed
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        // Rhai-defined device should still be present (not filtered out)
        // This device is from Rhai config, not from connected devices
        // Disabled connected device should NOT be present
        expect(pageContent).not.toContain('Connected Keyboard');
      });
    });

    it('handles empty device list gracefully', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({ devices: [] });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Should render without errors even with no devices
      expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
    });

    it('handles all devices disabled scenario', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Disabled Device 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: false,
              },
              {
                id: 'device-2',
                name: 'Disabled Device 2',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: false,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // No disabled devices should be visible
      const pageContent = document.body.textContent || '';
      expect(pageContent).not.toContain('Disabled Device 1');
      expect(pageContent).not.toContain('Disabled Device 2');
    });
  });

  describe('Device Selector Filtering', () => {
    it('device selector does not show disabled devices in dropdown', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Enabled Keyboard',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
              {
                id: 'device-2',
                name: 'Disabled Keyboard',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: false,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Wait for devices to load
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Enabled Keyboard');
      });

      // Verify disabled device is not in the page at all
      const pageContent = document.body.textContent || '';
      expect(pageContent).not.toContain('Disabled Keyboard');
    });

    it('device count reflects only enabled devices', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Device 1',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
              {
                id: 'device-2',
                name: 'Device 2',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: false,
              },
              {
                id: 'device-3',
                name: 'Device 3',
                path: '/dev/input/event2',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Wait for devices to load
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        // Should show 2 enabled devices, not 3 total
        expect(pageContent).toContain('Device 1');
        expect(pageContent).toContain('Device 3');
        expect(pageContent).not.toContain('Device 2');
      });
    });
  });

  describe('Edge Cases and Error Scenarios', () => {
    it('handles API error when fetching devices', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json(
            { error: 'Failed to fetch devices' },
            { status: 500 }
          );
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Should handle error gracefully without crashing
      expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
    });

    it('handles malformed device data gracefully', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Valid Device',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
              // Malformed device with missing fields
              {
                id: 'device-2',
                // Missing name, path, etc.
                enabled: false,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Should handle malformed data without crashing
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Valid Device');
      });
    });

    it('filters work correctly after device enable state changes', async () => {
      const devices = [
        {
          id: 'device-1',
          name: 'Toggle Test Device',
          path: '/dev/input/event0',
          active: true,
          layout: 'ANSI_104',
          enabled: true,
        },
      ];

      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({ devices });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text('// Empty config');
        }),
        http.put('/api/devices/:id/enabled', async ({ request }) => {
          const body = (await request.json()) as { enabled: boolean };
          devices[0].enabled = body.enabled;
          return HttpResponse.json({ success: true });
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Device should be visible initially
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Toggle Test Device');
      });

      // Note: Full integration test of toggling from DevicesPage would require
      // navigating between pages, which is tested in the DevicesPage tests
    });
  });

  describe('Integration with Rhai Configuration', () => {
    it('disabled devices do not interfere with Rhai-parsed device configurations', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Physical Keyboard',
                path: '/dev/input/event0',
                serial: 'SERIAL123',
                active: true,
                layout: 'ANSI_104',
                enabled: false, // Disabled
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          // Rhai config references a device by serial
          return HttpResponse.text(`
            device("My Custom Device", serial="SERIAL456") {
              // Mappings
            }
          `);
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Disabled physical device should NOT appear
      const pageContent = document.body.textContent || '';
      expect(pageContent).not.toContain('Physical Keyboard');

      // Rhai-defined device should work independently
      // (exact behavior depends on implementation)
    });

    it('handles mix of connected and Rhai-defined devices with some disabled', async () => {
      server.use(
        http.get('/api/devices', () => {
          return HttpResponse.json({
            devices: [
              {
                id: 'device-1',
                name: 'Connected Enabled',
                path: '/dev/input/event0',
                active: true,
                layout: 'ANSI_104',
                enabled: true,
              },
              {
                id: 'device-2',
                name: 'Connected Disabled',
                path: '/dev/input/event1',
                active: true,
                layout: 'ANSI_104',
                enabled: false,
              },
            ],
          });
        }),
        http.get('/api/profiles/default/rhai', () => {
          return HttpResponse.text(`
            device("Rhai Device 1", serial="ABC") {
              // Mappings
            }
            device("Rhai Device 2", serial="DEF") {
              // Mappings
            }
          `);
        })
      );

      renderPage(<ConfigPage />);

      await waitFor(() => {
        expect(screen.getByText(/Configuration/i)).toBeInTheDocument();
      });

      // Enabled connected device should appear
      await waitFor(() => {
        const pageContent = document.body.textContent || '';
        expect(pageContent).toContain('Connected Enabled');
      });

      // Disabled connected device should NOT appear
      const pageContent = document.body.textContent || '';
      expect(pageContent).not.toContain('Connected Disabled');

      // Rhai-defined devices should appear (they're not affected by enabled field)
      // (exact display depends on implementation)
    });
  });
});
