# LX-Proxy Go SDK

Go client library for LX-Proxy API - Xray management panel

[![Go Reference](https://pkg.go.dev/badge/github.com/lx-proxy/sdk/go.svg)](https://pkg.go.dev/github.com/lx-proxy/sdk/go)
[![Go Report Card](https://goreportcard.com/badge/github.com/lx-proxy/sdk/go)](https://goreportcard.com/report/github.com/lx-proxy/sdk/go)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Installation

```bash
go get github.com/lx-proxy/sdk/go
```

## Quick Start

```go
package main

import (
    "context"
    "fmt"
    "log"

    "github.com/lx-proxy/sdk/go/lxproxy"
)

func main() {
    // Create client
    client := lxproxy.NewClientWithBaseURL("http://localhost:8080")
    ctx := context.Background()

    // Login
    resp, err := client.Login(ctx, "admin", "admin123")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Logged in as: %s\n", resp.User.Username)

    // List users
    users, err := client.ListUsers(ctx)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Total users: %d\n", len(users))

    // Create inbound
    trafficLimit := int64(10737418240) // 10GB
    inbound, err := client.CreateInbound(ctx, &lxproxy.CreateInboundRequest{
        Tag:          "vmess-1000",
        Protocol:     "vmess",
        Port:         1000,
        TrafficLimit: &trafficLimit,
    })
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Created inbound on port: %d\n", inbound.Port)

    // Get stats
    stats, err := client.GetStats(ctx)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Total traffic used: %d bytes\n", stats.TrafficUsed)

    // Logout
    client.Logout(ctx)
}
```

## API Reference

### Client Creation

```go
// With default config
client := lxproxy.NewClient(nil)

// With custom config
config := &lxproxy.Config{
    BaseURL: "http://localhost:8080",
    Timeout: 60 * time.Second,
}
client := lxproxy.NewClient(config)

// Quick creation with base URL
client := lxproxy.NewClientWithBaseURL("http://localhost:8080")
```

### Authentication

#### Login

```go
resp, err := client.Login(ctx, "admin", "admin123")
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Token: %s\n", resp.Token)
```

#### Logout

```go
err := client.Logout(ctx)
if err != nil {
    log.Fatal(err)
}
```

#### Get Current User

```go
user, err := client.GetCurrentUser(ctx)
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Username: %s\n", user.Username)
```

### User Management

#### List Users

```go
users, err := client.ListUsers(ctx)
if err != nil {
    log.Fatal(err)
}
for _, user := range users {
    fmt.Printf("- %s (%s)\n", user.Username, user.Role)
}
```

#### Create User

```go
user, err := client.CreateUser(ctx, "newuser", "password123", "user")
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Created user: %s\n", user.Username)
```

#### Get User

```go
user, err := client.GetUser(ctx, "user-id")
if err != nil {
    log.Fatal(err)
}
```

#### Update User

```go
updated, err := client.UpdateUser(ctx, "user-id", map[string]interface{}{
    "role": "admin",
})
if err != nil {
    log.Fatal(err)
}
```

#### Delete User

```go
err := client.DeleteUser(ctx, "user-id")
if err != nil {
    log.Fatal(err)
}
```

### Inbound Management

#### List Inbounds

```go
inbounds, err := client.ListInbounds(ctx)
if err != nil {
    log.Fatal(err)
}
for _, inbound := range inbounds {
    fmt.Printf("- %s (Port: %d)\n", inbound.Tag, inbound.Port)
}
```

#### Create Inbound

```go
trafficLimit := int64(10737418240) // 10GB
inbound, err := client.CreateInbound(ctx, &lxproxy.CreateInboundRequest{
    Tag:          "vmess-1000",
    Protocol:     "vmess",
    Port:         1000,
    TrafficLimit: &trafficLimit,
})
if err != nil {
    log.Fatal(err)
}
```

#### Update Inbound

```go
newLimit := int64(21474836480) // 20GB
updated, err := client.UpdateInbound(ctx, "inbound-id", &lxproxy.UpdateInboundRequest{
    TrafficLimit: &newLimit,
})
if err != nil {
    log.Fatal(err)
}
```

#### Delete Inbound

```go
err := client.DeleteInbound(ctx, "inbound-id")
if err != nil {
    log.Fatal(err)
}
```

### System & Statistics

#### Get Stats

```go
stats, err := client.GetStats(ctx)
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Total users: %d\n", stats.TotalUsers)
fmt.Printf("Total inbounds: %d\n", stats.TotalInbounds)
```

#### Get System Status

```go
status, err := client.GetSystemStatus(ctx)
if err != nil {
    log.Fatal(err)
}
fmt.Printf("CPU: %.1f%%\n", status.CPUUsage)
fmt.Printf("Memory: %d bytes\n", status.MemoryUsed)
fmt.Printf("Xray Running: %v\n", status.XrayRunning)
```

## Error Handling

```go
import "github.com/lx-proxy/sdk/go/lxproxy"

resp, err := client.Login(ctx, "admin", "wrong-password")
if err != nil {
    if lxproxy.IsAuthenticationError(err) {
        log.Printf("Authentication failed: %v", err)
    } else if lxproxy.IsNotFoundError(err) {
        log.Printf("Resource not found: %v", err)
    } else if lxproxy.IsValidationError(err) {
        log.Printf("Validation error: %v", err)
    } else if lxproxy.IsAPIError(err) {
        apiErr := err.(*lxproxy.APIError)
        log.Printf("API error %d: %v", apiErr.StatusCode, apiErr.Message)
    } else {
        log.Printf("Unknown error: %v", err)
    }
}
```

## Advanced Usage

### Custom HTTP Client

```go
config := lxproxy.DefaultConfig()
config.BaseURL = "http://localhost:8080"
config.Timeout = 60 * time.Second

client := lxproxy.NewClient(config)
```

### Manual Token Management

```go
client := lxproxy.NewClientWithBaseURL("http://localhost:8080")

// Set token
client.SetAPIKey("your-jwt-token")

// Get token
token := client.GetAPIKey()

// Clear token
client.ClearAPIKey()
```

### Context with Timeout

```go
ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
defer cancel()

users, err := client.ListUsers(ctx)
if err != nil {
    log.Fatal(err)
}
```

## Development

### Setup

```bash
# Clone repository
git clone https://github.com/x64arm/lx-proxy.git
cd lx-proxy/sdk/go

# Install dependencies
go mod tidy

# Run tests
go test -v ./...

# Run tests with coverage
go test -v -cover ./...

# Build
go build ./...
```

### Project Structure

```
sdk/go/
├── lxproxy/                 # SDK source code
│   ├── lxproxy.go          # Package documentation
│   ├── client.go           # Main client
│   ├── types.go            # Type definitions
│   ├── errors.go           # Error types
│   └── client_test.go      # Tests
├── examples/                # Examples
│   └── main.go             # Basic usage example
├── go.mod                  # Go module
└── README.md               # This file
```

### Running Examples

```bash
cd examples
go run main.go
```

## Requirements

- Go 1.21 or later

## License

MIT License

## Links

- **Go Module:** https://pkg.go.dev/github.com/lx-proxy/sdk/go
- **GitHub:** https://github.com/x64arm/lx-proxy/tree/main/sdk/go
- **Documentation:** https://github.com/x64arm/lx-proxy
- **Issues:** https://github.com/x64arm/lx-proxy/issues

## Related Packages

- **Python SDK:** https://pypi.org/project/lx-proxy/
- **Node.js SDK:** https://www.npmjs.com/package/@lx-proxy/sdk
- **LX-Proxy Backend:** https://github.com/x64arm/lx-proxy
