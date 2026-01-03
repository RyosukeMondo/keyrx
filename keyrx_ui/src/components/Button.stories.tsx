/**
 * Button Component Stories
 *
 * This file contains Storybook stories for the Button component,
 * demonstrating all its variants and states.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { fn } from '@storybook/test';
import { Button } from './Button';

const meta = {
  title: 'Components/Button',
  component: Button,
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/configure/story-layout
    layout: 'centered',
    // Enable accessibility addon
    a11y: {
      element: 'button',
      config: {
        rules: [
          {
            // Ensure buttons have accessible text
            id: 'button-name',
            enabled: true,
          },
        ],
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'danger'],
      description: 'Visual style variant',
    },
    size: {
      control: 'select',
      options: ['small', 'medium', 'large'],
      description: 'Button size',
    },
    disabled: {
      control: 'boolean',
      description: 'Whether the button is disabled',
    },
    onClick: {
      action: 'clicked',
      description: 'Click handler function',
    },
  },
  args: {
    onClick: fn(),
  },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

// =============================================================================
// Basic Variants
// =============================================================================

export const Primary: Story = {
  args: {
    children: 'Primary Button',
    variant: 'primary',
  },
};

export const Secondary: Story = {
  args: {
    children: 'Secondary Button',
    variant: 'secondary',
  },
};

export const Danger: Story = {
  args: {
    children: 'Delete',
    variant: 'danger',
  },
};

// =============================================================================
// Sizes
// =============================================================================

export const Small: Story = {
  args: {
    children: 'Small Button',
    size: 'small',
  },
};

export const Medium: Story = {
  args: {
    children: 'Medium Button',
    size: 'medium',
  },
};

export const Large: Story = {
  args: {
    children: 'Large Button',
    size: 'large',
  },
};

// =============================================================================
// States
// =============================================================================

export const Disabled: Story = {
  args: {
    children: 'Disabled Button',
    disabled: true,
  },
};

export const Loading: Story = {
  args: {
    children: 'Loading...',
    disabled: true,
  },
};

// =============================================================================
// Interactive Examples
// =============================================================================

export const WithIcon: Story = {
  args: {
    children: (
      <>
        <span aria-hidden="true">+</span> Add Profile
      </>
    ),
    variant: 'primary',
  },
};

export const FullWidth: Story = {
  args: {
    children: 'Full Width Button',
    style: { width: '100%' },
  },
};

// =============================================================================
// Accessibility Scenarios
// =============================================================================

export const AccessibleName: Story = {
  args: {
    children: 'Save',
    'aria-label': 'Save profile configuration',
  },
  parameters: {
    docs: {
      description: {
        story: 'Button with explicit aria-label for screen readers',
      },
    },
  },
};

export const IconOnly: Story = {
  args: {
    children: <span>×</span>,
    'aria-label': 'Close dialog',
  },
  parameters: {
    docs: {
      description: {
        story: 'Icon-only buttons MUST have aria-label for accessibility',
      },
    },
  },
};
