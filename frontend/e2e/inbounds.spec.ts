import { test, expect } from '@playwright/test';

test.describe('Inbound Configuration', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');

    // Navigate to inbounds page
    await page.click('text=入站配置');
    await page.waitForURL('/inbounds');
  });

  test('should display inbounds list', async ({ page }) => {
    // Wait for table to load
    await page.waitForSelector('.el-table');
    
    // Verify table headers
    await expect(page.locator('text=名称')).toBeVisible();
    await expect(page.locator('text=协议')).toBeVisible();
    await expect(page.locator('text=端口')).toBeVisible();
    await expect(page.locator('text=流量")).toBeVisible();
  });

  test('should create a new Vmess inbound', async ({ page }) => {
    // Click create button
    await page.click('button:has-text("新建入站")');
    
    // Wait for dialog
    await expect(page.locator('.el-dialog__title:has-text("新建入站")')).toBeVisible();
    
    // Fill basic info
    await page.fill('input[placeholder="名称"]', 'test-vmess-e2e');
    await page.fill('input[placeholder="端口"]', '10080');
    
    // Select protocol
    await page.selectOption('select', 'vmess');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
    
    // Verify inbound appears in table
    await expect(page.locator('text=test-vmess-e2e')).toBeVisible();
  });

  test('should create a new Vless inbound', async ({ page }) => {
    // Click create button
    await page.click('button:has-text("新建入站")');
    
    // Fill basic info
    await page.fill('input[placeholder="名称"]', 'test-vless-e2e');
    await page.fill('input[placeholder="端口"]', '10081');
    
    // Select protocol
    await page.selectOption('select', 'vless');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });

  test('should create a new Trojan inbound', async ({ page }) => {
    // Click create button
    await page.click('button:has-text("新建入站")');
    
    // Fill basic info
    await page.fill('input[placeholder="名称"]', 'test-trojan-e2e');
    await page.fill('input[placeholder="端口"]', '10082');
    
    // Select protocol
    await page.selectOption('select', 'trojan');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });

  test('should edit inbound', async ({ page }) => {
    // Click edit button for first inbound
    await page.click('button:has-text("编辑") >> nth=0');
    
    // Wait for dialog
    await expect(page.locator('.el-dialog__title:has-text("编辑入站")')).toBeVisible();
    
    // Update remark
    await page.fill('input[placeholder="备注"]', 'Updated via E2E test');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });

  test('should toggle inbound enable/disable', async ({ page }) => {
    // Find toggle switch for first inbound
    const toggleSwitch = page.locator('.el-switch').first();
    
    // Click to toggle
    await toggleSwitch.click();
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });

  test('should reset inbound traffic', async ({ page }) => {
    // Click reset button
    await page.click('button:has-text("重置流量") >> nth=0');
    
    // Confirm reset
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });

  test('should view subscription links', async ({ page }) => {
    // Click subscription link button
    await page.click('button:has-text("订阅链接") >> nth=0');
    
    // Wait for dialog
    await expect(page.locator('.el-dialog__title:has-text("订阅链接")')).toBeVisible();
    
    // Verify QR code is visible
    await expect(page.locator('img[alt="QR Code"]')).toBeVisible();
    
    // Verify subscription link is visible
    await expect(page.locator('text=vmess://')).toBeVisible();
  });

  test('should delete inbound', async ({ page }) => {
    // Click delete button for test inbound
    await page.click('button:has-text("删除") >> nth=0');
    
    // Confirm deletion
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });
});
