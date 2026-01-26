# Design Patterns for API and Frontend Development

**Updated**: 2026-01-25
**Wave**: 9 (API Migration)
**Related**: [CODING_PATTERNS.md](CODING_PATTERNS.md), [ARCHITECTURE.md](ARCHITECTURE.md)

## Overview

This document defines patterns for the FastAPI backend and React frontend being added in Wave 9. These patterns complement the existing CLI patterns in [CODING_PATTERNS.md](CODING_PATTERNS.md) and ensure consistency across the web layer.

---

## API Patterns (FastAPI Backend)

### 1. Project Structure

```
api/
├── main.py                 # FastAPI app, CORS, lifespan
├── config.py               # Settings via pydantic-settings
├── database.py             # SQLAlchemy engine, session
├── models/                 # SQLAlchemy ORM models
│   ├── __init__.py
│   ├── run.py              # Run, RunState, RunConfig
│   ├── district.py         # District, DistrictMetrics
│   └── state.py            # State, StateConfig
├── schemas/                # Pydantic request/response schemas
│   ├── __init__.py
│   ├── run.py              # RunCreate, RunResponse, RunList
│   ├── district.py         # DistrictResponse, DistrictList
│   └── state.py            # StateResponse, StateConfig
├── routers/                # API route handlers
│   ├── __init__.py
│   ├── runs.py             # /api/runs/*
│   ├── districts.py        # /api/districts/*
│   ├── states.py           # /api/states/*
│   └── pipeline.py         # /api/pipeline/* (execution)
├── services/               # Business logic
│   ├── __init__.py
│   ├── run_service.py      # Run CRUD operations
│   ├── pipeline_service.py # Pipeline execution
│   └── result_service.py   # Result aggregation
├── workers/                # Background task execution
│   ├── __init__.py
│   └── pipeline_worker.py  # Async subprocess management
└── utils/                  # Shared utilities
    ├── __init__.py
    └── status_parser.py    # STATUS protocol parsing
```

### 2. REST Endpoint Design

**URL Convention**:
```
/api/v1/{resource}              # Collection (GET, POST)
/api/v1/{resource}/{id}         # Item (GET, PUT, DELETE)
/api/v1/{resource}/{id}/{sub}   # Nested resource
/api/v1/{resource}/{id}/actions/{action}  # Actions (POST)
```

**Examples**:
```python
# routers/runs.py
from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy.orm import Session
from typing import List, Optional

router = APIRouter(prefix="/api/v1/runs", tags=["runs"])

@router.get("", response_model=RunListResponse)
async def list_runs(
    status: Optional[str] = Query(None, description="Filter by status"),
    year: Optional[str] = Query(None, description="Filter by year"),
    limit: int = Query(50, ge=1, le=100),
    offset: int = Query(0, ge=0),
    db: Session = Depends(get_db)
):
    """List all pipeline runs with optional filtering."""
    runs = run_service.list_runs(db, status=status, year=year, limit=limit, offset=offset)
    return RunListResponse(runs=runs, total=len(runs))

@router.post("", response_model=RunResponse, status_code=201)
async def create_run(
    run_create: RunCreate,
    db: Session = Depends(get_db)
):
    """Start a new pipeline run."""
    run = run_service.create_run(db, run_create)
    return run

@router.get("/{run_id}", response_model=RunDetailResponse)
async def get_run(run_id: int, db: Session = Depends(get_db)):
    """Get detailed run information including progress."""
    run = run_service.get_run(db, run_id)
    if not run:
        raise HTTPException(status_code=404, detail="Run not found")
    return run

@router.post("/{run_id}/actions/cancel", response_model=RunResponse)
async def cancel_run(run_id: int, db: Session = Depends(get_db)):
    """Cancel an active run."""
    run = run_service.cancel_run(db, run_id)
    return run
```

### 3. Request/Response Schemas

**Pattern**: Separate schemas for create, update, and response.

```python
# schemas/run.py
from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import datetime
from enum import Enum

class RunStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"

class RunCreate(BaseModel):
    """Schema for creating a new run."""
    version: str = Field(..., min_length=1, max_length=50, description="Version identifier")
    years: List[str] = Field(..., min_items=1, description="Census years to process")
    states: Optional[List[str]] = Field(None, description="States to process (all if None)")
    workers: int = Field(4, ge=1, le=16, description="Number of parallel workers")
    dpi: int = Field(150, ge=72, le=600, description="Map DPI")
    partition_mode: str = Field("edge-weighted", description="METIS partition mode")

    class Config:
        json_schema_extra = {
            "example": {
                "version": "v2",
                "years": ["2020", "2010"],
                "states": ["CA", "TX", "NY"],
                "workers": 4,
                "dpi": 150,
                "partition_mode": "edge-weighted"
            }
        }

class RunResponse(BaseModel):
    """Schema for run response."""
    id: int
    version: str
    status: RunStatus
    years: List[str]
    states: Optional[List[str]]
    workers: int
    dpi: int
    partition_mode: str
    created_at: datetime
    started_at: Optional[datetime]
    completed_at: Optional[datetime]
    error_message: Optional[str]

    class Config:
        from_attributes = True

class RunDetailResponse(RunResponse):
    """Extended run response with progress."""
    progress: dict  # Year -> {states_complete, states_total, stage, ...}
    districts_count: int
    duration_seconds: Optional[int]

class RunListResponse(BaseModel):
    """Paginated run list response."""
    runs: List[RunResponse]
    total: int
    limit: int = 50
    offset: int = 0
```

