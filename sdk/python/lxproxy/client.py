"""
LX-Proxy API Client

Main client class for interacting with LX-Proxy API
"""

from __future__ import annotations

import asyncio
from typing import Any
from datetime import datetime

import httpx
from pydantic import BaseModel, Field

from .exceptions import (
    LXProxyError,
    AuthenticationError,
    APIError,
    NotFoundError,
    ValidationError,
)


# ============== Models ==============

class User(BaseModel):
    """User model"""
    id: str
    username: str
    role: str
    created_at: datetime
    updated_at: datetime


class CreateUserRequest(BaseModel):
    """Request to create a user"""
    username: str
    password: str
    role: str | None = "user"


class InboundConfig(BaseModel):
    """Inbound configuration model"""
    id: str
    user_id: str | None
    tag: str
    protocol: str
    port: int
    enable: bool
    traffic_used: int
    traffic_limit: int | None
    expire_at: datetime | None
    created_at: datetime
    updated_at: datetime


class CreateInboundRequest(BaseModel):
    """Request to create an inbound"""
    user_id: str | None = None
    tag: str
    protocol: str
    port: int
    settings: dict = Field(default_factory=dict)
    traffic_limit: int | None = None
    expire_at: datetime | None = None


class SystemStatus(BaseModel):
    """System status model"""
    cpu_usage: float
    memory_total: int
    memory_used: int
    memory_free: int
    uptime: int
    xray_running: bool
    connections: int


class Stats(BaseModel):
    """Statistics model"""
    total_users: int
    total_inbounds: int
    enabled_inbounds: int
    total_traffic_used: int
    total_traffic_limit: int | None


# ============== Client ==============

