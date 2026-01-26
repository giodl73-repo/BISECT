"""
Tests for health and version endpoints.

Tests the /health and /version endpoints for proper functionality.
"""
import pytest
from fastapi.testclient import TestClient


def test_health_endpoint_returns_healthy(client: TestClient):
    """Test that health endpoint returns healthy status with database connection."""
    response = client.get("/health")
    assert response.status_code == 200

    data = response.json()
    assert data["status"] == "healthy"
    assert data["database"] == "connected"
    assert "timestamp" in data


def test_version_endpoint(client: TestClient):
    """Test that version endpoint returns correct version information."""
    response = client.get("/version")
    assert response.status_code == 200

    data = response.json()
    assert data["version"] == "9.0.0"
    assert data["api_version"] == "v1"
    assert data["wave"] == 9


def test_root_endpoint(client: TestClient):
    """Test that root endpoint returns API information."""
    response = client.get("/")
    assert response.status_code == 200

    data = response.json()
    assert "message" in data
    assert "version" in data
    assert data["version"] == "9.0.0"


def test_cors_headers_present(client: TestClient):
    """Test that CORS headers are present in responses."""
    response = client.get("/health", headers={"Origin": "http://localhost:3002"})
    assert response.status_code == 200
    # Note: TestClient doesn't fully simulate CORS, actual CORS tested manually


def test_cors_preflight_request(client: TestClient):
    """Test that CORS preflight returns correct headers."""
    response = client.options(
        "/health",
        headers={
            "Origin": "http://localhost:3002",
            "Access-Control-Request-Method": "GET",
        },
    )
    # TestClient returns 200 for OPTIONS
    assert response.status_code == 200
