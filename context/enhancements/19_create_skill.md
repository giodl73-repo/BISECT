# Enhancement 19: Create-Skill Meta-Skill

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium (2-4 hours)
**Created**: January 15, 2026
**Completed**: January 15, 2026

## Current State

We have 25 skills across 4 phases, each manually created by following the skill pattern:
- YAML frontmatter with metadata (name, description, allowed-tools, user-invocable)
- Structured markdown content (Overview, Prerequisites, Workflow, etc.)
- Directory structure (`.claude/skills/{skill-name}/SKILL.md`)

Creating new skills requires:
1. Understanding the skill pattern from existing examples
2. Manually creating directory and SKILL.md file
3. Writing YAML frontmatter correctly
4. Following content structure conventions
5. Updating CLAUDE.md and SKILLS.md manually

This is time-consuming and error-prone, especially for:
- Choosing appropriate tool permissions
- Following naming conventions (kebab-case, uppercase SKILL.md)
- Ensuring consistent content structure
- Updating all documentation files

## Goal

Create a `/create-skill` meta-skill that:
1. **Guides skill creation** through interactive questions
2. **Generates skill files** following established patterns
3. **Automates tool permission selection** based on skill purpose
4. **Updates documentation** automatically (CLAUDE.md, SKILLS.md)
5. **Validates output** to ensure correctness

This will:
- Reduce skill creation time from 30-60 minutes to 5-10 minutes
- Eliminate pattern inconsistencies across skills
- Lower barrier to adding new skills
- Ensure documentation stays synchronized

## Implementation Plan

### Phase 1: Skill Metadata Gathering

- [ ] Use `AskUserQuestion` to gather core information:
  - Skill purpose and description (1-2 sentences)
  - Target user phrases (e.g., "create a map", "run tests")
  - Skill category (Enhancement/Pipeline/Visualization/Research/Organization)
  - Expected workflow complexity (simple/moderate/complex)
- [ ] Determine skill name from description (convert to kebab-case)
- [ ] Validate skill name is unique (check `.claude/skills/` directory)
- [ ] Infer initial tool permissions based on category and purpose

### Phase 2: Tool Permission Selection

- [ ] Use `AskUserQuestion` to confirm/refine tool permissions:
  - Read (always included for context gathering)
  - Write (for creating new files)
  - Edit (for modifying existing files)
  - Bash (for running commands/scripts)
  - Glob (for file pattern matching)
  - Grep (for content searching)
  - TodoWrite (for task tracking)
- [ ] Provide guidance on when each tool is appropriate
- [ ] Explain tool permission implications

### Phase 3: Content Structure Generation

- [ ] Generate YAML frontmatter:
  ```yaml
  ---
  name: skill-name-kebab-case
  description: User-provided description
  allowed-tools:
    - Read
    - [other tools based on selection]
  user-invocable: true
  ---
  ```
- [ ] Generate markdown content sections:
  - **Overview** - Brief description and purpose
  - **Prerequisites** - What must exist before using
  - **When to Use This Skill** - User trigger patterns
  - **Workflow** - Step-by-step approach with numbered steps
  - **[Category-Specific Sections]** - Based on skill type
  - **What You'll Get** - Expected outputs and results
  - **Next Steps** - Recommended follow-up actions
- [ ] Populate sections with guidance text and placeholders
- [ ] Add examples section if applicable

### Phase 4: File Creation & Documentation Updates

- [ ] Create skill directory: `.claude/skills/{skill-name}/`
- [ ] Write `SKILL.md` file with generated content
- [ ] Update `CLAUDE.md`:
  - Add skill to appropriate phase section
  - Update skill count totals
  - Add user invocation example
- [ ] Update `SKILLS.md` (if exists, otherwise note for future)
- [ ] Display summary of created files and changes

### Phase 5: Validation & Testing

- [ ] Verify skill file structure:
  - YAML frontmatter is valid
  - All required sections present
  - Tool permissions are reasonable
- [ ] Check naming conventions:
  - Directory name is kebab-case
  - SKILL.md uses uppercase (following 23/25 pattern)
  - Skill name in YAML matches directory
- [ ] Validate documentation updates:
  - CLAUDE.md entry added
  - Skill count updated correctly
  - Phase categorization is correct
- [ ] Provide validation report to user

## Files to Modify/Create

### Create

- `.claude/skills/create-skill/SKILL.md` - The meta-skill definition itself
- `.claude/skills/{user-skill-name}/SKILL.md` - Generated skill files (created dynamically)

### Modify

- `CLAUDE.md` - Add new skill to appropriate phase section, update counts
- `SKILLS.md` - Add comprehensive skill documentation (if file exists)
- `enhancements/INDEX.md` - Mark Enhancement 19 as complete

## Testing Plan