### 4. Error Handling

**Pattern**: Consistent error responses with proper HTTP status codes.

```python
# utils/exceptions.py
from fastapi import HTTPException, status

class APIError(HTTPException):
    """Base API exception."""
    def __init__(self, detail: str, status_code: int = 500):
        super().__init__(status_code=status_code, detail=detail)

class NotFoundError(APIError):
    def __init__(self, resource: str, id: any):
        super().__init__(f"{resource} with id {id} not found", status.HTTP_404_NOT_FOUND)

class ConflictError(APIError):
    def __init__(self, detail: str):
        super().__init__(detail, status.HTTP_409_CONFLICT)

class ValidationError(APIError):
    def __init__(self, detail: str):
        super().__init__(detail, status.HTTP_422_UNPROCESSABLE_ENTITY)

# Usage in router
@router.post("/{run_id}/actions/cancel")
async def cancel_run(run_id: int, db: Session = Depends(get_db)):
    run = run_service.get_run(db, run_id)
    if not run:
        raise NotFoundError("Run", run_id)
    if run.status != RunStatus.RUNNING:
        raise ConflictError(f"Cannot cancel run in {run.status} state")
    return run_service.cancel_run(db, run_id)

# Global exception handler
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error(f"Unhandled exception: {exc}", exc_info=True)
    return JSONResponse(
        status_code=500,
        content={"detail": "Internal server error", "type": type(exc).__name__}
    )
```

### 5. Async Execution Pattern

**Pattern**: Background tasks for long-running pipeline operations.

```python
# workers/pipeline_worker.py
import asyncio
from asyncio import subprocess
from typing import Callable, Optional

class PipelineWorker:
    """Async wrapper for pipeline subprocess execution."""

    def __init__(
        self,
        run_id: int,
        command: list[str],
        on_progress: Callable[[dict], None],
        on_complete: Callable[[int], None]
    ):
        self.run_id = run_id
        self.command = command
        self.on_progress = on_progress
        self.on_complete = on_complete
        self.process: Optional[asyncio.subprocess.Process] = None
        self._cancelled = False

    async def start(self):
        """Start pipeline subprocess and monitor output."""
        self.process = await asyncio.create_subprocess_exec(
            *self.command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=self._get_env()
        )

        # Monitor stdout for STATUS messages
        asyncio.create_task(self._monitor_stdout())
        asyncio.create_task(self._monitor_stderr())

        # Wait for completion
        returncode = await self.process.wait()
        self.on_complete(returncode)

    async def _monitor_stdout(self):
        """Read stdout and parse STATUS messages."""
        while True:
            line = await self.process.stdout.readline()
            if not line:
                break

            decoded = line.decode().strip()
            if decoded.startswith("STATUS:"):
                progress = self._parse_status(decoded)
                if progress:
                    self.on_progress(progress)

    async def cancel(self):
        """Cancel running pipeline."""
        self._cancelled = True
        if self.process:
            self.process.terminate()
            try:
                await asyncio.wait_for(self.process.wait(), timeout=5.0)
            except asyncio.TimeoutError:
                self.process.kill()

    def _parse_status(self, line: str) -> Optional[dict]:
        """Parse STATUS protocol message."""
        # STATUS:YEAR:2020:COMPLETE:24/50
        # STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps
        parts = line.split(":")
        if len(parts) < 3:
            return None

        msg_type = parts[1]
        if msg_type == "YEAR":
            return {
                "type": "year",
                "year": parts[2],
                "status": parts[3],
                "progress": parts[4] if len(parts) > 4 else None
            }
        elif msg_type == "WORKER":
            return {
                "type": "worker",
                "year": parts[2],
                "worker_id": parts[3],
                "state_progress": parts[5] if len(parts) > 5 else None,
                "state_name": parts[6] if len(parts) > 6 else None,
                "stage_progress": parts[8] if len(parts) > 8 else None,
                "stage_name": parts[9] if len(parts) > 9 else None
            }
        return None

    def _get_env(self) -> dict:
        """Build subprocess environment."""
        import os
        env = os.environ.copy()
        env["MULTI_YEAR_SUBPROCESS"] = "1"
        env["TQDM_POSITION"] = "999"
        return env
```

### 6. WebSocket Progress Streaming

**Pattern**: Real-time progress updates via WebSocket.

