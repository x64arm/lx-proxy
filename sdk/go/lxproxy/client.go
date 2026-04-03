// Package lxproxy provides a Go client library for LX-Proxy API
package lxproxy

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"time"
)

// Client configuration
type Config struct {
	BaseURL string
	APIKey  string
	Timeout time.Duration
}

// DefaultConfig returns a config with default values
func DefaultConfig() *Config {
	return &Config{
		BaseURL: "http://localhost:8080",
		Timeout: 30 * time.Second,
	}
}

// LXProxyClient is the main client for LX-Proxy API
type LXProxyClient struct {
	config  *Config
	client  *http.Client
	apiKey  string
}

// NewClient creates a new LX-Proxy client
func NewClient(config *Config) *LXProxyClient {
	if config == nil {
		config = DefaultConfig()
	}

	return &LXProxyClient{
		config: config,
		client: &http.Client{
			Timeout: config.Timeout,
		},
		apiKey: config.APIKey,
	}
}

// NewClientWithBaseURL creates a new client with just the base URL
func NewClientWithBaseURL(baseURL string) *LXProxyClient {
	config := DefaultConfig()
	config.BaseURL = baseURL
	return NewClient(config)
}

// SetAPIKey sets the API key (JWT token)
func (c *LXProxyClient) SetAPIKey(key string) {
	c.apiKey = key
}

// GetAPIKey returns the current API key
func (c *LXProxyClient) GetAPIKey() string {
	return c.apiKey
}

// ClearAPIKey clears the API key
func (c *LXProxyClient) ClearAPIKey() {
	c.apiKey = ""
}

// doRequest performs an HTTP request
func (c *LXProxyClient) doRequest(ctx context.Context, method, path string, body interface{}, result interface{}) error {
	var reqBody io.Reader
	if body != nil {
		jsonData, err := json.Marshal(body)
		if err != nil {
			return fmt.Errorf("marshal request body: %w", err)
		}
		reqBody = bytes.NewReader(jsonData)
	}

	url := c.config.BaseURL + path
	req, err := http.NewRequestWithContext(ctx, method, url, reqBody)
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json")
	if c.apiKey != "" {
		req.Header.Set("Authorization", "Bearer "+c.apiKey)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return fmt.Errorf("execute request: %w", err)
	}
	defer resp.Body.Close()

	// Handle errors
	if resp.StatusCode >= 400 {
		return c.handleError(resp)
	}

	// Parse response
	if result != nil {
		data, err := io.ReadAll(resp.Body)
		if err != nil {
			return fmt.Errorf("read response body: %w", err)
		}
		if len(data) > 0 {
			if err := json.Unmarshal(data, result); err != nil {
				return fmt.Errorf("unmarshal response: %w", err)
			}
		}
	}

	return nil
}

// handleError handles HTTP error responses
func (c *LXProxyClient) handleError(resp *http.Response) error {
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return &APIError{
			StatusCode: resp.StatusCode,
			Message:    fmt.Sprintf("request failed with status %d", resp.StatusCode),
		}
	}

	var errorResp map[string]interface{}
	var message string
	if err := json.Unmarshal(data, &errorResp); err == nil {
		if msg, ok := errorResp["error"].(string); ok {
			message = msg
		} else if msg, ok := errorResp["message"].(string); ok {
			message = msg
		}
	}

	if message == "" {
		message = fmt.Sprintf("request failed with status %d", resp.StatusCode)
	}

	switch resp.StatusCode {
	case http.StatusUnauthorized:
		return &AuthenticationError{Message: message}
	case http.StatusNotFound:
		return &NotFoundError{Message: message}
	case http.StatusBadRequest:
		return &ValidationError{Message: message}
	default:
		return &APIError{
			StatusCode: resp.StatusCode,
			Message:    message,
		}
	}
}

// ============== Authentication ==============

// LoginRequest represents a login request
type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

// LoginResponse represents a login response
type LoginResponse struct {
	Token string `json:"token"`
	User  struct {
		ID       string `json:"id"`
		Username string `json:"username"`
		Role     string `json:"role"`
	} `json:"user"`
}

// Login logs in to the API and stores the JWT token
func (c *LXProxyClient) Login(ctx context.Context, username, password string) (*LoginResponse, error) {
	req := &LoginRequest{
		Username: username,
		Password: password,
	}

	var resp LoginResponse
	if err := c.doRequest(ctx, http.MethodPost, "/api/auth/login", req, &resp); err != nil {
		return nil, err
	}

	// Store token
	c.apiKey = resp.Token
	return &resp, nil
}

