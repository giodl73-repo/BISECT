# Testing Patterns for API and Frontend Development

**Updated**: 2026-01-25
**Wave**: 9 (API Migration)
**Related**: [DESIGN_PATTERNS.md](DESIGN_PATTERNS.md), [CODING_PATTERNS.md](CODING_PATTERNS.md), [tests/README.md](../tests/README.md)

## Overview

This document defines testing patterns for the FastAPI backend and React frontend being added in Wave 9. These patterns complement the existing CLI test patterns in [tests/README.md](../tests/README.md) and ensure comprehensive coverage across the web layer.

---

## General Testing Principles

### Test Pyramid

Follow the test pyramid for balanced coverage and fast feedback:

```
           /\
          /E2E\        5-10% - Critical user flows only
         /------\
        /Integ   \     20-25% - API + component integration
       /----------\
      /    Unit    \   65-75% - Pure functions, services, components
     /--------------\
```

**Target Distribution for Wave 9**:
- **Unit Tests**: 70% (~50-60 tests) - Services, parsers, utilities, UI components
- **Integration Tests**: 20% (~15-20 tests) - API endpoints with test DB, component integration
- **E2E Tests**: 10% (~5-8 tests) - Critical flows: create run, monitor progress, view results

### Test Naming Conventions

**Python (pytest)**:
```python
# Pattern: test_{what}_{scenario}_{expected_result}
def test_create_run_with_valid_data_returns_pending_status():
def test_create_run_with_missing_version_raises_validation_error():
def test_cancel_run_while_running_updates_status_to_cancelled():
def test_parse_worker_message_extracts_all_fields():
```

**TypeScript (React Testing Library / Vitest)**:
```typescript
// Pattern: it('should {action} when {condition}')
it('should display loading spinner when data is fetching')
it('should show error banner when API request fails')
it('should update progress bar every 2 seconds while running')
it('should disable cancel button for completed runs')
```

### Test Organization Patterns

**Backend Tests** (`tests/api/`):
```
tests/api/
├── unit/                           # Isolated component tests
│   ├── test_status_parser.py       # STATUS protocol parsing
│   ├── test_progress_service.py    # Progress calculation
│   ├── test_run_service.py         # Run CRUD operations
│   └── test_file_progress.py       # File-based progress I/O
├── integration/                    # Multi-component tests
│   ├── test_runs_api.py            # Full API endpoint tests
│   ├── test_progress_polling.py    # Polling endpoint tests
│   └── test_database_migrations.py # Schema migration tests
├── e2e/                            # End-to-end pipeline tests
│   └── test_pipeline_execution.py  # Full run lifecycle (VT)
└── conftest.py                     # Shared fixtures
```

**Frontend Tests** (`frontend/src/__tests__/` or colocated):
```
frontend/src/
├── components/
│   └── ui/
│       ├── Button.tsx
│       └── Button.test.tsx         # Colocated unit test
├── features/
│   └── runs/
│       ├── RunList.tsx
│       ├── RunList.test.tsx        # Component integration test
│       └── hooks.test.ts           # Custom hook tests
├── __tests__/                      # E2E and shared tests
│   ├── setup.ts                    # Test setup
│   └── e2e/
│       └── run-flow.spec.ts        # Playwright E2E
└── mocks/
    ├── handlers.ts                 # MSW request handlers
    └── server.ts                   # MSW server setup
```

### Mock vs Integration Testing Guidelines

**Use Mocks When**:
- Testing isolated units (pure functions, single components)
- Testing error handling and edge cases
- Tests need to be deterministic and fast
- External dependencies are slow or unreliable

**Use Real Integrations When**:
- Testing database operations (use test database)
- Testing API contracts between frontend and backend
- Testing critical user flows (E2E)
- Testing subprocess communication patterns

### Test Data Management

**Fixture Patterns** (Python):
```python
# Session-scoped: Expensive setup, shared across all tests
@pytest.fixture(scope='session')
def test_database():
    """Create test database once per session."""
    engine = create_engine(TEST_DATABASE_URL)
    Base.metadata.create_all(engine)
    yield engine
    Base.metadata.drop_all(engine)

# Function-scoped: Fresh for each test
@pytest.fixture
def db_session(test_database):
    """Fresh database session for each test."""
    Session = sessionmaker(bind=test_database)
    session = Session()
    yield session
    session.rollback()
    session.close()

# Factory pattern for flexible test data
@pytest.fixture
def run_factory(db_session):
    """Factory for creating test runs."""
    def create_run(**overrides):
        defaults = {
            'version': 'test_v1',
            'status': RunStatus.PENDING,
            'config': {'years': ['2020'], 'workers': 1},
        }
        defaults.update(overrides)
        run = Run(**defaults)
        db_session.add(run)
        db_session.commit()
        return run
    return create_run
```

**Factory Patterns** (TypeScript):
```typescript
// Test data factories
export function createMockRun(overrides: Partial<Run> = {}): Run {
  return {
    id: 1,
    version: 'test_v1',
    status: 'pending',
    years: ['2020'],
    workers: 4,
    created_at: '2026-01-25T10:00:00Z',
    ...overrides,
  };
}

export function createMockProgress(overrides: Partial<RunProgress> = {}): RunProgress {
  return {
    run_id: 1,
    status: 'running',
    overall_progress: 0.5,
    years: {
      '2020': { states_completed: 25, states_total: 50, status: 'running' },
    },
    eta_seconds: 3600,
    ...overrides,
  };
}
```

---

## API Testing Patterns (pytest)

### FastAPI Test Client Setup

