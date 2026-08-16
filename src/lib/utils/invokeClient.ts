/**
 * InvokeClient - Axios-like client for Tauri commands and HTTP requests
 *
 * Provides a unified, configurable interface for calling backend functions
 * that works seamlessly in Tauri desktop app and browser environments.
 */

import { isTauriEnvironment, tauriInvoke } from './tauri';
import DeviceAuthService from '$lib/services/deviceAuthService';

/**
 * Get the base URL for HTTP API calls
 */
function getApiBaseUrl(): string {
  if (typeof window === 'undefined') return '';
  return `${window.location.protocol}//${window.location.host}`;
}

/**
 * Check if endpoint is a URL (vs Tauri command)
 */
function isUrl(endpoint: string): boolean {
  return (
    endpoint.startsWith('http://') ||
    endpoint.startsWith('https://') ||
    endpoint.startsWith('/api/')
  );
}

/**
 * Localhost strategy options
 */
export type LocalhostStrategy = 'http' | 'empty' | 'error';

/**
 * Request configuration
 */
export interface RequestConfig {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  data?: any;
  params?: Record<string, any>;
  headers?: Record<string, string>;
  timeout?: number;
  requireAuth?: boolean;
  localhostStrategy?: LocalhostStrategy;
  transformRequest?: (data: any) => any;
  transformResponse?: (data: any) => any;
}

/**
 * Client configuration
 */
export interface InvokeClientConfig {
  baseURL?: string;
  defaultHeaders?: Record<string, string>;
  timeout?: number;
  localhostStrategy?: LocalhostStrategy;
  requireAuth?: boolean;
  transformRequest?: (data: any) => any;
  transformResponse?: (data: any) => any;
}

/**
 * Interceptor types
 */
export type RequestInterceptor = (config: RequestConfig) => RequestConfig | Promise<RequestConfig>;
export type ResponseInterceptor<T = any> = (response: T) => T | Promise<T>;
export type ErrorInterceptor = (error: Error) => Error | Promise<Error>;

/**
 * Type helper to check if T is an array type
 */
type IsArray<T> = T extends (infer U)[] ? true : false;

/**
 * Type helper to get the element type of an array
 */
type ArrayElement<T> = T extends (infer U)[] ? U : never;

/**
 * Type helper to check if T is nullable (includes null or undefined)
 */
type IsNullable<T> = null extends T ? true : undefined extends T ? true : false;

/**
 * Type helper to get the non-nullable version of T
 */
type NonNullable<T> = T extends null | undefined ? never : T;

/**
 * Type helper to get appropriate empty value type
 * - Arrays always return empty array (never null)
 * - Non-nullable single items return a default value based on type
 * - Nullable types can return null
 */
type EmptyValue<T> =
  // If T is an array, always return empty array (never null)
  T extends (infer U)[]
    ? T
    : // If T is a number (non-nullable), return 0
      T extends number
      ? number
      : // If T is a string (non-nullable), return empty string
        T extends string
        ? string
        : // If T is a boolean (non-nullable), return false
          T extends boolean
          ? boolean
          : // If T is explicitly nullable (Language | null), allow null
            null extends T
            ? T
            : undefined extends T
              ? T
              : // For non-nullable single items, we need to return a value
                // But we can't create a valid instance, so we'll throw an error
                // or return null and let the caller handle it
                T | null;

/**
 * InvokeClient - Axios-like client for unified backend access
 */
export class InvokeClient {
  private config: Required<InvokeClientConfig>;
  private requestInterceptors: RequestInterceptor[] = [];
  private responseInterceptors: ResponseInterceptor[] = [];
  private errorInterceptors: ErrorInterceptor[] = [];

  /**
   * Static method to check if accessing from localhost
   * Can be reused anywhere in the codebase
   */
  static isLocalhost(): boolean {
    if (typeof window === 'undefined') return false;
    const hostname = window.location.hostname;
    return (
      hostname === 'localhost' ||
      hostname === '127.0.0.1' ||
      hostname === '::1' ||
      hostname === '[::1]'
    );
  }

  constructor(config: InvokeClientConfig = {}) {
    this.config = {
      baseURL: config.baseURL || '',
      defaultHeaders: config.defaultHeaders || {},
      timeout: config.timeout || 30000, // 30 seconds
      localhostStrategy: config.localhostStrategy || 'empty',
      requireAuth: config.requireAuth !== false, // Default true
      transformRequest: config.transformRequest || ((data) => data),
      transformResponse: config.transformResponse || ((data) => data),
    };
  }

