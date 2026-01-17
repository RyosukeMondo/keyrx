/**
 * Button Component Stories
 *
 * This file contains Storybook stories for the Button component,
 * demonstrating all its variants and states.
 */

import type { Meta, StoryObj } from '@storybook/react-vite';
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
      options: ['primary', 'secondary', 'danger', 'ghost'],
      description: 'Visual style variant',
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
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
    onClick: () => {},
    'aria-label': 'Button',
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
    'aria-label': 'Primary action button',
  },
};

export const Secondary: Story = {
  args: {
    children: 'Secondary Button',
    variant: 'secondary',
    'aria-label': 'Secondary action button',
  },
};

export const Danger: Story = {
  args: {
    children: 'Delete',
    variant: 'danger',
    'aria-label': 'Delete item',
  },
};

// =============================================================================
// Sizes
// =============================================================================

export const Small: Story = {
  args: {
    children: 'Small Button',
    size: 'sm',
    'aria-label': 'Small button',
  },
};

export const Medium: Story = {
  args: {
    children: 'Medium Button',
    size: 'md',
    'aria-label': 'Medium button',
  },
};

export const Large: Story = {
  args: {
    children: 'Large Button',
    size: 'lg',
    'aria-label': 'Large button',
  },
};

// =============================================================================
// States
// =============================================================================

export const Disabled: Story = {
  args: {
    children: 'Disabled Button',
    disabled: true,
    'aria-label': 'Disabled button',
  },
};

export const Loading: Story = {
  args: {
    children: 'Loading...',
    disabled: true,
    'aria-label': 'Loading button',
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
    'aria-label': 'Add new profile',
  },
};

export const FullWidth: Story = {
  args: {
    children: 'Full Width Button',
    'aria-label': 'Full width button',
    className: 'w-full',
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
