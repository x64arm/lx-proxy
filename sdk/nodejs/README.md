# LX-Proxy Node.js SDK

TypeScript/JavaScript client library for LX-Proxy API - Xray management panel

[![npm version](https://img.shields.io/npm/v/@lx-proxy/sdk.svg)](https://www.npmjs.com/package/@lx-proxy/sdk)
[![npm downloads](https://img.shields.io/npm/dm/@lx-proxy/sdk.svg)](https://www.npmjs.com/package/@lx-proxy/sdk)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.2-blue.svg)](https://www.typescriptlang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Installation

```bash
npm install @lx-proxy/sdk
```

or with yarn:

```bash
yarn add @lx-proxy/sdk
```

or with pnpm:

```bash
pnpm add @lx-proxy/sdk
```

## Quick Start

### TypeScript/ESM

```typescript
import { LXProxyClient } from '@lx-proxy/sdk';

async function main() {
  const client = new LXProxyClient({
    baseURL: 'http://localhost:8080',
  });

  // Login
  const { token, user } = await client.login('admin', 'admin123');
  console.log(`Logged in as: ${user.username}`);

  // List users
  const users = await client.listUsers();
  console.log(`Total users: ${users.length}`);

  // Create inbound
  const inbound = await client.createInbound({
    tag: 'vmess-1000',
    protocol: 'vmess',
    port: 1000,
    traffic_limit: 10737418240, // 10GB
  });
  console.log(`Created inbound on port ${inbound.port}`);

  // Get stats
  const stats = await client.getStats();
  console.log(`Total traffic used: ${stats.total_traffic_used} bytes`);

  // Logout
  await client.logout();
}

main().catch(console.error);
```

### CommonJS

```javascript
const { LXProxyClient } = require('@lx-proxy/sdk');

async function main() {
  const client = new LXProxyClient({
    baseURL: 'http://localhost:8080',
  });

  await client.login('admin', 'admin123');
  
  const users = await client.listUsers();
  console.log(`Total users: ${users.length}`);
}

main().catch(console.error);
```

## API Reference

### Constructor

```typescript
new LXProxyClient(config?: LXProxyClientConfig)
```

**Config options:**
- `baseURL` (optional) - API base URL (default: `'http://localhost:8080'`)
- `apiKey` (optional) - JWT token (can login later)
- `timeout` (optional) - Request timeout in ms (default: `30000`)

### Authentication

#### `login(username, password)`

Login and get JWT token.

```typescript
const { token, user } = await client.login('admin', 'admin123');
```

#### `logout()`

Logout and clear token.

```typescript
await client.logout();
```

#### `getCurrentUser()`

Get current user info.

```typescript
const user = await client.getCurrentUser();
```

### User Management

#### `listUsers()`

List all users.

```typescript
const users = await client.listUsers();
```

#### `createUser(data)`

Create new user.

```typescript
const user = await client.createUser({
  username: 'newuser',
  password: 'password123',
  role: 'user',
});
```

#### `getUser(userId)`

Get user by ID.

```typescript
const user = await client.getUser('user-id');
```

#### `updateUser(userId, data)`

Update user.

```typescript
await client.updateUser('user-id', {
  role: 'admin',
});
```

#### `deleteUser(userId)`

Delete user.

```typescript
await client.deleteUser('user-id');
```

### Inbound Management

#### `listInbounds()`

List all inbound configs.

```typescript
const inbounds = await client.listInbounds();
```

#### `createInbound(data)`

Create new inbound.

```typescript
const inbound = await client.createInbound({
  tag: 'vmess-1000',
  protocol: 'vmess',
  port: 1000,
  traffic_limit: 10737418240, // 10GB
});
```

#### `getInbound(inboundId)`

Get inbound by ID.

```typescript
const inbound = await client.getInbound('inbound-id');
```

#### `updateInbound(inboundId, data)`

Update inbound.

```typescript
await client.updateInbound('inbound-id', {
  enable: false,
  traffic_limit: 21474836480, // 20GB
});
```

#### `deleteInbound(inboundId)`

Delete inbound.

```typescript
await client.deleteInbound('inbound-id');
```

### System & Statistics

#### `getStats()`

Get system statistics.

```typescript
const stats = await client.getStats();
console.log(`Total users: ${stats.total_users}`);
console.log(`Total inbounds: ${stats.total_inbounds}`);
```

#### `getSystemStatus()`

Get real-time system status.

```typescript
const status = await client.getSystemStatus();
console.log(`CPU: ${status.cpu_usage}%`);
console.log(`Memory: ${status.memory_used} bytes`);
console.log(`Xray Running: ${status.xray_running}`);
```

## Error Handling

```typescript
import {
  LXProxyClient,
  AuthenticationError,
  APIError,
  NotFoundError,
  ValidationError,
} from '@lx-proxy/sdk';

const client = new LXProxyClient();

try {
  await client.login('admin', 'wrong-password');
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error(`Login failed: ${error.message}`);
  } else if (error instanceof NotFoundError) {
    console.error(`Resource not found: ${error.message}`);
  } else if (error instanceof APIError) {
    console.error(`API error: ${error.message} (Status: ${error.statusCode})`);
  } else if (error instanceof ValidationError) {
    console.error(`Validation errors: ${error.errors}`);
  } else {
    console.error(`Unexpected error: ${error}`);
  }
}
```

## Advanced Usage

### Using Existing Token

```typescript
const client = new LXProxyClient({
  baseURL: 'http://localhost:8080',
  apiKey: 'your-jwt-token-here',
});
```

### Manual Token Management

```typescript
const client = new LXProxyClient();

// Set token
client.setApiKey('new-token');

// Get token
const token = client.getApiKey();

// Clear token
client.clearApiKey();
```

### Custom Timeout

```typescript
const client = new LXProxyClient({
  baseURL: 'http://localhost:8080',
  timeout: 60000, // 60 seconds
});
```

## Development

### Setup

```bash
# Clone repository
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy/sdk/nodejs

# Install dependencies
npm install

# Build
npm run build

# Run tests
npm test

# Run with coverage
npm run test:coverage

# Lint
npm run lint

# Format
npm run format
```

### Project Structure

```
sdk/nodejs/
├── src/                   # Source code
│   ├── index.ts          # Entry point
│   ├── client.ts         # Main client class
│   ├── types.ts          # TypeScript types
│   └── errors.ts         # Error classes
├── test/                  # Tests
│   └── client.test.ts    # Client tests
├── examples/              # Examples
│   └── basic-usage.ts    # Basic usage example
├── package.json          # Package config
├── tsconfig.json         # TypeScript config
├── tsup.config.ts        # Build config
└── jest.config.js        # Jest config
```

## License

MIT License

## Links

- **npm:** https://www.npmjs.com/package/@lx-proxy/sdk
- **GitHub:** https://github.com/x64arm/lx-proxy/tree/main/sdk/nodejs
- **Documentation:** https://github.com/x64arm/lx-proxy
- **Issues:** https://github.com/x64arm/lx-proxy/issues

## Related Packages

- **Python SDK:** https://pypi.org/project/lx-proxy/
- **LX-Proxy Backend:** https://github.com/x64arm/lx-proxy