  /**
   * Add request interceptor
   */
  interceptors = {
    request: {
      use: (handler: RequestInterceptor) => {
        this.requestInterceptors.push(handler);
      },
    },
    response: {
      use: (handler: ResponseInterceptor) => {
        this.responseInterceptors.push(handler);
      },
    },
    error: {
      use: (handler: ErrorInterceptor) => {
        this.errorInterceptors.push(handler);
      },
    },
  };

  /**
   * Main request method - handles both Tauri commands and HTTP requests
   */
  async request<T = any>(endpoint: string, config: RequestConfig = {}): Promise<T> {
    try {
      // Merge config with defaults
      const mergedConfig: RequestConfig = {
        method: config.method || 'POST',
        data: config.data,
        params: config.params,
        headers: { ...this.config.defaultHeaders, ...config.headers },
        timeout: config.timeout ?? this.config.timeout,
        requireAuth: config.requireAuth ?? this.config.requireAuth,
        localhostStrategy: config.localhostStrategy ?? this.config.localhostStrategy,
        transformRequest: config.transformRequest ?? this.config.transformRequest,
        transformResponse: config.transformResponse ?? this.config.transformResponse,
      };

      // Apply request interceptors
      let finalConfig = mergedConfig;
      for (const interceptor of this.requestInterceptors) {
        finalConfig = await interceptor(finalConfig);
      }

      // Transform request data
      if (finalConfig.data && finalConfig.transformRequest) {
        finalConfig.data = finalConfig.transformRequest(finalConfig.data);
      }

      // Route to appropriate handler
      let response: any;
      if (isUrl(endpoint)) {
        response = await this.httpRequest<T>(endpoint, finalConfig);
      } else {
        response = await this.tauriRequest<T>(endpoint, finalConfig);
      }

      // Transform response data
      if (finalConfig.transformResponse) {
        response = finalConfig.transformResponse(response);
      }

      // Apply response interceptors
      for (const interceptor of this.responseInterceptors) {
        response = await interceptor(response);
      }

      return response;
    } catch (error) {
      // Apply error interceptors
      let finalError = error instanceof Error ? error : new Error(String(error));
      for (const interceptor of this.errorInterceptors) {
        finalError = await interceptor(finalError);
      }
      throw finalError;
    }
  }

  /**
   * HTTP request handler
   */
  private async httpRequest<T>(endpoint: string, config: RequestConfig): Promise<T> {
    // Build full URL
    let fullUrl = endpoint;
    if (this.config.baseURL && !endpoint.startsWith('http')) {
      fullUrl = `${this.config.baseURL}${endpoint}`;
    } else if (!endpoint.startsWith('http://') && !endpoint.startsWith('https://')) {
      fullUrl = `${getApiBaseUrl()}${endpoint}`;
    }

    // Prepare headers
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...config.headers,
    };

    // Add authentication for remote access (not localhost)
    if (config.requireAuth && !InvokeClient.isLocalhost()) {
      const accessToken = DeviceAuthService.getAccessToken();
      if (accessToken) {
        headers['Authorization'] = `Bearer ${accessToken}`;
      } else {
        throw new Error('Authentication required');
      }
    }

    // Prepare request options
    const requestOptions: RequestInit = {
      method: config.method,
      headers,
    };

