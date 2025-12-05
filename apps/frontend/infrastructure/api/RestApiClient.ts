import { API_TIMEOUT, DEFAULT_HEADERS } from '../config/apiConfig';

/**
 * API Error
 *
 * Custom error class for API-related failures.
 * Includes HTTP status code and user-friendly message.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
    public readonly originalError?: Error
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * RestApiClient
 *
 * Base HTTP client for communicating with the backend REST API.
 * Handles common concerns: error handling, timeouts, JSON serialization.
 * Infrastructure layer component - no domain logic.
 */
export class RestApiClient {
  /**
   * Execute a GET request
   *
   * @param url - Full URL to request
   * @param params - Optional URL query parameters
   * @returns Promise resolving to parsed JSON response
   * @throws ApiError if request fails
   */
  async get<T>(url: string, params?: Record<string, string | number>): Promise<T> {
    const urlWithParams = this.buildUrlWithParams(url, params);
    return await this.request<T>(urlWithParams, { method: 'GET' });
  }

  /**
   * Execute a POST request
   *
   * @param url - Full URL to request
   * @param body - Request body (will be JSON-serialized)
   * @returns Promise resolving to parsed JSON response
   * @throws ApiError if request fails
   */
  async post<T>(url: string, body?: unknown): Promise<T> {
    return await this.request<T>(url, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    });
  }

  /**
   * Execute a DELETE request
   *
   * @param url - Full URL to request
   * @returns Promise resolving to parsed JSON response
   * @throws ApiError if request fails
   */
  async delete<T>(url: string): Promise<T> {
    return await this.request<T>(url, { method: 'DELETE' });
  }

  /**
   * Core request method with error handling and timeout
   *
   * @param url - Full URL to request
   * @param options - Fetch API options
   * @returns Promise resolving to parsed JSON response
   * @throws ApiError if request fails
   */
  private async request<T>(url: string, options: RequestInit): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);

    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          ...DEFAULT_HEADERS,
          ...options.headers,
        },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      // Handle non-2xx responses
      if (!response.ok) {
        const errorMessage = await this.extractErrorMessage(response);
        throw new ApiError(
          errorMessage || `HTTP ${response.status}: ${response.statusText}`,
          response.status
        );
      }

      // Parse JSON response
      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);

      // Handle abort (timeout)
      if (error instanceof Error && error.name === 'AbortError') {
        throw new ApiError(
          'Request timeout - backend did not respond in time',
          408,
          error
        );
      }

      // Handle network errors
      if (error instanceof TypeError) {
        throw new ApiError(
          'Network error - unable to connect to backend. Is it running?',
          undefined,
          error
        );
      }

      // Re-throw ApiError as-is
      if (error instanceof ApiError) {
        throw error;
      }

      // Wrap unknown errors
      throw new ApiError(
        'Unexpected error during API request',
        undefined,
        error as Error
      );
    }
  }

  /**
   * Build URL with query parameters
   *
   * @param url - Base URL
   * @param params - Query parameters
   * @returns URL string with encoded parameters
   */
  private buildUrlWithParams(
    url: string,
    params?: Record<string, string | number>
  ): string {
    if (!params || Object.keys(params).length === 0) {
      return url;
    }

    const searchParams = new URLSearchParams();
    for (const [key, value] of Object.entries(params)) {
      searchParams.append(key, String(value));
    }

    return `${url}?${searchParams.toString()}`;
  }

  /**
   * Extract user-friendly error message from response
   *
   * @param response - Failed HTTP response
   * @returns Error message string
   */
  private async extractErrorMessage(response: Response): Promise<string | null> {
    try {
      const contentType = response.headers.get('content-type');
      if (contentType?.includes('application/json')) {
        const errorBody = await response.json();
        return errorBody.message || errorBody.error || null;
      }
    } catch {
      // If JSON parsing fails, return null (fallback to status text)
    }
    return null;
  }
}
