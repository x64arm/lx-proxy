"""
Tests for LX-Proxy Python SDK
"""

import pytest
from unittest.mock import AsyncMock, Mock, patch
import httpx

from lxproxy import LXProxyClient, AuthenticationError, APIError, NotFoundError
from lxproxy.client import User, InboundConfig, Stats, SystemStatus


@pytest.fixture
def mock_response():
    """Create mock HTTP response"""
    def _mock_response(status_code=200, json_data=None):
        response = Mock(spec=httpx.Response)
        response.status_code = status_code
        response.json.return_value = json_data or {}
        response.content = b'{}' if json_data is None else str(json_data).encode()
        response.url = "http://test/endpoint"
        return response
    return _mock_response


class TestUserModel:
    """Test User model"""
    
    def test_user_creation(self):
        user = User(
            id="123e4567-e89b-12d3-a456-426614174000",
            username="testuser",
            role="user",
            created_at="2024-01-01T00:00:00Z",
            updated_at="2024-01-01T00:00:00Z"
        )
        assert user.username == "testuser"
        assert user.role == "user"
        assert str(user.id) == "123e4567-e89b-12d3-a456-426614174000"


class TestInboundConfigModel:
    """Test InboundConfig model"""
    
    def test_inbound_creation(self):
        inbound = InboundConfig(
            id="123e4567-e89b-12d3-a456-426614174000",
            user_id=None,
            tag="vmess-1000",
            protocol="vmess",
            port=1000,
            enable=True,
            traffic_used=0,
            traffic_limit=10737418240,
            expire_at=None,
            created_at="2024-01-01T00:00:00Z",
            updated_at="2024-01-01T00:00:00Z"
        )
        assert inbound.tag == "vmess-1000"
        assert inbound.protocol == "vmess"
        assert inbound.port == 1000
        assert inbound.enable is True


class TestClientInitialization:
    """Test client initialization"""
    
    def test_default_init(self):
        client = LXProxyClient()
        assert client.base_url == "http://localhost:8080"
        assert client.api_key is None
        assert client.timeout == 30.0
    
    def test_custom_init(self):
        client = LXProxyClient(
            base_url="http://example.com:9000",
            api_key="test-token",
            timeout=60.0
        )
        assert client.base_url == "http://example.com:9000"
        assert client.api_key == "test-token"
        assert client.timeout == 60.0
    
    def test_base_url_trailing_slash_removal(self):
        client = LXProxyClient(base_url="http://example.com/")
        assert client.base_url == "http://example.com"


class TestAuthentication:
    """Test authentication methods"""
    
    @pytest.mark.asyncio
    async def test_async_login_success(self, mock_response):
        client = LXProxyClient()
        
        mock_data = {
            "token": "test-jwt-token",
            "user": {
                "id": "123",
                "username": "admin",
                "role": "admin"
            }
        }
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.post = AsyncMock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            result = await client.async_login("admin", "password123")
            
            assert result["token"] == "test-jwt-token"
            assert client.api_key == "test-jwt-token"
            mock_client.post.assert_called_once_with(
                "/api/auth/login",
                json={"username": "admin", "password": "password123"}
            )
    
    @pytest.mark.asyncio
    async def test_async_login_failure(self, mock_response):
        client = LXProxyClient()
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.post = AsyncMock(return_value=mock_response(401, {"error": "Invalid credentials"}))
            mock_get_client.return_value = mock_client
            
            with pytest.raises(AuthenticationError):
                await client.async_login("admin", "wrong-password")
    
    def test_sync_login_success(self, mock_response):
        client = LXProxyClient()
        
        mock_data = {
            "token": "test-jwt-token",
            "user": {
                "id": "123",
                "username": "admin",
                "role": "admin"
            }
        }
        
        with patch.object(client, '_get_sync_client') as mock_get_client:
            mock_client = Mock()
            mock_client.post = Mock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            result = client.login("admin", "password123")
            
            assert result["token"] == "test-jwt-token"
            assert client.api_key == "test-jwt-token"
    
    def test_headers_with_auth(self):
        client = LXProxyClient(api_key="test-token")
        headers = client._get_headers()
        
        assert headers["Authorization"] == "Bearer test-token"
        assert headers["Content-Type"] == "application/json"
    
    def test_headers_without_auth(self):
        client = LXProxyClient()
        headers = client._get_headers()
        
        assert "Authorization" not in headers
        assert headers["Content-Type"] == "application/json"


