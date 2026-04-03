/**
 * LX-Proxy API Client
 * 
 * TypeScript/JavaScript client for LX-Proxy API
 * 
 * @example
 * ```typescript
 * import { LXProxyClient } from '@lx-proxy/sdk';
 * 
 * const client = new LXProxyClient('http://localhost:8080');
 * 
 * // Login
 * await client.login('admin', 'admin123');
 * 
 * // List users
 * const users = await client.listUsers();
 * 
 * // Create inbound
 * const inbound = await client.createInbound({
 *   tag: 'vmess-1000',
 *   protocol: 'vmess',
 *   port: 1000,
 * });
 * ```
 */

import axios, { AxiosInstance, AxiosError } from 'axios';
import {
  User,
  CreateUserRequest,
  UpdateUserRequest,
  InboundConfig,
  CreateInboundRequest,
  UpdateInboundRequest,
  Stats,
  SystemStatus,
  LoginResponse,
} from './types';
import {
  LXProxyError,
  AuthenticationError,
  APIError,
  NotFoundError,
  ValidationError,
} from './errors';

export interface LXProxyClientConfig {
  baseURL?: string;
  apiKey?: string;
  timeout?: number;
}

export class LXProxyClient {
  private client: AxiosInstance;
  private apiKey?: string;

  /**
   * Create LX-Proxy client
   * 
   * @param config - Client configuration
   */
  constructor(config: LXProxyClientConfig = {}) {
    const {
      baseURL = 'http://localhost:8080',
      apiKey,
      timeout = 30000,
    } = config;

    this.apiKey = apiKey;

    this.client = axios.create({
      baseURL: baseURL.replace(/\/$/, ''),
      timeout,
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
    });

    // Request interceptor to add auth header
    this.client.interceptors.request.use((config) => {
      if (this.apiKey) {
        config.headers.Authorization = `Bearer ${this.apiKey}`;
      }
      return config;
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        throw this.handleError(error);
      }
    );
  }

  /**
   * Handle HTTP errors
   */
  private handleError(error: AxiosError): LXProxyError {
    const status = error.response?.status;
    const data = error.response?.data as Record<string, unknown> | undefined;

    if (status === 401) {
      return new AuthenticationError('Authentication failed. Please login again.');
    }

    if (status === 404) {
      return new NotFoundError('Resource not found');
    }

    if (status === 400) {
      const errors = Array.isArray(data?.errors) ? data.errors : [];
      return new ValidationError(
        (data?.message as string) || 'Validation error',
        errors as string[]
      );
    }

    const message = (data?.message as string) || data?.error as string || `API error: ${status}`;
    return new APIError(message, status, data);
  }

  /**
   * ============== Authentication ==============
   */

  /**
   * Login to LX-Proxy API
   * 
   * @param username - Username
   * @param password - Password
   * @returns Login response with token and user info
   */
  async login(username: string, password: string): Promise<LoginResponse> {
    const response = await this.client.post<LoginResponse>('/api/auth/login', {
      username,
      password,
    });

    // Store token for subsequent requests
    if (response.data.token) {
      this.apiKey = response.data.token;
    }

    return response.data;
  }

  /**
   * Logout from LX-Proxy API
   */
  async logout(): Promise<void> {
    await this.client.post('/api/auth/logout');
    this.apiKey = undefined;
  }

  /**
   * Get current user info
   */
  async getCurrentUser(): Promise<User> {
    const response = await this.client.get<User>('/api/auth/me');
    return response.data;
  }

  /**
   * ============== User Management ==============
   */

  /**
   * List all users
   */
  async listUsers(): Promise<User[]> {
    const response = await this.client.get<User[]>('/api/users');
    return response.data;
  }

  /**
   * Create a new user
   * 
   * @param data - User creation data
   */
  async createUser(data: CreateUserRequest): Promise<User> {
    const response = await this.client.post<User>('/api/users', data);
    return response.data;
  }

  /**
   * Get user by ID
   * 
   * @param userId - User ID
   */
  async getUser(userId: string): Promise<User> {
    const response = await this.client.get<User>(`/api/users/${userId}`);
    return response.data;
  }

  /**
   * Update user
   * 
   * @param userId - User ID
   * @param data - Update data
   */
  async updateUser(userId: string, data: UpdateUserRequest): Promise<User> {
    const response = await this.client.put<User>(`/api/users/${userId}`, data);
    return response.data;
  }

  /**
   * Delete user
   * 
   * @param userId - User ID
   */
  async deleteUser(userId: string): Promise<void> {
    await this.client.delete(`/api/users/${userId}`);
  }

  /**
   * ============== Inbound Management ==============
   */

  /**
   * List all inbound configs
   */
  async listInbounds(): Promise<InboundConfig[]> {
    const response = await this.client.get<InboundConfig[]>('/api/inbounds');
    return response.data;
  }

  /**
   * Create new inbound
   * 
   * @param data - Inbound creation data
   */
  async createInbound(data: CreateInboundRequest): Promise<InboundConfig> {
    const response = await this.client.post<InboundConfig>('/api/inbounds', data);
    return response.data;
  }

  /**
   * Get inbound by ID
   * 
   * @param inboundId - Inbound ID
   */
  async getInbound(inboundId: string): Promise<InboundConfig> {
    const response = await this.client.get<InboundConfig>(`/api/inbounds/${inboundId}`);
    return response.data;
  }

  /**
   * Update inbound
   * 
   * @param inboundId - Inbound ID
   * @param data - Update data
   */
  async updateInbound(inboundId: string, data: UpdateInboundRequest): Promise<InboundConfig> {
    const response = await this.client.put<InboundConfig>(`/api/inbounds/${inboundId}`, data);
    return response.data;
  }

  /**
   * Delete inbound
   * 
   * @param inboundId - Inbound ID
   */
  async deleteInbound(inboundId: string): Promise<void> {
    await this.client.delete(`/api/inbounds/${inboundId}`);
  }

  /**
   * ============== System & Statistics ==============
   */

  /**
   * Get system statistics
   */
  async getStats(): Promise<Stats> {
    const response = await this.client.get<Stats>('/api/stats');
    return response.data;
  }

  /**
   * Get system status
   */
  async getSystemStatus(): Promise<SystemStatus> {
    const response = await this.client.get<SystemStatus>('/api/system/status');
    return response.data;
  }

  /**
   * ============== Utility Methods ==============
   */

  /**
   * Set API key (JWT token)
   * 
   * @param token - JWT token
   */
  setApiKey(token: string): void {
    this.apiKey = token;
  }

  /**
   * Get API key
   */
  getApiKey(): string | undefined {
    return this.apiKey;
  }

  /**
   * Clear API key
   */
  clearApiKey(): void {
    this.apiKey = undefined;
  }
}