```python
# routers/websocket.py
from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from typing import Dict, Set
import asyncio
import json

router = APIRouter()

class ConnectionManager:
    """Manage WebSocket connections for progress streaming."""

    def __init__(self):
        # run_id -> set of WebSocket connections
        self.active_connections: Dict[int, Set[WebSocket]] = {}

    async def connect(self, websocket: WebSocket, run_id: int):
        await websocket.accept()
        if run_id not in self.active_connections:
            self.active_connections[run_id] = set()
        self.active_connections[run_id].add(websocket)

    def disconnect(self, websocket: WebSocket, run_id: int):
        if run_id in self.active_connections:
            self.active_connections[run_id].discard(websocket)
            if not self.active_connections[run_id]:
                del self.active_connections[run_id]

    async def broadcast_progress(self, run_id: int, progress: dict):
        """Send progress update to all connected clients."""
        if run_id not in self.active_connections:
            return

        message = json.dumps({"type": "progress", "data": progress})
        disconnected = set()

        for websocket in self.active_connections[run_id]:
            try:
                await websocket.send_text(message)
            except Exception:
                disconnected.add(websocket)

        # Clean up disconnected
        for ws in disconnected:
            self.disconnect(ws, run_id)

manager = ConnectionManager()

@router.websocket("/ws/runs/{run_id}")
async def websocket_endpoint(websocket: WebSocket, run_id: int):
    """WebSocket endpoint for run progress updates."""
    await manager.connect(websocket, run_id)
    try:
        while True:
            # Keep connection alive, handle client messages
            data = await websocket.receive_text()
            if data == "ping":
                await websocket.send_text("pong")
    except WebSocketDisconnect:
        manager.disconnect(websocket, run_id)

# Usage in pipeline worker
def on_progress(progress: dict):
    # Called from async context
    asyncio.create_task(manager.broadcast_progress(run_id, progress))
```

### 7. Database Session Pattern

**Pattern**: Dependency injection for database sessions.

```python
# database.py
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, Session
from contextlib import contextmanager
from typing import Generator

from .config import settings

engine = create_engine(
    settings.database_url,
    pool_pre_ping=True,
    pool_size=5,
    max_overflow=10
)

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

def get_db() -> Generator[Session, None, None]:
    """Dependency for database session."""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

@contextmanager
def get_db_context() -> Generator[Session, None, None]:
    """Context manager for database session (non-dependency use)."""
    db = SessionLocal()
    try:
        yield db
        db.commit()
    except Exception:
        db.rollback()
        raise
    finally:
        db.close()
```

---

## Frontend Patterns (React Dashboard)

### 1. Project Structure

```
frontend/
├── src/
│   ├── main.tsx              # Entry point
│   ├── App.tsx               # Root component, routing
│   ├── api/                  # API client
│   │   ├── client.ts         # Axios instance
│   │   ├── runs.ts           # Run API calls
│   │   ├── districts.ts      # District API calls
│   │   └── websocket.ts      # WebSocket connection
│   ├── components/           # Reusable components
│   │   ├── ui/               # Generic UI (Button, Card, etc.)
│   │   ├── forms/            # Form components
│   │   ├── maps/             # Map visualization
│   │   └── tables/           # Data tables
│   ├── features/             # Feature modules
│   │   ├── runs/             # Run management
│   │   │   ├── RunList.tsx
│   │   │   ├── RunDetail.tsx
│   │   │   ├── RunForm.tsx
│   │   │   └── hooks.ts
│   │   ├── districts/        # District visualization
│   │   │   ├── DistrictMap.tsx
│   │   │   ├── DistrictTable.tsx
│   │   │   └── hooks.ts
│   │   └── dashboard/        # Main dashboard
│   │       ├── Dashboard.tsx
│   │       └── components/
│   ├── hooks/                # Shared hooks
│   │   ├── useApi.ts         # Data fetching
│   │   ├── useWebSocket.ts   # WebSocket connection
│   │   └── useLocalStorage.ts
│   ├── types/                # TypeScript types
│   │   ├── run.ts
│   │   ├── district.ts
│   │   └── api.ts
│   └── utils/                # Utilities
│       ├── format.ts         # Number/date formatting
│       └── constants.ts      # App constants
├── public/
└── index.html
```

### 2. Component Architecture

**Pattern**: Container (data) + Presentational (UI) separation.

```tsx
// features/runs/RunList.tsx (Container)
import { useRuns } from './hooks';
import { RunTable } from './components/RunTable';
import { RunFilters } from './components/RunFilters';
import { LoadingSpinner, ErrorBanner } from '@/components/ui';

export function RunList() {
  const {
    runs,
    isLoading,
    error,
    filters,
    setFilters,
    refetch
  } = useRuns();

  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorBanner error={error} onRetry={refetch} />;

  return (
    <div className="space-y-4">
      <RunFilters
        filters={filters}
        onChange={setFilters}
      />
      <RunTable
        runs={runs}
        onRowClick={(run) => navigate(`/runs/${run.id}`)}
      />
    </div>
  );
}

// features/runs/components/RunTable.tsx (Presentational)
import { Run } from '@/types/run';
import { formatDate, formatDuration } from '@/utils/format';
import { StatusBadge } from '@/components/ui';

interface RunTableProps {
  runs: Run[];
  onRowClick: (run: Run) => void;
}

export function RunTable({ runs, onRowClick }: RunTableProps) {
  return (
    <table className="min-w-full divide-y divide-gray-200">
      <thead>
        <tr>
          <th>Version</th>
          <th>Status</th>
          <th>Years</th>
          <th>Started</th>
          <th>Duration</th>
        </tr>
      </thead>
      <tbody>
        {runs.map((run) => (
          <tr
            key={run.id}
            onClick={() => onRowClick(run)}
            className="cursor-pointer hover:bg-gray-50"
          >
            <td>{run.version}</td>
            <td><StatusBadge status={run.status} /></td>
            <td>{run.years.join(', ')}</td>
            <td>{formatDate(run.started_at)}</td>
            <td>{formatDuration(run.duration_seconds)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

### 3. Data Fetching Pattern

**Pattern**: Custom hooks with React Query (or SWR).

```tsx
// features/runs/hooks.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { runApi } from '@/api/runs';
import { Run, RunCreate, RunFilters } from '@/types/run';