class TestUserManagement:
    """Test user management operations"""
    
    @pytest.mark.asyncio
    async def test_list_users(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        mock_data = [
            {
                "id": "123",
                "username": "admin",
                "role": "admin",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "456",
                "username": "user1",
                "role": "user",
                "created_at": "2024-01-02T00:00:00Z",
                "updated_at": "2024-01-02T00:00:00Z"
            }
        ]
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            users = await client.async_list_users()
            
            assert len(users) == 2
            assert users[0].username == "admin"
            assert users[1].role == "user"
            mock_client.get.assert_called_once_with("/api/users")
    
    @pytest.mark.asyncio
    async def test_create_user(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        mock_data = {
            "id": "789",
            "username": "newuser",
            "role": "user",
            "created_at": "2024-01-03T00:00:00Z",
            "updated_at": "2024-01-03T00:00:00Z"
        }
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.post = AsyncMock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            user = await client.async_create_user(
                username="newuser",
                password="password123",
                role="user"
            )
            
            assert user.username == "newuser"
            assert user.id == "789"
    
    @pytest.mark.asyncio
    async def test_get_user_not_found(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response(404, {"error": "User not found"}))
            mock_get_client.return_value = mock_client
            
            with pytest.raises(NotFoundError):
                await client.async_get_user("non-existent-id")


class TestInboundManagement:
    """Test inbound management operations"""
    
    @pytest.mark.asyncio
    async def test_list_inbounds(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        mock_data = [
            {
                "id": "123",
                "user_id": None,
                "tag": "vmess-1000",
                "protocol": "vmess",
                "port": 1000,
                "enable": True,
                "traffic_used": 0,
                "traffic_limit": 10737418240,
                "expire_at": None,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            inbounds = await client.async_list_inbounds()
            
            assert len(inbounds) == 1
            assert inbounds[0].tag == "vmess-1000"
            assert inbounds[0].protocol == "vmess"


class TestSystemStats:
    """Test system statistics operations"""
    
    @pytest.mark.asyncio
    async def test_get_stats(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        mock_data = {
            "total_users": 10,
            "total_inbounds": 25,
            "enabled_inbounds": 20,
            "total_traffic_used": 1073741824,
            "total_traffic_limit": 107374182400
        }
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            stats = await client.async_get_stats()
            
            assert stats.total_users == 10
            assert stats.total_inbounds == 25
            assert stats.total_traffic_used == 1073741824
    
    @pytest.mark.asyncio
    async def test_get_system_status(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        mock_data = {
            "cpu_usage": 25.5,
            "memory_total": 17179869184,
            "memory_used": 8589934592,
            "memory_free": 8589934592,
            "uptime": 86400,
            "xray_running": True,
            "connections": 150
        }
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response(200, mock_data))
            mock_get_client.return_value = mock_client
            
            status = await client.async_get_system_status()
            
            assert status.cpu_usage == 25.5
            assert status.xray_running is True
            assert status.connections == 150


class TestContextManager:
    """Test context manager functionality"""
    
    @pytest.mark.asyncio
    async def test_async_context_manager(self):
        client = LXProxyClient()
        
        async with client as c:
            assert c is client
            assert c._async_client is not None
        
        assert c._async_client.is_closed
    
    def test_sync_context_manager(self):
        client = LXProxyClient()
        
        with client as c:
            assert c is client
            assert c._sync_client is not None
        
        assert c._sync_client.is_closed


class TestErrorHandling:
    """Test error handling"""
    
    @pytest.mark.asyncio
    async def test_api_error_with_details(self, mock_response):
        client = LXProxyClient(api_key="test-token")
        
        error_data = {
            "error": "Validation failed",
            "details": ["Username is required"]
        }
        
        with patch.object(client, '_get_async_client') as mock_get_client:
            mock_client = AsyncMock()
            mock_client.get = AsyncMock(return_value=mock_response(400, error_data))
            mock_get_client.return_value = mock_client
            
            with pytest.raises(APIError) as exc_info:
                await client.async_list_users()
            
            assert exc_info.value.status_code == 400
            assert exc_info.value.response_data == error_data
    
    def test_exception_inheritance(self):
        from lxproxy.exceptions import LXProxyError
        
        error = AuthenticationError()
        assert isinstance(error, LXProxyError)
        assert error.status_code == 401


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
