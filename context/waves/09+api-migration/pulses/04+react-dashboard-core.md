---
wave_uuid: 3861b5
slug: react-dashboard-core
uuid: ff7bf8
---
# E63: React Dashboard Core

**Status**: PLANNED
**Wave**: Wave 9 (API-MIGRATION)
**Priority**: High
**Estimated Complexity**: High
**Estimated Hours**: 20-24 hours
**Created**: 2026-01-25

---

## Description

Build the React dashboard core with UI component library, run management pages, and progress display. Following the Senior Designer's recommendation, this enhancement starts with a reusable UI component library before building feature components. Uses React Query for server state management and polling-based progress updates.

---

## Tasks

### Phase 1: Shared UI Components Integration (2-3 hours)

- [ ] Install shared UI packages from App Manager
  ```bash
  cd frontend
  pnpm add @common/ui@workspace:* @common/types@workspace:* @common/api-client@workspace:*
  ```
- [ ] Configure symlinks to App Manager packages
  - `pnpm link ../../appmanager/packages/common-ui`
  - `pnpm link ../../appmanager/packages/common-types`
  - `pnpm link ../../appmanager/packages/common-api-client`
- [ ] **Use existing components from @common/ui**:
  ```typescript
  import { Button, LoadingSpinner, StatusIndicator } from '@common/ui';

  // Available components (from appmanager/packages/common-ui):
  // - Button: Primary/secondary variants with loading states
  // - LoadingSpinner: Consistent loading indicator
  // - StatusIndicator: Status badges (running, completed, failed)
  ```
- [ ] Build **app-specific** components only (not generic UI):
  ```typescript
  // frontend/src/components/app/
  // - RunCard.tsx          # Display run summary card
  // - ProgressBar.tsx      # Show pipeline progress
  // - DistrictTable.tsx    # Show district data
  // - StateSelector.tsx    # Multi-state selection
  ```
- [ ] Create Tailwind config extending App Manager color palette

### Phase 2: Navigation & Layout (2-3 hours)

- [ ] Create `frontend/src/components/layout/` directory
  ```typescript
  // Layout.tsx - Main application shell
  function Layout({ children }: { children: React.ReactNode }) {
    return (
      <div className="min-h-screen bg-gray-50">
        <Navbar />
        <main className="max-w-7xl mx-auto px-4 py-8">
          {children}
        </main>
      </div>
    );
  }

  // Navbar.tsx - Top navigation
  function Navbar() {
    return (
      <nav className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <Link to="/">Redistricting Dashboard</Link>
            </div>
            <div className="flex items-center space-x-4">
              <NavLink to="/runs">Runs</NavLink>
              <NavLink to="/runs/new">New Run</NavLink>
            </div>
          </div>
        </div>
      </nav>
    );
  }
  ```
- [ ] Set up React Router with routes:
  - `/` - Dashboard home (redirect to /runs)
  - `/runs` - Run list
  - `/runs/new` - Create new run
  - `/runs/:id` - Run detail with progress
- [ ] Add breadcrumb navigation component

### Phase 3: API Client & React Query Setup (3-4 hours)

- [ ] Create `frontend/src/api/client.ts` with Axios instance
  ```typescript
  import axios from 'axios';

  export const apiClient = axios.create({
    baseURL: import.meta.env.VITE_API_URL || 'http://localhost:8002',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // Request/response interceptors for error handling
  apiClient.interceptors.response.use(
    response => response,
    error => {
      // Handle common errors (401, 500, network)
      return Promise.reject(error);
    }
  );
  ```
- [ ] Create `frontend/src/api/runs.ts` with run API functions
  ```typescript
  export const runApi = {
    list: (filters?: RunFilters) =>
      apiClient.get<RunListResponse>('/api/v1/runs', { params: filters }),

    get: (id: number) =>
      apiClient.get<RunDetailResponse>(`/api/v1/runs/${id}`),

    create: (data: RunCreate) =>
      apiClient.post<RunResponse>('/api/v1/runs', data),

    start: (id: number) =>
      apiClient.post<RunResponse>(`/api/v1/runs/${id}/actions/start`),

    cancel: (id: number) =>
      apiClient.post<RunResponse>(`/api/v1/runs/${id}/actions/cancel`),

    getProgress: (id: number) =>
      apiClient.get<RunProgressResponse>(`/api/v1/runs/${id}/progress`),
  };
  ```
