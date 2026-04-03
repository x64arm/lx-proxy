# LX-Proxy Python SDK

Python client library for LX-Proxy API - Xray management panel

## Installation

```bash
pip install lx-proxy
```

For development:

```bash
pip install lx-proxy[dev]
```

## Quick Start

### Async Usage (Recommended)

```python
import asyncio
from lxproxy import LXProxyClient

async def main():
    async with LXProxyClient("http://localhost:8080") as client:
        # Login
        result = await client.async_login("admin", "admin123")
        print(f"Logged in as: {result['user']['username']}")
        
        # List users
        users = await client.async_list_users()
        print(f"Total users: {len(users)}")
        
        # Create a new user
        new_user = await client.async_create_user(
            username="testuser",
            password="securepassword123",
            role="user"
        )
        print(f"Created user: {new_user.username}")
        
        # List inbounds
        inbounds = await client.async_list_inbounds()
        print(f"Total inbounds: {len(inbounds)}")
        
        # Get system stats
        stats = await client.async_get_stats()
        print(f"Total traffic used: {stats.total_traffic_used} bytes")

asyncio.run(main())
```

### Sync Usage

```python
from lxproxy import LXProxyClient

# Create client
client = LXProxyClient("http://localhost:8080")

# Login
result = client.login("admin", "admin123")
print(f"Token: {result['token']}")

# List users
users = client.list_users()
for user in users:
    print(f"- {user.username} ({user.role})")

# Create inbound
inbound = client.create_inbound(
    tag="vmess-1000",
    protocol="vmess",
    port=1000,
    traffic_limit=10737418240,  # 10GB
)
print(f"Created inbound on port {inbound.port}")

# Get system status
status = client.get_system_status()
print(f"CPU: {status.cpu_usage}%, Memory: {status.memory_used} bytes")

# Logout
client.logout()
```

## API Reference

### Authentication

- `login(username, password)` - Login and get JWT token
- `logout()` - Logout (invalidate token)
- `get_current_user()` - Get current user info

### User Management

- `list_users()` - List all users
- `create_user(username, password, role)` - Create new user
- `get_user(user_id)` - Get user by ID
- `update_user(user_id, **kwargs)` - Update user
- `delete_user(user_id)` - Delete user

### Inbound Management

- `list_inbounds()` - List all inbound configs
- `create_inbound(**kwargs)` - Create new inbound
- `get_inbound(inbound_id)` - Get inbound by ID
- `update_inbound(inbound_id, **kwargs)` - Update inbound
- `delete_inbound(inbound_id)` - Delete inbound

### System & Statistics

- `get_stats()` - Get system statistics
- `get_system_status()` - Get real-time system status

## Error Handling

```python
from lxproxy import (
    LXProxyClient,
    AuthenticationError,
    APIError,
    NotFoundError,
    ValidationError,
)

client = LXProxyClient("http://localhost:8080")

try:
    client.login("admin", "wrong_password")
except AuthenticationError as e:
    print(f"Login failed: {e}")
except NotFoundError as e:
    print(f"Resource not found: {e}")
except APIError as e:
    print(f"API error: {e} (status: {e.status_code})")
except ValidationError as e:
    print(f"Validation error: {e}")
    for error in e.errors:
        print(f"  - {error}")
```

## Advanced Usage

### Custom Timeout

```python
client = LXProxyClient(
    "http://localhost:8080",
    timeout=60.0  # 60 seconds
)
```

### With API Key

```python
# If you already have a JWT token
client = LXProxyClient(
    "http://localhost:8080",
    api_key="your-jwt-token-here"
)
```

### Batch Operations

```python
async def batch_create_users():
    async with LXProxyClient("http://localhost:8080") as client:
        await client.async_login("admin", "admin123")
        
        # Create multiple users
        for i in range(10):
            try:
                user = await client.async_create_user(
                    username=f"user{i}",
                    password=f"password{i}"
                )
                print(f"Created: {user.username}")
            except Exception as e:
                print(f"Failed to create user{i}: {e}")
```

## Development

### Setup

```bash
# Clone repository
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy/sdk/python

# Install in development mode
pip install -e ".[dev]"
```

### Run Tests

```bash
pytest tests/ -v --cov=lxproxy
```

### Code Formatting

```bash
black lxproxy tests
ruff check lxproxy tests
```

### Type Checking

```bash
mypy lxproxy
```

## License

MIT License

## Links

- **GitHub:** https://github.com/x64arm/lx-proxy
- **PyPI:** https://pypi.org/project/lx-proxy/
- **Documentation:** https://github.com/x64arm/lx-proxy/tree/main/sdk/python
- **Issues:** https://github.com/x64arm/lx-proxy/issues