```python
# tests/api/conftest.py
import pytest
from fastapi.testclient import TestClient
from httpx import AsyncClient
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.pool import StaticPool

from api.main import app
from api.database import get_db, Base
from api.config import settings

# Use in-memory SQLite for fast tests
TEST_DATABASE_URL = "sqlite:///:memory:"

@pytest.fixture(scope='session')
def test_engine():
    """Create test database engine."""
    engine = create_engine(
        TEST_DATABASE_URL,
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    Base.metadata.create_all(bind=engine)
    yield engine
    Base.metadata.drop_all(bind=engine)

@pytest.fixture
def db_session(test_engine):
    """Create fresh database session for each test."""
    TestingSessionLocal = sessionmaker(
        autocommit=False, autoflush=False, bind=test_engine
    )
    session = TestingSessionLocal()

    # Override app dependency
    def override_get_db():
        try:
            yield session
        finally:
            pass  # Don't close - we handle cleanup

    app.dependency_overrides[get_db] = override_get_db

    yield session

    # Rollback changes after each test
    session.rollback()
    session.close()
    app.dependency_overrides.clear()

@pytest.fixture
def client(db_session):
    """Sync test client for simple tests."""
    with TestClient(app) as c:
        yield c

@pytest.fixture
async def async_client(db_session):
    """Async test client for async endpoint tests."""
    async with AsyncClient(app=app, base_url="http://test") as ac:
        yield ac
```

### Database Test Fixtures with Transactions and Rollbacks

```python
# tests/api/conftest.py (continued)

@pytest.fixture
def transactional_session(test_engine):
    """Session with automatic rollback - for testing transactions."""
    connection = test_engine.connect()
    transaction = connection.begin()
    session = sessionmaker(bind=connection)()

    # Override app dependency
    def override_get_db():
        yield session

    app.dependency_overrides[get_db] = override_get_db

    yield session

    transaction.rollback()
    connection.close()
    app.dependency_overrides.clear()

# For testing with PostgreSQL (integration tests)
@pytest.fixture(scope='session')
def postgres_engine():
    """Real PostgreSQL engine for integration tests."""
    engine = create_engine(settings.test_database_url)
    Base.metadata.create_all(bind=engine)
    yield engine
    Base.metadata.drop_all(bind=engine)
```

### Async Test Patterns

```python
# tests/api/unit/test_pipeline_executor.py
import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

@pytest.mark.asyncio
async def test_executor_starts_subprocess():
    """Test pipeline executor creates subprocess correctly."""
    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', 'print("STATUS:YEAR:2020:COMPLETE:1/1")'],
        on_progress=AsyncMock(),
        on_complete=AsyncMock(),
    )

    await executor.start()

    executor.on_complete.assert_awaited_once_with(0)  # Exit code 0

@pytest.mark.asyncio
async def test_executor_parses_status_messages():
    """Test executor correctly parses STATUS output."""
    progress_updates = []

    async def capture_progress(progress):
        progress_updates.append(progress)

    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', 'print("STATUS:YEAR:2020:COMPLETE:24/50")'],
        on_progress=capture_progress,
        on_complete=AsyncMock(),
    )

    await executor.start()

    assert len(progress_updates) == 1
    assert progress_updates[0]['year'] == '2020'
    assert progress_updates[0]['completed'] == 24

@pytest.mark.asyncio
async def test_executor_cancellation():
    """Test executor handles cancellation gracefully."""
    executor = PipelineExecutor(
        run_id=1,
        command=['python', '-c', 'import time; time.sleep(10)'],
        on_progress=AsyncMock(),
        on_complete=AsyncMock(),
    )

    # Start executor
    task = asyncio.create_task(executor.start())

    # Wait for process to start
    await asyncio.sleep(0.1)

    # Cancel
    await executor.cancel()

    # Should complete (with cancellation)
    await task

    assert executor._cancelled is True
```

### Mock Subprocess Execution

```python
# tests/api/unit/test_executor_mock.py
import pytest
from unittest.mock import AsyncMock, patch, MagicMock
import asyncio

@pytest.mark.asyncio
async def test_executor_with_mock_subprocess():
    """Test executor with mocked subprocess for fast unit tests."""

    # Create mock process
    mock_process = MagicMock()
    mock_process.returncode = None

    # Mock stdout that yields STATUS messages
    async def mock_readline():
        yield b"STATUS:YEAR:2020:COMPLETE:24/50\n"
        yield b"STATUS:YEAR:2020:COMPLETE:50/50\n"
        yield b""  # EOF

    mock_stdout = MagicMock()
    mock_stdout.__aiter__ = lambda self: mock_readline()
    mock_process.stdout = mock_stdout
    mock_process.stderr = AsyncMock(readline=AsyncMock(return_value=b""))
    mock_process.wait = AsyncMock(return_value=0)

    with patch('asyncio.create_subprocess_exec', return_value=mock_process):
        progress_updates = []

        executor = PipelineExecutor(
            run_id=1,
            command=['python', 'script.py'],
            on_progress=lambda p: progress_updates.append(p),
            on_complete=AsyncMock(),
        )

        await executor.start()

        assert len(progress_updates) == 2
        assert progress_updates[0]['completed'] == 24
        assert progress_updates[1]['completed'] == 50
```

### WebSocket Testing (if implemented later)

```python
# tests/api/integration/test_websocket.py
import pytest
from fastapi.testclient import TestClient

def test_websocket_connection(client):
    """Test WebSocket connection and message flow."""
    with client.websocket_connect("/ws/runs/1") as ws:
        # Send ping
        ws.send_text("ping")

        # Receive pong
        data = ws.receive_text()
        assert data == "pong"

def test_websocket_progress_broadcast(client, db_session, run_factory):
    """Test progress updates are broadcast to WebSocket clients."""
    run = run_factory(status=RunStatus.RUNNING)

    with client.websocket_connect(f"/ws/runs/{run.id}") as ws:
        # Simulate progress update
        from api.utils.ws_manager import manager
        import asyncio

        asyncio.get_event_loop().run_until_complete(
            manager.broadcast_progress(run.id, {'overall_progress': 0.5})
        )

        # Receive progress
        data = ws.receive_json()
        assert data['type'] == 'progress'
        assert data['data']['overall_progress'] == 0.5
```

### Error Scenario Testing