- [ ] Set up React Query provider with default options
  ```typescript
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30000,
        retry: 3,
        refetchOnWindowFocus: false,
      },
    },
  });
  ```
- [ ] Create custom hooks for data fetching
  ```typescript
  // frontend/src/features/runs/hooks.ts
  export function useRuns(filters?: RunFilters) {
    return useQuery({
      queryKey: ['runs', filters],
      queryFn: () => runApi.list(filters),
    });
  }

  export function useRun(id: number) {
    return useQuery({
      queryKey: ['runs', id],
      queryFn: () => runApi.get(id),
      refetchInterval: (data) =>
        data?.status === 'running' ? 2000 : false,
    });
  }

  export function useCreateRun() {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: runApi.create,
      onSuccess: () => queryClient.invalidateQueries(['runs']),
    });
  }
  ```

### Phase 4: Run List Page (3-4 hours)

- [ ] Create `frontend/src/features/runs/RunList.tsx`
  ```typescript
  function RunList() {
    const [filters, setFilters] = useState<RunFilters>({});
    const { data, isLoading, error } = useRuns(filters);

    if (isLoading) return <Spinner size="lg" />;
    if (error) return <ErrorBanner error={error} />;

    return (
      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <h1 className="text-2xl font-bold">Pipeline Runs</h1>
          <Link to="/runs/new">
            <Button variant="primary">New Run</Button>
          </Link>
        </div>

        <RunFilters filters={filters} onChange={setFilters} />

        <RunTable
          runs={data.runs}
          onRowClick={(run) => navigate(`/runs/${run.id}`)}
        />

        <Pagination
          total={data.total}
          limit={data.limit}
          offset={data.offset}
          onChange={handlePageChange}
        />
      </div>
    );
  }
  ```
- [ ] Create `RunFilters` component (status, year, version dropdowns)
- [ ] Create `RunTable` component with sortable columns
- [ ] Add pagination support
- [ ] Style status badges (pending, running, completed, failed, cancelled)

### Phase 5: Run Detail Page with Progress (4-5 hours)

- [ ] Create `frontend/src/features/runs/RunDetail.tsx`
  ```typescript
  function RunDetail() {
    const { id } = useParams<{ id: string }>();
    const { data: run, isLoading, error } = useRun(Number(id));

    if (isLoading) return <Spinner size="lg" />;
    if (error) return <ErrorBanner error={error} />;

    return (
      <div className="space-y-6">
        <RunHeader run={run} />

        {run.status === 'running' && (
          <RunProgress run={run} />
        )}

        <RunConfig config={run.config} />

        {run.status === 'completed' && (
          <RunResults run={run} />
        )}

        {run.error_message && (
          <ErrorBanner message={run.error_message} />
        )}
      </div>
    );
  }
  ```
- [ ] Create `RunProgress` component with visual progress bars
  ```typescript
  function RunProgress({ run }: { run: RunDetail }) {
    // Poll progress every 2 seconds
    const { data: progress } = useQuery({
      queryKey: ['runs', run.id, 'progress'],
      queryFn: () => runApi.getProgress(run.id),
      refetchInterval: 2000,
      enabled: run.status === 'running',
    });

    return (
      <Card title="Progress">
        <div className="space-y-4">
          {/* Overall progress bar */}
          <ProgressBar
            value={progress.overall_progress * 100}
            label={`${Math.round(progress.overall_progress * 100)}%`}
          />

          {/* ETA */}
          <p className="text-sm text-gray-500">
            Estimated time remaining: {formatDuration(progress.eta_seconds)}
          </p>

          {/* Per-year progress */}
          {Object.entries(progress.years).map(([year, yearProgress]) => (
            <YearProgress key={year} year={year} progress={yearProgress} />
          ))}
        </div>
      </Card>
    );
  }
  ```
- [ ] Create `YearProgress` component showing states completed
- [ ] Create `RunActions` component (Start, Cancel buttons)
- [ ] Add run duration and timestamps display

### Phase 6: Run Creation Form (3-4 hours)