export function useRuns(filters?: RunFilters) {
  return useQuery({
    queryKey: ['runs', filters],
    queryFn: () => runApi.list(filters),
    staleTime: 30000, // 30 seconds
  });
}

export function useRun(runId: number) {
  return useQuery({
    queryKey: ['runs', runId],
    queryFn: () => runApi.get(runId),
    refetchInterval: (data) => {
      // Refetch every 2s if running, stop if complete
      return data?.status === 'running' ? 2000 : false;
    },
  });
}

export function useCreateRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: RunCreate) => runApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['runs'] });
    },
  });
}

export function useCancelRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (runId: number) => runApi.cancel(runId),
    onSuccess: (_, runId) => {
      queryClient.invalidateQueries({ queryKey: ['runs', runId] });
      queryClient.invalidateQueries({ queryKey: ['runs'] });
    },
  });
}
```

### 4. WebSocket Integration

**Pattern**: Shared WebSocket hook with auto-reconnect.

```tsx
// hooks/useWebSocket.ts
import { useEffect, useRef, useCallback, useState } from 'react';

interface WebSocketOptions {
  onMessage?: (data: any) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  reconnectAttempts?: number;
  reconnectInterval?: number;
}

export function useWebSocket(url: string | null, options: WebSocketOptions = {}) {
  const wsRef = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<any>(null);
  const reconnectCount = useRef(0);

  const {
    onMessage,
    onConnect,
    onDisconnect,
    reconnectAttempts = 5,
    reconnectInterval = 3000,
  } = options;

  const connect = useCallback(() => {
    if (!url) return;

    const ws = new WebSocket(url);

    ws.onopen = () => {
      setIsConnected(true);
      reconnectCount.current = 0;
      onConnect?.();
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setLastMessage(data);
      onMessage?.(data);
    };

    ws.onclose = () => {
      setIsConnected(false);
      onDisconnect?.();

      // Auto-reconnect
      if (reconnectCount.current < reconnectAttempts) {
        reconnectCount.current++;
        setTimeout(connect, reconnectInterval);
      }
    };

    wsRef.current = ws;
  }, [url, onMessage, onConnect, onDisconnect, reconnectAttempts, reconnectInterval]);

  useEffect(() => {
    connect();
    return () => {
      wsRef.current?.close();
    };
  }, [connect]);

  const send = useCallback((data: any) => {
    wsRef.current?.send(JSON.stringify(data));
  }, []);

  return { isConnected, lastMessage, send };
}

// Usage in component
function RunProgress({ runId }: { runId: number }) {
  const [progress, setProgress] = useState<RunProgress | null>(null);

  useWebSocket(
    runId ? `ws://localhost:8002/ws/runs/${runId}` : null,
    {
      onMessage: (data) => {
        if (data.type === 'progress') {
          setProgress(data.data);
        }
      },
    }
  );

  return progress ? <ProgressDisplay progress={progress} /> : null;
}
```

### 5. Error Boundaries and Loading States

**Pattern**: Consistent error and loading UI.

```tsx
// components/ui/ErrorBoundary.tsx
import { Component, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="p-4 bg-red-50 border border-red-200 rounded">
          <h2 className="text-red-800 font-semibold">Something went wrong</h2>
          <p className="text-red-600 text-sm">{this.state.error?.message}</p>
        </div>
      );
    }
    return this.props.children;
  }
}

// components/ui/LoadingStates.tsx
export function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center p-8">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
    </div>
  );
}

export function LoadingSkeleton({ lines = 3 }: { lines?: number }) {
  return (
    <div className="space-y-3 animate-pulse">
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className="h-4 bg-gray-200 rounded"
          style={{ width: `${100 - i * 10}%` }}
        />
      ))}
    </div>
  );
}

// components/ui/ErrorBanner.tsx
interface ErrorBannerProps {
  error: Error;
  onRetry?: () => void;
}

export function ErrorBanner({ error, onRetry }: ErrorBannerProps) {
  return (
    <div className="p-4 bg-red-50 border border-red-200 rounded flex items-center justify-between">
      <div>
        <p className="text-red-800 font-medium">Error loading data</p>
        <p className="text-red-600 text-sm">{error.message}</p>
      </div>
      {onRetry && (
        <button
          onClick={onRetry}
          className="px-3 py-1 bg-red-100 text-red-700 rounded hover:bg-red-200"
        >
          Retry
        </button>
      )}
    </div>
  );
}
```

### 6. Map Visualization Pattern

**Pattern**: Leaflet integration with GeoJSON district boundaries.

```tsx
// components/maps/DistrictMap.tsx
import { MapContainer, TileLayer, GeoJSON, useMap } from 'react-leaflet';
import { useMemo, useRef } from 'react';
import { District } from '@/types/district';
import { getDistrictColor, getDistrictPopup } from './utils';

interface DistrictMapProps {
  districts: District[];
  colorBy: 'default' | 'compactness' | 'partisan' | 'demographic';
  selectedId?: number;
  onDistrictClick?: (district: District) => void;
}