// Logout logs out and clears the token
func (c *LXProxyClient) Logout(ctx context.Context) error {
	_, err := c.doRequest(ctx, http.MethodPost, "/api/auth/logout", nil, nil)
	if err != nil {
		return err
	}
	c.ClearAPIKey()
	return nil
}

// GetCurrentUser gets the current user info
func (c *LXProxyClient) GetCurrentUser(ctx context.Context) (*User, error) {
	var user User
	if err := c.doRequest(ctx, http.MethodGet, "/api/auth/me", nil, &user); err != nil {
		return nil, err
	}
	return &user, nil
}

// ============== User Management ==============

// ListUsers lists all users
func (c *LXProxyClient) ListUsers(ctx context.Context) ([]*User, error) {
	var users []*User
	if err := c.doRequest(ctx, http.MethodGet, "/api/users", nil, &users); err != nil {
		return nil, err
	}
	return users, nil
}

// CreateUser creates a new user
func (c *LXProxyClient) CreateUser(ctx context.Context, username, password string, role ...string) (*User, error) {
	req := map[string]interface{}{
		"username": username,
		"password": password,
	}
	if len(role) > 0 && role[0] != "" {
		req["role"] = role[0]
	}

	var user User
	if err := c.doRequest(ctx, http.MethodPost, "/api/users", req, &user); err != nil {
		return nil, err
	}
	return &user, nil
}

// GetUser gets a user by ID
func (c *LXProxyClient) GetUser(ctx context.Context, userID string) (*User, error) {
	var user User
	if err := c.doRequest(ctx, http.MethodGet, "/api/users/"+userID, nil, &user); err != nil {
		return nil, err
	}
	return &user, nil
}

// UpdateUser updates a user
func (c *LXProxyClient) UpdateUser(ctx context.Context, userID string, data map[string]interface{}) (*User, error) {
	var user User
	if err := c.doRequest(ctx, http.MethodPut, "/api/users/"+userID, data, &user); err != nil {
		return nil, err
	}
	return &user, nil
}

// DeleteUser deletes a user
func (c *LXProxyClient) DeleteUser(ctx context.Context, userID string) error {
	_, err := c.doRequest(ctx, http.MethodDelete, "/api/users/"+userID, nil, nil)
	return err
}

// ============== Inbound Management ==============

// ListInbounds lists all inbound configs
func (c *LXProxyClient) ListInbounds(ctx context.Context) ([]*InboundConfig, error) {
	var inbounds []*InboundConfig
	if err := c.doRequest(ctx, http.MethodGet, "/api/inbounds", nil, &inbounds); err != nil {
		return nil, err
	}
	return inbounds, nil
}

// CreateInbound creates a new inbound
func (c *LXProxyClient) CreateInbound(ctx context.Context, req *CreateInboundRequest) (*InboundConfig, error) {
	var inbound InboundConfig
	if err := c.doRequest(ctx, http.MethodPost, "/api/inbounds", req, &inbound); err != nil {
		return nil, err
	}
	return &inbound, nil
}

// GetInbound gets an inbound by ID
func (c *LXProxyClient) GetInbound(ctx context.Context, inboundID string) (*InboundConfig, error) {
	var inbound InboundConfig
	if err := c.doRequest(ctx, http.MethodGet, "/api/inbounds/"+inboundID, nil, &inbound); err != nil {
		return nil, err
	}
	return &inbound, nil
}

// UpdateInbound updates an inbound
func (c *LXProxyClient) UpdateInbound(ctx context.Context, inboundID string, data *UpdateInboundRequest) (*InboundConfig, error) {
	var inbound InboundConfig
	if err := c.doRequest(ctx, http.MethodPut, "/api/inbounds/"+inboundID, data, &inbound); err != nil {
		return nil, err
	}
	return &inbound, nil
}

// DeleteInbound deletes an inbound
func (c *LXProxyClient) DeleteInbound(ctx context.Context, inboundID string) error {
	_, err := c.doRequest(ctx, http.MethodDelete, "/api/inbounds/"+inboundID, nil, nil)
	return err
}

// ============== System & Statistics ==============

// GetStats gets system statistics
func (c *LXProxyClient) GetStats(ctx context.Context) (*Stats, error) {
	var stats Stats
	if err := c.doRequest(ctx, http.MethodGet, "/api/stats", nil, &stats); err != nil {
		return nil, err
	}
	return &stats, nil
}

// GetSystemStatus gets system status
func (c *LXProxyClient) GetSystemStatus(ctx context.Context) (*SystemStatus, error) {
	var status SystemStatus
	if err := c.doRequest(ctx, http.MethodGet, "/api/system/status", nil, &status); err != nil {
		return nil, err
	}
	return &status, nil
}