```python
# tests/api/integration/test_error_handling.py
import pytest
from fastapi import HTTPException

def test_create_run_missing_version(client):
    """Test validation error for missing version."""
    response = client.post("/api/v1/runs", json={
        "years": ["2020"],
        "workers": 4
    })

    assert response.status_code == 422
    assert "version" in response.json()["detail"][0]["loc"]

def test_get_nonexistent_run(client):
    """Test 404 for nonexistent run."""
    response = client.get("/api/v1/runs/99999")

    assert response.status_code == 404
    assert "not found" in response.json()["detail"].lower()

def test_start_already_running_run(client, run_factory):
    """Test conflict error for starting already running run."""
    run = run_factory(status=RunStatus.RUNNING)

    response = client.post(f"/api/v1/runs/{run.id}/actions/start")

    assert response.status_code == 409
    assert "already" in response.json()["detail"].lower()

def test_cancel_completed_run(client, run_factory):
    """Test conflict error for cancelling completed run."""
    run = run_factory(status=RunStatus.COMPLETED)

    response = client.post(f"/api/v1/runs/{run.id}/actions/cancel")

    assert response.status_code == 409

def test_database_error_handling(client, db_session):
    """Test handling of database connection errors."""
    with patch('api.database.SessionLocal', side_effect=Exception("DB error")):
        response = client.get("/api/v1/runs")

        assert response.status_code == 500
        # Should not leak internal error details
        assert "Internal server error" in response.json()["detail"]
```

---

## Frontend Testing Patterns (React Testing Library)

### Component Testing Setup

```typescript
// frontend/src/__tests__/setup.ts
import '@testing-library/jest-dom';
import { afterAll, afterEach, beforeAll } from 'vitest';
import { cleanup } from '@testing-library/react';
import { server } from '../mocks/server';

// Start MSW server before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));

// Reset handlers after each test
afterEach(() => {
  cleanup();
  server.resetHandlers();
});

// Close server after all tests
afterAll(() => server.close());
```

### Custom Render with Providers

```typescript
// frontend/src/__tests__/test-utils.tsx
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';
import { ReactElement } from 'react';

// Create fresh QueryClient for each test
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        cacheTime: 0,
        staleTime: 0,
      },
    },
    logger: {
      log: console.log,
      warn: console.warn,
      error: () => {},  // Suppress errors in tests
    },
  });
}

interface WrapperProps {
  children: React.ReactNode;
}

function AllTheProviders({ children }: WrapperProps) {
  const queryClient = createTestQueryClient();

  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        {children}
      </BrowserRouter>
    </QueryClientProvider>
  );
}

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) => render(ui, { wrapper: AllTheProviders, ...options });

export * from '@testing-library/react';
export { customRender as render };
```

### Custom Hook Testing

```typescript
// frontend/src/features/runs/hooks.test.ts
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useRuns, useRun, useCreateRun } from './hooks';
import { server } from '../../mocks/server';
import { rest } from 'msw';

const wrapper = ({ children }) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('useRuns', () => {
  it('should fetch runs successfully', async () => {
    const { result } = renderHook(() => useRuns(), { wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data?.runs).toHaveLength(2);
    expect(result.current.data?.runs[0].version).toBe('v1');
  });

  it('should handle fetch error', async () => {
    server.use(
      rest.get('/api/v1/runs', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ detail: 'Server error' }));
      })
    );

    const { result } = renderHook(() => useRuns(), { wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error?.message).toContain('500');
  });
});

describe('useRun', () => {
  it('should poll while run is running', async () => {
    let pollCount = 0;

    server.use(
      rest.get('/api/v1/runs/1', (req, res, ctx) => {
        pollCount++;
        return res(ctx.json({
          id: 1,
          status: pollCount < 3 ? 'running' : 'completed',
          // ...
        }));
      })
    );

    const { result } = renderHook(() => useRun(1), { wrapper });

    await waitFor(() => expect(result.current.data?.status).toBe('completed'), {
      timeout: 10000,
    });

    expect(pollCount).toBeGreaterThanOrEqual(2);
  });
});
```

### Mock Service Worker (MSW) Setup

```typescript
// frontend/src/mocks/handlers.ts
import { rest } from 'msw';
import { createMockRun, createMockProgress } from '../__tests__/factories';

export const handlers = [
  // List runs
  rest.get('/api/v1/runs', (req, res, ctx) => {
    const status = req.url.searchParams.get('status');
    const runs = [
      createMockRun({ id: 1, version: 'v1', status: 'completed' }),
      createMockRun({ id: 2, version: 'v2', status: 'running' }),
    ];

    const filtered = status
      ? runs.filter(r => r.status === status)
      : runs;

    return res(ctx.json({
      runs: filtered,
      total: filtered.length,
      limit: 50,
      offset: 0,
    }));
  }),

  // Get single run
  rest.get('/api/v1/runs/:id', (req, res, ctx) => {
    const { id } = req.params;
    return res(ctx.json(createMockRun({ id: Number(id) })));
  }),

  // Create run
  rest.post('/api/v1/runs', async (req, res, ctx) => {
    const body = await req.json();
    return res(
      ctx.status(201),
      ctx.json(createMockRun({
        id: Date.now(),
        version: body.version,
        status: 'pending',
        config: body,
      }))
    );
  }),

  // Get progress
  rest.get('/api/v1/runs/:id/progress', (req, res, ctx) => {
    return res(ctx.json(createMockProgress({
      run_id: Number(req.params.id),
    })));
  }),

  // Start run
  rest.post('/api/v1/runs/:id/actions/start', (req, res, ctx) => {
    return res(ctx.json(createMockRun({
      id: Number(req.params.id),
      status: 'running',
    })));
  }),

  // Cancel run
  rest.post('/api/v1/runs/:id/actions/cancel', (req, res, ctx) => {
    return res(ctx.json(createMockRun({
      id: Number(req.params.id),
      status: 'cancelled',
    })));
  }),
];

// frontend/src/mocks/server.ts
import { setupServer } from 'msw/node';
import { handlers } from './handlers';

export const server = setupServer(...handlers);
```

### User Interaction Testing

