# Wave 9: API & Dashboard Quick Start

Quick start guide for developing and running the Apportionment API and Dashboard (Wave 9).

## App Manager Integration

Wave 9 integrates with the **unified App Manager system** at `C:\src\appmanager`. This provides:
- **Shared UI components** (@common/ui) - Button, LoadingSpinner, StatusIndicator
- **Shared backend utilities** (common-backend-utils) - Exceptions, database patterns
- **Centralized PM2** process management
- **Unified dashboard** at http://localhost:9000

## Prerequisites

- **Docker Desktop** (for PostgreSQL database)
- **Python 3.11+** with Poetry
- **Node.js 18+** with pnpm
- **PM2** (`npm install -g pm2`)
- **App Manager setup complete** (see `C:\src\appmanager\EXECUTE_APPMANAGER.md`)

## Option 1: PM2 via App Manager (Recommended)

Start all services from the centralized App Manager:

```bash
# Start databases
cd C:\src\appmanager
docker compose up -d

# Start apportionment services (or all services)
pm2 start infrastructure/ecosystem.config.js --only apportionment-backend
pm2 start infrastructure/ecosystem.config.js --only apportionment-frontend

# Or start everything
pm2 start infrastructure/ecosystem.config.js

# View status
pm2 list
pm2 logs apportionment-backend
pm2 logs apportionment-frontend
```

Services will be available at:
- **API**: http://localhost:8002
- **API Docs**: http://localhost:8002/docs
- **Frontend**: http://localhost:3002
- **PostgreSQL**: localhost:5434
- **App Manager**: http://localhost:9000

## Option 2: Local Development (Standalone)

### Backend API

```bash
# 1. Install Python dependencies
cd backend
poetry install

# 2. Set up environment
cp .env.example .env
# Edit .env if needed (database URL, CORS, etc.)

# 3. Run API server
poetry run uvicorn app.main:app --reload --host 127.0.0.1 --port 8002
```

API will be available at http://localhost:8002

### Frontend Dashboard

```bash
# 1. Install Node dependencies
cd frontend
pnpm install

# 2. Link shared packages from App Manager
pnpm link ../../appmanager/packages/common-ui
pnpm link ../../appmanager/packages/common-types
pnpm link ../../appmanager/packages/common-api-client

# 3. Set up environment
cp .env.example .env

# 4. Run dev server
pnpm dev
```

Frontend will be available at http://localhost:3002

## Testing

### Backend Tests

```bash
cd backend
poetry run pytest tests/ -v

# With coverage
poetry run pytest tests/ --cov=app --cov-report=html
```

### Frontend Tests

```bash
cd frontend
pnpm lint
```

## Verification Checklist

After starting the services, verify:

- [ ] API health endpoint: http://localhost:8002/health
- [ ] API version: http://localhost:8002/version
- [ ] API docs: http://localhost:8002/docs
- [ ] Frontend loads: http://localhost:3002
- [ ] Frontend shows API connection status using shared @common/ui components
- [ ] App Manager dashboard shows apportionment services: http://localhost:9000

## Shared Package Integration

This project uses shared packages from App Manager:

**Backend** (`backend/pyproject.toml`):
```toml
common-backend-utils = {path = "../../appmanager/packages/common-backend-utils", develop = true}
```

**Frontend** (`frontend/package.json`):
```json
"pnpm": {
  "overrides": {
    "@common/ui": "link:../../appmanager/packages/common-ui",
    "@common/types": "link:../../appmanager/packages/common-types"
  }
}
```

**Usage**:
```typescript
// Frontend
import { Button, LoadingSpinner, StatusIndicator } from '@common/ui';
```

```python
# Backend
from common_backend_utils.exceptions import NotFoundException, ValidationException
```

## Troubleshooting

### Port Conflicts

If ports 8002 or 3002 are in use, check PM2:
```bash
pm2 list
pm2 stop apportionment-backend
pm2 stop apportionment-frontend
```

### Database Connection Failed

- **Docker**: Ensure PostgreSQL container is running
  ```bash
  docker ps | grep postgres
  cd C:\src\appmanager && docker compose up -d
  ```
- **Local**: Verify PostgreSQL is running on port 5434

### Shared Package Import Errors

If you get import errors for @common/ui or common-backend-utils:

**Frontend**:
```bash
cd frontend
pnpm link ../../appmanager/packages/common-ui
pnpm link ../../appmanager/packages/common-types
```

**Backend**:
```bash
cd backend
poetry install  # Reinstalls path dependencies
```

### API Not Accessible from Frontend

- Verify API is running on port 8002
- Check CORS origins in `backend/.env` include frontend URL
- In dev mode, Vite proxy should handle API requests

## Next Steps

After completing Enhancement 60 setup:

1. **Enhancement 61**: Run management API with database schema
2. **Enhancement 62**: Pipeline execution engine
3. **Enhancement 63**: React dashboard core features (using @common/ui)
4. **Enhancement 64**: District visualization with maps

## Documentation

- Backend README: `backend/README.md`
- Frontend README: `frontend/README.md`
- Wave 9 Plan: `context/waves/WAVE09-api-migration.md`
- Enhancement 60: `context/enhancements/60_api_project_setup.md`
- App Manager Master Plan: `C:\src\appmanager\MASTER_PLAN.md`
- App Manager Architecture: `C:\src\appmanager\architecture.md`
