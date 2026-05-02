/**
 * Profile RPC methods for RpcClient
 */

import type { RpcApi } from '../../types/api';
import type { Profile, ProfileConfig } from '../types';
import { validateApiResponse, ProfileConfigRpcSchema } from '../schemas';
import type { z } from 'zod';

/**
 * Profile RPC methods
 */
export class ProfileRpcMethods {
  constructor(private api: RpcApi) {}

  /**
   * Get all profiles.
   */
  async getProfiles(): Promise<Profile[]> {
    return this.api.query<Profile[]>('get_profiles');
  }

  /**
   * Create a new profile.
   */
  async createProfile(name: string, template?: string): Promise<void> {
    return this.api.command<void>('create_profile', { name, template });
  }

  /**
   * Activate a profile.
   */
  async activateProfile(name: string): Promise<void> {
    return this.api.command<void>('activate_profile', { name });
  }

  /**
   * Delete a profile.
   */
  async deleteProfile(name: string): Promise<void> {
    return this.api.command<void>('delete_profile', { name });
  }

  /**
   * Duplicate a profile.
   */
  async duplicateProfile(sourceName: string, newName: string): Promise<void> {
    return this.api.command<void>('duplicate_profile', {
      source_name: sourceName,
      new_name: newName,
    });
  }

  /**
   * Rename a profile.
   */
  async renameProfile(oldName: string, newName: string): Promise<void> {
    return this.api.command<void>('rename_profile', {
      old_name: oldName,
      new_name: newName,
    });
  }

  /**
   * Get profile configuration source code.
   */
  async getProfileConfig(name: string): Promise<ProfileConfig> {
    const response = await this.api.query<z.infer<typeof ProfileConfigRpcSchema>>(
      'get_profile_config',
      { name }
    );
    const validated = validateApiResponse(
      ProfileConfigRpcSchema,
      response,
      'get_profile_config'
    );
    return {
      name: validated.name,
      source: validated.source,
    };
  }

  /**
   * Set profile configuration source code.
   */
  async setProfileConfig(name: string, source: string): Promise<void> {
    await this.api.command<void>('set_profile_config', { name, source });
  }

  /**
   * Get the currently active profile name.
   */
  async getActiveProfile(): Promise<string | null> {
    return this.api.query<string | null>('get_active_profile');
  }
}