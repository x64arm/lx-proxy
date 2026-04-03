/**
 * Example: Basic usage of LX-Proxy Node.js SDK
 */

import { LXProxyClient, AuthenticationError, APIError } from '../src/index';

async function exampleBasicUsage() {
  console.log('=== Basic Usage Example ===\n');

  // Create client
  const client = new LXProxyClient({
    baseURL: 'http://localhost:8080',
    timeout: 30000,
  });

  try {
    // Login
    console.log('Logging in...');
    const loginResult = await client.login('admin', 'admin123');
    console.log(`✅ Logged in as: ${loginResult.user.username}`);
    console.log(`   Role: ${loginResult.user.role}\n`);

    // Get current user
    console.log('Getting current user info...');
    const user = await client.getCurrentUser();
    console.log(`   User ID: ${user.id}`);
    console.log(`   Username: ${user.username}\n`);

    // List users
    console.log('Listing users...');
    const users = await client.listUsers();
    console.log(`   Total users: ${users.length}`);
    for (const u of users) {
      console.log(`   - ${u.username} (${u.role})`);
    }
    console.log();

    // Get system stats
    console.log('Getting system statistics...');
    const stats = await client.getStats();
    console.log(`   Total users: ${stats.total_users}`);
    console.log(`   Total inbounds: ${stats.total_inbounds}`);
    console.log(`   Enabled inbounds: ${stats.enabled_inbounds}`);
    console.log(`   Traffic used: ${stats.total_traffic_used.toLocaleString()} bytes\n`);

    // Get system status
    console.log('Getting system status...');
    const status = await client.getSystemStatus();
    console.log(`   CPU Usage: ${status.cpu_usage.toFixed(1)}%`);
    console.log(`   Memory: ${status.memory_used.toLocaleString()} / ${status.memory_total.toLocaleString()} bytes`);
    console.log(`   Xray Running: ${status.xray_running}`);
    console.log(`   Active Connections: ${status.connections}\n`);

    // List inbounds
    console.log('Listing inbounds...');
    const inbounds = await client.listInbounds();
    console.log(`   Total inbounds: ${inbounds.length}`);
    for (const inbound of inbounds) {
      const statusIcon = inbound.enable ? '✅' : '❌';
      console.log(`   ${statusIcon} ${inbound.tag} (Port: ${inbound.port}, Protocol: ${inbound.protocol})`);
    }
    console.log();

  } catch (error) {
    if (error instanceof AuthenticationError) {
      console.error(`❌ Authentication failed: ${error.message}`);
    } else if (error instanceof APIError) {
      console.error(`❌ API error: ${error.message} (Status: ${error.statusCode})`);
    } else {
      console.error(`❌ Unexpected error: ${error}`);
    }
  } finally {
    // Logout
    await client.logout();
    console.log('✅ Logged out\n');
  }
}

async function exampleUserManagement() {
  console.log('\n=== User Management Example ===\n');

  const client = new LXProxyClient('http://localhost:8080');

  try {
    await client.login('admin', 'admin123');

    // Create a new user
    console.log('Creating new user...');
    const newUser = await client.createUser({
      username: 'demo_user',
      password: 'demo_password_123',
      role: 'user',
    });
    console.log(`✅ Created user: ${newUser.username} (ID: ${newUser.id})`);

    // Get user
    console.log('\nGetting user details...');
    const user = await client.getUser(newUser.id);
    console.log(`   Username: ${user.username}`);
    console.log(`   Role: ${user.role}`);
    console.log(`   Created: ${user.created_at}`);

    // Update user
    console.log('\nUpdating user role...');
    const updated = await client.updateUser(newUser.id, { role: 'admin' });
    console.log(`✅ Updated role to: ${updated.role}`);

    // Delete user
    console.log('\nDeleting user...');
    await client.deleteUser(newUser.id);
    console.log(`✅ Deleted user: ${newUser.username}`);

  } catch (error) {
    if (error instanceof APIError) {
      console.error(`❌ Error: ${error.message} (Status: ${error.statusCode})`);
    } else {
      console.error(`❌ Error: ${error}`);
    }
  }
}

async function exampleInboundManagement() {
  console.log('\n=== Inbound Management Example ===\n');

  const client = new LXProxyClient('http://localhost:8080');

  try {
    await client.login('admin', 'admin123');

    // Create a new inbound
    console.log('Creating new inbound...');
    const inbound = await client.createInbound({
      tag: 'vmess-demo',
      protocol: 'vmess',
      port: 10086,
      traffic_limit: 10737418240, // 10GB
    });
    console.log(`✅ Created inbound: ${inbound.tag}`);
    console.log(`   ID: ${inbound.id}`);
    console.log(`   Port: ${inbound.port}`);
    console.log(`   Protocol: ${inbound.protocol}`);
    console.log(`   Traffic Limit: ${inbound.traffic_limit?.toLocaleString()} bytes`);

    // Get inbound
    console.log('\nGetting inbound details...');
    const retrieved = await client.getInbound(inbound.id);
    console.log(`   Tag: ${retrieved.tag}`);
    console.log(`   Enabled: ${retrieved.enable}`);

    // Update inbound
    console.log('\nUpdating inbound...');
    const updated = await client.updateInbound(inbound.id, {
      traffic_limit: 21474836480, // 20GB
    });
    console.log(`✅ Updated traffic limit to: ${updated.traffic_limit?.toLocaleString()} bytes`);

    // Delete inbound
    console.log('\nDeleting inbound...');
    await client.deleteInbound(inbound.id);
    console.log(`✅ Deleted inbound: ${inbound.tag}`);

  } catch (error) {
    if (error instanceof APIError) {
      console.error(`❌ Error: ${error.message} (Status: ${error.statusCode})`);
    } else {
      console.error(`❌ Error: ${error}`);
    }
  }
}

async function main() {
  console.log('LX-Proxy Node.js SDK - Examples\n');
  console.log('='.repeat(50));

  // Run examples
  await exampleBasicUsage();
  await exampleUserManagement();
  await exampleInboundManagement();

  console.log('\n' + '='.repeat(50));
  console.log('✅ All examples completed!\n');
}

// Run if executed directly
if (require.main === module) {
  main().catch(console.error);
}
