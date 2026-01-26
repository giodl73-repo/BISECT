# Apportionment Dashboard

React frontend for the Congressional Redistricting project, integrated with the unified App Manager system.

## Quick Start

### Development (via PM2 - Recommended)

The frontend is managed by the centralized PM2 ecosystem in the App Manager:

```bash
# From C:\src\appmanager
pm2 start infrastructure/ecosystem.config.js --only apportionment-frontend

# Or start all services
pm2 start infrastructure/ecosystem.config.js
```

### Local Development (Standalone)

```bash
# Install dependencies
cd frontend
pnpm install

# Link shared packages from App Manager
pnpm link ../../appmanager/packages/common-ui
pnpm link ../../appmanager/packages/common-types
pnpm link ../../appmanager/packages/common-api-client

# Set environment variables
cp .env.example .env

# Run dev server
pnpm dev
```

Frontend will be available at http://localhost:3002

## Integration with App Manager

This frontend uses shared packages from the App Manager system:

**Shared TypeScript Packages**:
- `@common/ui` - Button, LoadingSpinner, StatusIndicator components
- `@common/types` - TypeScript type definitions
- `@common/api-client` - Axios instance and React Query setup (future)

**Usage**:
```typescript
import { Button, LoadingSpinner, StatusIndicator } from '@common/ui';
```

**Benefits**:
- Consistent UI across all apps (TCM, NHL, Apportionment)
- Faster development (no need to build generic components)
- Shared Tailwind styling

## Build for Production

```bash
# Build optimized production bundle
pnpm build

# Preview production build locally
pnpm preview
```

## Project Structure

```
frontend/
├── src/
│   ├── main.tsx          # Entry point with React Query setup
│   ├── App.tsx           # Root component
│   ├── api/
│   │   └── client.ts     # Axios instance with base URL
│   ├── components/
│   │   └── app/          # App-specific components (E63)
│   ├── features/
│   │   ├── runs/         # Run list, detail, form (E63)
│   │   └── districts/    # District table, map (E64)
│   ├── hooks/
│   │   └── useApi.ts     # React Query hooks (E63)
│   └── types/
│       └── index.ts      # TypeScript interfaces
├── public/               # Static assets
├── index.html            # HTML template
├── vite.config.ts        # Vite configuration with API proxy
├── tailwind.config.js    # Tailwind CSS configuration
└── package.json          # Dependencies and scripts
```

## Tech Stack

- **React 18** - UI library
- **TypeScript** - Type safety
- **Vite** - Build tool and dev server
- **Tailwind CSS** - Utility-first CSS framework
- **React Query** - Server state management
- **Axios** - HTTP client
- **Leaflet** - Map visualization (Enhancement 64)
- **React Router** - Client-side routing (Enhancement 63)
- **@common/ui** - Shared components from App Manager

## Configuration

Environment variables are loaded from `.env` files.

Key settings:
- `VITE_API_URL` - Backend API base URL (default: http://localhost:8002)

## Development

### API Proxy

Vite dev server proxies API requests to the backend:
- `/api/*` → http://localhost:8002/api/*
- `/health` → http://localhost:8002/health
- `/version` → http://localhost:8002/version

This avoids CORS issues during development.

### Linting

```bash
pnpm lint
```

## Wave 9 Enhancements

This dashboard is built across multiple enhancements:
- **Enhancement 60**: Project setup (current) ✓
- **Enhancement 63**: React dashboard core
- **Enhancement 64**: District visualization

## PM2 Integration

This frontend is registered in the centralized PM2 ecosystem at:
`C:\src\appmanager\infrastructure\ecosystem.config.js`

Configuration:
- **Name**: `apportionment-frontend`
- **Port**: 3002
- **Logs**: `C:\src\appmanager\infrastructure\logs/apportionment-frontend-*.log`

## Shared Components

This project uses shared components from `@common/ui` instead of building from scratch:

**Available**:
- `Button` - Primary/secondary/danger/ghost variants with loading states
- `LoadingSpinner` - Consistent loading indicator (sm/md/lg sizes)
- `StatusIndicator` - Status badges (success/warning/error/info)

**App-specific** (to be built in Enhancement 63):
- `RunCard` - Display run summary
- `ProgressBar` - Show pipeline progress
- `DistrictTable` - Show district data
- `StateSelector` - Multi-state selection