- [ ] Create `frontend/src/features/runs/RunForm.tsx`
  ```typescript
  function RunForm() {
    const navigate = useNavigate();
    const createRun = useCreateRun();
    const { data: stateConfig } = useStateConfig();

    const [formData, setFormData] = useState<RunCreate>({
      version: '',
      years: ['2020'],
      states: null, // null = all states
      workers: 4,
      dpi: 150,
      partition_mode: 'edge-weighted',
    });

    const handleSubmit = async (e: FormEvent) => {
      e.preventDefault();
      const run = await createRun.mutateAsync(formData);
      navigate(`/runs/${run.id}`);
    };

    return (
      <form onSubmit={handleSubmit} className="space-y-6">
        <Card title="Create New Run">
          {/* Version input */}
          <Input
            label="Version"
            value={formData.version}
            onChange={(v) => setFormData({ ...formData, version: v })}
            placeholder="v1, test, experiment_1"
            required
          />

          {/* Years multi-select */}
          <MultiSelect
            label="Census Years"
            options={['2020', '2010', '2000']}
            value={formData.years}
            onChange={(years) => setFormData({ ...formData, years })}
          />

          {/* States selector (optional) */}
          <StateSelector
            label="States (optional, leave empty for all)"
            states={stateConfig?.states}
            value={formData.states}
            onChange={(states) => setFormData({ ...formData, states })}
          />

          {/* Workers slider */}
          <Slider
            label="Parallel Workers"
            min={1}
            max={16}
            value={formData.workers}
            onChange={(workers) => setFormData({ ...formData, workers })}
          />

          {/* Submit button */}
          <Button
            type="submit"
            variant="primary"
            loading={createRun.isLoading}
          >
            Create Run
          </Button>
        </Card>
      </form>
    );
  }
  ```
- [ ] Add form validation with helpful error messages
- [ ] Create state selector with search and multi-select
- [ ] Add version name validation (check for existing)
- [ ] Show disk space warning for large runs

### Phase 7: Error Handling & Loading States (2-3 hours)

- [ ] Create `ErrorBoundary` component for catching render errors
  ```typescript
  class ErrorBoundary extends React.Component<Props, State> {
    static getDerivedStateFromError(error: Error): State {
      return { hasError: true, error };
    }

    render() {
      if (this.state.hasError) {
        return (
          <Card className="bg-red-50 border-red-200">
            <h2 className="text-red-800 font-semibold">Something went wrong</h2>
            <p className="text-red-600">{this.state.error?.message}</p>
            <Button onClick={() => window.location.reload()}>
              Reload Page
            </Button>
          </Card>
        );
      }
      return this.props.children;
    }
  }
  ```
- [ ] Create loading skeleton components for better UX
- [ ] Add retry buttons on error states
- [ ] Handle network errors gracefully
- [ ] Add toast notifications for actions (run created, cancelled)

### Phase 8: Testing (3-4 hours)

- [ ] Unit tests for UI components
  ```typescript
  describe('Button', () => {
    it('renders with correct variant styles', () => {
      render(<Button variant="primary">Click me</Button>);
      expect(screen.getByRole('button')).toHaveClass('bg-blue-600');
    });

    it('shows spinner when loading', () => {
      render(<Button loading>Loading</Button>);
      expect(screen.getByTestId('spinner')).toBeInTheDocument();
    });
  });
  ```
- [ ] Integration tests for pages with MSW for API mocking
  ```typescript
  describe('RunList', () => {
    it('displays runs from API', async () => {
      server.use(
        rest.get('/api/v1/runs', (req, res, ctx) => {
          return res(ctx.json({ runs: mockRuns, total: 2 }));
        })
      );

      render(<RunList />);

      await waitFor(() => {
        expect(screen.getByText('v1')).toBeInTheDocument();
      });
    });
  });
  ```
- [ ] Add Playwright E2E tests for critical flows
  - Create run flow
  - View run progress
  - Filter runs list

---

## Architecture Changes

