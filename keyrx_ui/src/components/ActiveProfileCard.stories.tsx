/**
 * ActiveProfileCard Component Stories
 *
 * Demonstrates the ActiveProfileCard component with realistic data
 * using faker-js factories for consistency.
 */

import type { Meta, StoryObj } from '@storybook/react-vite';
import { ActiveProfileCard } from './ActiveProfileCard';
import { createActiveProfile, seed } from '../../tests/factories';

const meta = {
  title: 'Components/ActiveProfileCard',
  component: ActiveProfileCard,
  parameters: {
    layout: 'padded',
    a11y: {
      config: {
        rules: [
          {
            id: 'landmark-unique',
            enabled: true,
          },
        ],
      },
    },
  },
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <div style={{ maxWidth: '400px', margin: '0 auto' }}>
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ActiveProfileCard>;

export default meta;
type Story = StoryObj<typeof meta>;

// =============================================================================
// Realistic Data Using Factories
// =============================================================================

// Seed for consistent visual regression testing
seed(42);

export const ActiveProfile: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Gaming',
      layers: 2,
      mappings: 24,
    }),
  },
};

export const MultipleDevices: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Work Setup',
      layers: 5,
      mappings: 84,
    }),
  },
};

export const MinimalProfile: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Simple',
      layers: 1,
      mappings: 4,
    }),
  },
};

export const ManyKeys: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Power User',
      layers: 3,
      mappings: 104,
    }),
  },
};

// =============================================================================
// Interactive States
// =============================================================================

export const WithCustomProfile: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Test Profile',
      layers: 3,
      mappings: 42,
    }),
  },
  parameters: {
    docs: {
      description: {
        story: 'Card with custom profile data',
      },
    },
  },
};

// =============================================================================
// Responsive Behavior
// =============================================================================

export const MobileView: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Mobile Profile',
      layers: 1,
      mappings: 12,
    }),
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile1',
    },
  },
};

export const TabletView: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Tablet Profile',
      layers: 2,
      mappings: 48,
    }),
  },
  parameters: {
    viewport: {
      defaultViewport: 'tablet',
    },
  },
};

// =============================================================================
// Edge Cases
// =============================================================================

export const LongName: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Super Long Profile Name That Should Truncate Gracefully',
      layers: 3,
      mappings: 64,
    }),
  },
};

export const NoDevices: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Empty Profile',
      layers: 0,
      mappings: 0,
    }),
  },
  parameters: {
    docs: {
      description: {
        story: 'Profile with no configured devices or keys',
      },
    },
  },
};

// =============================================================================
// Visual Regression Testing
// =============================================================================

export const VisualRegression: Story = {
  args: {
    profile: createActiveProfile({
      name: 'Baseline',
      layers: 2,
      mappings: 32,
      modifiedAt: '2024-01-02T12:30:00Z',
    }),
  },
  parameters: {
    chromatic: {
      // Take snapshots at multiple viewport sizes
      viewports: [320, 768, 1024, 1920],
    },
    docs: {
      description: {
        story: 'Baseline story for visual regression testing with Chromatic',
      },
    },
  },
};
