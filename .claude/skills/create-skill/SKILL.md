---
name: create-skill
description: Create new Claude Code skills following established patterns. Guides skill creation through interactive questions, generates YAML frontmatter and markdown content, automates tool permission selection, and updates documentation. Reduces skill creation time from 30-60 minutes to 5-10 minutes.
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
user-invocable: true
---

# Create Skill

Create new Claude Code skills by following the established pattern from 31+ existing skills. This meta-skill automates skill creation through interactive guidance, generates properly structured SKILL.md files, and updates all relevant documentation.

## Prerequisites
No specific prerequisites required. Can be used anytime to create new skills.

## When to Use
User says "Create new skill for [purpose]/I want to add skill that does [action]", after planning enhancement that requires new skill, when identifying repetitive workflow that should be automated, when expanding skill library with new capabilities

## Skill Categories

**Phase 1 - Enhancement & Pipeline** (9 skills): Enhancement workflow (plan/implement/document), pipeline execution + debugging, data management (census download/adjacency build/validation)

**Phase 2 - Visualization & Documentation** (7 skills): Map generation (state/national/pedagogical), documentation management, architecture diagrams

**Phase 3 - Research & Analysis** (6 skills): Statistical analysis + experiments, presentation figure generation, compactness validation, parameter sweeps

**Phase 4 - Code Organization** (3 skills): Directory reorganization, script consolidation, pattern refactoring

**Phase 5 - Editorial** (New): Paper editing (journal style), presentation editing (conference style)

## Workflow

### Step 1: Gather Skill Information
Use `AskUserQuestion` to collect core details:

**Question 1: Skill Purpose**: "What is the main purpose of this skill? (1-2 sentences)" → Get clear description (becomes YAML frontmatter description)

**Question 2: User Invocation Patterns**: "What phrases would users say to invoke this skill?" → Examples: "create a map", "run tests", "edit paper" → Helps with skill discovery

**Question 3: Skill Category**: Options (Enhancement & Pipeline, Visualization & Documentation, Research & Analysis, Code Organization, Editorial) → Determines tool permissions + workflow structure

**Question 4: Workflow Complexity**: Options (Simple 2-3 steps, Moderate 4-6 steps, Complex 7+ steps) → Affects how detailed the workflow section should be

### Step 2: Generate Skill Name
Convert description to kebab-case: "Create State Map" → `create-state-map`, "Edit Paper" → `edit-paper`, "Run Experiment" → `run-experiment`

**Validation**: Check name is unique (scan `.claude/skills/` directory), follow pattern (verb-noun or verb-adjective-noun), use hyphens not underscores, all lowercase

### Step 3: Determine Tool Permissions
Based on skill category, infer tool needs:

**Tool Permission Heuristics**: Read (always 100%), Bash (Pipeline/Research/Data 84%), Glob (code/data analysis 76%), Grep (code/data analysis 76%), Write (Enhancement/Documentation 20%), Edit (Enhancement/Documentation/Editorial 24%), TodoWrite (Enhancement/Testing/Research 16%), AskUserQuestion (skill needs user input during execution)

**Category Defaults**:
- **Enhancement & Pipeline**: Read, Write, Edit, Bash, Glob, Grep, TodoWrite
- **Visualization & Documentation**: Read, Edit, Bash, Glob, Grep
- **Research & Analysis**: Read, Bash, TodoWrite, Glob, Grep
- **Code Organization**: Read, Write, Edit, Glob, Grep, TodoWrite
- **Editorial**: Read, Edit, Glob, Grep, AskUserQuestion, Bash

Use `AskUserQuestion` to confirm/refine these defaults (show pre-selected tools based on category, allow user to add/remove tools, explain what each tool does)

### Step 4: Generate YAML Frontmatter
```yaml
---
name: skill-name-kebab-case
description: [User-provided description from Step 1]
allowed-tools:
  - Read
  - [other tools from Step 3]
user-invocable: true
---
```

**Validation**: YAML syntax is valid (no tabs, proper indentation), `name` matches directory name exactly, `description` is 1-3 sentences no line breaks, `user-invocable` always `true` for new skills, all tool names valid

### Step 5: Generate Markdown Content