```typescript
// frontend/src/features/runs/RunForm.test.tsx
import { render, screen, waitFor } from '../../__tests__/test-utils';
import userEvent from '@testing-library/user-event';
import { RunForm } from './RunForm';
import { server } from '../../mocks/server';
import { rest } from 'msw';

describe('RunForm', () => {
  const mockNavigate = vi.fn();

  beforeEach(() => {
    vi.mock('react-router-dom', async () => ({
      ...(await vi.importActual('react-router-dom')),
      useNavigate: () => mockNavigate,
    }));
  });

  it('should submit form with valid data', async () => {
    const user = userEvent.setup();
    let submittedData: any;

    server.use(
      rest.post('/api/v1/runs', async (req, res, ctx) => {
        submittedData = await req.json();
        return res(ctx.status(201), ctx.json({ id: 1, ...submittedData }));
      })
    );

    render(<RunForm />);

    // Fill form
    await user.type(screen.getByLabelText(/version/i), 'test_v1');
    await user.click(screen.getByLabelText(/2020/i));
    await user.click(screen.getByRole('button', { name: /create/i }));

    await waitFor(() => {
      expect(submittedData).toEqual(expect.objectContaining({
        version: 'test_v1',
        years: ['2020'],
      }));
    });

    expect(mockNavigate).toHaveBeenCalledWith('/runs/1');
  });

  it('should show validation error for empty version', async () => {
    const user = userEvent.setup();

    render(<RunForm />);

    // Submit without filling version
    await user.click(screen.getByRole('button', { name: /create/i }));

    expect(screen.getByText(/version is required/i)).toBeInTheDocument();
  });

  it('should disable submit button while loading', async () => {
    const user = userEvent.setup();

    server.use(
      rest.post('/api/v1/runs', async (req, res, ctx) => {
        await new Promise(r => setTimeout(r, 100));
        return res(ctx.status(201), ctx.json({ id: 1 }));
      })
    );

    render(<RunForm />);

    await user.type(screen.getByLabelText(/version/i), 'test');
    await user.click(screen.getByRole('button', { name: /create/i }));

    expect(screen.getByRole('button', { name: /creating/i })).toBeDisabled();
  });
});
```

### Error Boundary Testing

```typescript
// frontend/src/components/ErrorBoundary.test.tsx
import { render, screen } from '@testing-library/react';
import { ErrorBoundary } from './ErrorBoundary';

const ThrowError = () => {
  throw new Error('Test error');
};

describe('ErrorBoundary', () => {
  beforeEach(() => {
    // Suppress console.error for expected errors
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should render children when no error', () => {
    render(
      <ErrorBoundary>
        <div>Child content</div>
      </ErrorBoundary>
    );

    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('should render error UI when child throws', () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText(/something went wrong/i)).toBeInTheDocument();
    expect(screen.getByText(/test error/i)).toBeInTheDocument();
  });

  it('should render custom fallback when provided', () => {
    render(
      <ErrorBoundary fallback={<div>Custom error UI</div>}>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText('Custom error UI')).toBeInTheDocument();
  });
});
```

### Performance Testing (React Profiler)

```typescript
// frontend/src/features/districts/DistrictMap.perf.test.tsx
import { render } from '../../__tests__/test-utils';
import { Profiler, ProfilerOnRenderCallback } from 'react';
import { DistrictMap } from './DistrictMap';
import { createMockDistricts } from '../../__tests__/factories';

describe('DistrictMap Performance', () => {
  it('should render 435 districts in under 500ms', () => {
    const districts = createMockDistricts(435);
    let renderTime = 0;

    const onRender: ProfilerOnRenderCallback = (
      id,
      phase,
      actualDuration
    ) => {
      if (phase === 'mount') {
        renderTime = actualDuration;
      }
    };

    render(
      <Profiler id="DistrictMap" onRender={onRender}>
        <DistrictMap
          districts={districts}
          colorBy="default"
        />
      </Profiler>
    );

    expect(renderTime).toBeLessThan(500);
  });

  it('should not re-render on unrelated prop changes', () => {
    const districts = createMockDistricts(50);
    let renderCount = 0;

    const onRender: ProfilerOnRenderCallback = () => {
      renderCount++;
    };

    const { rerender } = render(
      <Profiler id="DistrictMap" onRender={onRender}>
        <DistrictMap
          districts={districts}
          colorBy="default"
          selectedId={undefined}
        />
      </Profiler>
    );

    // Re-render with same props (should not trigger component re-render)
    rerender(
      <Profiler id="DistrictMap" onRender={onRender}>
        <DistrictMap
          districts={districts}
          colorBy="default"
          selectedId={undefined}
        />
      </Profiler>
    );

    // Only mount render, no update
    expect(renderCount).toBe(1);
  });
});
```

---

## E2E Testing Patterns (Playwright)

### Test Organization and Page Objects

```typescript
// frontend/e2e/pages/RunsPage.ts
import { Page, Locator, expect } from '@playwright/test';

export class RunsPage {
  readonly page: Page;
  readonly runTable: Locator;
  readonly newRunButton: Locator;
  readonly statusFilter: Locator;

  constructor(page: Page) {
    this.page = page;
    this.runTable = page.locator('[data-testid="run-table"]');
    this.newRunButton = page.locator('a[href="/runs/new"]');
    this.statusFilter = page.locator('[data-testid="status-filter"]');
  }

  async goto() {
    await this.page.goto('/runs');
    await this.runTable.waitFor();
  }

  async filterByStatus(status: string) {
    await this.statusFilter.selectOption(status);
    await this.page.waitForResponse('/api/v1/runs*');
  }

  async clickRun(id: number) {
    await this.page.click(`[data-testid="run-row-${id}"]`);
    await this.page.waitForURL(`/runs/${id}`);
  }

  async createRun() {
    await this.newRunButton.click();
    await this.page.waitForURL('/runs/new');
  }
}

// frontend/e2e/pages/RunDetailPage.ts
export class RunDetailPage {
  readonly page: Page;
  readonly progressBar: Locator;
  readonly cancelButton: Locator;
  readonly viewDistrictsButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.progressBar = page.locator('[data-testid="progress-bar"]');
    this.cancelButton = page.locator('button:has-text("Cancel")');
    this.viewDistrictsButton = page.locator('a:has-text("View Districts")');
  }

  async goto(runId: number) {
    await this.page.goto(`/runs/${runId}`);
  }

  async waitForCompletion(timeout = 300000) {
    await expect(this.page.locator('text=completed')).toBeVisible({
      timeout,
    });
  }

  async cancel() {
    await this.cancelButton.click();
    await expect(this.page.locator('text=cancelled')).toBeVisible();
  }
}
```

