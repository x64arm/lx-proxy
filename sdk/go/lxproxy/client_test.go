package lxproxy

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestNewClient(t *testing.T) {
	t.Run("default config", func(t *testing.T) {
		client := NewClient(nil)
		if client == nil {
			t.Fatal("client should not be nil")
		}
		if client.config.BaseURL != "http://localhost:8080" {
			t.Errorf("expected default baseURL, got %s", client.config.BaseURL)
		}
	})

	t.Run("custom config", func(t *testing.T) {
		config := &Config{
			BaseURL: "http://test:9000",
			Timeout: 60 * time.Second,
		}
		client := NewClient(config)
		if client.config.BaseURL != "http://test:9000" {
			t.Errorf("expected custom baseURL, got %s", client.config.BaseURL)
		}
	})
}

func TestAPIKeyManagement(t *testing.T) {
	client := NewClient(nil)

	t.Run("set api key", func(t *testing.T) {
		client.SetAPIKey("test-token")
		if client.GetAPIKey() != "test-token" {
			t.Errorf("expected test-token, got %s", client.GetAPIKey())
		}
	})

	t.Run("clear api key", func(t *testing.T) {
		client.ClearAPIKey()
		if client.GetAPIKey() != "" {
			t.Errorf("expected empty api key, got %s", client.GetAPIKey())
		}
	})
}

func TestLogin(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Errorf("expected POST, got %s", r.Method)
		}
		if r.URL.Path != "/api/auth/login" {
			t.Errorf("expected /api/auth/login, got %s", r.URL.Path)
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"token":"test-jwt-token","user":{"id":"123","username":"admin","role":"admin"}}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	ctx := context.Background()

	resp, err := client.Login(ctx, "admin", "password123")
	if err != nil {
		t.Fatalf("login failed: %v", err)
	}

	if resp.Token != "test-jwt-token" {
		t.Errorf("expected test-jwt-token, got %s", resp.Token)
	}
	if resp.User.Username != "admin" {
		t.Errorf("expected admin, got %s", resp.User.Username)
	}

	// Check token was stored
	if client.GetAPIKey() != "test-jwt-token" {
		t.Errorf("expected token to be stored, got %s", client.GetAPIKey())
	}
}

func TestAuthenticationError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
		w.Write([]byte(`{"error":"Invalid credentials"}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	ctx := context.Background()

	_, err := client.Login(ctx, "admin", "wrong-password")
	if err == nil {
		t.Fatal("expected error, got nil")
	}

	if !IsAuthenticationError(err) {
		t.Errorf("expected AuthenticationError, got %T", err)
	}
}

func TestListUsers(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("Authorization") == "" {
			t.Error("expected Authorization header")
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`[{"id":"1","username":"admin","role":"admin","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}]`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	users, err := client.ListUsers(ctx)
	if err != nil {
		t.Fatalf("list users failed: %v", err)
	}

	if len(users) != 1 {
		t.Errorf("expected 1 user, got %d", len(users))
	}
	if users[0].Username != "admin" {
		t.Errorf("expected admin, got %s", users[0].Username)
	}
}

func TestCreateUser(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"id":"123","username":"newuser","role":"user","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	user, err := client.CreateUser(ctx, "newuser", "password123", "user")
	if err != nil {
		t.Fatalf("create user failed: %v", err)
	}

	if user.Username != "newuser" {
		t.Errorf("expected newuser, got %s", user.Username)
	}
}

func TestNotFoundError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte(`{"error":"User not found"}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	_, err := client.GetUser(ctx, "non-existent")
	if err == nil {
		t.Fatal("expected error, got nil")
	}

	if !IsNotFoundError(err) {
		t.Errorf("expected NotFoundError, got %T", err)
	}
}

func TestGetStats(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"total_users":10,"total_inbounds":25,"enabled_inbounds":20,"total_traffic_used":1073741824}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	stats, err := client.GetStats(ctx)
	if err != nil {
		t.Fatalf("get stats failed: %v", err)
	}

	if stats.TotalUsers != 10 {
		t.Errorf("expected 10 users, got %d", stats.TotalUsers)
	}
	if stats.TotalInbounds != 25 {
		t.Errorf("expected 25 inbounds, got %d", stats.TotalInbounds)
	}
}

func TestGetSystemStatus(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"cpu_usage":25.5,"memory_total":17179869184,"memory_used":8589934592,"memory_free":8589934592,"uptime":86400,"xray_running":true,"connections":150}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	status, err := client.GetSystemStatus(ctx)
	if err != nil {
		t.Fatalf("get system status failed: %v", err)
	}

	if status.CPUUsage != 25.5 {
		t.Errorf("expected CPU usage 25.5, got %f", status.CPUUsage)
	}
	if !status.XrayRunning {
		t.Error("expected xray to be running")
	}
	if status.Connections != 150 {
		t.Errorf("expected 150 connections, got %d", status.Connections)
	}
}

func TestListInbounds(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`[{"id":"1","tag":"vmess-1000","protocol":"vmess","port":1000,"enable":true,"traffic_used":0,"traffic_limit":10737418240,"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}]`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	inbounds, err := client.ListInbounds(ctx)
	if err != nil {
		t.Fatalf("list inbounds failed: %v", err)
	}

	if len(inbounds) != 1 {
		t.Errorf("expected 1 inbound, got %d", len(inbounds))
	}
	if inbounds[0].Tag != "vmess-1000" {
		t.Errorf("expected vmess-1000, got %s", inbounds[0].Tag)
	}
	if inbounds[0].Port != 1000 {
		t.Errorf("expected port 1000, got %d", inbounds[0].Port)
	}
}

func TestCreateInbound(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"id":"123","tag":"vmess-2000","protocol":"vmess","port":2000,"enable":true,"traffic_used":0,"traffic_limit":21474836480,"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}`))
	}))
	defer server.Close()

	client := NewClientWithBaseURL(server.URL)
	client.SetAPIKey("test-token")
	ctx := context.Background()

	trafficLimit := int64(21474836480)
	inbound, err := client.CreateInbound(ctx, &CreateInboundRequest{
		Tag:          "vmess-2000",
		Protocol:     "vmess",
		Port:         2000,
		TrafficLimit: &trafficLimit,
	})
	if err != nil {
		t.Fatalf("create inbound failed: %v", err)
	}

	if inbound.Tag != "vmess-2000" {
		t.Errorf("expected vmess-2000, got %s", inbound.Tag)
	}
	if inbound.Port != 2000 {
		t.Errorf("expected port 2000, got %d", inbound.Port)
	}
}

func TestErrorTypes(t *testing.T) {
	t.Run("authentication error", func(t *testing.T) {
		err := &AuthenticationError{Message: "test"}
		if !IsAuthenticationError(err) {
			t.Error("expected authentication error")
		}
	})

	t.Run("not found error", func(t *testing.T) {
		err := &NotFoundError{Message: "test"}
		if !IsNotFoundError(err) {
			t.Error("expected not found error")
		}
	})

	t.Run("validation error", func(t *testing.T) {
		err := &ValidationError{Message: "test"}
		if !IsValidationError(err) {
			t.Error("expected validation error")
		}
	})

	t.Run("api error", func(t *testing.T) {
		err := &APIError{StatusCode: 500, Message: "test"}
		if !IsAPIError(err) {
			t.Error("expected api error")
		}
	})
}