**Required Sections** (All Skills):
1. **Title**: `# [Skill Title]` (Use Title Case, match skill purpose)
2. **Overview**: 2-3 sentence description of what skill does + when to use it
3. **Prerequisites** (if applicable): Required + Recommended (optional)
4. **When to Use This Skill**: User says "[phrase 1]" / User says "[phrase 2]" / When [scenario 1] / When [scenario 2]
5. **Workflow**: ### Step 1: [First Action] / ### Step 2: [Second Action] / ...
6. **What You'll Get**: After successful execution, you will have: Output 1 / Output 2 / Output 3
7. **Next Steps**: After completing this skill: Recommended action 1 / Recommended action 2 / Related skill to use next

**Optional Sections** (Category-Specific):
- **Visualization skills**: Map Types/Visualization Options, Styling Guidelines, Output Locations, Performance Notes
- **Pipeline/Research skills**: Experiment Types/Analysis Types, Parameters Table, Troubleshooting, Performance Notes
- **Enhancement skills**: Enhancement Phases Template, Testing Plan, Success Criteria
- **Editorial skills**: Editing Levels (Light/Standard/Heavy), Style Guidelines, Quality Checklist
- **All skills** (if applicable): Examples, Troubleshooting, Advanced Usage, Related Skills, Performance Notes

### Step 6: Populate Content Templates
Based on category, populate sections with appropriate guidance:

**Enhancement Skills**: (1) Gather Context → Read docs/code, (2) Plan Changes → Create implementation plan, (3) Implement Changes → Make modifications following patterns, (4) Test Changes → Validate with print-only → small state → full test, (5) Update Documentation → Update all affected docs

**Pipeline Skills**: (1) Validate Prerequisites → Check required data/config, (2) Configure Parameters → Set command-line arguments, (3) Execute Pipeline → Run with progress monitoring, (4) Verify Outputs → Check results/logs, (5) Handle Errors → Debug failures

**Visualization Skills**: (1) Identify Requirements → Determine what to visualize, (2) Validate Data Availability → Ensure input data exists, (3) Set Parameters → Configure visualization options, (4) Generate Visualization → Create map/chart/diagram, (5) Verify Output → Check quality/accuracy

**Research Skills**: (1) Define Experiment → Research question + hypothesis, (2) Select Test Cases → Choose states/parameters, (3) Run Experiments → Execute test conditions, (4) Collect Metrics → Gather quantitative data, (5) Analyze Results → Statistical testing + visualization, (6) Document Findings → Create report with results

**Editorial Skills**: (1) Analyze Content → Read and assess current state, (2) Identify Issues → Flag problems and opportunities, (3) Apply Edits → Make corrections and improvements, (4) Verify Quality → Check for errors and consistency, (5) Generate Summary → Document changes made

### Step 7: Create Skill File
**Create directory**: `mkdir -p .claude/skills/{skill-name}/`
**Write SKILL.md**: Combine YAML frontmatter + markdown content, use uppercase `SKILL.md` (following 23/25 pattern), ensure proper formatting (no tabs, consistent indentation)

**File structure**:
```
.claude/skills/{skill-name}/
└── SKILL.md
```

### Step 8: Update Documentation
**Update CLAUDE.md**: Add skill to appropriate phase section (`**Available Skills (Phase N - Category Name)**: /skill-name - Brief description`), update skill count (`**Phase N Skills**: ✅ X skills (Category)`), add user invocation example (`**How to use**: Simply describe what you want to do naturally. Examples: "I want to [action]" → Claude offers /skill-name`)

**Update ../../context/SKILLS.md** (if exists): Add comprehensive entry with skill name + description, when to use, parameters, examples, related skills

**Update ../../context/enhancements/INDEX.md** (if part of enhancement): Mark skill creation enhancement as complete or in progress

### Step 9: Validate Output
**Check file structure**: [ ] Directory created `.claude/skills/{skill-name}/`, [ ] File created `.claude/skills/{skill-name}/SKILL.md`, [ ] Filename is uppercase `SKILL.md`
**Check YAML frontmatter**: [ ] Valid YAML syntax (no tabs), [ ] `name` matches directory name, [ ] `description` is clear and concise, [ ] `allowed-tools` list is appropriate, [ ] `user-invocable: true`
**Check markdown content**: [ ] All required sections present, [ ] Workflow has numbered steps, [ ] Examples are relevant, [ ] No placeholders left unfilled
**Check naming conventions**: [ ] Directory name is kebab-case, [ ] Skill name in YAML matches directory, [ ] No typos or inconsistencies
**Check documentation updates**: [ ] CLAUDE.md updated with skill entry, [ ] Skill count updated correctly, [ ] Phase categorization is correct

