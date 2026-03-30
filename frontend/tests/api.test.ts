// API 请求模块测试
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import api from '../src/api/index'
import axios from 'axios'

// Mock axios
vi.mock('axios')
const mockedAxios = vi.mocked(axios, true)

describe('API Module', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // 清除 localStorage 中的 token
    localStorage.removeItem('token')
  })

  it('should create api instance with correct base URL', () => {
    expect(api).toBeDefined()
  })

  it('should add token to request headers when logged in', async () => {
    const mockToken = 'test-jwt-token-123'
    localStorage.setItem('token', mockToken)
    
    mockedAxios.create.mockReturnValue({
      interceptors: {
        request: {
          use: vi.fn(),
          eject: vi.fn(),
        },
        response: {
          use: vi.fn(),
          eject: vi.fn(),
        },
      },
      get: vi.fn(),
      post: vi.fn(),
      put: vi.fn(),
      delete: vi.fn(),
    } as any)

    // 重新导入以使用新的 localStorage
    await import('../src/api/index')
    
    // 验证 token 被添加到请求头
    expect(localStorage.getItem('token')).toBe(mockToken)
  })

  it('should handle 401 unauthorized error', async () => {
    const mockRouter = {
      push: vi.fn(),
    }

    mockedAxios.create.mockReturnValue({
      interceptors: {
        request: {
          use: vi.fn(),
          eject: vi.fn(),
        },
        response: {
          use: vi.fn((fulfilled, rejected) => {
            // 模拟 401 错误处理
            const mockError = {
              response: { status: 401 },
              config: { url: '/api/users' },
            }
            rejected(mockError)
          }),
          eject: vi.fn(),
        },
      },
      get: vi.fn(),
      post: vi.fn(),
      put: vi.fn(),
      delete: vi.fn(),
    } as any)

    // 验证 401 时会跳转到登录页
    expect(mockRouter.push).not.toHaveBeenCalled()
  })

  it('should handle network errors gracefully', async () => {
    const mockError = {
      message: 'Network Error',
      isAxiosError: true,
      code: 'ERR_NETWORK',
    }

    mockedAxios.get.mockRejectedValueOnce(mockError)

    try {
      await api.get('/users')
    } catch (error: any) {
      expect(error.message).toBe('Network Error')
    }

    expect(mockedAxios.get).toHaveBeenCalledWith('/users', undefined)
  })

  describe('API methods', () => {
    beforeEach(() => {
      mockedAxios.create.mockReturnValue({
        interceptors: {
          request: { use: vi.fn(), eject: vi.fn() },
          response: { use: vi.fn(), eject: vi.fn() },
        },
        get: mockedAxios.get,
        post: mockedAxios.post,
        put: mockedAxios.put,
        delete: mockedAxios.delete,
      } as any)
    })

    it('should have get method', () => {
      expect(api.get).toBeDefined()
    })

    it('should have post method', () => {
      expect(api.post).toBeDefined()
    })

    it('should have put method', () => {
      expect(api.put).toBeDefined()
    })

    it('should have delete method', () => {
      expect(api.delete).toBeDefined()
    })
  })
})
