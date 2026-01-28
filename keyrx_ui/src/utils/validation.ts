/**
 * Input validation utilities
 * Provides client-side validation for forms and user inputs
 */

import { z } from 'zod';

/**
 * Profile name validation schema
 */
export const profileNameSchema = z
  .string()
  .min(1, 'Profile name is required')
  .max(50, 'Profile name must be 50 characters or less')
  .regex(/^[a-zA-Z0-9_-]+$/, 'Profile name can only contain letters, numbers, hyphens, and underscores');

/**
 * Device name validation schema
 */
export const deviceNameSchema = z
  .string()
  .min(1, 'Device name is required')
  .max(64, 'Device name must be 64 characters or less');

/**
 * Key code validation schema
 */
export const keyCodeSchema = z
  .string()
  .min(1, 'Key code is required')
  .regex(/^[A-Z0-9_]+$/, 'Key code must be uppercase alphanumeric with underscores');

/**
 * Validate profile name
 */
export function validateProfileName(name: string): { valid: boolean; error?: string } {
  const result = profileNameSchema.safeParse(name);
  if (!result.success) {
    return { valid: false, error: result.error.errors[0]?.message };
  }
  return { valid: true };
}

/**
 * Validate device name
 */
export function validateDeviceName(name: string): { valid: boolean; error?: string } {
  const result = deviceNameSchema.safeParse(name);
  if (!result.success) {
    return { valid: false, error: result.error.errors[0]?.message };
  }
  return { valid: true };
}

/**
 * Validate email address
 */
export const emailSchema = z.string().email('Invalid email address');

/**
 * Validate URL
 */
export const urlSchema = z.string().url('Invalid URL');

/**
 * Sanitize user input to prevent XSS
 */
export function sanitizeInput(input: string): string {
  return input
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;');
}

/**
 * Validate and sanitize form data
 */
export function validateFormData<T extends Record<string, unknown>>(
  data: T,
  schema: z.ZodSchema<T>
): { success: true; data: T } | { success: false; errors: Record<string, string> } {
  const result = schema.safeParse(data);

  if (!result.success) {
    const errors: Record<string, string> = {};
    result.error.errors.forEach((err) => {
      const path = err.path.join('.');
      errors[path] = err.message;
    });
    return { success: false, errors };
  }

  return { success: true, data: result.data };
}