**New Files**:
```
frontend/src/
├── components/
│   ├── ui/
│   │   ├── Button.tsx
│   │   ├── Card.tsx
│   │   ├── Table.tsx
│   │   ├── Input.tsx
│   │   ├── Select.tsx
│   │   ├── Badge.tsx
│   │   ├── Spinner.tsx
│   │   ├── ErrorBanner.tsx
│   │   ├── ProgressBar.tsx
│   │   └── index.ts
│   └── layout/
│       ├── Layout.tsx
│       ├── Navbar.tsx
│       └── Breadcrumbs.tsx
├── features/
│   └── runs/
│       ├── RunList.tsx
│       ├── RunDetail.tsx
│       ├── RunForm.tsx
│       ├── RunProgress.tsx
│       ├── RunTable.tsx
│       ├── RunFilters.tsx
│       ├── components/
│       │   ├── YearProgress.tsx
│       │   ├── StateSelector.tsx
│       │   └── RunActions.tsx
│       └── hooks.ts
├── api/
│   ├── client.ts
│   └── runs.ts
├── types/
│   ├── run.ts
│   └── api.ts
└── utils/
    ├── format.ts
    └── constants.ts
```

**Related DESIGN_PATTERNS.md Sections**:
- Frontend Patterns: Component Architecture
- Frontend Patterns: Data Fetching Pattern
- Frontend Patterns: Error Boundaries and Loading States

---

## Testing Strategy

**Test Coverage Target**: 80% minimum

### Unit Tests (18-22 tests)
- UI components (Button, Card, Table, etc.)
- Utility functions (formatters, validators)
- Custom hooks with mock queries
- Accessibility attributes (ARIA labels, roles)

### Integration Tests (10-12 tests)
- RunList page with mocked API
- RunDetail page with progress updates
- RunForm validation and submission
- Error handling scenarios
- Polling state transitions (pending -> running -> completed/failed)
- Error recovery tests (network failures, retry logic)

### E2E Tests (5-7 tests)
- Full run creation flow
- Run progress monitoring
- Run cancellation flow
- Error recovery flows
- Accessibility verification

### Manual Testing
1. Create new run with various configurations
2. Monitor progress during execution
3. Cancel running run
4. Filter and paginate runs list
5. Test error scenarios (API down, invalid data)
6. Test keyboard navigation
7. Test screen reader compatibility

### Test Effort Estimate
- Unit tests: 15-20 component tests
- Integration tests: 10-12 integration tests
- E2E tests: 3-4 E2E tests
- Total estimated effort: 20% of wave testing time

---

## Testing Assessment (from Senior Tester)

| Attribute | Value |
|-----------|-------|
| **Risk Rating** | MEDIUM |
| **Original Assessment** | ADEQUATE with recommendations |
| **Testing Priority** | 3 |
| **Recommended Effort** | 20% of total testing effort |

### Gap Analysis

| Test Type | Originally Proposed | Recommended | Gap |
|-----------|---------------------|-------------|-----|
| Unit | 15-20 | 18-22 | Add accessibility tests |
| Integration | 8-10 | 10-12 | Add polling state transition tests |
| E2E | 3-5 | 5-7 | Add error recovery flows |

### Testing Gaps Identified (from Senior Tester)

**1. Missing: Polling State Transitions**
```typescript
describe('Progress Polling', () => {
  it('should start polling when run status is running', async () => {
    server.use(
      rest.get('/api/v1/runs/1', (req, res, ctx) => {
        return res(ctx.json({ id: 1, status: 'running' }));
      })
    );

    render(<RunDetail runId={1} />);

    // Verify polling started (refetchInterval is active)
    await waitFor(() => {
      expect(mockApiCalls.length).toBeGreaterThan(1);
    }, { timeout: 5000 });
  });

  it('should stop polling when run status changes to completed', async () => {
    let callCount = 0;
    server.use(
      rest.get('/api/v1/runs/1', (req, res, ctx) => {
        callCount++;
        return res(ctx.json({
          id: 1,
          status: callCount < 3 ? 'running' : 'completed'
        }));
      })
    );

    render(<RunDetail runId={1} />);

    // Wait for completion
    await waitFor(() => {
      expect(screen.getByText(/completed/i)).toBeInTheDocument();
    });

    // Wait and verify polling stopped
    const callsAtComplete = callCount;
    await new Promise(r => setTimeout(r, 5000));
    expect(callCount).toBe(callsAtComplete);  // No new calls
  });
});
```