### Critical User Flow Tests

```typescript
// frontend/e2e/run-lifecycle.spec.ts
import { test, expect } from '@playwright/test';
import { RunsPage } from './pages/RunsPage';
import { RunDetailPage } from './pages/RunDetailPage';

test.describe('Run Lifecycle', () => {
  test('create and monitor run to completion', async ({ page }) => {
    const runsPage = new RunsPage(page);
    const detailPage = new RunDetailPage(page);

    // Navigate to runs list
    await runsPage.goto();

    // Create new run
    await runsPage.createRun();

    // Fill form
    await page.fill('[data-testid="version-input"]', 'e2e_test');
    await page.click('[data-testid="year-2020"]');
    await page.click('[data-testid="state-VT"]');  // Vermont only for speed
    await page.fill('[data-testid="workers-input"]', '1');

    // Submit
    await page.click('button[type="submit"]');

    // Wait for redirect to detail page
    await page.waitForURL(/\/runs\/\d+/);

    // Start the run
    await page.click('button:has-text("Start")');

    // Wait for progress to appear
    await expect(detailPage.progressBar).toBeVisible();

    // Wait for completion (5 minute timeout for VT)
    await detailPage.waitForCompletion(300000);

    // Verify results link appears
    await expect(detailPage.viewDistrictsButton).toBeVisible();
  });

  test('cancel running run', async ({ page }) => {
    // Create and start a run first
    await page.goto('/runs/new');
    await page.fill('[data-testid="version-input"]', 'cancel_test');
    await page.click('[data-testid="year-2020"]');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/runs\/\d+/);
    await page.click('button:has-text("Start")');

    // Wait for running state
    await expect(page.locator('text=running')).toBeVisible();

    // Cancel
    await page.click('button:has-text("Cancel")');

    // Confirm cancellation
    await expect(page.locator('text=cancelled')).toBeVisible();

    // Start button should be disabled
    await expect(page.locator('button:has-text("Start")')).toBeDisabled();
  });
});
```

### Error Scenario Tests

```typescript
// frontend/e2e/error-handling.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Error Handling', () => {
  test('shows error when API is unavailable', async ({ page }) => {
    // Block API requests
    await page.route('**/api/v1/**', route => {
      route.abort('connectionrefused');
    });

    await page.goto('/runs');

    // Should show error state
    await expect(page.locator('text=Error loading')).toBeVisible();

    // Should have retry button
    const retryButton = page.locator('button:has-text("Retry")');
    await expect(retryButton).toBeVisible();
  });

  test('handles run not found', async ({ page }) => {
    await page.goto('/runs/99999');

    await expect(page.locator('text=not found')).toBeVisible();
    await expect(page.locator('a:has-text("Back to runs")')).toBeVisible();
  });

  test('handles validation errors on form', async ({ page }) => {
    await page.goto('/runs/new');

    // Submit empty form
    await page.click('button[type="submit"]');

    // Should show validation errors
    await expect(page.locator('text=Version is required')).toBeVisible();
    await expect(page.locator('text=Select at least one year')).toBeVisible();
  });
});
```

### Visual Regression Testing

```typescript
// frontend/e2e/visual-regression.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Visual Regression', () => {
  test('run list page appearance', async ({ page }) => {
    await page.goto('/runs');
    await page.waitForSelector('[data-testid="run-table"]');

    // Full page screenshot
    await expect(page).toHaveScreenshot('run-list.png', {
      maxDiffPixels: 100,
    });
  });

  test('district map appearance', async ({ page }) => {
    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-container');

    // Wait for map tiles to load
    await page.waitForTimeout(2000);

    await expect(page.locator('.leaflet-container')).toHaveScreenshot(
      'district-map.png',
      { maxDiffPixels: 500 }  // Allow for map tile variations
    );
  });

  test('progress display during run', async ({ page }) => {
    // Mock a running state
    await page.route('**/api/v1/runs/1', route => {
      route.fulfill({
        json: {
          id: 1,
          status: 'running',
          progress: { overall_progress: 0.45 },
        },
      });
    });

    await page.goto('/runs/1');
    await page.waitForSelector('[data-testid="progress-bar"]');

    await expect(page.locator('[data-testid="run-progress"]')).toHaveScreenshot(
      'run-progress.png'
    );
  });
});
```

### Performance Testing

```typescript
// frontend/e2e/performance.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Performance', () => {
  test('initial page load under 3 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/runs');
    await page.waitForSelector('[data-testid="run-table"]');

    const loadTime = Date.now() - startTime;

    expect(loadTime).toBeLessThan(3000);
  });

  test('district map loads 435 districts in under 3 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-interactive');

    const loadTime = Date.now() - startTime;

    expect(loadTime).toBeLessThan(3000);

    // Verify all districts rendered
    const districtCount = await page.locator('.leaflet-interactive').count();
    expect(districtCount).toBe(435);
  });

  test('map maintains 60fps during interaction', async ({ page }) => {
    await page.goto('/runs/1/districts');
    await page.waitForSelector('.leaflet-container');

    // Measure FPS during pan
    const fps = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        let frameCount = 0;
        const startTime = performance.now();

        function countFrame() {
          frameCount++;
          if (performance.now() - startTime < 1000) {
            requestAnimationFrame(countFrame);
          } else {
            resolve(frameCount);
          }
        }

        // Trigger pan
        const map = document.querySelector('.leaflet-container');
        const event = new MouseEvent('mousedown', { clientX: 200, clientY: 200 });
        map?.dispatchEvent(event);

        requestAnimationFrame(countFrame);
      });
    });

    expect(fps).toBeGreaterThan(55);
  });
});
```

