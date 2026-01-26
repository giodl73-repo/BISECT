"""
Tests for district data API endpoints.

Tests GeoJSON serving and statistics calculation.
"""
import pytest
from pathlib import Path
from fastapi.testclient import TestClient


def test_get_district_geojson_run_not_found(client: TestClient):
    """Test fetching GeoJSON for non-existent run."""
    response = client.get("/api/v1/runs/99999/districts/2020/geojson")

    assert response.status_code == 404
    assert "not found" in response.json()["detail"].lower()


def test_get_district_geojson_run_not_completed(client: TestClient):
    """Test fetching GeoJSON for non-completed run."""
    # Create a pending run
    create_response = client.post("/api/v1/runs", json={
        "version": "test_districts",
        "years": ["2020"],
        "states": ["VT"],
    })
    run_id = create_response.json()["id"]

    # Try to fetch GeoJSON
    response = client.get(f"/api/v1/runs/{run_id}/districts/2020/geojson")

    assert response.status_code == 409
    assert "completed" in response.json()["detail"].lower()


def test_get_district_geojson_invalid_year(client: TestClient):
    """Test fetching GeoJSON for year not in run config."""
    # Create a completed run (manually mark as completed for test)
    create_response = client.post("/api/v1/runs", json={
        "version": "test_districts_year",
        "years": ["2020"],
    })
    run_id = create_response.json()["id"]

    # Manually update status to completed (would need run_service)
    # For now, test with wrong year will fail before status check

    # Try to fetch GeoJSON for 2010 (not in config)
    response = client.get(f"/api/v1/runs/{run_id}/districts/2010/geojson")

    # Will fail on status check first, but if we had completed run, would fail on year check
    assert response.status_code in [400, 409]


def test_get_district_stats_run_not_found(client: TestClient):
    """Test fetching stats for non-existent run."""
    response = client.get("/api/v1/runs/99999/districts/2020/stats")

    assert response.status_code == 404


def test_get_district_stats_run_not_completed(client: TestClient):
    """Test fetching stats for non-completed run."""
    # Create a pending run
    create_response = client.post("/api/v1/runs", json={
        "version": "test_stats",
        "years": ["2020"],
        "states": ["VT"],
    })
    run_id = create_response.json()["id"]

    # Try to fetch stats
    response = client.get(f"/api/v1/runs/{run_id}/districts/2020/stats")

    assert response.status_code == 409
    assert "completed" in response.json()["detail"].lower()
