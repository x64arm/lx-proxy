package lxproxy

import "time"

// User represents a user account
type User struct {
	ID        string    `json:"id"`
	Username  string    `json:"username"`
	Role      string    `json:"role"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

// InboundConfig represents an inbound configuration
type InboundConfig struct {
	ID             string                 `json:"id"`
	UserID         *string                `json:"user_id"`
	Tag            string                 `json:"tag"`
	Protocol       string                 `json:"protocol"`
	Port           int                    `json:"port"`
	Settings       map[string]interface{} `json:"settings"`
	StreamSettings map[string]interface{} `json:"stream_settings,omitempty"`
	Sniffing       map[string]interface{} `json:"sniffing,omitempty"`
	Enable         bool                   `json:"enable"`
	TrafficUsed    int64                  `json:"traffic_used"`
	TrafficLimit   *int64                 `json:"traffic_limit"`
	ExpireAt       *time.Time            `json:"expire_at"`
	IPLimit        *int                  `json:"ip_limit"`
	CreatedAt      time.Time             `json:"created_at"`
	UpdatedAt      time.Time             `json:"updated_at"`
}

// CreateInboundRequest represents a request to create an inbound
type CreateInboundRequest struct {
	UserID         *string                `json:"user_id,omitempty"`
	Tag            string                 `json:"tag"`
	Protocol       string                 `json:"protocol"`
	Port           int                    `json:"port"`
	Settings       map[string]interface{} `json:"settings,omitempty"`
	StreamSettings map[string]interface{} `json:"stream_settings,omitempty"`
	Sniffing       map[string]interface{} `json:"sniffing,omitempty"`
	TrafficLimit   *int64                 `json:"traffic_limit,omitempty"`
	ExpireAt       *time.Time            `json:"expire_at,omitempty"`
	IPLimit        *int                  `json:"ip_limit,omitempty"`
}

// UpdateInboundRequest represents a request to update an inbound
type UpdateInboundRequest struct {
	Tag            *string                `json:"tag,omitempty"`
	Port           *int                   `json:"port,omitempty"`
	Settings       map[string]interface{} `json:"settings,omitempty"`
	StreamSettings map[string]interface{} `json:"stream_settings,omitempty"`
	Sniffing       map[string]interface{} `json:"sniffing,omitempty"`
	Enable         *bool                  `json:"enable,omitempty"`
	TrafficLimit   *int64                 `json:"traffic_limit,omitempty"`
	ExpireAt       *time.Time            `json:"expire_at,omitempty"`
	IPLimit        *int                  `json:"ip_limit,omitempty"`
}

// Stats represents system statistics
type Stats struct {
	TotalUsers        int    `json:"total_users"`
	TotalInbounds     int    `json:"total_inbounds"`
	EnabledInbounds   int    `json:"enabled_inbounds"`
	TrafficUsed       int64  `json:"total_traffic_used"`
	TrafficLimit      *int64 `json:"total_traffic_limit"`
}

// SystemStatus represents real-time system status
type SystemStatus struct {
	CPUUsage    float32 `json:"cpu_usage"`
	MemoryTotal uint64  `json:"memory_total"`
	MemoryUsed  uint64  `json:"memory_used"`
	MemoryFree  uint64  `json:"memory_free"`
	Uptime      uint64  `json:"uptime"`
	XrayRunning bool    `json:"xray_running"`
	Connections uint64  `json:"connections"`
}
