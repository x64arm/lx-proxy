"""
LX-Proxy SDK Exceptions
"""


class LXProxyError(Exception):
    """Base exception for LX-Proxy SDK"""

    def __init__(self, message: str, status_code: int | None = None):
        self.message = message
        self.status_code = status_code
        super().__init__(self.message)


class AuthenticationError(LXProxyError):
    """Authentication failed (401)"""

    def __init__(self, message: str = "Authentication failed"):
        super().__init__(message, status_code=401)


class APIError(LXProxyError):
    """API request failed"""

    def __init__(self, message: str, status_code: int | None = None, response_data: dict | None = None):
        self.response_data = response_data
        super().__init__(message, status_code=status_code)


class NotFoundError(LXProxyError):
    """Resource not found (404)"""

    def __init__(self, message: str = "Resource not found"):
        super().__init__(message, status_code=404)


class ValidationError(LXProxyError):
    """Validation error (400)"""

    def __init__(self, message: str, errors: list | None = None):
        self.errors = errors or []
        super().__init__(message, status_code=400)
