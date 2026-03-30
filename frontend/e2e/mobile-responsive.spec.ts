import { test, expect, devices } from '@playwright/test';

/**
 * P12 移动端适配测试
 * 测试移动端响应式布局、触摸交互、卡片视图等功能
 */

// 移动端测试设备
const MOBILE_DEVICES = [
  { name: 'iPhone SE', ...devices['iPhone SE'] },
  { name: 'iPhone 12', ...devices['iPhone 12'] },
  { name: 'Pixel 5', ...devices['Pixel 5'] },
];

// 平板设备
const TABLET_DEVICES = [
  { name: 'iPad Mini', ...devices['iPad Mini'] },
  { name: 'iPad Air', ...devices['iPad Air'] },
];

test.describe('P12 移动端适配 - 登录页面', () => {
  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上正常显示登录页面`, async ({ page }) => {
      await page.goto('/login');
      
      // 检查页面标题
      await expect(page).toHaveTitle(/LX-Proxy/);
      
      // 检查登录框显示
      const loginBox = page.locator('.login-box');
      await expect(loginBox).toBeVisible();
      
      // 检查标题显示
      const title = page.locator('.title');
      await expect(title).toContainText('🚀 LX-Proxy');
      
      // 检查表单输入框
      const usernameInput = page.locator('input[type="text"]');
      await expect(usernameInput).toBeVisible();
      
      const passwordInput = page.locator('input[type="password"]');
      await expect(passwordInput).toBeVisible();
      
      // 检查登录按钮
      const loginButton = page.locator('button[type="submit"]');
      await expect(loginButton).toBeVisible();
      
      // 检查按钮最小高度（触摸友好）
      const buttonHeight = await loginButton.boundingBox();
      expect(buttonHeight?.height).toBeGreaterThanOrEqual(48);
    });
  }
});

test.describe('P12 移动端适配 - 主布局', () => {
  test.beforeEach(async ({ page }) => {
    // 模拟登录
    await page.goto('/login');
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上显示汉堡菜单`, async ({ page }) => {
      // 检查汉堡菜单按钮
      const menuButton = page.locator('.mobile-menu-btn');
      await expect(menuButton).toBeVisible();
      
      // 点击汉堡菜单
      await menuButton.click();
      
      // 检查侧边栏抽屉是否打开
      const drawer = page.locator('.mobile-sidebar-drawer');
      await expect(drawer).toBeVisible();
      
      // 检查菜单项
      const menuItems = drawer.locator('.el-menu-item');
      await expect(menuItems.first()).toBeVisible();
    });

    test(`应在 ${device.name} 上隐藏桌面端侧边栏`, async ({ page }) => {
      const desktopSidebar = page.locator('.desktop-sidebar');
      await expect(desktopSidebar).not.toBeVisible();
    });
  }

  for (const device of TABLET_DEVICES) {
    test(`应在 ${device.name} 上适配平板布局`, async ({ page }) => {
      // 平板应该显示桌面端布局
      const desktopSidebar = page.locator('.desktop-sidebar');
      await expect(desktopSidebar).toBeVisible();
    });
  }
});

test.describe('P12 移动端适配 - 用户列表', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
    await page.goto('/users');
  });

  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上显示卡片视图`, async ({ page }) => {
      // 检查卡片视图容器
      const cardsContainer = page.locator('.cards-container');
      await expect(cardsContainer).toBeVisible();
      
      // 检查用户卡片
      const userCards = page.locator('.user-card');
      const cardCount = await userCards.count();
      expect(cardCount).toBeGreaterThan(0);
      
      // 检查卡片结构
      const firstCard = userCards.first();
      await expect(firstCard.locator('.card-header')).toBeVisible();
      await expect(firstCard.locator('.card-body')).toBeVisible();
      await expect(firstCard.locator('.card-actions')).toBeVisible();
    });

    test(`应在 ${device.name} 上支持视图切换`, async ({ page }) => {
      // 检查视图切换按钮
      const viewToggleBtn = page.locator('.view-toggle-btn');
      await expect(viewToggleBtn).toBeVisible();
      
      // 切换到表格视图
      await viewToggleBtn.click();
      
      // 等待视图切换
      await page.waitForTimeout(500);
      
      // 检查表格视图是否显示
      const tableCard = page.locator('.table-card');
      await expect(tableCard).toBeVisible();
    });

    test(`应在 ${device.name} 上操作按钮全宽显示`, async ({ page }) => {
      const actionButtons = page.locator('.card-actions .el-button');
      const buttonCount = await actionButtons.count();
      
      for (let i = 0; i < buttonCount; i++) {
        const button = actionButtons.nth(i);
        const buttonWidth = await button.boundingBox();
        expect(buttonWidth?.width).toBeGreaterThan(100); // 全宽按钮
      }
    });
  }
});

test.describe('P12 移动端适配 - 入站列表', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
    await page.goto('/inbounds');
  });

  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上显示入站卡片视图`, async ({ page }) => {
      const cardsContainer = page.locator('.cards-container');
      await expect(cardsContainer).toBeVisible();
      
      const inboundCards = page.locator('.inbound-card');
      const cardCount = await inboundCards.count();
      expect(cardCount).toBeGreaterThan(0);
      
      // 检查卡片头部（备注 + 标签）
      const firstCard = inboundCards.first();
      await expect(firstCard.locator('.card-remark')).toBeVisible();
      await expect(firstCard.locator('.el-tag')).toBeVisible();
    });

    test(`应在 ${device.name} 上显示流量信息`, async ({ page }) => {
      const trafficDisplay = page.locator('.inbound-card .card-value').first();
      await expect(trafficDisplay).toBeVisible();
    });
  }
});

