"""
District data API endpoints.

Serves district GeoJSON and statistics for completed runs.
"""
from pathlib import Path
from typing import Optional
from fastapi import APIRouter, HTTPException, Depends, status
from fastapi.responses import FileResponse, JSONResponse
from sqlalchemy.orm import Session
import json

from app.database import get_db
from app.services import run_service
from app.models.run import RunStatus

router = APIRouter()


@router.get("/{run_id}/districts/{year}/geojson")
async def get_district_geojson(
    run_id: int,
    year: str,
    db: Session = Depends(get_db),
):
    """
    Get district GeoJSON for a completed run and year.

    Returns GeoJSON FeatureCollection with district geometries and properties.
    """
    # Verify run exists and is completed
    run = run_service.get_run(db, run_id)
    if not run:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Run {run_id} not found",
        )

    if run.status != RunStatus.COMPLETED:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail=f"Run must be completed. Current status: {run.status}",
        )

    # Check if year is in run config
    if year not in run.config.get("years", []):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Year {year} not in run configuration",
        )

    # Build path to district GeoJSON file
    # Format: outputs/{version}/results/{year}/data/national_districts.geojson
    version = run.version
    geojson_path = (
        Path("outputs") / version / "results" / year / "data" / "national_districts.geojson"
    )

    if not geojson_path.exists():
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"District GeoJSON not found for year {year}. Pipeline may not have completed visualization stage.",
        )

    # Return GeoJSON file
    return FileResponse(
        path=str(geojson_path),
        media_type="application/geo+json",
        filename=f"districts_{year}.geojson",
    )


@router.get("/{run_id}/districts/{year}/stats")
async def get_district_stats(
    run_id: int,
    year: str,
    db: Session = Depends(get_db),
):
    """
    Get district statistics for a completed run and year.

    Returns aggregate statistics across all districts.
    """
    # Verify run exists and is completed
    run = run_service.get_run(db, run_id)
    if not run:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"Run {run_id} not found",
        )

    if run.status != RunStatus.COMPLETED:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail=f"Run must be completed. Current status: {run.status}",
        )

    # Check if year is in run config
    if year not in run.config.get("years", []):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Year {year} not in run configuration",
        )

    # Build path to stats CSV (or calculate from GeoJSON)
    version = run.version
    stats_csv_path = (
        Path("outputs") / version / "results" / year / "data" / "district_summary.csv"
    )

    # If stats CSV exists, parse and return summary
    if stats_csv_path.exists():
        try:
            import csv
            with open(stats_csv_path, 'r') as f:
                reader = csv.DictReader(f)
                districts = list(reader)

            total_districts = len(districts)
            total_population = sum(int(d.get('population', 0)) for d in districts)
            total_area = sum(float(d.get('area_sq_km', 0)) for d in districts)
            avg_compactness = sum(float(d.get('compactness_polsby_popper', 0)) for d in districts) / total_districts if total_districts > 0 else 0
            avg_population = total_population / total_districts if total_districts > 0 else 0

            return {
                "total_districts": total_districts,
                "avg_compactness": avg_compactness,
                "avg_population": avg_population,
                "total_population": total_population,
                "total_area": total_area,
            }
        except Exception as e:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail=f"Failed to parse stats: {str(e)}",
            )

    # Fallback: Calculate from GeoJSON
    geojson_path = (
        Path("outputs") / version / "results" / year / "data" / "national_districts.geojson"
    )

    if not geojson_path.exists():
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"District data not found for year {year}",
        )

    try:
        with open(geojson_path, 'r') as f:
            geojson = json.load(f)

        features = geojson.get('features', [])
        total_districts = len(features)

        if total_districts == 0:
            return {
                "total_districts": 0,
                "avg_compactness": 0,
                "avg_population": 0,
                "total_population": 0,
                "total_area": 0,
            }

        total_population = sum(f['properties'].get('population', 0) for f in features)
        total_area = sum(f['properties'].get('area_sq_km', 0) for f in features)
        avg_compactness = sum(f['properties'].get('compactness_polsby_popper', 0) for f in features) / total_districts
        avg_population = total_population / total_districts

        return {
            "total_districts": total_districts,
            "avg_compactness": avg_compactness,
            "avg_population": avg_population,
            "total_population": total_population,
            "total_area": total_area,
        }
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Failed to calculate stats: {str(e)}",
        )
