# Windows Command Wrappers

The `BISECT` CLI is the canonical pipeline entry point. `setup_env.bat` exposes
`run` and `runtest` aliases directly against that CLI rather than routing
through additional batch files.

The remaining wrappers have distinct responsibilities:

| Script | Purpose |
|---|---|
| `compile_artifacts.bat` | Build the complete research artifact set. |
| `create_test_data.bat` | Materialize quick, standard, or full test data. |
| `deploy_master.bat` | Generate the master dashboard. |
| `deploy_web.bat` | Generate or deploy a versioned web dashboard. |
| `run_dashboard_tests.bat` | Run dashboard-specific test modes. |
| `run_master.bat` | Open an already-generated master dashboard. |
| `run_tests.bat` | Run unit, integration, end-to-end, coverage, marker, or acceptance tests. |

Use `run_tests.bat acceptance <version> [year]` for the behavior previously
provided by `run_acceptance_tests.bat`.
