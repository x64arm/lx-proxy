package main

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/lx-proxy/sdk/go/lxproxy"
)

func main() {
	fmt.Println("LX-Proxy Go SDK - Examples")
	fmt.Println("========================================")

	// Create client
	client := lxproxy.NewClientWithBaseURL("http://localhost:8080")
	ctx := context.Background()

	// Example 1: Basic usage
	fmt.Println("\n=== Basic Usage Example ===\n")
	if err := exampleBasicUsage(client, ctx); err != nil {
		log.Printf("❌ Basic usage error: %v\n", err)
	}

	// Example 2: User management
	fmt.Println("\n=== User Management Example ===\n")
	if err := exampleUserManagement(client, ctx); err != nil {
		log.Printf("❌ User management error: %v\n", err)
	}

	// Example 3: Inbound management
	fmt.Println("\n=== Inbound Management Example ===\n")
	if err := exampleInboundManagement(client, ctx); err != nil {
		log.Printf("❌ Inbound management error: %v\n", err)
	}

	fmt.Println("\n========================================")
	fmt.Println("✅ All examples completed!")
}

func exampleBasicUsage(client *lxproxy.LXProxyClient, ctx context.Context) error {
	// Login
	fmt.Println("Logging in...")
	resp, err := client.Login(ctx, "admin", "admin123")
	if err != nil {
		return fmt.Errorf("login failed: %w", err)
	}
	fmt.Printf("✅ Logged in as: %s (Role: %s)\n\n", resp.User.Username, resp.User.Role)

	// Get current user
	fmt.Println("Getting current user...")
	user, err := client.GetCurrentUser(ctx)
	if err != nil {
		return fmt.Errorf("get current user: %w", err)
	}
	fmt.Printf("   User ID: %s\n", user.ID)
	fmt.Printf("   Username: %s\n\n", user.Username)

	// List users
	fmt.Println("Listing users...")
	users, err := client.ListUsers(ctx)
	if err != nil {
		return fmt.Errorf("list users: %w", err)
	}
	fmt.Printf("   Total users: %d\n", len(users))
	for _, u := range users {
		fmt.Printf("   - %s (%s)\n", u.Username, u.Role)
	}
	fmt.Println()

	// Get stats
	fmt.Println("Getting system statistics...")
	stats, err := client.GetStats(ctx)
	if err != nil {
		return fmt.Errorf("get stats: %w", err)
	}
	fmt.Printf("   Total users: %d\n", stats.TotalUsers)
	fmt.Printf("   Total inbounds: %d\n", stats.TotalInbounds)
	fmt.Printf("   Enabled inbounds: %d\n", stats.EnabledInbounds)
	fmt.Printf("   Traffic used: %d bytes\n\n", stats.TrafficUsed)

	// Get system status
	fmt.Println("Getting system status...")
	status, err := client.GetSystemStatus(ctx)
	if err != nil {
		return fmt.Errorf("get system status: %w", err)
	}
	fmt.Printf("   CPU Usage: %.1f%%\n", status.CPUUsage)
	fmt.Printf("   Memory: %d / %d bytes\n", status.MemoryUsed, status.MemoryTotal)
	fmt.Printf("   Xray Running: %v\n", status.XrayRunning)
	fmt.Printf("   Connections: %d\n\n", status.Connections)

	// List inbounds
	fmt.Println("Listing inbounds...")
	inbounds, err := client.ListInbounds(ctx)
	if err != nil {
		return fmt.Errorf("list inbounds: %w", err)
	}
	fmt.Printf("   Total inbounds: %d\n", len(inbounds))
	for _, inbound := range inbounds {
		statusIcon := "✅"
		if !inbound.Enable {
			statusIcon = "❌"
		}
		fmt.Printf("   %s %s (Port: %d, Protocol: %s)\n", statusIcon, inbound.Tag, inbound.Port, inbound.Protocol)
	}
	fmt.Println()

	// Logout
	fmt.Println("Logging out...")
	if err := client.Logout(ctx); err != nil {
		return fmt.Errorf("logout: %w", err)
	}
	fmt.Println("✅ Logged out")

	return nil
}