**2. Missing: Accessibility Tests**
```typescript
import { axe, toHaveNoViolations } from 'jest-axe';
expect.extend(toHaveNoViolations);

describe('Accessibility', () => {
  it('RunList should have no accessibility violations', async () => {
    const { container } = render(<RunList />);
    await waitFor(() => screen.getByRole('table'));

    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it('Progress bar should have proper ARIA attributes', () => {
    render(<ProgressBar value={50} max={100} />);

    const progressbar = screen.getByRole('progressbar');
    expect(progressbar).toHaveAttribute('aria-valuenow', '50');
    expect(progressbar).toHaveAttribute('aria-valuemax', '100');
  });

  it('should support keyboard navigation', async () => {
    render(<RunList />);
    await waitFor(() => screen.getByRole('table'));

    // Tab to first row
    await userEvent.tab();
    expect(document.activeElement).toHaveRole('row');
  });
});
```

**3. Missing: Optimistic Update Rollback Test**
```typescript
it('should rollback optimistic update on error', async () => {
  server.use(
    rest.post('/api/v1/runs/1/actions/cancel', (req, res, ctx) => {
      return res(ctx.status(500), ctx.json({ detail: 'Server error' }));
    })
  );

  render(<RunDetail runId={1} />);

  // Run should be running initially
  await waitFor(() => screen.getByText(/running/i));

  // Click cancel
  await userEvent.click(screen.getByRole('button', { name: /cancel/i }));

  // Should briefly show cancelled (optimistic)
  // Then revert to running (on error)
  await waitFor(() => {
    expect(screen.getByText(/running/i)).toBeInTheDocument();
  });

  // Should show error message
  expect(screen.getByText(/error/i)).toBeInTheDocument();
});
```

**4. Missing: Network Disconnection Test**
```typescript
it('should handle network disconnection gracefully', async () => {
  render(<RunList />);
  await waitFor(() => screen.getByRole('table'));

  // Simulate network disconnect
  server.use(
    rest.get('/api/v1/runs', (req, res) => {
      return res.networkError('Network error');
    })
  );

  // Trigger refetch
  await userEvent.click(screen.getByRole('button', { name: /refresh/i }));

  // Should show offline/error state
  await waitFor(() => {
    expect(screen.getByText(/connection error|offline/i)).toBeInTheDocument();
  });
});
```

### Quality Gates for This Enhancement

- [ ] All UI components render correctly with all prop variations
- [ ] Progress polling starts/stops based on run status
- [ ] Error boundaries catch and display component failures
- [ ] Network errors handled gracefully with retry options
- [ ] Keyboard navigation works for all interactive elements
- [ ] No accessibility violations (axe audit passes)
- [ ] Test coverage >= 80%

---

## Dependencies

**Prerequisites**:
- E60 (Project Setup) - REQUIRED (frontend scaffold)
- E61 (Run Management API) - REQUIRED (API endpoints)
- E62 (Pipeline Execution) - REQUIRED (progress endpoint)
- **App Manager shared packages** (appmanager\packages\)

**Shared Packages** (workspace dependencies):
- `@common/ui@workspace:*` - Button, LoadingSpinner, StatusIndicator
- `@common/types@workspace:*` - TypeScript type definitions
- `@common/api-client@workspace:*` - Axios instance and React Query setup

**Node Packages** (add to package.json):
- @tanstack/react-query (may already be in @common/api-client)
- axios (may already be in @common/api-client)
- react-router-dom
- msw (for testing)
- @testing-library/react
- @playwright/test

**Important**: Use `pnpm link` to symlink App Manager packages during development

---

## Success Criteria

- [ ] UI component library complete with all components
- [ ] Run list displays with filtering and pagination
- [ ] Run creation form validates and submits correctly
- [ ] Progress displays with 2-second polling
- [ ] Progress bars show per-year and overall progress
- [ ] Error boundaries catch and display errors gracefully
- [ ] Loading states provide good UX
- [ ] All tests pass (unit + integration)
- [ ] E2E tests pass for critical flows

---

## Design Notes (from Senior Designer)

### Shared UI Components from App Manager
**Use existing components from @common/ui** instead of building from scratch:
- **Rationale**: Consistency across all apps (TCM, NHL, Apportionment), faster development
- **Available**: Button, LoadingSpinner, StatusIndicator
- **Build app-specific only**: RunCard, ProgressBar, DistrictTable, StateSelector
- **Pattern**: Import from @common/ui, extend with composition for app-specific needs