test.describe('P12 移动端适配 - 系统设置', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
    await page.goto('/settings');
  });

  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上表单标签垂直布局`, async ({ page }) => {
      const formLabels = page.locator('.mobile-form .el-form-item__label');
      const labelCount = await formLabels.count();
      
      for (let i = 0; i < labelCount; i++) {
        const label = formLabels.nth(i);
        const labelWidth = await label.boundingBox();
        // 移动端标签应该全宽
        expect(labelWidth?.width).toBeGreaterThan(200);
      }
    });

    test(`应在 ${device.name} 上输入框全宽显示`, async ({ page }) => {
      const inputs = page.locator('.mobile-form .el-input');
      const inputCount = await inputs.count();
      
      for (let i = 0; i < inputCount; i++) {
        const input = inputs.nth(i);
        const inputWidth = await input.boundingBox();
        expect(inputWidth?.width).toBeGreaterThan(250); // 全宽输入框
      }
    });

    test(`应在 ${device.name} 上底部操作按钮全宽`, async ({ page }) => {
      const formActions = page.locator('.form-actions');
      const buttons = formActions.locator('.el-button');
      
      // 检查保存按钮
      const saveButton = buttons.first();
      const buttonWidth = await saveButton.boundingBox();
      expect(buttonWidth?.width).toBeGreaterThan(250); // 全宽按钮
    });
  }
});

test.describe('P12 移动端适配 - 流量统计', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
    await page.goto('/traffic');
  });

  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上图表自适应尺寸`, async ({ page }) => {
      // 检查图表容器
      const chartContainer = page.locator('[ref="chartRef"]');
      await expect(chartContainer).toBeVisible();
      
      // 检查图表高度（移动端应该更小）
      const chartHeight = await chartContainer.boundingBox();
      expect(chartHeight?.height).toBeLessThanOrEqual(300); // 移动端图表高度
    });

    test(`应在 ${device.name} 上统计卡片单列显示`, async ({ page }) => {
      const statCards = page.locator('.el-statistic');
      const cardCount = await statCards.count();
      expect(cardCount).toBeGreaterThanOrEqual(3);
      
      // 检查卡片是否垂直排列（单列）
      const firstCard = statCards.first();
      const secondCard = statCards.nth(1);
      
      const firstBox = await firstCard.boundingBox();
      const secondBox = await secondCard.boundingBox();
      
      // 第二个卡片应该在第一个卡片下方
      expect(secondBox?.y).toBeGreaterThan(firstBox?.y || 0);
    });
  }
});

test.describe('P12 移动端适配 - 触摸友好性', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/dashboard/);
  });

  for (const device of MOBILE_DEVICES) {
    test(`应在 ${device.name} 上按钮最小高度 44px`, async ({ page }) => {
      const buttons = page.locator('.el-button');
      const buttonCount = await buttons.count();
      
      for (let i = 0; i < Math.min(buttonCount, 5); i++) {
        const button = buttons.nth(i);
        const buttonHeight = await button.boundingBox();
        expect(buttonHeight?.height).toBeGreaterThanOrEqual(44);
      }
    });

    test(`应在 ${device.name} 上菜单项最小高度 44px`, async ({ page }) => {
      // 打开侧边栏
      const menuButton = page.locator('.mobile-menu-btn');
      await menuButton.click();
      
      const menuItems = page.locator('.el-menu-item');
      const itemCount = await menuItems.count();
      
      for (let i = 0; i < Math.min(itemCount, 3); i++) {
        const item = menuItems.nth(i);
        const itemHeight = await item.boundingBox();
        expect(itemHeight?.height).toBeGreaterThanOrEqual(44);
      }
    });
  }
});

test.describe('P12 移动端适配 - 响应式断点', () => {
  test('应在 375px 宽度显示移动端布局', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/login');
    
    const loginBox = page.locator('.login-box');
    await expect(loginBox).toBeVisible();
    
    // 检查是否有移动端优化
    const padding = await loginBox.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return parseFloat(style.padding);
    });
    expect(padding).toBeLessThanOrEqual(24); // 移动端内边距更小
  });

  test('应在 768px 宽度显示平板布局', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/login');
    
    // 检查布局切换
    const loginBox = page.locator('.login-box');
    await expect(loginBox).toBeVisible();
  });

  test('应在 1024px 宽度显示桌面布局', async ({ page }) => {
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.goto('/login');
    
    const loginBox = page.locator('.login-box');
    await expect(loginBox).toBeVisible();
  });
});
