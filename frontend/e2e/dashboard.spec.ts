import { test, expect } from '@playwright/test';

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
  });

  test('should load dashboard successfully', async ({ page }) => {
    // Verify dashboard is visible
    await expect(page.locator('text=仪表盘')).toBeVisible();
    await expect(page).toHaveURL('/dashboard');
  });

  test('should display statistics cards', async ({ page }) => {
    // Wait for statistics to load
    await page.waitForSelector('.el-statistic');
    
    // Verify statistic cards are visible
    await expect(page.locator('.el-statistic:has-text("总用户")')).toBeVisible();
    await expect(page.locator('.el-statistic:has-text("入站配置")')).toBeVisible();
    await expect(page.locator('.el-statistic:has-text("总流量")')).toBeVisible();
  });

  test('should display traffic chart', async ({ page }) => {
    // Wait for chart to load
    await page.waitForSelector('#trafficChart');
    
    // Verify chart canvas exists
    const canvas = page.locator('#trafficChart canvas');
    await expect(canvas).toBeVisible();
  });

  test('should refresh statistics on manual refresh', async ({ page }) => {
    // Click refresh button
    await page.click('button:has-text("刷新")');
    
    // Wait for data to reload (should show loading state)
    await page.waitForLoadState('networkidle');
    
    // Verify data is still visible after refresh
    await expect(page.locator('.el-statistic:has-text("总用户")')).toBeVisible();
  });

  test('should display system status', async ({ page }) => {
    // Wait for system status section
    await page.waitForSelector('text=系统状态');
    
    // Verify CPU and Memory info
    await expect(page.locator('text=CPU')).toBeVisible();
    await expect(page.locator('text=内存')).toBeVisible();
  });
});
