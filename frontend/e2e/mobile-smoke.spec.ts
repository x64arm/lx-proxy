import { test, expect } from '@playwright/test';

/**
 * P12 移动端适配 - 基础冒烟测试
 * 简化版本，验证核心功能
 */

test.describe('P12 移动端基础测试', () => {
  
  test('应能访问登录页面', async ({ page }) => {
    await page.goto('/login');
    
    // 检查页面标题
    await expect(page).toHaveTitle(/LX-Proxy/);
    
    // 检查登录框
    const loginBox = page.locator('.login-container');
    await expect(loginBox).toBeVisible();
    
    console.log('✅ 登录页面可访问');
  });

  test('应能在移动端显示响应式布局', async ({ page }) => {
    // 设置为手机视口
    await page.setViewportSize({ width: 375, height: 667 });
    
    await page.goto('/login');
    
    // 检查登录框在移动端正常显示
    const loginBox = page.locator('.login-box');
    await expect(loginBox).toBeVisible();
    
    // 检查输入框
    const usernameInput = page.locator('input[type="text"]');
    await expect(usernameInput).toBeVisible();
    
    console.log('✅ 移动端布局正常');
  });

  test('应能访问首页', async ({ page }) => {
    await page.goto('/');
    
    // 应该重定向到登录页或显示首页
    const url = page.url();
    expect(url).toMatch(/\/(login|dashboard)/);
    
    console.log('✅ 首页可访问，当前 URL:', url);
  });
});
