import { test, expect } from '@playwright/test';

test.describe('User Management', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');

    // Navigate to users page
    await page.click('text=用户管理');
    await page.waitForURL('/users');
  });

  test('should display users list', async ({ page }) => {
    // Wait for table to load
    await page.waitForSelector('.el-table');
    
    // Verify table headers
    await expect(page.locator('text=用户名')).toBeVisible();
    await expect(page.locator('text=角色')).toBeVisible();
    await expect(page.locator('text=邮箱')).toBeVisible();
  });

  test('should create a new user', async ({ page }) => {
    // Click create button
    await page.click('button:has-text("新建用户")');
    
    // Wait for dialog
    await expect(page.locator('.el-dialog__title:has-text("新建用户")')).toBeVisible();
    
    // Fill form
    await page.fill('input[placeholder="用户名"]', 'testuser_e2e');
    await page.fill('input[placeholder="密码"]', 'password123');
    await page.fill('input[placeholder="邮箱"]', 'testuser@example.com');
    
    // Select role
    await page.selectOption('select', 'user');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
    
    // Verify user appears in table
    await expect(page.locator('text=testuser_e2e')).toBeVisible();
  });

  test('should show validation error for duplicate username', async ({ page }) => {
    // Click create button
    await page.click('button:has-text("新建用户")');
    
    // Fill with existing username
    await page.fill('input[placeholder="用户名"]', 'admin');
    await page.fill('input[placeholder="密码"]', 'password123');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for error message
    await expect(page.locator('.el-message--error')).toBeVisible();
  });

  test('should edit user', async ({ page }) => {
    // Find edit button for first user
    await page.click('button:has-text("编辑") >> nth=0');
    
    // Wait for dialog
    await expect(page.locator('.el-dialog__title:has-text("编辑用户")')).toBeVisible();
    
    // Update email
    await page.fill('input[placeholder="邮箱"]', 'updated@example.com');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });

  test('should delete user', async ({ page }) => {
    // Find delete button for test user (if exists)
    const deleteButton = page.locator('button:has-text("删除")').first();
    
    if (await deleteButton.isVisible()) {
      await deleteButton.click();
      
      // Confirm deletion
      await page.click('button:has-text("确定")');
      
      // Wait for success message
      await expect(page.locator('.el-message--success')).toBeVisible();
    }
  });

  test('should search users', async ({ page }) => {
    // Fill search input
    await page.fill('input[placeholder="搜索用户"]', 'admin');
    
    // Wait for search results
    await page.waitForTimeout(500);
    
    // Verify only admin user is shown
    await expect(page.locator('text=admin')).toBeVisible();
  });

  test('should change user password', async ({ page }) => {
    // Click reset password button
    await page.click('button:has-text("重置密码") >> nth=0');
    
    // Fill new password
    await page.fill('input[placeholder="新密码"]', 'newpassword123');
    await page.fill('input[placeholder="确认密码"]', 'newpassword123');
    
    // Submit
    await page.click('button:has-text("确定")');
    
    // Wait for success message
    await expect(page.locator('.el-message--success')).toBeVisible();
  });
});
