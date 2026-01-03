/**
 * ActiveProfileCard Component Stories
 *
 * Demonstrates the ActiveProfileCard component with realistic data
 * using faker-js factories for consistency.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { ActiveProfileCard } from './ActiveProfileCard';
import { createProfile, seed } from '../../tests/factories';

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
    profile: createProfile({
      name: 'Gaming',
      isActive: true,
      deviceCount: 2,
      keyCount: 24,
    }),
  },
};

export const MultipleDevices: Story = {
  args: {
    profile: createProfile({
      name: 'Work Setup',
      isActive: true,
      deviceCount: 5,
      keyCount: 84,
    }),
  },
};

export const MinimalProfile: Story = {
  args: {
    profile: createProfile({
      name: 'Simple',
      isActive: true,
      deviceCount: 1,
      keyCount: 4,
    }),
  },
};

export const ManyKeys: Story = {
  args: {
    profile: createProfile({
      name: 'Power User',
      isActive: true,
      deviceCount: 3,
      keyCount: 104,
    }),
  },
};

// =============================================================================
// Interactive States
// =============================================================================

export const WithCustomActions: Story = {
  args: {
    profile: createProfile({
      name: 'Test Profile',
      isActive: true,
    }),
    onEdit: () => alert('Edit clicked!'),
    onDuplicate: () => alert('Duplicate clicked!'),
  },
  parameters: {
    docs: {
      description: {
        story: 'Card with interactive action buttons',
      },
    },
  },
};

// =============================================================================
// Responsive Behavior
// =============================================================================

export const MobileView: Story = {
  args: {
    profile: createProfile({
      name: 'Mobile Profile',
      isActive: true,
      deviceCount: 1,
      keyCount: 12,
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
    profile: createProfile({
      name: 'Tablet Profile',
      isActive: true,
      deviceCount: 2,
      keyCount: 48,
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
    profile: createProfile({
      name: 'Super Long Profile Name That Should Truncate Gracefully',
      isActive: true,
      deviceCount: 3,
      keyCount: 64,
    }),
  },
};

export const NoDevices: Story = {
  args: {
    profile: createProfile({
      name: 'Empty Profile',
      isActive: true,
      deviceCount: 0,
      keyCount: 0,
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
    profile: createProfile({
      name: 'Baseline',
      isActive: true,
      deviceCount: 2,
      keyCount: 32,
      createdAt: '2024-01-01T00:00:00Z',
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