1. **Self-test** - Use `/create-skill` to create itself (meta-test)
2. **Simple skill test** - Create a trivial skill with minimal tools (Read only)
3. **Complex skill test** - Create a skill with multiple tools and workflow steps
4. **Documentation verification** - Verify CLAUDE.md and SKILLS.md updated correctly
5. **Validation test** - Intentionally provide edge cases (long names, special chars)

After initial testing, use `/create-skill` to create:
- `/edit-paper` skill (journal editor)
- `/edit-presentation` skill (conference editor)

## Success Criteria

- [ ] `/create-skill` skill file created in `.claude/skills/create-skill/`
- [ ] Interactive questions gather all necessary information
- [ ] Generated YAML frontmatter is valid
- [ ] Generated content follows established patterns
- [ ] Tool permissions are appropriate for skill purpose
- [ ] Skill directory and SKILL.md created correctly
- [ ] CLAUDE.md updated with new skill entry and counts
- [ ] Naming conventions followed (kebab-case, uppercase SKILL.md)
- [ ] Validation checks pass for generated files
- [ ] Can successfully create 2-3 test skills without errors
- [ ] Documentation accurately reflects new skills

## Benefits

- **Time savings**: Reduce skill creation from 30-60 minutes to 5-10 minutes
- **Consistency**: Eliminate pattern variations across skills
- **Lower barrier**: Non-experts can create skills following best practices
- **Auto-documentation**: CLAUDE.md and SKILLS.md stay synchronized
- **Validation**: Catch errors early with automated checks
- **Scalability**: Makes it easy to expand skill library (targeting 30-40 skills)

## Dependencies

- **AskUserQuestion tool** - Required for interactive guidance
- **Existing skill patterns** - 25 existing skills provide template examples
- **CLAUDE.md structure** - Must understand phase categories
- **Enhancement system** - Will help create editing skills in next step

## Risks & Mitigations

- **Risk 1**: Generated content might be too generic or template-like
  - *Mitigation*: Provide rich examples and guidance text; user can edit after generation

- **Risk 2**: Tool permission selection might be unclear to users
  - *Mitigation*: Include descriptions and examples for each tool with AskUserQuestion options

- **Risk 3**: YAML frontmatter parsing errors if format is incorrect
  - *Mitigation*: Use strict template with validation; test YAML syntax before writing

- **Risk 4**: Concurrent skill creation could conflict (multiple Claude instances)
  - *Mitigation*: Use file existence checks; low probability in practice

## Implementation Notes

### Key Design Decisions

**Question Flow Strategy**:
1. Start with high-level questions (purpose, category)
2. Infer tool needs from category
3. Ask confirmation questions with pre-selected defaults
4. Allow custom additions/removals

**Content Generation Approach**:
- Use template strings with variable substitution
- Provide rich placeholder text as guidance
- Include examples from similar existing skills
- Make it easy for user to refine after generation

**Tool Permission Heuristics**:
- **Read**: Always include (100% of skills use it)
- **Bash**: Include if category is Pipeline/Research/Data (21/25 skills)
- **Glob/Grep**: Include if category involves code/data analysis (19/25 skills)
- **Write**: Include only for Enhancement/Documentation skills (5/25 skills)
- **Edit**: Include only for Enhancement/Documentation skills (6/25 skills)
- **TodoWrite**: Include only for Enhancement/Testing/Research skills (4/25 skills)

**Naming Conventions**:
- Directory: `kebab-case` (e.g., `create-skill`)
- File: `SKILL.md` (uppercase, 23/25 existing skills use this)
- YAML name: Match directory exactly
- Title in markdown: Title Case with hyphens (e.g., "Create-Skill Meta-Skill")

### Section Templates by Category

**Enhancement Skills**:
- Workflow includes: Gather Context → Plan → Implement → Test → Document
- Tools: Read, Write, Edit, TodoWrite, Glob, Grep
- Examples: enhancement-plan, enhancement-implement

**Pipeline Skills**:
- Workflow includes: Validate Prerequisites → Execute → Monitor → Report
- Tools: Read, Bash, Glob, Grep, TodoWrite
- Examples: run-redistricting, pipeline-debug

**Visualization Skills**:
- Workflow includes: Load Data → Generate Visualization → Save Output
- Tools: Read, Bash, Glob
- Examples: create-state-map, create-national-map

**Research Skills**:
- Workflow includes: Design Experiment → Execute → Analyze → Report
- Tools: Read, Bash, TodoWrite, Glob, Grep
- Examples: run-experiment, parameter-sweep

**Documentation Skills**:
- Workflow includes: Review Current → Identify Updates → Edit → Verify
- Tools: Read, Edit, Glob, Grep
- Examples: update-docs, create-architecture-diagram

## Related Documentation

- Enhancement #20: Edit-Paper Skill (next step)
- Enhancement #21: Edit-Presentation Skill (next step)
- [CLAUDE.md](../../CLAUDE.md) - Anthropic Skills section
- [SKILLS.md](../SKILLS.md) - Comprehensive skill documentation
- Existing skills in `.claude/skills/` - Pattern examples