export function DistrictMap({
  districts,
  colorBy,
  selectedId,
  onDistrictClick,
}: DistrictMapProps) {
  const geoJsonRef = useRef<L.GeoJSON>(null);

  // Convert districts to GeoJSON FeatureCollection
  const geojsonData = useMemo(() => ({
    type: 'FeatureCollection' as const,
    features: districts.map((d) => ({
      type: 'Feature' as const,
      id: d.id,
      geometry: d.geometry,
      properties: d,
    })),
  }), [districts]);

  const styleFeature = (feature: any) => {
    const district = feature.properties as District;
    const isSelected = district.id === selectedId;

    return {
      fillColor: getDistrictColor(district, colorBy),
      weight: isSelected ? 3 : 1,
      opacity: 1,
      color: isSelected ? '#000' : '#666',
      fillOpacity: 0.7,
    };
  };

  const onEachFeature = (feature: any, layer: L.Layer) => {
    const district = feature.properties as District;

    layer.bindPopup(getDistrictPopup(district));

    layer.on({
      click: () => onDistrictClick?.(district),
      mouseover: (e) => {
        const layer = e.target;
        layer.setStyle({ weight: 2, fillOpacity: 0.9 });
      },
      mouseout: (e) => {
        geoJsonRef.current?.resetStyle(e.target);
      },
    });
  };

  return (
    <MapContainer
      center={[39.8283, -98.5795]}  // Center of US
      zoom={4}
      className="h-full w-full"
    >
      <TileLayer
        attribution='&copy; OpenStreetMap'
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
      />
      <GeoJSON
        ref={geoJsonRef}
        data={geojsonData}
        style={styleFeature}
        onEachFeature={onEachFeature}
      />
      <AlaskaHawaiiInsets districts={districts} colorBy={colorBy} />
    </MapContainer>
  );
}

// Separate component for AK/HI inset maps
function AlaskaHawaiiInsets({ districts, colorBy }: { districts: District[], colorBy: string }) {
  // Implementation for positioned inset maps
  return null; // Simplified
}
```

---

## Database Patterns

### 1. Schema Design

**Pattern**: Normalize for flexibility, denormalize for query performance.

```python
# models/run.py
from sqlalchemy import Column, Integer, String, DateTime, Enum, JSON, ForeignKey
from sqlalchemy.orm import relationship
from datetime import datetime
import enum

