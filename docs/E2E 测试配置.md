# Playwright E2E 测试配置

**完成时间：** 2026-03-25  
**状态：** ✅ 完成

---

## 📦 安装依赖

```bash
cd frontend

# 安装 Playwright
npm install -D @playwright/test

# 安装 Playwright 浏览器
npx playwright install

# 安装 Playwright UI 模式（可选）
npm install -D @playwright/test
```

---

## 📁 测试文件结构

```
frontend/e2e/
├── example.spec.ts           # 示例测试
├── auth.spec.ts              # 认证流程测试
├── dashboard.spec.ts         # 仪表盘测试
├── users.spec.ts             # 用户管理测试
├── inbounds.spec.ts          # 入站配置测试
├── traffic.spec.ts           # 流量统计测试
└── settings.spec.ts          # 系统设置测试
```

---

## 🚀 运行测试

### 运行所有测试
```bash
npx playwright test
```

### 运行特定测试文件
```bash
npx playwright test e2e/auth.spec.ts
```

### 运行特定测试用例
```bash
npx playwright test -g "should login successfully"
```

### UI 模式运行
```bash
npx playwright test --ui
```

### 生成 HTML 报告
```bash
npx playwright test --reporter=html
npx playwright show-report
```

---

## 📊 测试覆盖率目标

| 模块 | 测试数量 | 覆盖场景 |
|------|---------|----------|
| 认证 | 6 | 登录、登出、错误处理、TOTP |
| 仪表盘 | 4 | 数据加载、图表显示、刷新 |
| 用户管理 | 8 | CRUD 操作、搜索、分页 |
| 入站配置 | 8 | CRUD 操作、协议切换、订阅链接 |
| 流量统计 | 5 | 日期筛选、图表交互、导出 |
| 系统设置 | 5 | 配置修改、保存、重置 |

**总计：36 个 E2E 测试用例**

---

## 🔧 配置说明

### playwright.config.ts
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
});
```

---

## 📝 测试示例

### 登录测试
```typescript
import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('should login successfully', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('text=仪表盘')).toBeVisible();
  });

  test('should show error on invalid credentials', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'wrong');
    await page.click('button[type="submit"]');
    
    await expect(page.locator('.el-message--error')).toBeVisible();
  });
});
```

### 用户管理测试
```typescript
import { test, expect } from '@playwright/test';

test.describe('User Management', () => {
  test.beforeEach(async ({ page }) => {
    // 登录
    await page.goto('/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
  });

  test('should create a new user', async ({ page }) => {
    await page.goto('/users');
    
    await page.click('button:has-text("新建用户")');
    await page.fill('input[placeholder="用户名"]', 'testuser');
    await page.fill('input[placeholder="密码"]', 'password123');
    await page.selectOption('select', 'user');
    await page.click('button:has-text("确定")');
    
    await expect(page.locator('text=testuser')).toBeVisible();
  });

  test('should edit user', async ({ page }) => {
    await page.goto('/users');
    
    await page.click('button:has-text("编辑")');
    await page.fill('input[placeholder="邮箱"]', 'test@example.com');
    await page.click('button:has-text("确定")');
    
    await expect(page.locator('text=test@example.com')).toBeVisible();
  });
});
```

---

## 🎯 最佳实践

### 1. 使用 Page Object 模式
```typescript
// e2e/pages/LoginPage.ts
export class LoginPage {
  constructor(private page: Page) {}

  async goto() {
    await this.page.goto('/login');
  }

  async login(username: string, password: string) {
    await this.page.fill('input[name="username"]', username);
    await this.page.fill('input[name="password"]', password);
    await this.page.click('button[type="submit"]');
  }
}

// 测试中使用
test('should login', async ({ page }) => {
  const loginPage = new LoginPage(page);
  await loginPage.goto();
  await loginPage.login('admin', 'admin123');
});
```

### 2. 使用 Fixtures
```typescript
// e2e/fixtures.ts
import { test as base } from '@playwright/test';

export const test = base.extend<{
  loggedInPage: Page;
}>({
  loggedInPage: async ({ page }, use) => {
    await page.goto('/login');
    await page.fill('input[name="username"]', 'admin');
    await page.fill('input[name="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
    await use(page);
  },
});
```

### 3. 数据清理
```typescript
test.afterEach(async ({ page }) => {
  // 清理测试数据
  await page.request.delete('/api/test-cleanup');
});
```

---

## 📈 CI/CD 集成

### GitHub Actions
```yaml
name: E2E Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        run: |
          cd frontend
          npm ci
          npx playwright install --with-deps
      
      - name: Build frontend
        run: |
          cd frontend
          npm run build
      
      - name: Start services
        run: |
          docker-compose up -d
          sleep 10
      
      - name: Run E2E tests
        run: |
          cd frontend
          npx playwright test
      
      - name: Upload test report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: frontend/playwright-report/
```

---

## 🔍 调试技巧

### 1. 慢动作模式
```bash
npx playwright test --debug
```

### 2. 截图
```typescript
await page.screenshot({ path: 'screenshot.png' });
```

### 3. 录制视频
```typescript
// playwright.config.ts
use: {
  video: 'on-first-retry',
}
```

### 4. 追踪
```typescript
// playwright.config.ts
use: {
  trace: 'on-first-retry',
}

// 查看追踪
npx playwright show-trace trace.zip
```

---

## ✅ 验收标准

- [x] Playwright 环境配置完成
- [x] 认证流程测试覆盖
- [x] 核心功能 E2E 测试
- [x] CI/CD 集成配置
- [x] 测试报告生成
- [ ] 跨浏览器测试（Chrome/Firefox/Safari）
- [ ] 移动端测试（可选）

---

**🎉 E2E 测试框架搭建完成！**

*最后更新：2026-03-25*
