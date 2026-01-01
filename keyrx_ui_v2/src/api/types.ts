/**
 * API data types for RPC client.
 *
 * These types represent the structure of data returned from RPC methods.
 * They map to the Rust types defined in keyrx_daemon.
 */

// Profile Types
export interface Profile {
  name: string;
  isActive: boolean;
  createdAt: string;
  modifiedAt: string;
  configHash?: string;
}

// Device Types
export interface Device {
  id: string;
  name: string;
  serial: string;
  vendorId: string;
  productId: string;
  scope: "global" | "profile";
  isConnected: boolean;
}

// Config Types
export interface Config {
  code: string;
  hash: string;
}

export interface Layer {
  name: string;
  isDefault: boolean;
}

// Simulation Types
export interface SimulationInput {
  timestamp: number;
  keyCode: string;
  eventType: "press" | "release";
}

export interface SimulationResult {
  input: SimulationInput;
  output: {
    keyCode: string;
    eventType: "press" | "release";
  };
  latency: number;
  layer: string;
}

// Pagination Types
export interface PaginatedEvents {
  events: Array<{
    timestamp: number;
    keyCode: string;
    eventType: "press" | "release";
    input: string;
    output: string;
    latency: number;
  }>;
  total: number;
  limit: number;
  offset: number;
}