---

## Integration Testing Patterns

### CLI Subprocess Integration Tests

```python
# tests/api/integration/test_cli_subprocess.py
import pytest
import asyncio
import subprocess
from pathlib import Path

@pytest.mark.integration
@pytest.mark.timeout(30)
def test_cli_produces_status_output():
    """Test CLI script outputs STATUS messages when configured."""
    env = {
        **os.environ,
        'MULTI_YEAR_SUBPROCESS': '1',
        'TQDM_POSITION': '999',
    }

    result = subprocess.run(
        ['python', 'scripts/pipeline/run_state_redistricting.py',
         '--year', '2020', '--state', 'VT', '--version', 'test', '-p'],
        capture_output=True,
        text=True,
        env=env,
        timeout=30,
    )

    # Should contain STATUS messages
    assert 'STATUS:' in result.stdout

@pytest.mark.integration
@pytest.mark.timeout(300)  # 5 minutes for VT
async def test_full_pipeline_integration():
    """Test full pipeline execution from API to CLI completion."""
    from api.services.pipeline_service import PipelineService
    from api.workers.executor import PipelineExecutor

    service = PipelineService(Path('.'))
    command = service.build_command({
        'version': 'integration_test',
        'years': ['2020'],
        'states': ['VT'],
        'workers': 1,
    })

    progress_updates = []
    final_code = None

    async def on_progress(progress):
        progress_updates.append(progress)

    async def on_complete(code):
        nonlocal final_code
        final_code = code

    executor = PipelineExecutor(
        run_id=1,
        command=command,
        on_progress=on_progress,
        on_complete=on_complete,
    )

    await executor.start()

    assert final_code == 0
    assert len(progress_updates) > 0

    # Verify output files created
    output_dir = Path('outputs/integration_test/2020/states/vermont')
    assert (output_dir / 'data' / 'final_assignments.pkl').exists()
```

### Database Migration Testing

```python
# tests/api/integration/test_migrations.py
import pytest
from alembic.config import Config
from alembic import command
from sqlalchemy import create_engine, text

@pytest.fixture(scope='module')
def migration_engine():
    """Create separate engine for migration testing."""
    engine = create_engine('sqlite:///./test_migrations.db')
    yield engine
    engine.dispose()
    Path('test_migrations.db').unlink(missing_ok=True)

def test_migrations_upgrade_downgrade(migration_engine):
    """Test migrations can upgrade and downgrade cleanly."""
    alembic_cfg = Config("alembic.ini")
    alembic_cfg.set_main_option("sqlalchemy.url", str(migration_engine.url))

    # Upgrade to head
    command.upgrade(alembic_cfg, "head")

    # Verify tables exist
    with migration_engine.connect() as conn:
        result = conn.execute(text(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ))
        tables = [row[0] for row in result]

        assert 'runs' in tables
        assert 'run_years' in tables

    # Downgrade to base
    command.downgrade(alembic_cfg, "base")

    # Verify tables removed
    with migration_engine.connect() as conn:
        result = conn.execute(text(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ))
        tables = [row[0] for row in result]

        assert 'runs' not in tables

def test_data_preserved_across_migrations(migration_engine):
    """Test existing data survives migration upgrades."""
    alembic_cfg = Config("alembic.ini")

    # Upgrade to initial
    command.upgrade(alembic_cfg, "001_initial")

    # Insert test data
    with migration_engine.connect() as conn:
        conn.execute(text(
            "INSERT INTO runs (version, status, config) VALUES ('test', 'pending', '{}')"
        ))
        conn.commit()

    # Upgrade to head
    command.upgrade(alembic_cfg, "head")

    # Verify data still exists
    with migration_engine.connect() as conn:
        result = conn.execute(text("SELECT version FROM runs"))
        row = result.fetchone()
        assert row[0] == 'test'
```

### File I/O Testing

```python
# tests/api/integration/test_file_progress.py
import pytest
import asyncio
from pathlib import Path
from api.utils.file_progress import FileProgressManager

@pytest.fixture
def progress_manager(tmp_path):
    """Create progress manager with temp directory."""
    return FileProgressManager(tmp_path, 'test_version')

@pytest.mark.asyncio
async def test_write_and_read_progress(progress_manager):
    """Test atomic write and read of progress."""
    progress = {
        'overall_progress': 0.5,
        'years': {'2020': {'states_completed': 25}},
    }

    await progress_manager.write(progress)

    read_progress = await progress_manager.read()

    assert read_progress == progress

@pytest.mark.asyncio
async def test_concurrent_writes(progress_manager):
    """Test concurrent writes don't corrupt file."""
    async def write_progress(value):
        await progress_manager.write({'value': value})

    # Write concurrently
    await asyncio.gather(
        write_progress(1),
        write_progress(2),
        write_progress(3),
    )

    # File should be valid (last write wins)
    progress = await progress_manager.read()
    assert progress is not None
    assert 'value' in progress

@pytest.mark.asyncio
async def test_read_nonexistent_file(progress_manager):
    """Test reading nonexistent progress file returns None."""
    progress = await progress_manager.read()
    assert progress is None
```

### Cross-Layer Integration Tests

```python
# tests/api/integration/test_api_to_database.py
import pytest

def test_create_run_persists_to_database(client, db_session):
    """Test run creation is persisted correctly."""
    response = client.post('/api/v1/runs', json={
        'version': 'test',
        'years': ['2020'],
        'workers': 4,
    })

    assert response.status_code == 201
    run_id = response.json()['id']

    # Query database directly
    from api.models import Run
    run = db_session.query(Run).filter(Run.id == run_id).first()

    assert run is not None
    assert run.version == 'test'
    assert run.config['years'] == ['2020']

def test_progress_update_reflects_in_api(client, db_session, run_factory):
    """Test database progress updates appear in API."""
    run = run_factory(status=RunStatus.RUNNING)

    # Update progress in database
    run.progress = {'overall_progress': 0.5}
    db_session.commit()

    # Fetch via API
    response = client.get(f'/api/v1/runs/{run.id}/progress')

    assert response.status_code == 200
    assert response.json()['overall_progress'] == 0.5
```