func exampleUserManagement(client *lxproxy.LXProxyClient, ctx context.Context) error {
	// Login
	if _, err := client.Login(ctx, "admin", "admin123"); err != nil {
		return fmt.Errorf("login: %w", err)
	}
	defer client.Logout(ctx)

	// Create user
	fmt.Println("Creating new user...")
	newUser, err := client.CreateUser(ctx, "go_demo_user", "demo_password_123", "user")
	if err != nil {
		return fmt.Errorf("create user: %w", err)
	}
	fmt.Printf("✅ Created user: %s (ID: %s)\n\n", newUser.Username, newUser.ID)

	// Get user
	fmt.Println("Getting user details...")
	user, err := client.GetUser(ctx, newUser.ID)
	if err != nil {
		return fmt.Errorf("get user: %w", err)
	}
	fmt.Printf("   Username: %s\n", user.Username)
	fmt.Printf("   Role: %s\n", user.Role)
	fmt.Printf("   Created: %s\n\n", user.CreatedAt.Format(time.RFC3339))

	// Update user
	fmt.Println("Updating user role...")
	updated, err := client.UpdateUser(ctx, newUser.ID, map[string]interface{}{
		"role": "admin",
	})
	if err != nil {
		return fmt.Errorf("update user: %w", err)
	}
	fmt.Printf("✅ Updated role to: %s\n\n", updated.Role)

	// Delete user
	fmt.Println("Deleting user...")
	if err := client.DeleteUser(ctx, newUser.ID); err != nil {
		return fmt.Errorf("delete user: %w", err)
	}
	fmt.Printf("✅ Deleted user: %s\n", newUser.Username)

	return nil
}

func exampleInboundManagement(client *lxproxy.LXProxyClient, ctx context.Context) error {
	// Login
	if _, err := client.Login(ctx, "admin", "admin123"); err != nil {
		return fmt.Errorf("login: %w", err)
	}
	defer client.Logout(ctx)

	// Create inbound
	fmt.Println("Creating new inbound...")
	trafficLimit := int64(10737418240) // 10GB
	inbound, err := client.CreateInbound(ctx, &lxproxy.CreateInboundRequest{
		Tag:          "go-vmess-demo",
		Protocol:     "vmess",
		Port:         10086,
		TrafficLimit: &trafficLimit,
	})
	if err != nil {
		return fmt.Errorf("create inbound: %w", err)
	}
	fmt.Printf("✅ Created inbound: %s\n", inbound.Tag)
	fmt.Printf("   ID: %s\n", inbound.ID)
	fmt.Printf("   Port: %d\n", inbound.Port)
	fmt.Printf("   Protocol: %s\n", inbound.Protocol)
	if inbound.TrafficLimit != nil {
		fmt.Printf("   Traffic Limit: %d bytes\n\n", *inbound.TrafficLimit)
	}

	// Get inbound
	fmt.Println("Getting inbound details...")
	retrieved, err := client.GetInbound(ctx, inbound.ID)
	if err != nil {
		return fmt.Errorf("get inbound: %w", err)
	}
	fmt.Printf("   Tag: %s\n", retrieved.Tag)
	fmt.Printf("   Enabled: %v\n\n", retrieved.Enable)

	// Update inbound
	fmt.Println("Updating inbound...")
	newLimit := int64(21474836480) // 20GB
	updated, err := client.UpdateInbound(ctx, inbound.ID, &lxproxy.UpdateInboundRequest{
		TrafficLimit: &newLimit,
	})
	if err != nil {
		return fmt.Errorf("update inbound: %w", err)
	}
	if updated.TrafficLimit != nil {
		fmt.Printf("✅ Updated traffic limit to: %d bytes\n\n", *updated.TrafficLimit)
	}

	// Delete inbound
	fmt.Println("Deleting inbound...")
	if err := client.DeleteInbound(ctx, inbound.ID); err != nil {
		return fmt.Errorf("delete inbound: %w", err)
	}
	fmt.Printf("✅ Deleted inbound: %s\n", inbound.Tag)

	return nil
}