class RunStatus(enum.Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"

class Run(Base):
    __tablename__ = "runs"

    id = Column(Integer, primary_key=True, index=True)
    version = Column(String(50), nullable=False, index=True)
    status = Column(Enum(RunStatus), default=RunStatus.PENDING, index=True)

    # Configuration (stored as JSON for flexibility)
    config = Column(JSON, nullable=False)  # {years, states, workers, dpi, partition_mode}

    # Timestamps
    created_at = Column(DateTime, default=datetime.utcnow)
    started_at = Column(DateTime)
    completed_at = Column(DateTime)

    # Results
    error_message = Column(String)
    output_path = Column(String)  # Path to outputs/{version}/{year}/

    # Progress (denormalized for fast queries)
    progress_json = Column(JSON)  # {year: {states_complete, stage, ...}}

    # Relationships
    districts = relationship("District", back_populates="run", cascade="all, delete-orphan")
    year_results = relationship("YearResult", back_populates="run", cascade="all, delete-orphan")

class YearResult(Base):
    __tablename__ = "year_results"

    id = Column(Integer, primary_key=True)
    run_id = Column(Integer, ForeignKey("runs.id"), nullable=False)
    year = Column(String(4), nullable=False)

    status = Column(Enum(RunStatus), default=RunStatus.PENDING)
    states_completed = Column(Integer, default=0)
    states_total = Column(Integer, default=50)

    started_at = Column(DateTime)
    completed_at = Column(DateTime)
    error_message = Column(String)

    run = relationship("Run", back_populates="year_results")

class District(Base):
    __tablename__ = "districts"

    id = Column(Integer, primary_key=True)
    run_id = Column(Integer, ForeignKey("runs.id"), nullable=False)
    year = Column(String(4), nullable=False, index=True)
    state = Column(String(50), nullable=False, index=True)
    district_num = Column(Integer, nullable=False)

    # Core metrics
    population = Column(Integer)
    num_tracts = Column(Integer)

    # Compactness
    polsby_popper = Column(Float)
    reock = Column(Float)
    convex_hull_ratio = Column(Float)

    # Political (optional)
    partisan_lean = Column(Float)  # -1 (D) to +1 (R)

    # Demographics (optional)
    white_pct = Column(Float)
    black_pct = Column(Float)
    hispanic_pct = Column(Float)
    asian_pct = Column(Float)

    # Geometry (stored separately for performance)
    geometry_wkt = Column(Text)  # WKT for smaller districts
    geometry_file = Column(String)  # Path to GeoJSON for large geometries

    run = relationship("Run", back_populates="districts")

    __table_args__ = (
        Index('ix_district_state_year', 'state', 'year'),
    )
```

### 2. Migration Strategy

**Pattern**: Use Alembic for versioned migrations.

```python
# alembic/versions/001_initial.py
"""Initial schema

Revision ID: 001
"""
from alembic import op
import sqlalchemy as sa

def upgrade():
    op.create_table(
        'runs',
        sa.Column('id', sa.Integer(), primary_key=True),
        sa.Column('version', sa.String(50), nullable=False),
        sa.Column('status', sa.Enum('pending', 'running', 'completed', 'failed', 'cancelled'), default='pending'),
        sa.Column('config', sa.JSON(), nullable=False),
        sa.Column('created_at', sa.DateTime(), server_default=sa.func.now()),
        sa.Column('started_at', sa.DateTime()),
        sa.Column('completed_at', sa.DateTime()),
        sa.Column('error_message', sa.String()),
        sa.Column('output_path', sa.String()),
        sa.Column('progress_json', sa.JSON()),
    )
    op.create_index('ix_runs_version', 'runs', ['version'])
    op.create_index('ix_runs_status', 'runs', ['status'])

def downgrade():
    op.drop_index('ix_runs_status')
    op.drop_index('ix_runs_version')
    op.drop_table('runs')
```

### 3. Query Optimization

**Pattern**: Eager loading for related data, pagination for large result sets.

```python
# services/run_service.py
from sqlalchemy.orm import joinedload, selectinload
from sqlalchemy import and_

def get_run_with_districts(db: Session, run_id: int) -> Optional[Run]:
    """Get run with all related data (eager loaded)."""
    return db.query(Run)\
        .options(
            selectinload(Run.year_results),
            selectinload(Run.districts)
        )\
        .filter(Run.id == run_id)\
        .first()

def list_runs(
    db: Session,
    status: Optional[str] = None,
    year: Optional[str] = None,
    limit: int = 50,
    offset: int = 0
) -> List[Run]:
    """List runs with filtering and pagination."""
    query = db.query(Run)

    # Apply filters
    if status:
        query = query.filter(Run.status == status)
    if year:
        # Filter by year in config JSON
        query = query.filter(Run.config['years'].contains([year]))

    # Order by newest first, paginate
    return query\
        .order_by(Run.created_at.desc())\
        .offset(offset)\
        .limit(limit)\
        .all()

def get_districts_by_state(
    db: Session,
    state: str,
    year: str,
    run_id: Optional[int] = None
) -> List[District]:
    """Get districts for a state, optionally from specific run."""
    query = db.query(District)\
        .filter(
            District.state == state,
            District.year == year
        )

    if run_id:
        query = query.filter(District.run_id == run_id)
    else:
        # Get from most recent completed run
        subquery = db.query(Run.id)\
            .filter(Run.status == RunStatus.COMPLETED)\
            .order_by(Run.completed_at.desc())\
            .limit(1)\
            .scalar_subquery()
        query = query.filter(District.run_id == subquery)

    return query.all()
```

---

## Integration Patterns

### 1. Pipeline Subprocess Management

**Pattern**: Wrap existing CLI without modification.

```python
# services/pipeline_service.py
import sys
import asyncio
from pathlib import Path
from typing import List

class PipelineService:
    """Service for executing redistricting pipeline."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.script_path = project_root / "scripts" / "pipeline" / "run_complete_redistricting.py"

    def build_command(self, config: dict) -> List[str]:
        """Build command line for pipeline execution."""
        cmd = [
            sys.executable,
            str(self.script_path),
            "--version", config["version"],
        ]

        # Add years
        for year in config["years"]:
            cmd.extend(["--year", year])

        # Add optional states
        if config.get("states"):
            cmd.extend(["--states", ",".join(config["states"])])

        # Add other options
        cmd.extend([
            "--workers", str(config.get("workers", 4)),
            "--dpi", str(config.get("dpi", 150)),
            "--partition-mode", config.get("partition_mode", "edge-weighted"),
        ])

        return cmd

    async def execute(
        self,
        config: dict,
        on_progress: callable,
        on_complete: callable
    ):
        """Execute pipeline asynchronously."""
        command = self.build_command(config)
        worker = PipelineWorker(
            run_id=config["run_id"],
            command=command,
            on_progress=on_progress,
            on_complete=on_complete
        )
        await worker.start()
```

### 2. STATUS Protocol Bridge

**Pattern**: Parse STATUS messages from CLI and emit to WebSocket/database.

```python
# utils/status_bridge.py
import re
from typing import Dict, Any, Optional, Callable
from dataclasses import dataclass

@dataclass
class StatusMessage:
    """Parsed STATUS message."""
    msg_type: str
    year: Optional[str]
    data: Dict[str, Any]

class StatusBridge:
    """Bridge between CLI STATUS protocol and API/WebSocket."""

    # STATUS:YEAR:2020:COMPLETE:24/50
    YEAR_PATTERN = re.compile(r"STATUS:YEAR:(\d{4}):(\w+):?(.*)$")

    # STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps
    WORKER_PATTERN = re.compile(
        r"STATUS:WORKER:(\d{4}):(\d+):STATE:(\d+)/(\d+):(\w+):STAGE:(\d+)/(\d+):(.+)$"
    )

    # STATUS:CENSUS:2020:WORKER:0:5/50:california:1/3:Parsing PL 94-171
    CENSUS_PATTERN = re.compile(
        r"STATUS:CENSUS:(\d{4}):WORKER:(\d+):(\d+)/(\d+):(\w+):(\d+)/(\d+):(.+)$"
    )

    def __init__(
        self,
        on_year_progress: Optional[Callable] = None,
        on_worker_progress: Optional[Callable] = None,
        on_census_progress: Optional[Callable] = None
    ):
        self.on_year_progress = on_year_progress
        self.on_worker_progress = on_worker_progress
        self.on_census_progress = on_census_progress

    def parse_line(self, line: str) -> Optional[StatusMessage]:
        """Parse a STATUS line and dispatch to appropriate handler."""
        line = line.strip()
        if not line.startswith("STATUS:"):
            return None

        # Try YEAR pattern
        match = self.YEAR_PATTERN.match(line)
        if match:
            year, status, progress = match.groups()
            msg = StatusMessage(
                msg_type="year",
                year=year,
                data={"status": status, "progress": progress}
            )
            if self.on_year_progress:
                self.on_year_progress(msg)
            return msg

        # Try WORKER pattern
        match = self.WORKER_PATTERN.match(line)
        if match:
            year, worker_id, state_current, state_total, state_name, \
                stage_current, stage_total, stage_name = match.groups()
            msg = StatusMessage(
                msg_type="worker",
                year=year,
                data={
                    "worker_id": int(worker_id),
                    "state_current": int(state_current),
                    "state_total": int(state_total),
                    "state_name": state_name,
                    "stage_current": int(stage_current),
                    "stage_total": int(stage_total),
                    "stage_name": stage_name
                }
            )
            if self.on_worker_progress:
                self.on_worker_progress(msg)
            return msg

        # Try CENSUS pattern
        match = self.CENSUS_PATTERN.match(line)
        if match:
            year, worker_id, state_current, state_total, state_name, \
                stage_current, stage_total, stage_name = match.groups()
            msg = StatusMessage(
                msg_type="census",
                year=year,
                data={
                    "worker_id": int(worker_id),
                    "state_current": int(state_current),
                    "state_total": int(state_total),
                    "state_name": state_name,
                    "stage_current": int(stage_current),
                    "stage_total": int(stage_total),
                    "stage_name": stage_name
                }
            )
            if self.on_census_progress:
                self.on_census_progress(msg)
            return msg

        return None
```

### 3. Result Storage

**Pattern**: Import results from file outputs to database.

```python
# services/result_service.py
import pandas as pd
from pathlib import Path
from sqlalchemy.orm import Session
from models import Run, District

class ResultService:
    """Service for importing and querying redistricting results."""

    def __init__(self, outputs_dir: Path):
        self.outputs_dir = outputs_dir

    def import_run_results(self, db: Session, run: Run):
        """Import results from completed pipeline run to database."""
        for year in run.config["years"]:
            year_dir = self.outputs_dir / run.version / year
            self._import_year_results(db, run, year, year_dir)

        db.commit()

    def _import_year_results(
        self,
        db: Session,
        run: Run,
        year: str,
        year_dir: Path
    ):
        """Import results for a single year."""
        # Read district summary
        summary_file = year_dir / "data" / "us_district_summary.csv"
        if not summary_file.exists():
            return

        df = pd.read_csv(summary_file)

        for _, row in df.iterrows():
            district = District(
                run_id=run.id,
                year=year,
                state=row["state"],
                district_num=row["district"],
                population=row.get("population"),
                num_tracts=row.get("num_tracts"),
                polsby_popper=row.get("polsby_popper"),
                reock=row.get("reock"),
                convex_hull_ratio=row.get("convex_hull_ratio"),
                partisan_lean=row.get("partisan_lean"),
            )
            db.add(district)

    def get_district_geometry(
        self,
        db: Session,
        district: District
    ) -> dict:
        """Get GeoJSON geometry for a district."""
        # Check if geometry is inline
        if district.geometry_wkt:
            from shapely import wkt
            from shapely.geometry import mapping
            return mapping(wkt.loads(district.geometry_wkt))

        # Load from file
        if district.geometry_file:
            import json
            geom_path = self.outputs_dir / district.geometry_file
            if geom_path.exists():
                with open(geom_path) as f:
                    return json.load(f)

        # Generate from state data
        return self._generate_district_geometry(district)
```

### 4. File Serving

**Pattern**: Serve generated maps and data files.

```python
# routers/files.py
from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse, StreamingResponse
from pathlib import Path
import mimetypes

router = APIRouter(prefix="/api/v1/files", tags=["files"])

# Configure outputs directory
OUTPUTS_DIR = Path("outputs")

@router.get("/maps/{version}/{year}/states/{state}/{filename}")
async def get_state_map(version: str, year: str, state: str, filename: str):
    """Serve state-level map images."""
    file_path = OUTPUTS_DIR / version / year / "states" / state / "maps" / filename
    return serve_file(file_path, f"{state}_{filename}")

@router.get("/maps/{version}/{year}/national/{filename}")
async def get_national_map(version: str, year: str, filename: str):
    """Serve national-level map images."""
    file_path = OUTPUTS_DIR / version / year / "maps" / filename
    return serve_file(file_path, filename)

@router.get("/data/{version}/{year}/{filename}")
async def get_data_file(version: str, year: str, filename: str):
    """Serve data files (CSV, JSON)."""
    file_path = OUTPUTS_DIR / version / year / "data" / filename
    return serve_file(file_path, filename)

def serve_file(file_path: Path, download_name: str):
    """Serve a file with proper content type."""
    if not file_path.exists():
        raise HTTPException(status_code=404, detail="File not found")

    # Security: Ensure path is within outputs directory
    try:
        file_path.resolve().relative_to(OUTPUTS_DIR.resolve())
    except ValueError:
        raise HTTPException(status_code=403, detail="Access denied")

    media_type, _ = mimetypes.guess_type(str(file_path))
    return FileResponse(
        path=file_path,
        media_type=media_type,
        filename=download_name
    )
```

---

## Testing Patterns for API/Frontend

### 1. API Integration Tests

```python
# tests/api/test_runs.py
import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from api.main import app
from api.database import get_db, Base

# Test database
TEST_DATABASE_URL = "sqlite:///./test.db"
engine = create_engine(TEST_DATABASE_URL)
TestingSessionLocal = sessionmaker(bind=engine)

@pytest.fixture
def client():
    Base.metadata.create_all(bind=engine)

    def override_get_db():
        db = TestingSessionLocal()
        try:
            yield db
        finally:
            db.close()

    app.dependency_overrides[get_db] = override_get_db

    with TestClient(app) as c:
        yield c

    Base.metadata.drop_all(bind=engine)

def test_create_run(client):
    response = client.post("/api/v1/runs", json={
        "version": "test_v1",
        "years": ["2020"],
        "workers": 2
    })

    assert response.status_code == 201
    data = response.json()
    assert data["version"] == "test_v1"
    assert data["status"] == "pending"

def test_list_runs_with_filter(client):
    # Create test runs
    client.post("/api/v1/runs", json={"version": "v1", "years": ["2020"]})
    client.post("/api/v1/runs", json={"version": "v2", "years": ["2010"]})

    # Filter by year
    response = client.get("/api/v1/runs?year=2020")
    assert response.status_code == 200
    data = response.json()
    assert len(data["runs"]) == 1
    assert data["runs"][0]["version"] == "v1"
```

### 2. Frontend Component Tests

```tsx
// tests/features/runs/RunTable.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { RunTable } from '@/features/runs/components/RunTable';

const mockRuns = [
  {
    id: 1,
    version: 'v1',
    status: 'completed',
    years: ['2020'],
    started_at: '2026-01-20T10:00:00Z',
    duration_seconds: 3600,
  },
];

describe('RunTable', () => {
  it('renders runs correctly', () => {
    render(<RunTable runs={mockRuns} onRowClick={jest.fn()} />);

    expect(screen.getByText('v1')).toBeInTheDocument();
    expect(screen.getByText('completed')).toBeInTheDocument();
    expect(screen.getByText('2020')).toBeInTheDocument();
  });

  it('calls onRowClick when row is clicked', () => {
    const handleClick = jest.fn();
    render(<RunTable runs={mockRuns} onRowClick={handleClick} />);

    fireEvent.click(screen.getByText('v1'));

    expect(handleClick).toHaveBeenCalledWith(mockRuns[0]);
  });
});
```

---

## Anti-Patterns to Avoid

### API

1. **Sync blocking in async endpoints**: Use `asyncio.to_thread()` for CPU-bound operations
2. **N+1 queries**: Use `selectinload()` / `joinedload()` for relationships
3. **Missing validation**: Always use Pydantic schemas for input validation
4. **Exposing internal errors**: Use exception handlers to sanitize error messages
5. **Hardcoded paths**: Use settings/config for all paths

### Frontend

1. **Prop drilling**: Use context or state management for deeply nested state
2. **Inline styles**: Use Tailwind classes or CSS modules
3. **No error boundaries**: Wrap feature components in ErrorBoundary
4. **Missing loading states**: Always show loading indicators during fetches
5. **Direct API calls in components**: Use custom hooks for data fetching

### Database

1. **Missing indexes**: Add indexes for all filtered/sorted columns
2. **Storing large blobs**: Use file references for geometries > 1MB
3. **No migrations**: Always use Alembic for schema changes
4. **Unbounded queries**: Always paginate list endpoints

---

## Quick Reference

### Starting New API Endpoint

```python
# 1. Define schema (schemas/resource.py)
class ResourceCreate(BaseModel): ...
class ResourceResponse(BaseModel): ...

# 2. Add model if needed (models/resource.py)
class Resource(Base): ...

# 3. Create service (services/resource_service.py)
def create_resource(db, data): ...

# 4. Create router (routers/resource.py)
@router.post("", response_model=ResourceResponse)
async def create(data: ResourceCreate, db=Depends(get_db)): ...

# 5. Register router (main.py)
app.include_router(resource_router)

# 6. Add tests (tests/api/test_resource.py)
def test_create_resource(client): ...
```

### Starting New React Feature

```tsx
// 1. Define types (types/resource.ts)
interface Resource { ... }

// 2. Create API client (api/resource.ts)
export const resourceApi = { list, get, create };

// 3. Create hooks (features/resource/hooks.ts)
export function useResources() { ... }

// 4. Create components (features/resource/components/)
export function ResourceList() { ... }
export function ResourceDetail() { ... }

// 5. Add route (App.tsx)
<Route path="/resources" element={<ResourceList />} />

// 6. Add tests (tests/features/resource/)
describe('ResourceList', () => { ... });
```

---

## Related Documentation

- [CODING_PATTERNS.md](CODING_PATTERNS.md) - CLI patterns (STATUS protocol, subprocess management)
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [TESTING.md](TESTING.md) - Testing patterns and requirements
- [Wave 9 Plan](waves/WAVE09-api-migration.md) - Wave 9 implementation plan