    // Add body for non-GET requests
    if (config.method !== 'GET' && config.data !== undefined) {
      requestOptions.body = JSON.stringify(config.data);
    } else if (config.method === 'GET' && (config.params || config.data)) {
      // For GET requests, add params/data as query params
      const params = new URLSearchParams();
      const queryData = config.params || config.data || {};
      Object.entries(queryData).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          params.append(key, String(value));
        }
      });
      if (params.toString()) {
        fullUrl += `?${params.toString()}`;
      }
    }

    // Create abort controller for timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), config.timeout);

    try {
      // Make the request
      const response = await fetch(fullUrl, {
        ...requestOptions,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
        try {
          const error = await response.json();
          errorMessage = error.error || error.message || errorMessage;
          // For 501 errors, provide a more helpful message
          if (response.status === 501) {
            errorMessage = `This feature requires the desktop app. Tauri commands cannot be executed from browser. Please use the Tauri desktop application for full functionality.`;
          }
        } catch {
          // If JSON parsing fails, use default error message
        }
        throw new Error(errorMessage);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error(`Request timeout after ${config.timeout}ms`, {
          cause: error,
        });
      }
      throw error;
    }
  }

  /**
   * Tauri command request handler
   */
  private async tauriRequest<T>(command: string, config: RequestConfig): Promise<T> {
    if (isTauriEnvironment()) {
      // Tauri app - direct invoke (fastest)
      const args = config.data || config.params || {};
      return await tauriInvoke<T>(command, args);
    } else if (InvokeClient.isLocalhost()) {
      // Localhost browser - handle based on strategy
      return this.handleLocalhostStrategy<T>(command, config);
    } else {
      // Remote browser. There is no transport for this yet.
      //
      // This used to POST `/api/tauri/{command}`, a SvelteKit route that
      // returned 501 and was never in the build at all — `adapter-static` in
      // SPA mode drops `src/routes/api/` — so the call could only ever fail
      // with an HTML parse error. The route is deleted; this says so plainly.
      //
      // The real transport is the daemon's `POST /rpc/{command}`, but it binds
      // loopback and refuses any request carrying an `Origin` header, so a
      // browser cannot reach it by design. Pairing, LAN binding and per-route
      // capabilities are phase 4 of docs/development/REMOTE_ARCHITECTURE.md.
      throw new Error(
        `Command '${command}' is not reachable from a remote browser yet: the daemon ` +
          `binds loopback and refuses browser origins. See REMOTE_ARCHITECTURE.md phase 4.`
      );
    }
  }

  /**
   * Handle localhost strategy for browser access
   */
  private async handleLocalhostStrategy<T>(command: string, config: RequestConfig): Promise<T> {
    const strategy = config.localhostStrategy || this.config.localhostStrategy;

    switch (strategy) {
      case 'http':
        // Kept as a strategy name so callers do not have to change, but there is
        // nothing to call: the SvelteKit proxy is deleted and the daemon refuses
        // browser origins (see `tauriRequest`). Behaves as 'empty' until phase 4
        // gives the browser a real transport.
        return this.getEmptyValue<T>(command);

      case 'empty':
        // Return empty/default values
        return this.getEmptyValue<T>(command);

      case 'error':
      default:
        // Throw error indicating Tauri required
        throw new Error(
          `Tauri command '${command}' is not available in browser. ` +
            `Please use the Tauri desktop app for full functionality.`
        );
    }
  }

  /**
   * Get empty value based on expected return type
   * Uses runtime heuristics (command name patterns) to determine appropriate empty values
   *
   * Type safety:
   * - Arrays (T extends U[]): Always return [] (never null)
   * - Numbers: Return 0
   * - Strings: Return ''
   * - Booleans: Return false
   * - Nullable types (T | null): Can return null
   * - Non-nullable single items: Return null (caller should handle gracefully)
   *
   * Note: TypeScript types are erased at runtime, so we use command name patterns
   * as heuristics. TypeScript will enforce type safety at compile time.
   */
  private getEmptyValue<T>(command?: string): T {
    // Runtime check: Use command name patterns as heuristic
    // This works even though TypeScript types are erased at runtime
    if (command) {
      // Array-returning commands: always return empty array (never null)
      const arrayPatterns = [
        /^get_all_/,
        /^get_.*_list$/,
        /^list_/,
        /^search_/,
        /^get_tasks$/,
        /^get_projects$/,
        /^get_subtasks$/,
        /^get_main_tasks$/,
        /^get_overdue_tasks$/,
        /^get_due_today_tasks$/,
        /^get_unestimated_tasks$/,
        /^get_all_ides$/,
        /^get_all_frameworks$/,
        /^get_all_package_managers$/,
        /^get_all_languages$/,
        /^get_suggested_/,
        /^detect_installed_/,
        /^get_all_framework_ide_mappings$/,
        // Kubernetes commands that return arrays
        /^k8s_load_clusters$/,
        /^k8s_list_/,
      ];

      if (arrayPatterns.some((pattern) => pattern.test(command))) {
        // Return empty array for array-returning commands
        // TypeScript ensures T extends U[] at compile time, so [] is assignable to T
        return [] as T;
      }

      // Number-returning commands: return 0
      const numberPatterns = [/^get_.*_count$/, /^count_/];

      if (numberPatterns.some((pattern) => pattern.test(command))) {
        // Return 0 for number-returning commands
        return 0 as T;
      }

      // String-returning commands: return empty string
      const stringPatterns = [/^get_.*_path$/, /^get_.*_url$/];

      if (stringPatterns.some((pattern) => pattern.test(command))) {
        // Return empty string for string-returning commands
        return '' as T;
      }
    }

    // Default: return null
    // - For nullable types (T | null): This is correct
    // - For non-nullable single items: TypeScript will catch type mismatches at compile time
    //   In practice, most commands return arrays or nullable types, so this is safe
    // - For arrays: This should never happen due to pattern matching above
    return null as T;
  }

  /**
   * HTTP method shortcuts
   */
  async get<T = any>(
    endpoint: string,
    config?: Omit<RequestConfig, 'method' | 'data'>
  ): Promise<T> {
    return this.request<T>(endpoint, { ...config, method: 'GET' });
  }

  async post<T = any>(
    endpoint: string,
    data?: any,
    config?: Omit<RequestConfig, 'method' | 'data'>
  ): Promise<T> {
    return this.request<T>(endpoint, { ...config, method: 'POST', data });
  }

  async put<T = any>(
    endpoint: string,
    data?: any,
    config?: Omit<RequestConfig, 'method' | 'data'>
  ): Promise<T> {
    return this.request<T>(endpoint, { ...config, method: 'PUT', data });
  }

  async delete<T = any>(
    endpoint: string,
    config?: Omit<RequestConfig, 'method' | 'data'>
  ): Promise<T> {
    return this.request<T>(endpoint, { ...config, method: 'DELETE' });
  }

  async patch<T = any>(
    endpoint: string,
    data?: any,
    config?: Omit<RequestConfig, 'method' | 'data'>
  ): Promise<T> {
    return this.request<T>(endpoint, { ...config, method: 'PATCH', data });
  }

  /**
   * Live/streaming request handler for two-way communication
   * Supports:
   * - Tauri: Uses Tauri events for streaming
   * - Browser: Uses WebSocket or SSE for streaming
   *
   * @param endpoint Command name or WebSocket URL
   * @param config Request configuration
   * @param callbacks Callbacks for handling streaming data
   * @returns Promise that resolves when stream is closed
   */
  async live<T = any>(
    endpoint: string,
    config: RequestConfig & {
      onData?: (data: T) => void;
      onError?: (error: Error) => void;
      onClose?: () => void;
      streamId?: string; // Unique ID for this stream
    } = {}
  ): Promise<() => void> {
    const { onData, onError, onClose, streamId, ...requestConfig } = config;
    const uniqueStreamId =
      streamId || `stream-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    if (isTauriEnvironment()) {
      // Tauri: Use Tauri events for streaming
      return this.tauriLive<T>(endpoint, {
        ...requestConfig,
        streamId: uniqueStreamId,
        onData,
        onError,
        onClose,
      });
    }

    // There is no browser transport for streaming. This used to route to
    // `/api/stream/{cmd}` (SSE) and `/ws/{cmd}` (WebSocket) — neither endpoint
    // has ever existed: both would have had to be SvelteKit server routes, and
    // `adapter-static` never built those. The daemon's `GET /events` WebSocket
    // is the replacement when a non-webview client needs a live stream, but it
    // refuses browser origins by design. See REMOTE_ARCHITECTURE.md phase 3.
    throw new Error(`Streaming '${endpoint}' is not available in a browser. Use the desktop app.`);
  }

  /**
   * Tauri live streaming using events
   */
  private async tauriLive<T>(
    command: string,
    config: RequestConfig & {
      streamId: string;
      onData?: (data: T) => void;
      onError?: (error: Error) => void;
      onClose?: () => void;
    }
  ): Promise<() => void> {
    const { streamId, onData, onError, onClose, ...requestConfig } = config;

    try {
      // Import Tauri event listener
      const { listen } = await import('@tauri-apps/api/event');

      // Set up event listeners for streaming data
      const eventName = `${command}:stream:${streamId}`;
      const errorEventName = `${command}:stream:${streamId}:error`;
      const closeEventName = `${command}:stream:${streamId}:close`;

      const unsubscribers: (() => void)[] = [];

      // Listen for data events
      if (onData) {
        const unsubscribe = await listen<T>(eventName, (event) => {
          onData(event.payload);
        });
        unsubscribers.push(unsubscribe);
      }

      // Listen for error events
      if (onError) {
        const unsubscribe = await listen<{ message: string }>(errorEventName, (event) => {
          onError(new Error(event.payload.message));
        });
        unsubscribers.push(unsubscribe);
      }

      // Listen for close events
      if (onClose) {
        const unsubscribe = await listen(closeEventName, () => {
          onClose();
        });
        unsubscribers.push(unsubscribe);
      }

      // Start the streaming command
      const args = {
        ...(requestConfig.data || requestConfig.params || {}),
        streamId,
      };

      // Invoke the command (it will start streaming via events)
      await tauriInvoke(command, args).catch((error) => {
        if (onError) {
          onError(error instanceof Error ? error : new Error(String(error)));
        }
      });

      // Return cleanup function
      return () => {
        unsubscribers.forEach((unsub) => unsub());
        // Optionally send stop command
        tauriInvoke(`${command}:stop`, { streamId }).catch(() => {
          // Ignore errors when stopping
        });
      };
    } catch (error) {
      if (onError) {
        onError(error instanceof Error ? error : new Error(String(error)));
      }
      return () => {}; // Return no-op cleanup
    }
  }
}

/**
 * Default InvokeClient instance
 */
export const invokeClient = new InvokeClient();

/**
 * Create a new InvokeClient instance with custom configuration
 */
export function createInvokeClient(config: InvokeClientConfig): InvokeClient {
  return new InvokeClient(config);
}
