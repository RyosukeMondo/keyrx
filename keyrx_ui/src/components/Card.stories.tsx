/**
 * Card Component Stories
 *
 * This file contains Storybook stories for the Card component,
 * demonstrating various use cases and content patterns.
 */

import type { Meta, StoryObj } from '@storybook/react';
import { Card } from './Card';
import { Button } from './Button';

const meta = {
  title: 'Components/Card',
  component: Card,
  parameters: {
    layout: 'padded',
    a11y: {
      config: {
        rules: [
          {
            // Ensure cards have proper semantic structure
            id: 'region',
            enabled: true,
          },
        ],
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    elevation: {
      control: 'select',
      options: ['none', 'low', 'medium', 'high'],
      description: 'Shadow elevation level',
    },
    padding: {
      control: 'select',
      options: ['none', 'small', 'medium', 'large'],
      description: 'Internal padding size',
    },
  },
} satisfies Meta<typeof Card>;

export default meta;
type Story = StoryObj<typeof meta>;

// =============================================================================
// Basic Variants
// =============================================================================

export const Default: Story = {
  args: {
    children: (
      <>
        <h3>Card Title</h3>
        <p>This is a basic card with default styling.</p>
      </>
    ),
  },
};

export const WithElevation: Story = {
  args: {
    elevation: 'medium',
    children: (
      <>
        <h3>Elevated Card</h3>
        <p>This card has a shadow for visual depth.</p>
      </>
    ),
  },
};

export const NoPadding: Story = {
  args: {
    padding: 'none',
    children: (
      <div style={{ background: '#f0f0f0', padding: '16px' }}>
        <h3>Custom Padding</h3>
        <p>This card has no default padding, allowing full control.</p>
      </div>
    ),
  },
};

// =============================================================================
// Content Patterns
// =============================================================================

export const ProfileCard: Story = {
  args: {
    children: (
      <>
        <h3>Gaming Profile</h3>
        <p style={{ color: '#666' }}>Modified 2 hours ago</p>
        <p>
          <strong>Devices:</strong> 2 | <strong>Keys:</strong> 24
        </p>
        <div style={{ marginTop: '16px', display: 'flex', gap: '8px' }}>
          <Button variant="primary" size="small">
            Activate
          </Button>
          <Button variant="secondary" size="small">
            Edit
          </Button>
        </div>
      </>
    ),
  },
};

export const DeviceCard: Story = {
  args: {
    children: (
      <>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
          <div>
            <h3>Logitech G915 TKL</h3>
            <p style={{ color: '#666', margin: '4px 0' }}>
              Vendor: 046d | Product: c33f
            </p>
            <p style={{ color: '#00aa00', fontWeight: 'bold', fontSize: '14px' }}>
              ● Connected
            </p>
          </div>
          <Button variant="secondary" size="small">
            Configure
          </Button>
        </div>
      </>
    ),
  },
};

export const MetricsCard: Story = {
  args: {
    children: (
      <>
        <h3>Latency Statistics</h3>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px', marginTop: '16px' }}>
          <div>
            <p style={{ color: '#666', fontSize: '14px', margin: 0 }}>Average</p>
            <p style={{ fontSize: '24px', fontWeight: 'bold', margin: '4px 0 0 0' }}>125μs</p>
          </div>
          <div>
            <p style={{ color: '#666', fontSize: '14px', margin: 0 }}>P95</p>
            <p style={{ fontSize: '24px', fontWeight: 'bold', margin: '4px 0 0 0' }}>245μs</p>
          </div>
          <div>
            <p style={{ color: '#666', fontSize: '14px', margin: 0 }}>P99</p>
            <p style={{ fontSize: '24px', fontWeight: 'bold', margin: '4px 0 0 0' }}>380μs</p>
          </div>
          <div>
            <p style={{ color: '#666', fontSize: '14px', margin: 0 }}>Samples</p>
            <p style={{ fontSize: '24px', fontWeight: 'bold', margin: '4px 0 0 0' }}>1,245</p>
          </div>
        </div>
      </>
    ),
  },
};

// =============================================================================
// Loading States
// =============================================================================

export const LoadingState: Story = {
  args: {
    children: (
      <>
        <div style={{ opacity: 0.5 }}>
          <h3>Loading...</h3>
          <p>Fetching data from daemon...</p>
        </div>
      </>
    ),
  },
};

export const EmptyState: Story = {
  args: {
    children: (
      <div style={{ textAlign: 'center', padding: '32px 16px', color: '#999' }}>
        <p style={{ fontSize: '48px', margin: '0 0 16px 0' }}>📋</p>
        <h3>No profiles yet</h3>
        <p>Create your first profile to get started</p>
        <Button variant="primary" style={{ marginTop: '16px' }}>
          Create Profile
        </Button>
      </div>
    ),
  },
};

// =============================================================================
// Error States
// =============================================================================

export const ErrorState: Story = {
  args: {
    children: (
      <div style={{ padding: '16px', background: '#fff3f3', border: '1px solid #ffcccc', borderRadius: '4px' }}>
        <h3 style={{ color: '#cc0000', margin: '0 0 8px 0' }}>⚠ Error</h3>
        <p style={{ color: '#660000', margin: 0 }}>
          Failed to connect to daemon. Please ensure the daemon is running.
        </p>
        <Button variant="danger" size="small" style={{ marginTop: '12px' }}>
          Retry Connection
        </Button>
      </div>
    ),
  },
};
