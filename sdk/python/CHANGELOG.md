# Changelog

All notable changes to the LX-Proxy Python SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Traffic statistics API endpoints
- Batch operations support
- Automatic token refresh
- Request retry mechanism
- Connection pooling optimization

## [0.1.0] - 2026-04-03

### Added

#### Core Functionality
- **Async/Sync Dual Mode Support** - Both asynchronous and synchronous API clients
- **Authentication** - Login, logout, and JWT token management
- **User Management** - Full CRUD operations for users
- **Inbound Management** - Full CRUD operations for inbound configs
- **System Statistics** - Query system stats and real-time status

#### Data Models
- `User` - User account model
- `InboundConfig` - Inbound configuration model
- `CreateUserRequest` - User creation request
- `CreateInboundRequest` - Inbound creation request
- `SystemStatus` - Real-time system status
- `Stats` - System statistics

#### Error Handling
- `LXProxyError` - Base exception class
- `AuthenticationError` - Authentication failures (401)
- `APIError` - General API errors
- `NotFoundError` - Resource not found (404)
- `ValidationError` - Validation errors (400)

#### Developer Experience
- Type hints with Pydantic models
- Comprehensive test suite (pytest)
- Usage examples in `examples/`
- Complete README documentation
- Development guide

#### Infrastructure
- GitHub Actions CI/CD workflow
- PyPI publishing automation
- Code quality checks (black, ruff, mypy)
- Test coverage reporting

### Technical Details

#### Dependencies
- httpx >= 0.25.0
- pydantic >= 2.0.0

#### Development Dependencies
- pytest >= 7.4.0
- pytest-asyncio >= 0.21.0
- pytest-cov >= 4.1.0
- black >= 23.0.0
- ruff >= 0.1.0
- mypy >= 1.5.0

#### Python Versions Supported
- Python 3.8
- Python 3.9
- Python 3.10
- Python 3.11
- Python 3.12

### Known Issues
- Token auto-refresh not yet implemented
- Batch operations pending
- Traffic statistics API not exposed in SDK

### Security
- Secure JWT token handling
- HTTPS support (via base_url configuration)
- No sensitive data logging

---

## Legend

- **Added** - New features
- **Changed** - Changes in existing functionality
- **Deprecated** - Soon-to-be removed features
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Security improvements

---

*Last updated: 2026-04-03*
