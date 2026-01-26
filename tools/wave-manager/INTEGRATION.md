# Wave Manager Integration with App Manager Dashboard

## Overview

The AppManager Wave Manager is now fully integrated with the PM2 ecosystem and App Manager Dashboard.

## What Was Updated

### 1. PM2 Ecosystem Configuration

Added `appmanager-wave-manager` service to:
- ✅ `/infrastructure/ecosystem.config.js` (full ecosystem)
- ✅ `/infrastructure/ecosystem-appmanager-only.config.js` (app manager only)

### 2. Service Details

```javascript
{
  name: 'appmanager-wave-manager',
  script: 'python',
  args: 'app.py',
  cwd: 'C:/src/appmanager/tools/wave-manager',
  env: {
    PYTHONUNBUFFERED: '1'
  },
  port: 5101,  // Configured in config.py
  autorestart: true,
  max_memory_restart: '200M'
}
```

### 3. Wave Manager Configuration

File: `tools/wave-manager/config.py`

```python
PORT = 5101
HOST = '0.0.0.0'
PROJECT_NAME = "App Manager"
PROJECT_COLOR = "#3b82f6"  # Blue
GITHUB_REPO = "https://github.com/giodl_microsoft/appmanager"
```

### 4. Dashboard Integration

The wave manager automatically appears in the App Manager Dashboard services section because:
- It's managed by PM2
- Dashboard queries `pm2.list()` to show all services
- Service card displays: status, uptime, memory, CPU, restart count
- Controls available: Start, Stop, Restart

### 5. Quick Links

Wave manager is also accessible via the Quick Links section:
- **AppManager** wave manager button (orange) → http://localhost:5101

## How to Use

### Start Everything (Recommended)

```bash
# From appmanager directory
./start-app-manager.bat
```

This automatically starts:
- Docker databases
- App Manager backend & frontend
- AppManager wave manager
- Performance wave manager
- Apportionment wave manager

### Start Just the Wave Manager

```bash
pm2 start infrastructure/ecosystem-appmanager-only.config.js --only appmanager-wave-manager
```

Or use the standalone script:
```bash
./start-wave-manager.bat
```

### Stop the Wave Manager

```bash
pm2 stop appmanager-wave-manager
```

Or use:
```bash
./stop-wave-manager.bat
```

### View Status

```bash
pm2 status
```

Shows all services including `appmanager-wave-manager`.

### View Logs

```bash
pm2 logs appmanager-wave-manager
```

Or check the dashboard's service card for live status.

## Accessing the Wave Manager

### Via Dashboard
1. Open http://localhost:9000
2. Look in the "Services" section
3. See `appmanager-wave-manager` with start/stop/restart controls
4. Click the "AppManager" button in Quick Links section

### Direct Access
- Wave Manager: http://localhost:5101

## Service Monitoring

The dashboard shows real-time information:
- **Status**: online, stopped, errored, or starting
- **Uptime**: How long the service has been running
- **Memory**: Current memory usage
- **CPU**: Current CPU usage
- **Restarts**: Number of times PM2 has restarted the service

## Architecture

```
App Manager Dashboard (Port 9000)
        │
        ├─ Manages via PM2
        │
        └─ AppManager Wave Manager (Port 5101)
           ├─ Reads: context/waves/*.md
           ├─ Reads: context/enhancements/*.md
           └─ Provides: Web UI for wave tracking
```

## Port Allocation

| Service | Port | Type |
|---------|------|------|
| App Manager Backend | 9001 | Express API |
| App Manager Frontend | 9000 | React/Vite |
| AppManager Wave Manager | 5101 | Flask |
| Performance Wave Manager | 5105 | Flask |
| TCM Wave Manager | 5102 | Flask |
| NHL Wave Manager | 5103 | Flask |
| Apportionment Wave Manager | 5104 | Flask |

## Features

### Enhanced Schema (v2.0)
- ✅ Explicit phase mappings
- ✅ Git commit parsing with auto GitHub URLs
- ✅ Wave body content rendering
- ✅ Phase validation tools

### Wave Manager UI
- View all waves and their phases
- Track enhancement status
- See git commits with clickable GitHub links
- Filter by status, priority, wave
- Search enhancements

### Integration with Dashboard
- Shows up as managed PM2 service
- Start/stop/restart via dashboard controls
- Monitor resource usage
- Quick access via orange "AppManager" button

## Validation

To validate your wave and enhancement documents:

```bash
cd tools/wave-manager
python validate_phases.py
```

This checks:
- Wave documents have proper phase mappings
- Enhancement files exist for all phase IDs
- Phase numbers in titles match wave definitions

## Troubleshooting

### Wave manager not showing in services
- Make sure PM2 is running: `pm2 status`
- Start it: `pm2 start infrastructure/ecosystem-appmanager-only.config.js --only appmanager-wave-manager`
- Check logs: `pm2 logs appmanager-wave-manager`

### Wave manager shows "errored" status
- Check Python is in PATH
- Check Flask is installed: `pip list | grep Flask`
- Check logs: `pm2 logs appmanager-wave-manager`
- Check port 5101 is not in use: `netstat -ano | findstr 5101`

### Can't access http://localhost:5101
- Check service is running: `pm2 status`
- Check HOST is set to '0.0.0.0' in config.py
- Check firewall is not blocking port 5101

## Next Steps

1. Start the App Manager: `./start-app-manager.bat`
2. Open dashboard: http://localhost:9000
3. Verify wave manager appears in Services section
4. Click "AppManager" in Quick Links to access wave manager
5. Create waves and enhancements following SCHEMA.md

## Documentation

- `QUICKSTART.md` - Quick start guide
- `SCHEMA.md` - Wave and enhancement schema documentation
- `README.md` - General wave manager documentation
- `../../../architecture.md` - Overall system architecture
- `../../../coding_patterns.md` - Coding best practices
