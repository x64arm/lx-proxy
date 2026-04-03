/**
 * LX-Proxy SDK Exceptions
 */

/** Base exception class */
export class LXProxyError extends Error {
  statusCode?: number;
  responseData?: Record<string, unknown>;

  constructor(message: string, statusCode?: number, responseData?: Record<string, unknown>) {
    super(message);
    this.name = 'LXProxyError';
    this.statusCode = statusCode;
    this.responseData = responseData;
    Object.setPrototypeOf(this, LXProxyError.prototype);
  }
}

/** Authentication error (401) */
export class AuthenticationError extends LXProxyError {
  constructor(message: string = 'Authentication failed') {
    super(message, 401);
    this.name = 'AuthenticationError';
    Object.setPrototypeOf(this, AuthenticationError.prototype);
  }
}

/** API error (4xx/5xx) */
export class APIError extends LXProxyError {
  constructor(message: string, statusCode?: number, responseData?: Record<string, unknown>) {
    super(message, statusCode, responseData);
    this.name = 'APIError';
    Object.setPrototypeOf(this, APIError.prototype);
  }
}

/** Resource not found (404) */
export class NotFoundError extends LXProxyError {
  constructor(message: string = 'Resource not found') {
    super(message, 404);
    this.name = 'NotFoundError';
    Object.setPrototypeOf(this, NotFoundError.prototype);
  }
}

/** Validation error (400) */
export class ValidationError extends LXProxyError {
  errors: string[];

  constructor(message: string, errors: string[] = []) {
    super(message, 400);
    this.name = 'ValidationError';
    this.errors = errors;
    Object.setPrototypeOf(this, ValidationError.prototype);
  }
}
