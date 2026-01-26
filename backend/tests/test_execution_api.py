"""
Tests for pipeline execution API endpoints.

Tests starting and cancelling runs.
"""
import pytest
from fastapi.testclient import TestClient


def test_start_run_endpoint(client: TestClient):
    """Test starting a run via API."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "test_exec",
        "years": ["2020"],
        "states": ["VT"],
        "workers": 1,
    })
    assert create_response.status_code == 201
    run_id = create_response.json()["id"]

    # Start the run
    start_response = client.post(f"/api/v1/runs/{run_id}/actions/start")

    assert start_response.status_code == 200
    data = start_response.json()
    assert data["id"] == run_id
    assert data["status"] == "running"


def test_start_run_not_found(client: TestClient):
    """Test starting a non-existent run."""
    response = client.post("/api/v1/runs/99999/actions/start")

    assert response.status_code == 404


def test_start_run_already_running(client: TestClient):
    """Test starting a run that is already running."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "test_already_running",
        "years": ["2020"],
        "states": ["VT"],
    })
    run_id = create_response.json()["id"]

    # Start the run
    client.post(f"/api/v1/runs/{run_id}/actions/start")

    # Try to start again
    response = client.post(f"/api/v1/runs/{run_id}/actions/start")

    assert response.status_code == 409
    assert "Cannot start run" in response.json()["detail"]


def test_cancel_run_endpoint(client: TestClient):
    """Test cancelling a running."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "test_cancel",
        "years": ["2020"],
        "states": ["VT"],
    })
    run_id = create_response.json()["id"]

    # Start the run
    client.post(f"/api/v1/runs/{run_id}/actions/start")

    # Cancel the run
    cancel_response = client.post(f"/api/v1/runs/{run_id}/actions/cancel")

    assert cancel_response.status_code == 200
    data = cancel_response.json()
    assert data["id"] == run_id
    assert data["status"] == "cancelled"


def test_cancel_run_not_running(client: TestClient):
    """Test cancelling a run that is not running."""
    # Create a run (pending status)
    create_response = client.post("/api/v1/runs", json={
        "version": "test_cancel_pending",
        "years": ["2020"],
    })
    run_id = create_response.json()["id"]

    # Try to cancel without starting
    response = client.post(f"/api/v1/runs/{run_id}/actions/cancel")

    assert response.status_code == 409
    assert "Cannot cancel run" in response.json()["detail"]


def test_cancel_run_not_found(client: TestClient):
    """Test cancelling a non-existent run."""
    response = client.post("/api/v1/runs/99999/actions/cancel")

    assert response.status_code == 404
