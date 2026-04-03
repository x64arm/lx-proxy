// Package lxproxy provides a Go client library for LX-Proxy API
//
// Example usage:
//
//	client := lxproxy.NewClientWithBaseURL("http://localhost:8080")
//
//	// Login
//	resp, err := client.Login(context.Background(), "admin", "admin123")
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Printf("Logged in as: %s\n", resp.User.Username)
//
//	// List users
//	users, err := client.ListUsers(context.Background())
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Printf("Total users: %d\n", len(users))
//
//	// Create inbound
//	inbound, err := client.CreateInbound(context.Background(), &lxproxy.CreateInboundRequest{
//	    Tag:      "vmess-1000",
//	    Protocol: "vmess",
//	    Port:     1000,
//	})
//	if err != nil {
//	    log.Fatal(err)
//	}
//	fmt.Printf("Created inbound on port: %d\n", inbound.Port)
package lxproxy
