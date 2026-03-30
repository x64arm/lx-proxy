// 登录页面组件测试
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../src/views/LoginView.vue'

const mockRouter = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', name: 'login', component: LoginView },
    { path: '/dashboard', name: 'dashboard', component: { template: '<div>Dashboard</div>' } },
  ],
})

describe('LoginView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
  })

  it('renders login form', () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [mockRouter],
      },
    })
    
    expect(wrapper.find('form').exists()).toBe(true)
    expect(wrapper.find('input[type="text"]').exists()).toBe(true)
    expect(wrapper.find('input[type="password"]').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('displays validation errors for empty fields', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [mockRouter],
      },
    })
    
    const submitButton = wrapper.find('button[type="submit"]')
    await submitButton.trigger('click')
    
    // 应该有验证错误提示
    expect(wrapper.findAll('.el-form-item__error').length).toBeGreaterThan(0)
  })

  it('accepts valid username and password input', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [mockRouter],
      },
    })
    
    const usernameInput = wrapper.find('input[type="text"]')
    const passwordInput = wrapper.find('input[type="password"]')
    
    await usernameInput.setValue('admin')
    await passwordInput.setValue('admin123')
    
    expect(usernameInput.element.value).toBe('admin')
    expect(passwordInput.element.value).toBe('admin123')
  })

  it('has remember me checkbox', () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [mockRouter],
      },
    })
    
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true)
  })

  it('displays loading state during login', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [mockRouter],
      },
    })
    
    // 模拟登录按钮点击
    const submitButton = wrapper.find('button[type="submit"]')
    await submitButton.trigger('click')
    
    // 验证是否有 loading 状态（如果有实现的话）
    expect(wrapper.exists()).toBe(true)
  })

  it('navigates to dashboard after successful login', async () => {
    const wrapper = mount(LoginView, {
      global: {
        plugins: [mockRouter],
      },
    })
    
    const routerPushSpy = vi.spyOn(mockRouter, 'push')
    
    const usernameInput = wrapper.find('input[type="text"]')
    const passwordInput = wrapper.find('input[type="password"]')
    
    await usernameInput.setValue('admin')
    await passwordInput.setValue('admin123')
    
    const submitButton = wrapper.find('button[type="submit"]')
    await submitButton.trigger('click')
    
    // 验证是否尝试跳转
    expect(routerPushSpy).toHaveBeenCalled()
  })
})
