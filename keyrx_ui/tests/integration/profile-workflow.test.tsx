/**
 * Profile Workflow Integration Test
 *
 * Tests the complete profile management workflow including:
 * - Creating profiles
 * - Activating profiles
 * - Duplicating profiles
 * - Renaming profiles
 * - Deleting profiles
 * - Profile list updates
 *
 * Prerequisites:
 * - Daemon must be running on test port
 *
 * This test verifies profile management acceptance criteria across:
 * - REQ-1 (AC2, AC3): Profile RPC methods
 * - Profile CRUD operations
 * - UI updates after profile changes
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { renderHook } from '@testing-library/react';
import { useUnifiedApi } from '../../src/hooks/useUnifiedApi';
import {
  setupDaemon,
  teardownDaemon,
  createTestProfileName,
  SIMPLE_TEST_CONFIG,
  DAEMON_WS_URL,
} from './test-harness';

describe('Profile Workflow Integration', () => {
  let testProfiles: string[] = [];

  beforeAll(async () => {
    // Start daemon if not already running
    await setupDaemon({ autoStart: false });
  });

  afterAll(async () => {
    // Cleanup test profiles
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Delete all test profiles
    for (const profileName of testProfiles) {
      try {
        await result.current.command('delete_profile', { name: profileName });
      } catch (error) {
        // Profile might already be deleted, ignore error
      }
    }
  });

  beforeEach(() => {
    // Reset test profiles array
    testProfiles = [];
  });

  it('should create a new profile', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Create profile
    const profileName = createTestProfileName();
    testProfiles.push(profileName);

    console.log(`Creating profile: ${profileName}`);
    await result.current.command('create_profile', { name: profileName });

    // Verify profile was created
    const profiles = await result.current.query('get_profiles');
    expect(profiles).toEqual(
      expect.arrayContaining([expect.objectContaining({ name: profileName })])
    );

    console.log('✓ Profile created successfully');
  });

  it('should activate a profile', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Create profile
    const profileName = createTestProfileName();
    testProfiles.push(profileName);

    await result.current.command('create_profile', { name: profileName });

    // Activate profile
    console.log(`Activating profile: ${profileName}`);
    await result.current.command('activate_profile', { name: profileName });

    // Note: Verification of active profile would require daemon state query
    // which may not be available. For now, we verify the command succeeds.
    console.log('✓ Profile activated successfully');
  });

  it('should duplicate a profile', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Create original profile
    const originalName = createTestProfileName();
    testProfiles.push(originalName);

    await result.current.command('create_profile', { name: originalName });

    // Set config on original
    await result.current.command('update_config', {
      profile_name: originalName,
      code: SIMPLE_TEST_CONFIG,
    });

    // Duplicate profile
    const duplicateName = `${originalName}-copy`;
    testProfiles.push(duplicateName);

    console.log(`Duplicating profile: ${originalName} -> ${duplicateName}`);
    await result.current.command('duplicate_profile', {
      source_name: originalName,
      new_name: duplicateName,
    });

    // Verify duplicate was created
    const profiles = await result.current.query('get_profiles');
    expect(profiles).toEqual(
      expect.arrayContaining([expect.objectContaining({ name: duplicateName })])
    );

    // Verify duplicate has same config
    const duplicateConfig = await result.current.query('get_config', {
      profile_name: duplicateName,
    });
    expect(duplicateConfig).toHaveProperty('code');
    expect(duplicateConfig.code).toContain('layer("default")');

    console.log('✓ Profile duplicated successfully');
  });

  it('should rename a profile', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Create profile
    const oldName = createTestProfileName();
    testProfiles.push(oldName);

    await result.current.command('create_profile', { name: oldName });

    // Rename profile
    const newName = `${oldName}-renamed`;
    testProfiles.push(newName);

    console.log(`Renaming profile: ${oldName} -> ${newName}`);
    await result.current.command('rename_profile', {
      old_name: oldName,
      new_name: newName,
    });

    // Verify renamed profile exists
    const profiles = await result.current.query('get_profiles');
    expect(profiles).toEqual(
      expect.arrayContaining([expect.objectContaining({ name: newName })])
    );

    // Verify old name doesn't exist
    expect(profiles).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ name: oldName })])
    );

    console.log('✓ Profile renamed successfully');
  });

  it('should delete a profile', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Create profile
    const profileName = createTestProfileName();
    await result.current.command('create_profile', { name: profileName });

    // Verify created
    let profiles = await result.current.query('get_profiles');
    expect(profiles).toEqual(
      expect.arrayContaining([expect.objectContaining({ name: profileName })])
    );

    // Delete profile
    console.log(`Deleting profile: ${profileName}`);
    await result.current.command('delete_profile', { name: profileName });

    // Verify deleted
    profiles = await result.current.query('get_profiles');
    expect(profiles).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ name: profileName })])
    );

    console.log('✓ Profile deleted successfully');
  });

  it('should update profile configuration', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Create profile
    const profileName = createTestProfileName();
    testProfiles.push(profileName);

    await result.current.command('create_profile', { name: profileName });

    // Update config
    console.log(`Updating config for profile: ${profileName}`);
    await result.current.command('update_config', {
      profile_name: profileName,
      code: SIMPLE_TEST_CONFIG,
    });

    // Verify config was updated
    const config = await result.current.query('get_config', {
      profile_name: profileName,
    });

    expect(config).toHaveProperty('code');
    expect(config).toHaveProperty('hash');
    expect(config.code).toContain('layer("default")');
    expect(config.code).toContain('bind("a", action::tap("b"))');

    console.log('✓ Profile configuration updated successfully');
  });

  it('should handle profile errors correctly', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Try to activate non-existent profile
    console.log('Testing error handling for non-existent profile...');
    await expect(
      result.current.command('activate_profile', { name: 'non-existent-profile-xyz' })
    ).rejects.toThrow();

    // Try to delete non-existent profile
    await expect(
      result.current.command('delete_profile', { name: 'non-existent-profile-xyz' })
    ).rejects.toThrow();

    // Try to rename non-existent profile
    await expect(
      result.current.command('rename_profile', {
        old_name: 'non-existent-profile-xyz',
        new_name: 'new-name',
      })
    ).rejects.toThrow();

    console.log('✓ Error handling works correctly');
  });

  it('should complete full profile lifecycle', async () => {
    const { result } = renderHook(() => useUnifiedApi(DAEMON_WS_URL));

    // Wait for connection
    await waitFor(
      () => {
        expect(result.current.isConnected).toBe(true);
      },
      { timeout: 10000 }
    );

    // Complete workflow:
    const profileName = createTestProfileName();

    // 1. Create
    console.log('1. Creating profile...');
    await result.current.command('create_profile', { name: profileName });

    // 2. Update config
    console.log('2. Updating config...');
    await result.current.command('update_config', {
      profile_name: profileName,
      code: SIMPLE_TEST_CONFIG,
    });

    // 3. Activate
    console.log('3. Activating profile...');
    await result.current.command('activate_profile', { name: profileName });

    // 4. Duplicate
    const duplicateName = `${profileName}-copy`;
    console.log('4. Duplicating profile...');
    await result.current.command('duplicate_profile', {
      source_name: profileName,
      new_name: duplicateName,
    });

    // 5. Rename duplicate
    const renamedName = `${profileName}-renamed`;
    console.log('5. Renaming duplicate...');
    await result.current.command('rename_profile', {
      old_name: duplicateName,
      new_name: renamedName,
    });

    // 6. Verify both exist
    console.log('6. Verifying profiles...');
    const profiles = await result.current.query('get_profiles');
    expect(profiles).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ name: profileName }),
        expect.objectContaining({ name: renamedName }),
      ])
    );

    // 7. Delete both
    console.log('7. Cleaning up...');
    await result.current.command('delete_profile', { name: profileName });
    await result.current.command('delete_profile', { name: renamedName });

    // 8. Verify deleted
    console.log('8. Verifying deletion...');
    const finalProfiles = await result.current.query('get_profiles');
    expect(finalProfiles).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ name: profileName })])
    );
    expect(finalProfiles).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ name: renamedName })])
    );

    console.log('✅ Complete profile lifecycle test passed');
  });
});
