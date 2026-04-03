package lxproxy

import (
	"fmt"
)

// LXProxyError is the base error type for LX-Proxy SDK
type LXProxyError struct {
	Message    string
	StatusCode int
}

func (e *LXProxyError) Error() string {
	if e.StatusCode > 0 {
		return fmt.Sprintf("%d: %s", e.StatusCode, e.Message)
	}
	return e.Message
}

// AuthenticationError represents authentication failures (401)
type AuthenticationError struct {
	Message string
}

func (e *AuthenticationError) Error() string {
	return fmt.Sprintf("authentication error: %s", e.Message)
}

// APIError represents general API errors
type APIError struct {
	StatusCode int
	Message    string
}

func (e *APIError) Error() string {
	return fmt.Sprintf("API error %d: %s", e.StatusCode, e.Message)
}

// NotFoundError represents resource not found errors (404)
type NotFoundError struct {
	Message string
}

func (e *NotFoundError) Error() string {
	return fmt.Sprintf("not found: %s", e.Message)
}

// ValidationError represents validation errors (400)
type ValidationError struct {
	Message string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("validation error: %s", e.Message)
}

// Error types for type checking
var (
	ErrAuthentication = &AuthenticationError{}
	ErrNotFound       = &NotFoundError{}
	ErrValidation     = &ValidationError{}
)

// IsAuthenticationError checks if an error is an authentication error
func IsAuthenticationError(err error) bool {
	_, ok := err.(*AuthenticationError)
	return ok
}

// IsNotFoundError checks if an error is a not found error
func IsNotFoundError(err error) bool {
	_, ok := err.(*NotFoundError)
	return ok
}

// IsValidationError checks if an error is a validation error
func IsValidationError(err error) bool {
	_, ok := err.(*ValidationError)
	return ok
}

// IsAPIError checks if an error is an API error
func IsAPIError(err error) bool {
	_, ok := err.(*APIError)
	return ok
}