### Container/Presentational Split
Separate data fetching from presentation:
- **Containers**: Handle data fetching, state management
- **Presentational**: Pure components, receive props
- **Benefit**: Easier testing, reusability

### Polling for Progress
Use React Query with `refetchInterval`:
```typescript
useQuery({
  queryKey: ['runs', id, 'progress'],
  refetchInterval: (data) => data?.status === 'running' ? 2000 : false,
});
```
- **Rationale**: Simpler than WebSocket for 1-4 hour runs
- **Automatic**: Stops polling when run completes

---

## Related Documentation

- [Wave 9 Plan](../waves/WAVE09-api-migration.md)
- [Design Patterns](../DESIGN_PATTERNS.md) - Frontend Patterns section
- [Senior Designer Review](../waves/wave09/01_senior_designer_review.md) - Frontend recommendations
- [Senior Engineer Review](../waves/wave09/02_senior_engineer_review.md) - React patterns and testing
- [Senior Tester Review](../waves/wave09/03_senior_tester_review.md) - Frontend testing recommendations
- [TESTING_PATTERNS.md](../TESTING_PATTERNS.md) - React Testing Library patterns, MSW setup

---

## Engineering Notes (from Senior Engineer)

### Risk Assessment: MEDIUM

Standard React patterns with medium risk around state synchronization bugs.

### Parallelization Opportunity

**This enhancement can be developed in parallel with E62** by different developers:
- Developer A: E62 (backend execution)
- Developer B: E63 (frontend, mock API responses using MSW)

This reduces calendar time from 6 weeks to 4-5 weeks.

### React Query Pattern Recommendations

Use React Query with proper cache invalidation:

```typescript
// Configure QueryClient with sensible defaults
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30000,        // Data fresh for 30 seconds
      retry: 3,                // Retry failed requests 3 times
      refetchOnWindowFocus: false,  // Don't refetch when tab gains focus
    },
  },
});

// Custom hook with conditional polling
export function useRun(id: number) {
  return useQuery({
    queryKey: ['runs', id],
    queryFn: () => runApi.get(id),
    // Only poll while running
    refetchInterval: (data) =>
      data?.status === 'running' ? 2000 : false,
  });
}

// Mutation with cache invalidation
export function useCreateRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: runApi.create,
    onSuccess: () => {
      // Invalidate runs list to show new run
      queryClient.invalidateQueries({ queryKey: ['runs'] });
    },
  });
}

// Cancel mutation with optimistic update
export function useCancelRun() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: runApi.cancel,
    onMutate: async (runId) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['runs', runId] });

      // Optimistically update to cancelled state
      const previousRun = queryClient.getQueryData(['runs', runId]);
      queryClient.setQueryData(['runs', runId], (old) => ({
        ...old,
        status: 'cancelled',
      }));

      return { previousRun };
    },
    onError: (err, runId, context) => {
      // Rollback on error
      queryClient.setQueryData(['runs', runId], context.previousRun);
    },
  });
}
```

### Container/Presentational Component Pattern

Per engineer recommendation, separate data fetching from presentation:

```typescript
// Container: handles data fetching, state management
// frontend/src/features/runs/RunListContainer.tsx
function RunListContainer() {
  const [filters, setFilters] = useState<RunFilters>({});
  const { data, isLoading, error } = useRuns(filters);

  if (isLoading) return <RunListSkeleton />;
  if (error) return <ErrorBanner error={error} onRetry={() => refetch()} />;

  return (
    <RunListPresenter
      runs={data.runs}
      filters={filters}
      onFiltersChange={setFilters}
      pagination={{ total: data.total, limit: data.limit, offset: data.offset }}
    />
  );
}

// Presentational: pure component, receives props
// frontend/src/features/runs/RunListPresenter.tsx
interface RunListPresenterProps {
  runs: Run[];
  filters: RunFilters;
  onFiltersChange: (filters: RunFilters) => void;
  pagination: PaginationInfo;
}

function RunListPresenter({ runs, filters, onFiltersChange, pagination }: RunListPresenterProps) {
  return (
    <div className="space-y-4">
      <RunFilters value={filters} onChange={onFiltersChange} />
      <RunTable runs={runs} />
      <Pagination {...pagination} />
    </div>
  );
}
```

