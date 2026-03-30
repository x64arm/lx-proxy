import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    // Fill login form
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    
    // Submit
    await page.click('button[type="submit"]');
    
    // Wait for navigation
    await page.waitForURL('/dashboard');
    
    // Verify dashboard is visible
    await expect(page.locator('text=仪表盘')).toBeVisible();
    await expect(page).toHaveTitle(/LX-Proxy/);
  });

  test('should show error on invalid credentials', async ({ page }) => {
    // Fill with wrong password
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'wrongpassword');
    
    // Submit
    await page.click('button[type="submit"]');
    
    // Wait for error message
    await expect(page.locator('.el-message--error')).toBeVisible();
    await expect(page.locator('text=用户名或密码错误')).toBeVisible();
    
    // Should stay on login page
    await expect(page).toHaveURL('/login');
  });

  test('should show error on empty username', async ({ page }) => {
    // Fill only password
    await page.fill('input[name="password"]', 'admin123');
    
    // Submit
    await page.click('button[type="submit"]');
    
    // Should show validation error
    await expect(page.locator('.el-form-item__error')).toBeVisible();
  });

  test('should show error on empty password', async ({ page }) => {
    // Fill only username
    await page.fill('input[name="username"]', 'admin');
    
    // Submit
    await page.click('button[type="submit"]');
    
    // Should show validation error
    await expect(page.locator('.el-form-item__error')).toBeVisible();
  });

  test('should logout successfully', async ({ page }) => {
    // Login first
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');

    // Logout
    await page.click('button:has-text("退出登录")');
    
    // Wait for redirect to login
    await page.waitForURL('/login');
    await expect(page).toHaveURL('/login');
  });

  test('should redirect to login when accessing protected route without auth', async ({ page }) => {
    // Try to access dashboard directly
    await page.goto('/dashboard');
    
    // Should redirect to login
    await page.waitForURL('/login');
    await expect(page).toHaveURL('/login');
  });
});
