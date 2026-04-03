# LX-Proxy Python SDK 开发文档

**版本：** 0.1.0  
**创建时间：** 2026-04-03  
**状态：** ✅ 已完成

---

## 📦 项目结构

```
sdk/python/
├── lxproxy/                 # SDK 源代码
│   ├── __init__.py         # 包入口
│   ├── client.py           # 主客户端类
│   └── exceptions.py       # 异常定义
├── tests/                   # 测试文件
│   ├── __init__.py
│   └── test_client.py      # 客户端测试
├── examples/                # 使用示例
│   └── basic_usage.py      # 基础用法示例
├── pyproject.toml          # 项目配置
├── README.md               # 使用文档
└── .gitignore              # Git 忽略文件
```

---

## 🎯 功能特性

### ✅ 已实现功能

1. **双模式支持**
   - 异步 API（推荐）：`async_login()`, `async_list_users()` 等
   - 同步 API：`login()`, `list_users()` 等

2. **完整 CRUD 操作**
   - 用户管理：创建、查询、更新、删除
   - 入站管理：创建、查询、更新、删除
   - 系统监控：统计信息、实时状态

3. **认证管理**
   - JWT Token 自动管理
   - 登录/登出功能
   - 自动刷新 Token（需扩展）

4. **错误处理**
   - 分层异常体系
   - HTTP 状态码映射
   - 详细错误信息

5. **类型注解**
   - Pydantic 数据模型
   - 完整的类型提示
   - IDE 友好

6. **测试覆盖**
   - 单元测试（pytest）
   - 异步测试支持
   - Mock 对象测试

---

## 🔧 技术栈

### 核心依赖
- **httpx** (>=0.25.0) - HTTP 客户端（支持 async/sync）
- **pydantic** (>=2.0.0) - 数据验证和序列化

### 开发依赖
- **pytest** (>=7.4.0) - 测试框架
- **pytest-asyncio** - 异步测试支持
- **pytest-cov** - 测试覆盖率
- **black** - 代码格式化
- **ruff** - 代码检查
- **mypy** - 类型检查

---

## 📝 API 设计

### 命名约定

```python
# 异步方法：async_前缀
await client.async_login()
await client.async_list_users()

# 同步方法：无前缀
client.login()
client.list_users()
```

### 数据模型

所有数据模型使用 Pydantic BaseModel：

```python
from pydantic import BaseModel, Field
from datetime import datetime

class User(BaseModel):
    id: str
    username: str
    role: str
    created_at: datetime
    updated_at: datetime
```

### 异常层次

```
LXProxyError (基类)
├── AuthenticationError (401)
├── APIError (其他 4xx/5xx)
├── NotFoundError (404)
└── ValidationError (400)
```

---

## 🧪 测试指南

### 运行测试

```bash
# 安装开发依赖
pip install -e ".[dev]"

# 运行所有测试
pytest tests/ -v

# 运行测试并生成覆盖率报告
pytest tests/ -v --cov=lxproxy --cov-report=html

# 运行特定测试
pytest tests/test_client.py::TestAuthentication -v
```

### 编写测试

```python
import pytest
from unittest.mock import AsyncMock, patch
from lxproxy import LXProxyClient

@pytest.mark.asyncio
async def test_login():
    client = LXProxyClient()
    
    with patch.object(client, '_get_async_client') as mock_get:
        mock_client = AsyncMock()
        mock_client.post.return_value = Mock(status_code=200, json=lambda: {"token": "test"})
        mock_get.return_value = mock_client
        
        result = await client.async_login("admin", "password")
        assert result["token"] == "test"
```

---

## 📦 发布流程

### 1. 更新版本号

编辑 `pyproject.toml`：

```toml
[project]
version = "0.1.0"  # 遵循 SemVer
```

### 2. 更新 CHANGELOG

编辑 `CHANGELOG.md`，记录变更内容。

### 3. 运行测试

```bash
pytest tests/ -v
black --check lxproxy tests
ruff check lxproxy tests
mypy lxproxy
```

### 4. 构建包

```bash
python -m build
```

### 5. 发布到 PyPI

```bash
# 测试发布
twine upload --repository testpypi dist/*

# 生产发布
twine upload dist/*
```

### 6. 创建 Git Tag

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## 🔄 版本历史

### v0.1.0 (2026-04-03)

**新增功能：**
- ✅ 初始版本发布
- ✅ 异步/同步双模式支持
- ✅ 用户管理 CRUD
- ✅ 入站管理 CRUD
- ✅ 系统统计查询
- ✅ 完整的异常处理
- ✅ Pydantic 数据模型
- ✅ 单元测试覆盖

**已知问题：**
- ⚠️ 流量统计 API 未实现（下一阶段）
- ⚠️ 批量操作未实现（下一阶段）
- ⚠️ Token 自动刷新未实现（下一阶段）

---

## 📋 待办事项

### 短期（v0.2.0）
- [ ] 流量统计 API
- [ ] 批量操作支持
- [ ] Token 自动刷新
- [ ] 请求重试机制
- [ ] 连接池优化

### 中期（v0.3.0）
- [ ] TOTP 双因素认证
- [ ] 邮件通知 API
- [ ] 订阅链接生成
- [ ] 节点管理 API
- [ ] 审计日志 API

### 长期（v1.0.0）
- [ ] WebSocket 实时推送
- [ ] 离线队列支持
- [ ] 插件系统 SDK
- [ ] 完整的文档站点
- [ ] 示例项目模板

---

## 🤝 贡献指南

### 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy/sdk/python

# 创建虚拟环境
python -m venv venv
source venv/bin/activate  # Linux/macOS
# or
.\venv\Scripts\activate  # Windows

# 安装开发依赖
pip install -e ".[dev]"
```

### 提交代码

1. Fork 仓库
2. 创建功能分支：`git checkout -b feature/my-feature`
3. 提交更改：`git commit -m 'feat: add my feature'`
4. 推送到分支：`git push origin feature/my-feature`
5. 创建 Pull Request

### 提交信息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat: 新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式
refactor: 重构
test: 测试
chore: 构建/工具
```

---

## 📚 资源链接

- **PyPI:** https://pypi.org/project/lx-proxy/
- **GitHub:** https://github.com/x64arm/lx-proxy/tree/main/sdk/python
- **Issue Tracker:** https://github.com/x64arm/lx-proxy/issues
- **LX-Proxy Docs:** https://github.com/x64arm/lx-proxy/tree/main/docs

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

*最后更新：2026-04-03*