### Step 10: Provide Summary
```
[OK] Created new skill: /skill-name

Location: .claude/skills/skill-name/SKILL.md
Category: Phase N - Category Name
Tools: Read, Edit, Bash, Glob, Grep
Status: Ready to use

Documentation Updated:
- CLAUDE.md (added to Phase N section)
- Skill count: 31 → 32

Next Steps:
- Test the skill by invoking it: /skill-name
- Refine content based on first use
- Add more examples if needed
```

## Tool Permission Guidance
**Read** - Read file contents (all skills - required for context gathering)
**Write** - Create new files (skills that generate new files - configs/reports/analysis scripts)
**Edit** - Modify existing files (skills that update existing files - documentation/source code)
**Bash** - Execute bash commands (skills that run scripts/build projects/execute tools)
**Glob** - Find files by pattern (skills that search for files by name)
**Grep** - Search file contents by regex (skills that search code/data for specific patterns)
**TodoWrite** - Create and manage task lists (skills with multi-step workflows benefiting from progress tracking)
**AskUserQuestion** - Prompt user for choices during execution (skills that need user input to proceed)

## Examples

**Example 1: Simple Visualization Skill**: Purpose "Generate pie charts showing district demographic composition" → Name `create-demographic-pie-chart`, Tools (Read/Bash/Glob), Workflow (4 steps: identify data → validate → generate → verify)

**Example 2: Complex Research Skill**: Purpose "Test redistricting algorithm with different population tolerance values and analyze impact on compactness" → Name `parameter-sweep`, Tools (Read/Bash/TodoWrite/Glob/Grep), Workflow (7 steps: define → select → run baseline → run treatments → collect → analyze → document)

**Example 3: Editorial Skill**: Purpose "Edit academic papers for journal submission with focus on clarity, conciseness, and page limits" → Name `edit-paper`, Tools (Read/Edit/AskUserQuestion/Bash/Glob/Grep), Workflow (5 steps: analyze → review → condense → polish → deliver)

## Troubleshooting
**Skill name already exists**: Directory .claude/skills/{name}/ already exists → Choose different name or update existing skill
**Invalid YAML frontmatter**: YAML parsing error (tabs/indentation) → Use spaces not tabs, check indentation is 2 spaces
**Tool selection unclear**: User unsure which tools to include → Start with category defaults, user can refine later
**Content too generic**: Generated content is template-like → Add specific examples relevant to skill purpose, user can refine after generation
**Documentation not updated**: CLAUDE.md or SKILLS.md not updated → Manually update these files after skill creation

## Naming Conventions
**Directory names**: Use kebab-case (`create-state-map`, `run-experiment`, `edit-paper`)
**File names**: Use uppercase `SKILL.md` (`.claude/skills/create-state-map/SKILL.md` - 23/25 existing skills use this)
**YAML name field**: Must match directory exactly (`name: create-state-map`)
**Markdown title**: Use Title Case (`# Create State Map`)
**Verbs to use**: create/generate/build (creation), run/execute (running processes), analyze/validate/check (analysis), update/edit/refactor (modification), debug/fix (problem-solving)

## Quality Checklist
[ ] YAML frontmatter is valid, [ ] Description is clear and concise (1-3 sentences), [ ] Tool permissions are appropriate, [ ] Workflow has clear numbered steps, [ ] Examples are relevant and helpful, [ ] "When to Use" section has 4-6 scenarios, [ ] "What You'll Get" lists concrete outputs, [ ] "Next Steps" suggests follow-up actions, [ ] No Lorem Ipsum or placeholder text, [ ] Spelling and grammar are correct, [ ] Formatting is consistent, [ ] Documentation files updated

## Performance Notes
**Skill creation time**: Without /create-skill (30-60 min manual), With /create-skill (5-10 min guided), Time savings (~80-90%)
**What takes time**: Gathering user input (2-3 min), generating content (1-2 min), updating documentation (1-2 min), validation (1 min)

## What You'll Get
New skill file (`.claude/skills/{skill-name}/SKILL.md`), updated documentation (CLAUDE.md with skill entry), skill count updated (total skill count incremented), category assignment (skill added to appropriate phase), ready to use (skill immediately available for invocation)

## Next Steps
Test the skill (invoke immediately to verify it works), refine content (update based on first use experience), add examples (include real-world usage examples), document in enhancement (if part of enhancement, update enhancement file), share (commit to git if skill is production-ready), create related skills (consider complementary skills)

## Related Skills
`/enhancement-plan` (plan new features - may include skill creation), `/enhancement-implement` (implement planned features), `/update-docs` (update documentation files)