**Benefits**:
- Easier testing (presentational components are pure)
- Reusability (presentational components can be used with different data sources)
- Clear separation of concerns

### Error Boundary Implementation

Per engineer recommendation, implement error boundaries:

```typescript
// frontend/src/components/ErrorBoundary.tsx
import React from 'react';
import { Card, Button } from '@/components/ui';

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo);
    // Could send to error reporting service
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <Card className="bg-red-50 border-red-200 p-6">
          <h2 className="text-red-800 font-semibold text-lg">
            Something went wrong
          </h2>
          <p className="text-red-600 mt-2">
            {this.state.error?.message || 'An unexpected error occurred'}
          </p>
          <Button
            variant="secondary"
            className="mt-4"
            onClick={() => window.location.reload()}
          >
            Reload Page
          </Button>
        </Card>
      );
    }

    return this.props.children;
  }
}

// Usage in App.tsx
function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
```

### Polling Pattern for Progress Updates

Per engineer recommendation, use React Query's `refetchInterval`:

```typescript
// frontend/src/features/runs/RunProgress.tsx
function RunProgress({ runId }: { runId: number }) {
  const { data: progress, isLoading } = useQuery({
    queryKey: ['runs', runId, 'progress'],
    queryFn: () => runApi.getProgress(runId),
    // Poll every 2 seconds while run is active
    refetchInterval: 2000,
    // Stop polling when component unmounts
    refetchIntervalInBackground: false,
  });

  // Stop polling when run completes
  useEffect(() => {
    if (progress?.status === 'completed' || progress?.status === 'failed') {
      // Invalidate to stop polling
      queryClient.invalidateQueries({ queryKey: ['runs', runId, 'progress'] });
    }
  }, [progress?.status]);

  if (isLoading) return <ProgressSkeleton />;

  return (
    <div className="space-y-4">
      <OverallProgressBar value={progress.overall_progress * 100} />
      <ETADisplay seconds={progress.eta_seconds} />
      {Object.entries(progress.years).map(([year, yearProgress]) => (
        <YearProgress key={year} year={year} progress={yearProgress} />
      ))}
    </div>
  );
}
```

### Testing with MSW (Mock Service Worker)

Per engineer recommendation, use MSW for API mocking:

```typescript
// frontend/src/mocks/handlers.ts
import { rest } from 'msw';

export const handlers = [
  rest.get('/api/v1/runs', (req, res, ctx) => {
    return res(ctx.json({
      runs: [
        { id: 1, version: 'v1', status: 'completed', years: ['2020'] },
        { id: 2, version: 'v2', status: 'running', years: ['2020', '2010'] },
      ],
      total: 2,
      limit: 50,
      offset: 0,
    }));
  }),

  rest.get('/api/v1/runs/:id/progress', (req, res, ctx) => {
    return res(ctx.json({
      run_id: Number(req.params.id),
      status: 'running',
      overall_progress: 0.45,
      years: {
        '2020': { states_completed: 24, states_total: 50, status: 'running' },
      },
      eta_seconds: 3600,
    }));
  }),
];

// frontend/src/mocks/server.ts
import { setupServer } from 'msw/node';
import { handlers } from './handlers';
export const server = setupServer(...handlers);

// frontend/src/setupTests.ts
import { server } from './mocks/server';
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### Frontend Testing Pyramid

Per engineer recommendation:

```
         /\
        /E2E\        3-5 tests (create run flow, progress monitoring)
       /------\
      /Integ   \     8-10 tests (pages with mocked API via MSW)
     /----------\
    /    Unit    \   15-20 tests (UI components, utilities, hooks)
   /--------------\
```

### Development Tools

Per engineer recommendation:
- Use `pnpm` over npm (faster, better disk usage)
- Use `@tanstack/react-query-devtools` for debugging queries
- Use `@playwright/test` for E2E tests
- Use `msw` for API mocking

---

**E63 Summary**: Build React dashboard with UI component library, run management pages, polling-based progress display, and comprehensive error handling.
