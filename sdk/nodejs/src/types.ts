/**
 * LX-Proxy SDK Type Definitions
 */

/** User model */
export interface User {
  id: string;
  username: string;
  role: string;
  created_at: string;
  updated_at: string;
}

/** Create user request */
export interface CreateUserRequest {
  username: string;
  password: string;
  role?: string;
}

/** Update user request */
export interface UpdateUserRequest {
  username?: string;
  role?: string;
}

/** Inbound configuration model */
export interface InboundConfig {
  id: string;
  user_id: string | null;
  tag: string;
  protocol: string;
  port: number;
  settings: Record<string, unknown>;
  stream_settings?: Record<string, unknown>;
  sniffing?: Record<string, unknown>;
  enable: boolean;
  traffic_used: number;
  traffic_limit: number | null;
  expire_at: string | null;
  ip_limit: number | null;
  created_at: string;
  updated_at: string;
}

/** Create inbound request */
export interface CreateInboundRequest {
  user_id?: string | null;
  tag: string;
  protocol: string;
  port: number;
  settings?: Record<string, unknown>;
  stream_settings?: Record<string, unknown>;
  sniffing?: Record<string, unknown>;
  traffic_limit?: number | null;
  expire_at?: string | null;
  ip_limit?: number | null;
}

/** Update inbound request */
export interface UpdateInboundRequest {
  tag?: string;
  port?: number;
  settings?: Record<string, unknown>;
  stream_settings?: Record<string, unknown>;
  sniffing?: Record<string, unknown>;
  enable?: boolean;
  traffic_limit?: number | null;
  expire_at?: string | null;
  ip_limit?: number | null;
}

/** System statistics */
export interface Stats {
  total_users: number;
  total_inbounds: number;
  enabled_inbounds: number;
  total_traffic_used: number;
  total_traffic_limit: number | null;
}

/** System status */
export interface SystemStatus {
  cpu_usage: number;
  memory_total: number;
  memory_used: number;
  memory_free: number;
  uptime: number;
  xray_running: boolean;
  connections: number;
}

/** Login request */
export interface LoginRequest {
  username: string;
  password: string;
}

/** Login response */
export interface LoginResponse {
  token: string;
  user: {
    id: string;
    username: string;
    role: string;
  };
}

/** API Error */
export interface ApiError {
  message: string;
  status_code?: number;
  response_data?: Record<string, unknown>;
}
