/**
 * API Configuration
 *
 * Centralized configuration for backend API communication.
 * Reads from environment variables with fallback defaults.
 */

/** Backend API base URL (from .env.local) */
export const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8081';

/** API endpoints */
export const API_ENDPOINTS = {
  search: '/api/search',
  collections: '/api/collections',
  index: '/api/index',
  indexStatus: (operationId: string) => `/api/index/${operationId}`,
} as const;

/** API request timeout (milliseconds) */
export const API_TIMEOUT = 30000; // 30 seconds

/** Default request headers */
export const DEFAULT_HEADERS = {
  'Content-Type': 'application/json',
  'Accept': 'application/json',
} as const;

/**
 * Build full API URL from endpoint path
 */
export function buildApiUrl(endpoint: string): string {
  return `${API_BASE_URL}${endpoint}`;
}