---

## Test Data Patterns

### Fixture Management

```python
# tests/api/fixtures/__init__.py

# Re-export all fixtures for easy importing
from .run_fixtures import run_factory, completed_run, running_run
from .district_fixtures import district_factory, mock_districts
from .progress_fixtures import mock_progress, progress_updates

# tests/api/fixtures/run_fixtures.py
import pytest
from api.models import Run, RunStatus

@pytest.fixture
def run_factory(db_session):
    """Factory for creating test runs with defaults."""
    created_runs = []

    def _create(**overrides):
        defaults = {
            'version': f'test_{len(created_runs)}',
            'status': RunStatus.PENDING,
            'config': {
                'years': ['2020'],
                'states': None,
                'workers': 4,
                'dpi': 150,
                'partition_mode': 'edge-weighted',
            },
        }
        defaults.update(overrides)

        run = Run(**defaults)
        db_session.add(run)
        db_session.commit()
        created_runs.append(run)
        return run

    yield _create

    # Cleanup after test
    for run in created_runs:
        db_session.delete(run)
    db_session.commit()

@pytest.fixture
def completed_run(run_factory):
    """Pre-made completed run for testing."""
    return run_factory(
        status=RunStatus.COMPLETED,
        output_path='outputs/test/2020',
    )

@pytest.fixture
def running_run(run_factory):
    """Pre-made running run for testing."""
    return run_factory(
        status=RunStatus.RUNNING,
        progress={'overall_progress': 0.5},
    )
```

### Factory Patterns (TypeScript)

```typescript
// frontend/src/__tests__/factories/index.ts
import { faker } from '@faker-js/faker';
import type { Run, District, RunProgress } from '../../types';

// Seeded faker for reproducible tests
faker.seed(12345);

export function createMockRun(overrides: Partial<Run> = {}): Run {
  return {
    id: faker.number.int({ min: 1, max: 1000 }),
    version: faker.string.alphanumeric(5),
    status: 'pending',
    years: ['2020'],
    states: null,
    workers: 4,
    dpi: 150,
    partition_mode: 'edge-weighted',
    created_at: faker.date.recent().toISOString(),
    started_at: null,
    completed_at: null,
    error_message: null,
    ...overrides,
  };
}

export function createMockDistrict(overrides: Partial<District> = {}): District {
  return {
    id: faker.number.int({ min: 1, max: 435 }),
    state: faker.helpers.arrayElement(['california', 'texas', 'florida']),
    district_num: faker.number.int({ min: 1, max: 52 }),
    population: faker.number.int({ min: 700000, max: 800000 }),
    polsby_popper: faker.number.float({ min: 0.1, max: 0.6 }),
    reock: faker.number.float({ min: 0.2, max: 0.5 }),
    partisan_lean: faker.number.float({ min: -0.3, max: 0.3 }),
    geometry: null,  // GeoJSON would go here
    ...overrides,
  };
}

export function createMockDistricts(count: number): District[] {
  return Array.from({ length: count }, (_, i) =>
    createMockDistrict({ id: i + 1 })
  );
}

export function createMockProgress(overrides: Partial<RunProgress> = {}): RunProgress {
  return {
    run_id: 1,
    status: 'running',
    overall_progress: 0.5,
    years: {
      '2020': {
        status: 'running',
        states_completed: 25,
        states_total: 50,
        current_stage: 'redistricting',
      },
    },
    eta_seconds: 3600,
    ...overrides,
  };
}
```

### Mock Data Generation

```python
# tests/api/mocks/status_messages.py
"""Generate realistic STATUS message sequences for testing."""

def generate_status_sequence(states_count=50, stages_per_state=7):
    """Generate a realistic sequence of STATUS messages."""
    messages = []

    for i in range(states_count):
        state_name = f"state_{i}"

        # Emit worker state updates
        for stage in range(1, stages_per_state + 1):
            messages.append(
                f"STATUS:WORKER:2020:0:STATE:{i+1}/{states_count}:{state_name}:STAGE:{stage}/{stages_per_state}:stage_{stage}"
            )

        # Emit year complete
        messages.append(f"STATUS:YEAR:2020:COMPLETE:{i+1}/{states_count}")

    return messages

def generate_error_sequence():
    """Generate STATUS messages that end in error."""
    return [
        "STATUS:WORKER:2020:0:STATE:1/50:vermont:STAGE:1/7:loading_data",
        "STATUS:WORKER:2020:0:STATE:1/50:vermont:STAGE:2/7:redistricting",
        "ERROR: METIS failed for vermont",
        "STATUS:YEAR:2020:ERROR:vermont failed",
    ]

def generate_cancellation_sequence():
    """Generate STATUS messages for a cancelled run."""
    return [
        "STATUS:YEAR:2020:COMPLETE:10/50",
        "STATUS:WORKER:2020:0:STATE:11/50:florida:STAGE:3/7:maps",
        "STATUS:YEAR:2020:CANCELLED",
    ]
```

### Test Database Seeding

