# Enhancement Index

Master index of all congressional redistricting pipeline enhancements.

**Last Updated**: January 18, 2026

## Quick Links

- **[Enhancement Template](templates/enhancement_template.md)** - Template for new enhancements
- **[Enhancement Manager](../../tools/enhancement_manager/)** - Web UI for viewing/editing enhancements

All enhancements are stored in this directory. Status is indicated in the frontmatter and table below.

## Enhancement Metadata Fields

As of Enhancement 48, all completed enhancements include:

**Commits**: Links to GitHub commits that implemented the enhancement
- Format: `[short_sha](github_url), [short_sha](github_url), ...`
- Automatically populated via `tools/enhancement_manager/capture_commits.py`
- Example: [abc1234](https://github.com/.../commit/abc1234), [def5678](...)

**Size**: Code change metrics
- Format: `{Category} - {Lines} lines changed ({Files} files)`
- Categories: XS (<100 lines), S (100-500), M (500-1500), L (1500-5000), XL (>5000)
- Example: `M - 1,250 lines changed (15 files)`
- Calculated from git diff-tree for all related commits

These fields enable:
- Traceability: Direct links from enhancement to implementation
- Code Review: Easy access to GitHub commit diffs
- Effort Estimation: Size metrics inform future planning
- Analytics: Size distribution reveals project patterns

## Status Overview

### ✅ Completed (29 enhancements)

| # | Title | Completion Date | Files |
|---|-------|----------------|-------|
| [1](01_compactness_integration.md) | Compactness Integration | Jan 10, 2026 | [View](01_compactness_integration.md) |
| [2](02_seat_totals.md) | D/R Seat Totals | Jan 11, 2026 | [View](02_seat_totals.md) |
| [3](03_national_maps.md) | National Maps | Jan 11, 2026 | [View](03_national_maps.md) |
| [4](04_metro_areas.md) | Urban Metro Areas | Jan 12, 2026 | [View](04_metro_areas.md) |
| [5](05_national_rounds.md) | National Round Progression | Jan 12, 2026 | [View](05_national_rounds.md) |
| [6](06_architecture_diagrams.md) | System Architecture Diagrams | Jan 12, 2026 | [View](06_architecture_diagrams.md) |
| [7](07_edge_weighted_bisection.md) | Edge-Weighted Recursive Bisection | Jan 12, 2026 | [View](07_edge_weighted_bisection.md) |
| [9](09_per_state_analysis.md) | Per-State Analysis Refactoring | Jan 12, 2026 | [View](09_per_state_analysis.md) |
| [10](10_per_state_urban.md) | Per-State Urban Area Processing | Jan 2026 | [View](10_per_state_urban.md) |
| [11](11_baseline_comparison.md) | Baseline Comparison to Enacted Districts | Jan 17, 2026 | [View](11_baseline_comparison.md) |
| [12](12_edge_weighted_analysis.md) | Edge-Weighted Algorithm Analysis | Jan 17, 2026 | [View](12_edge_weighted_analysis.md) |
| [13](13_directory_unification.md) | Directory Unification | Jan 14, 2026 | [View](13_directory_unification.md) |
| [14](14_validation_framework.md) | Pipeline Output Validation | Jan 14, 2026 | [View](14_validation_framework.md) |
| [15](15_multi_year_support.md) | Multi-Year Pipeline Support | Jan 14, 2026 | [View](15_multi_year_support.md) |
| [17](17_artifact_naming.md) | Artifact Naming Standardization | Jan 14, 2026 | [View](17_artifact_naming.md) |
| [18](18_figure_quality.md) | Figure Quality Improvement | Jan 15, 2026 | [View](18_figure_quality.md) |
| [19](19_create_skill.md) | Create-Skill Meta-Skill | Jan 15, 2026 | [View](19_create_skill.md) |
| [20](20_edit_paper.md) | Edit-Paper Skill | Jan 15, 2026 | [View](20_edit_paper.md) |
| [21](21_edit_presentation.md) | Edit-Presentation Skill | Jan 15, 2026 | [View](21_edit_presentation.md) |
| [29](29_artifacts_dashboard_tab.md) | Artifacts Dashboard Tab | Jan 16, 2026 | [View](29_artifacts_dashboard_tab.md) |
| [30](30_playwright_testing.md) | Playwright Test Harness for Dashboard Testing | Jan 16, 2026 | [View](30_playwright_testing.md) |
| [31](31_pipeline_test_system.md) | Pipeline Test System | Jan 16, 2026 | [View](31_pipeline_test_system.md) |
| [33](33_dashboard_mock_data.md) | Dashboard Mock Data Integration | Jan 16, 2026 | [View](33_dashboard_mock_data.md) |
| [34](34_test_execution_skills.md) | Test Execution and Debugging Skills | Jan 16, 2026 | [View](34_test_execution_skills.md) |
| [35](35_enhancement_manager_app.md) | Enhancement Manager Web App | Jan 17, 2026 | [View](35_enhancement_manager_app.md) |
| [37](37_parallel_multi_year_pipeline.md) | Parallel Multi-Year Pipeline with Enhanced Progress Visualization | Jan 17, 2026 | [View](37_parallel_multi_year_pipeline.md) |
| [38](38_streamline_claude_md.md) | Streamline CLAUDE.md Documentation | Jan 17, 2026 | [View](38_streamline_claude_md.md) |

| [39](39_pipeline_error_logging.md) | Comprehensive Pipeline Error Logging (MVP) | Jan 17, 2026 | [View](39_pipeline_error_logging.md) |
| [48](active/48_unified_download_orchestrator.md) | Unified Download Orchestrator with Parallel Processing | Jan 18, 2026 | [View](active/48_unified_download_orchestrator.md) |

### 🔄 In Progress (1 enhancement)

| # | Title | Status | Files |
|---|-------|--------|-------|
| [8](08_block_level_data.md) | Block-Level Data Support | Phase 0 Complete (2010), Partial (2000) | [View](08_block_level_data.md) |

### 📋 Planned (18 enhancements)

| # | Title | Priority | Files |
|---|-------|----------|-------|
| [49](49_pipeline_download_integration.md) | Pipeline Download Integration (Opt-In) | Low | [View](49_pipeline_download_integration.md) |
| [47](active/47_data_separation_restoration.md) | Data Separation and Restoration | Critical | [View](active/47_data_separation_restoration.md) |
| [42](42_research_narrative_policy_questions.md) | Research Narrative and Policy Questions | High | [View](42_research_narrative_policy_questions.md) |
| [45](45_baseline_data_organization.md) | Baseline Data Organization and Analysis | High | [View](45_baseline_data_organization.md) |
| [36](36_experimental_variants_config.md) | Experimental Variants Configuration System | High | [View](36_experimental_variants_config.md) |
| [43](43_cross_year_longitudinal_analysis.md) | Cross-Year Longitudinal Analysis | Medium | [View](43_cross_year_longitudinal_analysis.md) |
| [44](44_real_world_constraints.md) | Real-World Redistricting Constraints (VRA, COI, County Splitting) | Medium | [View](44_real_world_constraints.md) |
| [46](46_enhancement_priority_system.md) | Priority System for Enhancements and Enhancement Manager | Medium | [View](46_enhancement_priority_system.md) |
| [41](41_public_distribution.md) | Public Data and Dashboard Distribution | Medium | [View](41_public_distribution.md) |
| [32](32_reock_compactness_metric.md) | Reock Compactness Metric | Low | [View](32_reock_compactness_metric.md) |
| [40](40_corner_adjacency_filter.md) | Filter Corner Adjacencies from Adjacency Graphs | Low | [View](40_corner_adjacency_filter.md) |
| [16](16_metro_2000.md) | 2000 Census Metro Area Maps | Low | [View](16_metro_2000.md) |
| [22](22_national_redistricting.md) | National Redistricting (No State Boundaries) | Research | [View](22_national_redistricting.md) |
| [23](23_county_representation.md) | Direct County Representation | Research | [View](23_county_representation.md) |
| [24](24_party_based_allocation.md) | Party-Based District Allocation | Research | [View](24_party_based_allocation.md) |
| [25](25_committee_based_representation.md) | Committee-Based Representation | Research | [View](25_committee_based_representation.md) |
| [26](26_demographic_similarity_districts.md) | Demographic Similarity Districts | Research | [View](26_demographic_similarity.md) |
| [27](27_electoral_college_county_reform.md) | Electoral College County-Based Reform | Research | [View](27_electoral_college_county_reform.md) |
| [28](28_multi_member_districts.md) | Multi-Member Districts | Research | [View](28_multi_member_districts.md) |

## By Priority

Enhancements organized by priority level for better planning and focus.

### Critical (3 enhancements)

| # | Title | Status | Files |
|---|-------|--------|-------|
| [47](active/47_data_separation_restoration.md) | Data Separation and Restoration | Proposed | [View](active/47_data_separation_restoration.md) |
| [42](42_research_narrative_policy_questions.md) | Research Narrative and Policy Questions | Proposed | [View](42_research_narrative_policy_questions.md) |
| [45](45_baseline_data_organization.md) | Baseline Data Organization and Analysis | Proposed | [View](45_baseline_data_organization.md) |

### High (4 enhancements)

| # | Title | Status | Files |
|---|-------|--------|-------|
| [36](36_experimental_variants_config.md) | Experimental Variants Configuration System | Proposed | [View](36_experimental_variants_config.md) |
| [11](11_baseline_comparison.md) | Baseline Comparison to Enacted Districts | Completed (Jan 17, 2026) | [View](11_baseline_comparison.md) |
| [12](12_edge_weighted_analysis.md) | Edge-Weighted Algorithm Analysis | Completed (Jan 17, 2026) | [View](12_edge_weighted_analysis.md) |
| [10](10_per_state_urban.md) | Per-State Urban Area Processing | Completed (Jan 2026) | [View](10_per_state_urban.md) |

### Medium (32 enhancements)

| # | Title | Status | Files |
|---|-------|--------|-------|
| [48](active/48_unified_download_orchestrator.md) | Unified Download Orchestrator with Parallel Processing | Completed (Jan 18, 2026) | [View](active/48_unified_download_orchestrator.md) |
| [43](43_cross_year_longitudinal_analysis.md) | Cross-Year Longitudinal Analysis | Proposed | [View](43_cross_year_longitudinal_analysis.md) |
| [44](44_real_world_constraints.md) | Real-World Redistricting Constraints | Proposed | [View](44_real_world_constraints.md) |
| [46](46_enhancement_priority_system.md) | Priority System for Enhancements | In Progress | [View](46_enhancement_priority_system.md) |
| [41](41_public_distribution.md) | Public Data and Dashboard Distribution | Proposed | [View](41_public_distribution.md) |
| [1](01_compactness_integration.md) | Compactness Integration | Completed | [View](01_compactness_integration.md) |
| [2](02_seat_totals.md) | D/R Seat Totals | Completed | [View](02_seat_totals.md) |
| [3](03_national_maps.md) | National Maps | Completed | [View](03_national_maps.md) |
| [4](04_metro_areas.md) | Urban Metro Areas | Completed | [View](04_metro_areas.md) |
| [5](05_national_rounds.md) | National Round Progression | Completed | [View](05_national_rounds.md) |
| [6](06_architecture_diagrams.md) | System Architecture Diagrams | Completed | [View](06_architecture_diagrams.md) |
| [7](07_edge_weighted_bisection.md) | Edge-Weighted Recursive Bisection | Completed | [View](07_edge_weighted_bisection.md) |
| [9](09_per_state_analysis.md) | Per-State Analysis Refactoring | Completed | [View](09_per_state_analysis.md) |
| [13](13_directory_unification.md) | Directory Unification | Completed | [View](13_directory_unification.md) |
| [14](14_validation_framework.md) | Pipeline Output Validation | Completed | [View](14_validation_framework.md) |
| [15](15_multi_year_support.md) | Multi-Year Pipeline Support | Completed | [View](15_multi_year_support.md) |
| [17](17_artifact_naming.md) | Artifact Naming Standardization | Completed | [View](17_artifact_naming.md) |
| [18](18_figure_quality.md) | Figure Quality Improvement | Completed | [View](18_figure_quality.md) |
| [19](19_create_skill.md) | Create-Skill Meta-Skill | Completed | [View](19_create_skill.md) |
| [20](20_edit_paper.md) | Edit-Paper Skill | Completed | [View](20_edit_paper.md) |
| [21](21_edit_presentation.md) | Edit-Presentation Skill | Completed | [View](21_edit_presentation.md) |
| [29](29_artifacts_dashboard_tab.md) | Artifacts Dashboard Tab | Completed | [View](29_artifacts_dashboard_tab.md) |
| [30](30_playwright_testing.md) | Playwright Test Harness | Completed | [View](30_playwright_testing.md) |
| [31](31_pipeline_test_system.md) | Pipeline Test System | Completed | [View](31_pipeline_test_system.md) |
| [33](33_dashboard_mock_data.md) | Dashboard Mock Data Integration | Completed | [View](33_dashboard_mock_data.md) |
| [34](34_test_execution_skills.md) | Test Execution and Debugging Skills | Completed | [View](34_test_execution_skills.md) |
| [35](35_enhancement_manager_app.md) | Enhancement Manager Web App | Completed | [View](35_enhancement_manager_app.md) |
| [37](37_parallel_multi_year_pipeline.md) | Parallel Multi-Year Pipeline | Completed | [View](37_parallel_multi_year_pipeline.md) |
| [38](38_streamline_claude_md.md) | Streamline CLAUDE.md | Completed | [View](38_streamline_claude_md.md) |
| [39](39_pipeline_error_logging.md) | Pipeline Error Logging | Completed | [View](39_pipeline_error_logging.md) |

### Low (3 enhancements)

| # | Title | Status | Files |
|---|-------|--------|-------|
| [32](32_reock_compactness_metric.md) | Reock Compactness Metric | Proposed | [View](32_reock_compactness_metric.md) |
| [40](40_corner_adjacency_filter.md) | Filter Corner Adjacencies | Proposed | [View](40_corner_adjacency_filter.md) |
| [16](16_metro_2000.md) | 2000 Census Metro Area Maps | Proposed | [View](16_metro_2000.md) |

### Research (7 enhancements)

| # | Title | Status | Files |
|---|-------|--------|-------|
| [22](22_national_redistricting.md) | National Redistricting (No State Boundaries) | Proposed | [View](22_national_redistricting.md) |
| [23](23_county_representation.md) | Direct County Representation | Proposed | [View](23_county_representation.md) |
| [24](24_party_based_allocation.md) | Party-Based District Allocation | Proposed | [View](24_party_based_allocation.md) |
| [25](25_committee_based_representation.md) | Committee-Based Representation | Proposed | [View](25_committee_based_representation.md) |
| [26](26_demographic_similarity_districts.md) | Demographic Similarity Districts | Proposed | [View](26_demographic_similarity.md) |
| [27](27_electoral_college_county_reform.md) | Electoral College County-Based Reform | Proposed | [View](27_electoral_college_county_reform.md) |
| [28](28_multi_member_districts.md) | Multi-Member Districts | Proposed | [View](28_multi_member_districts.md) |

## Related Documentation

- **[ARCHITECTURE.md](../ARCHITECTURE.md)** - System design and architectural decisions
- **[CODING_PATTERNS.md](../CODING_PATTERNS.md)** - Implementation patterns and coding conventions
- **[CHANGELOG.md](../CHANGELOG.md)** - Version history and changes
- **[../CLAUDE.md](../../CLAUDE.md)** - AI assistant guide and quick reference

## Enhancement Numbering

Enhancements are numbered sequentially as they are proposed. Numbers are never reused, even if an enhancement is cancelled or superseded.

## Creating New Enhancements

1. Copy the [enhancement template](templates/enhancement_template.md)
2. Assign the next sequential number
3. Fill out all sections
4. Add to the appropriate category ( or planned)
5. Update this INDEX.md file