class LXProxyClient:
    """
    LX-Proxy API Client
    
    Supports both synchronous and asynchronous operations.
    
    Example:
        ```python
        # Async usage
        async with LXProxyClient("http://localhost:8080") as client:
            await client.login("admin", "admin123")
            users = await client.list_users()
            
        # Sync usage
        client = LXProxyClient("http://localhost:8080")
        client.login("admin", "admin123")
        users = client.list_users()
        ```
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        api_key: str | None = None,
        timeout: float = 30.0,
    ):
        """
        Initialize LX-Proxy client
        
        Args:
            base_url: LX-Proxy API base URL
            api_key: API key or JWT token (optional, can login later)
            timeout: Request timeout in seconds
        """
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout
        self._async_client: httpx.AsyncClient | None = None
        self._sync_client: httpx.Client | None = None

    def _get_headers(self) -> dict[str, str]:
        """Get request headers with authentication"""
        headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
        }
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers

    async def _get_async_client(self) -> httpx.AsyncClient:
        """Get or create async HTTP client"""
        if self._async_client is None or self._async_client.is_closed:
            self._async_client = httpx.AsyncClient(
                base_url=self.base_url,
                timeout=self.timeout,
                headers=self._get_headers(),
            )
        return self._async_client

    def _get_sync_client(self) -> httpx.Client:
        """Get or create sync HTTP client"""
        if self._sync_client is None or self._sync_client.is_closed:
            self._sync_client = httpx.Client(
                base_url=self.base_url,
                timeout=self.timeout,
                headers=self._get_headers(),
            )
        return self._sync_client

    async def _handle_response(self, response: httpx.Response) -> dict[str, Any]:
        """Handle HTTP response and raise appropriate exceptions"""
        if response.status_code == 401:
            raise AuthenticationError("Authentication failed. Please login again.")
        elif response.status_code == 404:
            raise NotFoundError(f"Resource not found: {response.url}")
        elif response.status_code >= 400:
            try:
                error_data = response.json()
                message = error_data.get("error", error_data.get("message", "API request failed"))
            except Exception:
                message = f"API request failed with status {response.status_code}"
            raise APIError(message, status_code=response.status_code, response_data=error_data)
        
        return response.json() if response.content else {}

    # ============== Authentication ==============

    async def async_login(self, username: str, password: str) -> dict[str, Any]:
        """
        Login to LX-Proxy API
        
        Args:
            username: Username
            password: Password
            
        Returns:
            dict with token and user info
        """
        client = await self._get_async_client()
        response = await client.post(
            "/api/auth/login",
            json={"username": username, "password": password},
        )
        data = await self._handle_response(response)
        
        # Store token for subsequent requests
        if "token" in data:
            self.api_key = data["token"]
            # Update headers for existing client
            self._async_client.headers["Authorization"] = f"Bearer {self.api_key}"
        
        return data

    def login(self, username: str, password: str) -> dict[str, Any]:
        """
        Login to LX-Proxy API (synchronous)
        
        Args:
            username: Username
            password: Password
            
        Returns:
            dict with token and user info
        """
        client = self._get_sync_client()
        response = client.post(
            "/api/auth/login",
            json={"username": username, "password": password},
        )
        data = self._handle_response_sync(response)
        
        # Store token for subsequent requests
        if "token" in data:
            self.api_key = data["token"]
            # Update headers for existing client
            self._sync_client.headers["Authorization"] = f"Bearer {self.api_key}"
        
        return data

    def _handle_response_sync(self, response: httpx.Response) -> dict[str, Any]:
        """Handle sync HTTP response"""
        if response.status_code == 401:
            raise AuthenticationError("Authentication failed. Please login again.")
        elif response.status_code == 404:
            raise NotFoundError(f"Resource not found: {response.url}")
        elif response.status_code >= 400:
            try:
                error_data = response.json()
                message = error_data.get("error", error_data.get("message", "API request failed"))
            except Exception:
                message = f"API request failed with status {response.status_code}"
            raise APIError(message, status_code=response.status_code, response_data=error_data)
        
        return response.json() if response.content else {}

    async def async_logout(self) -> None:
        """Logout from LX-Proxy API"""
        client = await self._get_async_client()
        await client.post("/api/auth/logout")
        self.api_key = None

    def logout(self) -> None:
        """Logout from LX-Proxy API (synchronous)"""
        client = self._get_sync_client()
        client.post("/api/auth/logout")
        self.api_key = None

    async def async_get_current_user(self) -> User:
        """Get current user info"""
        client = await self._get_async_client()
        response = await client.get("/api/auth/me")
        data = await self._handle_response(response)
        return User(**data)

    def get_current_user(self) -> User:
        """Get current user info (synchronous)"""
        client = self._get_sync_client()
        response = client.get("/api/auth/me")
        data = self._handle_response_sync(response)
        return User(**data)

    # ============== User Management ==============

    async def async_list_users(self) -> list[User]:
        """List all users"""
        client = await self._get_async_client()
        response = await client.get("/api/users")
        data = await self._handle_response(response)
        return [User(**user) for user in data]

    def list_users(self) -> list[User]:
        """List all users (synchronous)"""
        client = self._get_sync_client()
        response = client.get("/api/users")
        data = self._handle_response_sync(response)
        return [User(**user) for user in data]

    async def async_create_user(self, username: str, password: str, role: str | None = None) -> User:
        """Create a new user"""
        client = await self._get_async_client()
        request = CreateUserRequest(username=username, password=password, role=role)
        response = await client.post("/api/users", json=request.dict(exclude_none=True))
        data = await self._handle_response(response)
        return User(**data)

    def create_user(self, username: str, password: str, role: str | None = None) -> User:
        """Create a new user (synchronous)"""
        client = self._get_sync_client()
        request = CreateUserRequest(username=username, password=password, role=role)
        response = client.post("/api/users", json=request.dict(exclude_none=True))
        data = self._handle_response_sync(response)
        return User(**data)

    async def async_get_user(self, user_id: str) -> User:
        """Get user by ID"""
        client = await self._get_async_client()
        response = await client.get(f"/api/users/{user_id}")
        data = await self._handle_response(response)
        return User(**data)

    def get_user(self, user_id: str) -> User:
        """Get user by ID (synchronous)"""
        client = self._get_sync_client()
        response = client.get(f"/api/users/{user_id}")
        data = self._handle_response_sync(response)
        return User(**data)

    async def async_update_user(self, user_id: str, **kwargs) -> User:
        """Update user"""
        client = await self._get_async_client()
        response = await client.put(f"/api/users/{user_id}", json=kwargs)
        data = await self._handle_response(response)
        return User(**data)

    def update_user(self, user_id: str, **kwargs) -> User:
        """Update user (synchronous)"""
        client = self._get_sync_client()
        response = client.put(f"/api/users/{user_id}", json=kwargs)
        data = self._handle_response_sync(response)
        return User(**data)

    async def async_delete_user(self, user_id: str) -> None:
        """Delete user"""
        client = await self._get_async_client()
        await client.delete(f"/api/users/{user_id}")

    def delete_user(self, user_id: str) -> None:
        """Delete user (synchronous)"""
        client = self._get_sync_client()
        client.delete(f"/api/users/{user_id}")

    # ============== Inbound Management ==============

    async def async_list_inbounds(self) -> list[InboundConfig]:
        """List all inbound configs"""
        client = await self._get_async_client()
        response = await client.get("/api/inbounds")
        data = await self._handle_response(response)
        return [InboundConfig(**inbound) for inbound in data]

    def list_inbounds(self) -> list[InboundConfig]:
        """List all inbound configs (synchronous)"""
        client = self._get_sync_client()
        response = client.get("/api/inbounds")
        data = self._handle_response_sync(response)
        return [InboundConfig(**inbound) for inbound in data]

    async def async_create_inbound(self, **kwargs) -> InboundConfig:
        """Create new inbound"""
        client = await self._get_async_client()
        request = CreateInboundRequest(**kwargs)
        response = await client.post("/api/inbounds", json=request.dict(exclude_none=True))
        data = await self._handle_response(response)
        return InboundConfig(**data)

    def create_inbound(self, **kwargs) -> InboundConfig:
        """Create new inbound (synchronous)"""
        client = self._get_sync_client()
        request = CreateInboundRequest(**kwargs)
        response = client.post("/api/inbounds", json=request.dict(exclude_none=True))
        data = self._handle_response_sync(response)
        return InboundConfig(**data)

    async def async_get_inbound(self, inbound_id: str) -> InboundConfig:
        """Get inbound by ID"""
        client = await self._get_async_client()
        response = await client.get(f"/api/inbounds/{inbound_id}")
        data = await self._handle_response(response)
        return InboundConfig(**data)

    def get_inbound(self, inbound_id: str) -> InboundConfig:
        """Get inbound by ID (synchronous)"""
        client = self._get_sync_client()
        response = client.get(f"/api/inbounds/{inbound_id}")
        data = self._handle_response_sync(response)
        return InboundConfig(**data)

    async def async_update_inbound(self, inbound_id: str, **kwargs) -> InboundConfig:
        """Update inbound"""
        client = await self._get_async_client()
        response = await client.put(f"/api/inbounds/{inbound_id}", json=kwargs)
        data = await self._handle_response(response)
        return InboundConfig(**data)

    def update_inbound(self, inbound_id: str, **kwargs) -> InboundConfig:
        """Update inbound (synchronous)"""
        client = self._get_sync_client()
        response = client.put(f"/api/inbounds/{inbound_id}", json=kwargs)
        data = self._handle_response_sync(response)
        return InboundConfig(**data)

    async def async_delete_inbound(self, inbound_id: str) -> None:
        """Delete inbound"""
        client = await self._get_async_client()
        await client.delete(f"/api/inbounds/{inbound_id}")

    def delete_inbound(self, inbound_id: str) -> None:
        """Delete inbound (synchronous)"""
        client = self._get_sync_client()
        client.delete(f"/api/inbounds/{inbound_id}")

    # ============== System & Stats ==============

    async def async_get_stats(self) -> Stats:
        """Get system statistics"""
        client = await self._get_async_client()
        response = await client.get("/api/stats")
        data = await self._handle_response(response)
        return Stats(**data)

    def get_stats(self) -> Stats:
        """Get system statistics (synchronous)"""
        client = self._get_sync_client()
        response = client.get("/api/stats")
        data = self._handle_response_sync(response)
        return Stats(**data)

    async def async_get_system_status(self) -> SystemStatus:
        """Get system status"""
        client = await self._get_async_client()
        response = await client.get("/api/system/status")
        data = await self._handle_response(response)
        return SystemStatus(**data)

    def get_system_status(self) -> SystemStatus:
        """Get system status (synchronous)"""
        client = self._get_sync_client()
        response = client.get("/api/system/status")
        data = self._handle_response_sync(response)
        return SystemStatus(**data)

    # ============== Context Manager ==============

    async def __aenter__(self) -> LXProxyClient:
        """Async context manager entry"""
        await self._get_async_client()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Async context manager exit"""
        if self._async_client:
            await self._async_client.aclose()

    def __enter__(self) -> LXProxyClient:
        """Sync context manager entry"""
        self._get_sync_client()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """Sync context manager exit"""
        if self._sync_client:
            self._sync_client.close()
