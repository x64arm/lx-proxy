/**
 * Tests for LX-Proxy SDK Client
 */

import { LXProxyClient } from '../src/client';
import {
  AuthenticationError,
  APIError,
  NotFoundError,
  ValidationError,
} from '../src/errors';
import axios from 'axios';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('LXProxyClient', () => {
  let client: LXProxyClient;

  beforeEach(() => {
    client = new LXProxyClient({
      baseURL: 'http://test:8080',
      timeout: 5000,
    });
    jest.clearAllMocks();
  });

  describe('Initialization', () => {
    it('should create client with default config', () => {
      const defaultClient = new LXProxyClient();
      expect(defaultClient).toBeDefined();
    });

    it('should create client with custom config', () => {
      const customClient = new LXProxyClient({
        baseURL: 'http://custom:9000',
        apiKey: 'test-token',
        timeout: 60000,
      });
      expect(customClient).toBeDefined();
    });

    it('should remove trailing slash from baseURL', () => {
      const clientWithSlash = new LXProxyClient({
        baseURL: 'http://test:8080/',
      });
      expect(clientWithSlash).toBeDefined();
    });
  });

  describe('Authentication', () => {
    it('should login successfully', async () => {
      const mockResponse = {
        data: {
          token: 'test-jwt-token',
          user: {
            id: '123',
            username: 'admin',
            role: 'admin',
          },
        },
      };

      (mockedAxios.post as jest.Mock).mockResolvedValue(mockResponse);

      const result = await client.login('admin', 'password123');

      expect(result.token).toBe('test-jwt-token');
      expect(result.user.username).toBe('admin');
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/api/auth/login',
        { username: 'admin', password: 'password123' },
        expect.any(Object)
      );
    });

    it('should handle authentication error', async () => {
      const mockError = {
        response: {
          status: 401,
          data: { error: 'Invalid credentials' },
        },
      };

      (mockedAxios.post as jest.Mock).mockRejectedValue(mockError);

      await expect(client.login('admin', 'wrong-password'))
        .rejects
        .toThrow(AuthenticationError);
    });

    it('should logout and clear token', async () => {
      client.setApiKey('test-token');
      
      (mockedAxios.post as jest.Mock).mockResolvedValue({ data: {} });

      await client.logout();

      expect(client.getApiKey()).toBeUndefined();
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/auth/logout');
    });

    it('should get current user', async () => {
      const mockUser = {
        id: '123',
        username: 'admin',
        role: 'admin',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      };

      (mockedAxios.get as jest.Mock).mockResolvedValue({ data: mockUser });

      const user = await client.getCurrentUser();

      expect(user.username).toBe('admin');
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/auth/me');
    });
  });

  describe('User Management', () => {
    beforeEach(() => {
      client.setApiKey('test-token');
    });

    it('should list users', async () => {
      const mockUsers = [
        {
          id: '123',
          username: 'admin',
          role: 'admin',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
        {
          id: '456',
          username: 'user1',
          role: 'user',
          created_at: '2024-01-02T00:00:00Z',
          updated_at: '2024-01-02T00:00:00Z',
        },
      ];

      (mockedAxios.get as jest.Mock).mockResolvedValue({ data: mockUsers });

      const users = await client.listUsers();

      expect(users).toHaveLength(2);
      expect(users[0].username).toBe('admin');
      expect(users[1].role).toBe('user');
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/users');
    });

    it('should create user', async () => {
      const mockUser = {
        id: '789',
        username: 'newuser',
        role: 'user',
        created_at: '2024-01-03T00:00:00Z',
        updated_at: '2024-01-03T00:00:00Z',
      };

      (mockedAxios.post as jest.Mock).mockResolvedValue({ data: mockUser });

      const user = await client.createUser({
        username: 'newuser',
        password: 'password123',
        role: 'user',
      });

      expect(user.username).toBe('newuser');
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/api/users',
        expect.objectContaining({
          username: 'newuser',
          password: 'password123',
        }),
        expect.any(Object)
      );
    });

    it('should get user by ID', async () => {
      const mockUser = {
        id: '123',
        username: 'admin',
        role: 'admin',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      };

      (mockedAxios.get as jest.Mock).mockResolvedValue({ data: mockUser });

      const user = await client.getUser('123');

      expect(user.id).toBe('123');
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/users/123');
    });

    it('should handle not found error', async () => {
      const mockError = {
        response: {
          status: 404,
          data: { error: 'User not found' },
        },
      };

      (mockedAxios.get as jest.Mock).mockRejectedValue(mockError);

      await expect(client.getUser('non-existent'))
        .rejects
        .toThrow(NotFoundError);
    });

    it('should update user', async () => {
      const mockUser = {
        id: '123',
        username: 'updated',
        role: 'admin',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      };

      (mockedAxios.put as jest.Mock).mockResolvedValue({ data: mockUser });

      const user = await client.updateUser('123', { role: 'admin' });

      expect(user.role).toBe('admin');
      expect(mockedAxios.put).toHaveBeenCalledWith(
        '/api/users/123',
        expect.objectContaining({ role: 'admin' }),
        expect.any(Object)
      );
    });

    it('should delete user', async () => {
      (mockedAxios.delete as jest.Mock).mockResolvedValue({ data: {} });

      await client.deleteUser('123');

      expect(mockedAxios.delete).toHaveBeenCalledWith('/api/users/123');
    });
  });

  describe('Inbound Management', () => {
    beforeEach(() => {
      client.setApiKey('test-token');
    });

    it('should list inbounds', async () => {
      const mockInbounds = [
        {
          id: '123',
          user_id: null,
          tag: 'vmess-1000',
          protocol: 'vmess',
          port: 1000,
          enable: true,
          traffic_used: 0,
          traffic_limit: 10737418240,
          expire_at: null,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ];

      (mockedAxios.get as jest.Mock).mockResolvedValue({ data: mockInbounds });

      const inbounds = await client.listInbounds();

      expect(inbounds).toHaveLength(1);
      expect(inbounds[0].tag).toBe('vmess-1000');
      expect(inbounds[0].protocol).toBe('vmess');
    });

    it('should create inbound', async () => {
      const mockInbound = {
        id: '456',
        user_id: null,
        tag: 'vmess-2000',
        protocol: 'vmess',
        port: 2000,
        enable: true,
        traffic_used: 0,
        traffic_limit: 21474836480,
        expire_at: null,
        created_at: '2024-01-02T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      };

      (mockedAxios.post as jest.Mock).mockResolvedValue({ data: mockInbound });

      const inbound = await client.createInbound({
        tag: 'vmess-2000',
        protocol: 'vmess',
        port: 2000,
        traffic_limit: 21474836480,
      });

      expect(inbound.tag).toBe('vmess-2000');
      expect(inbound.port).toBe(2000);
    });

    it('should update inbound', async () => {
      const mockInbound = {
        id: '123',
        tag: 'updated-tag',
        protocol: 'vmess',
        port: 1000,
        enable: false,
        traffic_used: 0,
        traffic_limit: null,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-02T00:00:00Z',
      };

      (mockedAxios.put as jest.Mock).mockResolvedValue({ data: mockInbound });

      const inbound = await client.updateInbound('123', {
        tag: 'updated-tag',
        enable: false,
      });

      expect(inbound.tag).toBe('updated-tag');
      expect(inbound.enable).toBe(false);
    });
  });

  describe('System & Statistics', () => {
    beforeEach(() => {
      client.setApiKey('test-token');
    });

    it('should get stats', async () => {
      const mockStats = {
        total_users: 10,
        total_inbounds: 25,
        enabled_inbounds: 20,
        total_traffic_used: 1073741824,
        total_traffic_limit: 107374182400,
      };

      (mockedAxios.get as jest.Mock).mockResolvedValue({ data: mockStats });

      const stats = await client.getStats();

      expect(stats.total_users).toBe(10);
      expect(stats.total_inbounds).toBe(25);
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/stats');
    });

    it('should get system status', async () => {
      const mockStatus = {
        cpu_usage: 25.5,
        memory_total: 17179869184,
        memory_used: 8589934592,
        memory_free: 8589934592,
        uptime: 86400,
        xray_running: true,
        connections: 150,
      };

      (mockedAxios.get as jest.Mock).mockResolvedValue({ data: mockStatus });

      const status = await client.getSystemStatus();

      expect(status.cpu_usage).toBe(25.5);
      expect(status.xray_running).toBe(true);
      expect(status.connections).toBe(150);
    });
  });

  describe('Error Handling', () => {
    it('should handle validation error', async () => {
      const mockError = {
        response: {
          status: 400,
          data: {
            message: 'Validation failed',
            errors: ['Username is required'],
          },
        },
      };

      (mockedAxios.get as jest.Mock).mockRejectedValue(mockError);

      await expect(client.listUsers())
        .rejects
        .toThrow(ValidationError);
    });

    it('should handle generic API error', async () => {
      const mockError = {
        response: {
          status: 500,
          data: { error: 'Internal server error' },
        },
      };

      (mockedAxios.get as jest.Mock).mockRejectedValue(mockError);

      await expect(client.listUsers())
        .rejects
        .toThrow(APIError);
    });
  });

  describe('API Key Management', () => {
    it('should set API key', () => {
      client.setApiKey('new-token');
      expect(client.getApiKey()).toBe('new-token');
    });

    it('should clear API key', () => {
      client.setApiKey('test-token');
      client.clearApiKey();
      expect(client.getApiKey()).toBeUndefined();
    });
  });
});
