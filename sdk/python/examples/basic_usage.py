"""
Example: Basic usage of LX-Proxy Python SDK
"""

import asyncio
from lxproxy import LXProxyClient, AuthenticationError, APIError


async def example_basic_usage():
    """Basic async usage example"""
    print("=== Basic Usage Example ===\n")
    
    async with LXProxyClient("http://localhost:8080") as client:
        try:
            # Login
            print("Logging in...")
            result = await client.async_login("admin", "admin123")
            print(f"✅ Logged in as: {result['user']['username']}")
            print(f"   Role: {result['user']['role']}\n")
            
            # Get current user
            print("Getting current user info...")
            user = await client.async_get_current_user()
            print(f"   User ID: {user.id}")
            print(f"   Username: {user.username}\n")
            
            # List users
            print("Listing users...")
            users = await client.async_list_users()
            print(f"   Total users: {len(users)}")
            for u in users:
                print(f"   - {u.username} ({u.role})")
            print()
            
            # Get system stats
            print("Getting system statistics...")
            stats = await client.async_get_stats()
            print(f"   Total users: {stats.total_users}")
            print(f"   Total inbounds: {stats.total_inbounds}")
            print(f"   Enabled inbounds: {stats.enabled_inbounds}")
            print(f"   Traffic used: {stats.total_traffic_used:,} bytes\n")
            
            # Get system status
            print("Getting system status...")
            status = await client.async_get_system_status()
            print(f"   CPU Usage: {status.cpu_usage:.1f}%")
            print(f"   Memory: {status.memory_used:,} / {status.memory_total:,} bytes")
            print(f"   Xray Running: {status.xray_running}")
            print(f"   Active Connections: {status.connections}\n")
            
            # List inbounds
            print("Listing inbounds...")
            inbounds = await client.async_list_inbounds()
            print(f"   Total inbounds: {len(inbounds)}")
            for inbound in inbounds:
                status_icon = "✅" if inbound.enable else "❌"
                print(f"   {status_icon} {inbound.tag} (Port: {inbound.port}, Protocol: {inbound.protocol})")
            print()
            
        except AuthenticationError as e:
            print(f"❌ Authentication failed: {e}")
        except APIError as e:
            print(f"❌ API error: {e} (Status: {e.status_code})")
        except Exception as e:
            print(f"❌ Unexpected error: {e}")


async def example_user_management():
    """User management example"""
    print("\n=== User Management Example ===\n")
    
    async with LXProxyClient("http://localhost:8080") as client:
        await client.async_login("admin", "admin123")
        
        # Create a new user
        print("Creating new user...")
        try:
            new_user = await client.async_create_user(
                username="demo_user",
                password="demo_password_123",
                role="user"
            )
            print(f"✅ Created user: {new_user.username} (ID: {new_user.id})")
            
            # Get user
            print("\nGetting user details...")
            user = await client.async_get_user(new_user.id)
            print(f"   Username: {user.username}")
            print(f"   Role: {user.role}")
            print(f"   Created: {user.created_at}")
            
            # Update user
            print("\nUpdating user role...")
            updated = await client.async_update_user(new_user.id, role="admin")
            print(f"✅ Updated role to: {updated.role}")
            
            # Delete user
            print("\nDeleting user...")
            await client.async_delete_user(new_user.id)
            print(f"✅ Deleted user: {new_user.username}")
            
        except APIError as e:
            print(f"❌ Error: {e} (Status: {e.status_code})")


async def example_inbound_management():
    """Inbound management example"""
    print("\n=== Inbound Management Example ===\n")
    
    async with LXProxyClient("http://localhost:8080") as client:
        await client.async_login("admin", "admin123")
        
        # Create a new inbound
        print("Creating new inbound...")
        try:
            inbound = await client.async_create_inbound(
                tag="vmess-demo",
                protocol="vmess",
                port=10086,
                traffic_limit=10737418240,  # 10GB
            )
            print(f"✅ Created inbound: {inbound.tag}")
            print(f"   ID: {inbound.id}")
            print(f"   Port: {inbound.port}")
            print(f"   Protocol: {inbound.protocol}")
            print(f"   Traffic Limit: {inbound.traffic_limit:,} bytes")
            
            # Get inbound
            print("\nGetting inbound details...")
            retrieved = await client.async_get_inbound(inbound.id)
            print(f"   Tag: {retrieved.tag}")
            print(f"   Enabled: {retrieved.enable}")
            
            # Update inbound
            print("\nUpdating inbound...")
            updated = await client.async_update_inbound(
                inbound.id,
                traffic_limit=21474836480  # 20GB
            )
            print(f"✅ Updated traffic limit to: {updated.traffic_limit:,} bytes")
            
            # Delete inbound
            print("\nDeleting inbound...")
            await client.async_delete_inbound(inbound.id)
            print(f"✅ Deleted inbound: {inbound.tag}")
            
        except APIError as e:
            print(f"❌ Error: {e} (Status: {e.status_code})")


def example_sync_usage():
    """Synchronous usage example"""
    print("\n=== Sync Usage Example ===\n")
    
    client = LXProxyClient("http://localhost:8080")
    
    try:
        # Login
        print("Logging in (sync)...")
        result = client.login("admin", "admin123")
        print(f"✅ Logged in as: {result['user']['username']}")
        
        # List users
        print("\nListing users (sync)...")
        users = client.list_users()
        print(f"   Total users: {len(users)}")
        
        # Get stats
        print("\nGetting stats (sync)...")
        stats = client.get_stats()
        print(f"   Total inbounds: {stats.total_inbounds}")
        
    except Exception as e:
        print(f"❌ Error: {e}")
    finally:
        client.logout()
        print("\n✅ Logged out")


async def main():
    """Run all examples"""
    print("LX-Proxy Python SDK - Examples\n")
    print("=" * 50)
    
    # Run async examples
    await example_basic_usage()
    await example_user_management()
    await example_inbound_management()
    
    # Run sync example
    example_sync_usage()
    
    print("\n" + "=" * 50)
    print("✅ All examples completed!\n")


if __name__ == "__main__":
    asyncio.run(main())
