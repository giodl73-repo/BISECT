"""
Tests for run management API endpoints.

Integration tests for CRUD operations on runs.
"""
import pytest
from fastapi.testclient import TestClient


def test_create_run_endpoint(client: TestClient):
    """Test creating a run via API."""
    response = client.post("/api/v1/runs", json={
        "version": "test_v1",
        "years": ["2020"],
        "states": ["VT"],
        "workers": 1,
        "dpi": 150,
        "partition_mode": "edge-weighted",
    })

    assert response.status_code == 201
    data = response.json()
    assert data["version"] == "test_v1"
    assert data["status"] == "pending"
    assert "id" in data


def test_create_run_validation_empty_version(client: TestClient):
    """Test validation rejects empty version."""
    response = client.post("/api/v1/runs", json={
        "version": "",
        "years": ["2020"],
    })

    assert response.status_code == 422


def test_create_run_validation_invalid_year(client: TestClient):
    """Test validation rejects invalid year."""
    response = client.post("/api/v1/runs", json={
        "version": "v1",
        "years": ["1999"],  # Invalid year
    })

    assert response.status_code == 422


def test_create_run_validation_empty_years(client: TestClient):
    """Test validation rejects empty years list."""
    response = client.post("/api/v1/runs", json={
        "version": "v1",
        "years": [],
    })

    assert response.status_code == 422


def test_create_run_validation_workers_range(client: TestClient):
    """Test validation enforces workers range."""
    # Too few workers
    response = client.post("/api/v1/runs", json={
        "version": "v1",
        "years": ["2020"],
        "workers": 0,
    })
    assert response.status_code == 422

    # Too many workers
    response = client.post("/api/v1/runs", json={
        "version": "v1",
        "years": ["2020"],
        "workers": 100,
    })
    assert response.status_code == 422


def test_list_runs_endpoint(client: TestClient):
    """Test listing runs via API."""
    # Create a few runs
    for i in range(3):
        client.post("/api/v1/runs", json={
            "version": f"v{i}",
            "years": ["2020"],
        })

    response = client.get("/api/v1/runs")

    assert response.status_code == 200
    data = response.json()
    assert "runs" in data
    assert "total" in data
    assert data["total"] >= 3


def test_list_runs_with_status_filter(client: TestClient):
    """Test listing runs filtered by status."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "filter_test",
        "years": ["2020"],
    })
    run_id = create_response.json()["id"]

    # Filter by pending status
    response = client.get("/api/v1/runs?status=pending")

    assert response.status_code == 200
    data = response.json()
    assert all(r["status"] == "pending" for r in data["runs"])


def test_list_runs_pagination(client: TestClient):
    """Test pagination of run list."""
    # Create 15 runs
    for i in range(15):
        client.post("/api/v1/runs", json={
            "version": f"page_test_{i}",
            "years": ["2020"],
        })

    # Get first page
    response = client.get("/api/v1/runs?limit=10&offset=0")
    assert response.status_code == 200
    data = response.json()
    assert len(data["runs"]) == 10
    assert data["total"] >= 15

    # Get second page
    response = client.get("/api/v1/runs?limit=10&offset=10")
    assert response.status_code == 200
    data = response.json()
    assert len(data["runs"]) >= 5


def test_get_run_endpoint(client: TestClient):
    """Test getting a run by ID."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "get_test",
        "years": ["2020", "2010"],
        "workers": 4,
    })
    run_id = create_response.json()["id"]

    # Get the run
    response = client.get(f"/api/v1/runs/{run_id}")

    assert response.status_code == 200
    data = response.json()
    assert data["id"] == run_id
    assert data["version"] == "get_test"
    assert "config" in data
    assert data["config"]["years"] == ["2020", "2010"]
    assert "year_details" in data
    assert len(data["year_details"]) == 2


def test_get_run_not_found(client: TestClient):
    """Test getting a non-existent run."""
    response = client.get("/api/v1/runs/99999")

    assert response.status_code == 404


def test_delete_run_endpoint(client: TestClient):
    """Test deleting a run."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "delete_test",
        "years": ["2020"],
    })
    run_id = create_response.json()["id"]

    # Delete the run
    response = client.delete(f"/api/v1/runs/{run_id}")

    assert response.status_code == 204

    # Verify it's gone
    get_response = client.get(f"/api/v1/runs/{run_id}")
    assert get_response.status_code == 404


def test_delete_run_not_found(client: TestClient):
    """Test deleting a non-existent run."""
    response = client.delete("/api/v1/runs/99999")

    assert response.status_code == 404


def test_get_run_progress(client: TestClient):
    """Test getting run progress."""
    # Create a run
    create_response = client.post("/api/v1/runs", json={
        "version": "progress_test",
        "years": ["2020"],
    })
    run_id = create_response.json()["id"]

    # Get progress
    response = client.get(f"/api/v1/runs/{run_id}/progress")

    assert response.status_code == 200
    data = response.json()
    assert data["run_id"] == run_id
    assert data["status"] == "pending"
    assert "overall_progress" in data
    assert "years" in data
    assert "2020" in data["years"]


def test_get_state_config_2020(client: TestClient):
    """Test getting state configuration for 2020."""
    response = client.get("/api/v1/runs/config/states?year=2020")

    assert response.status_code == 200
    data = response.json()
    assert data["year"] == "2020"
    assert len(data["states"]) == 50
    assert all("code" in s and "name" in s and "districts" in s for s in data["states"])


def test_get_state_config_2010(client: TestClient):
    """Test getting state configuration for 2010."""
    response = client.get("/api/v1/runs/config/states?year=2010")

    assert response.status_code == 200
    data = response.json()
    assert data["year"] == "2010"
    assert len(data["states"]) == 50


def test_get_state_config_invalid_year(client: TestClient):
    """Test getting state configuration with invalid year."""
    response = client.get("/api/v1/runs/config/states?year=1999")

    assert response.status_code == 400
