import { test, expect } from '@playwright/test';

test.describe('P12 移动端适配 - 简化测试', () => {
  
  test('登录页面应该可访问', async ({ page }) => {
    await page.goto('/login', { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000); // 等待 Vue 渲染
    const title = await page.title();
    expect(title).toBeTruthy();
    console.log('✅ 登录页面可访问，标题:', title);
  });

  test('登录页面应该有输入框', async ({ page }) => {
    await page.goto('/login', { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000); // 等待 Vue 渲染
    const usernameInput = page.locator('#username');
    await expect(usernameInput).toBeVisible();
    console.log('✅ 用户名输入框可见');
  });

  test('移动端视口应该正常', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/login', { waitUntil: 'networkidle' });
    await page.waitForTimeout(2000); // 等待 Vue 渲染
    
    const loginBox = page.locator('.login-box');
    await expect(loginBox).toBeVisible();
    console.log('✅ 移动端布局正常');
  });
});