```python
# tests/api/fixtures/seed_database.py
"""Seed test database with realistic data."""

from api.models import Run, RunYear, RunStatus
from datetime import datetime, timedelta

def seed_runs(db_session, count=10):
    """Seed database with sample runs."""
    statuses = list(RunStatus)
    runs = []

    for i in range(count):
        status = statuses[i % len(statuses)]
        run = Run(
            version=f'v{i+1}',
            status=status,
            config={
                'years': ['2020'] if i % 2 == 0 else ['2020', '2010'],
                'workers': (i % 4) + 1,
            },
            created_at=datetime.utcnow() - timedelta(days=i),
        )

        if status in [RunStatus.RUNNING, RunStatus.COMPLETED]:
            run.started_at = run.created_at + timedelta(minutes=1)

        if status == RunStatus.COMPLETED:
            run.completed_at = run.started_at + timedelta(hours=2)
            run.output_path = f'outputs/v{i+1}/2020'

        if status == RunStatus.FAILED:
            run.error_message = 'Test error message'

        db_session.add(run)
        runs.append(run)

    db_session.commit()
    return runs
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  backend-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test_apportionment
        ports:
          - 5434:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.13'
          cache: 'pip'

      - name: Install dependencies
        run: |
          cd api
          pip install -r requirements.txt
          pip install pytest pytest-asyncio pytest-cov

      - name: Run unit tests
        run: pytest tests/api/unit/ -v --cov=api --cov-report=xml

      - name: Run integration tests
        run: pytest tests/api/integration/ -v
        env:
          DATABASE_URL: postgresql://test:test@localhost:5434/test_apportionment

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage.xml

  frontend-tests:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Set up Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
          cache-dependency-path: frontend/pnpm-lock.yaml

      - name: Install pnpm
        run: npm install -g pnpm

      - name: Install dependencies
        run: |
          cd frontend
          pnpm install

      - name: Run unit tests
        run: |
          cd frontend
          pnpm test:unit --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./frontend/coverage/lcov.info

  e2e-tests:
    runs-on: ubuntu-latest
    needs: [backend-tests, frontend-tests]

    steps:
      - uses: actions/checkout@v4

      - name: Set up services
        run: docker-compose -f docker-compose.test.yml up -d

      - name: Wait for services
        run: |
          ./scripts/wait-for-it.sh localhost:8002 -t 60
          ./scripts/wait-for-it.sh localhost:3002 -t 60

      - name: Install Playwright
        run: |
          cd frontend
          pnpm exec playwright install --with-deps chromium

      - name: Run E2E tests
        run: |
          cd frontend
          pnpm test:e2e

      - name: Upload test artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: frontend/playwright-report/
```

### Quality Gates

```yaml
# Quality gate configuration
quality-gates:
  unit-test-coverage:
    minimum: 80%
    files:
      - api/**/*.py
      - frontend/src/**/*.ts
      - frontend/src/**/*.tsx
    exclude:
      - '**/__tests__/**'
      - '**/mocks/**'

  integration-test-coverage:
    minimum: 60%

  e2e-tests:
    required:
      - create-run-flow
      - monitor-progress-flow
      - view-results-flow

  performance:
    api-response-time:
      p99: 100ms
    map-load-time:
      max: 3s
    page-load-time:
      max: 3s
```

---

## Anti-Patterns to Avoid

### Testing Anti-Patterns

1. **Testing Implementation Details**
   ```typescript
   // BAD: Testing internal state
   expect(component.state.isLoading).toBe(true);

   // GOOD: Testing user-visible behavior
   expect(screen.getByRole('progressbar')).toBeInTheDocument();
   ```

2. **Brittle Selectors**
   ```typescript
   // BAD: Relying on DOM structure
   wrapper.find('div > div > button').click();

   // GOOD: Using semantic selectors
   screen.getByRole('button', { name: /submit/i }).click();
   ```

3. **Not Waiting for Async Operations**
   ```typescript
   // BAD: No wait
   render(<Component />);
   expect(screen.getByText('data')).toBeInTheDocument();

   // GOOD: Wait for async
   render(<Component />);
   await waitFor(() => {
     expect(screen.getByText('data')).toBeInTheDocument();
   });
   ```

4. **Shared Mutable State Between Tests**
   ```python
   # BAD: Shared state
   runs = []

   def test_one():
       runs.append(create_run())

   def test_two():
       assert len(runs) == 0  # FAILS!

   # GOOD: Fixtures with cleanup
   @pytest.fixture
   def runs(db_session):
       runs = []
       yield runs
       for run in runs:
           db_session.delete(run)
   ```

5. **Over-Mocking**
   ```python
   # BAD: Mock everything
   @patch('api.services.run_service')
   @patch('api.database.get_db')
   @patch('api.models.Run')
   def test_create_run(...):
       pass  # Testing mocks, not code

   # GOOD: Mock at boundaries only
   def test_create_run(client, db_session):
       response = client.post('/api/v1/runs', json={...})
       assert response.status_code == 201
   ```

---

## Quick Reference

### Running Tests

```bash
# Backend tests
pytest tests/api/unit/ -v                    # Unit tests
pytest tests/api/integration/ -v             # Integration tests
pytest tests/api/e2e/ -v --timeout=300       # E2E tests (slow)
pytest tests/api/ --cov=api --cov-report=html  # With coverage

# Frontend tests
cd frontend
pnpm test                                    # All tests
pnpm test:unit                               # Unit tests only
pnpm test:watch                              # Watch mode
pnpm test:coverage                           # With coverage
pnpm test:e2e                                # Playwright E2E
pnpm test:e2e --headed                       # E2E with browser visible
```

### Test Markers

```python
# pytest markers
@pytest.mark.unit           # Unit test
@pytest.mark.integration    # Integration test
@pytest.mark.e2e           # End-to-end test
@pytest.mark.slow          # Slow test (>10s)
@pytest.mark.asyncio       # Async test
@pytest.mark.timeout(300)  # 5 minute timeout
```

### Coverage Targets

| Layer | Target | Actual (Goal) |
|-------|--------|---------------|
| API Unit Tests | 80% | 80%+ |
| API Integration | 60% | 60%+ |
| Frontend Unit | 80% | 80%+ |
| Frontend Integration | 60% | 60%+ |
| E2E Critical Flows | 100% | 100% |

---

## Related Documentation

- [CODING_PATTERNS.md](CODING_PATTERNS.md) - CLI patterns (STATUS protocol)
- [DESIGN_PATTERNS.md](DESIGN_PATTERNS.md) - API and frontend patterns
- [tests/README.md](../tests/README.md) - Existing test infrastructure
- [Wave 9 Plan](waves/WAVE09-api-migration.md) - Wave 9 implementation plan
