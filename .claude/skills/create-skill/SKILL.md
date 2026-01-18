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

## Overview

Create new Claude Code skills by following the established pattern from 25+ existing skills. This meta-skill automates skill creation through interactive guidance, generates properly structured SKILL.md files, and updates all relevant documentation.

## Prerequisites

No specific prerequisites required. This skill can be used at any time to create new skills.

## When to Use This Skill

- User says: "Create a new skill for [purpose]"
- User says: "I want to add a skill that does [action]"
- After planning an enhancement that requires a new skill
- When identifying a repetitive workflow that should be automated
- When expanding the skill library with new capabilities

## Skill Categories

Skills are organized into 5 categories (4 phases plus Editorial):

**Phase 1 - Enhancement & Pipeline** (9 skills):
- Enhancement workflow (plan, implement, document)
- Pipeline execution and debugging
- Data management (census download, adjacency build, validation)

**Phase 2 - Visualization & Documentation** (7 skills):
- Map generation (state, national, pedagogical)
- Documentation management
- Architecture diagrams

**Phase 3 - Research & Analysis** (6 skills):
- Statistical analysis and experiments
- Presentation figure generation
- Compactness validation
- Parameter sweeps

**Phase 4 - Code Organization** (3 skills):
- Directory reorganization
- Script consolidation
- Pattern refactoring

**Phase 5 - Editorial** (New):
- Paper editing (journal style)
- Presentation editing (conference style)

## Workflow

### Step 1: Gather Skill Information

Use `AskUserQuestion` to collect core details:

**Question 1: Skill Purpose**
- Question: "What is the main purpose of this skill? (1-2 sentences)"
- Get clear description of what the skill does
- This becomes the description in YAML frontmatter

**Question 2: User Invocation Patterns**
- Question: "What phrases would users say to invoke this skill?"
- Examples: "create a map", "run tests", "edit paper"
- Helps with skill discovery

**Question 3: Skill Category**
- Options:
  - Enhancement & Pipeline
  - Visualization & Documentation
  - Research & Analysis
  - Code Organization
  - Editorial (new category)
- Determines tool permissions and workflow structure

**Question 4: Workflow Complexity**
- Options: Simple (2-3 steps), Moderate (4-6 steps), Complex (7+ steps)
- Affects how detailed the workflow section should be

### Step 2: Generate Skill Name

Convert description to kebab-case:
- "Create State Map" → `create-state-map`
- "Edit Paper" → `edit-paper`
- "Run Experiment" → `run-experiment`

**Validation**:
- Check that name is unique (scan `.claude/skills/` directory)
- Follow pattern: verb-noun or verb-adjective-noun
- Use hyphens not underscores
- All lowercase

### Step 3: Determine Tool Permissions

Based on skill category, infer tool needs:

**Tool Permission Heuristics**:
- **Read**: Always include (100% of skills use it)
- **Bash**: Include if Pipeline/Research/Data category (21/25 = 84%)
- **Glob**: Include if involves code/data analysis (19/25 = 76%)
- **Grep**: Include if involves code/data analysis (19/25 = 76%)
- **Write**: Include only for Enhancement/Documentation (5/25 = 20%)
- **Edit**: Include only for Enhancement/Documentation/Editorial (6/25 = 24%)
- **TodoWrite**: Include only for Enhancement/Testing/Research (4/25 = 16%)
- **AskUserQuestion**: Include if skill needs user input during execution

**Category Defaults**:

**Enhancement & Pipeline**:
```yaml
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - TodoWrite
```

**Visualization & Documentation**:
```yaml
allowed-tools:
  - Read
  - Edit
  - Bash
  - Glob
  - Grep
```

**Research & Analysis**:
```yaml
allowed-tools:
  - Read
  - Bash
  - TodoWrite
  - Glob
  - Grep
```

**Code Organization**:
```yaml
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - TodoWrite
```

**Editorial**:
```yaml
allowed-tools:
  - Read
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
  - Bash
```

Use `AskUserQuestion` to confirm/refine these defaults:
- Show pre-selected tools based on category
- Allow user to add/remove tools
- Explain what each tool does

### Step 4: Generate YAML Frontmatter

Create frontmatter with collected information:

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

**Validation**:
- YAML syntax is valid (no tabs, proper indentation)
- `name` matches directory name exactly
- `description` is 1-3 sentences, no line breaks
- `user-invocable` is always `true` for new skills
- All tool names are valid

### Step 5: Generate Markdown Content

Create structured markdown sections following the pattern:

#### Required Sections (All Skills)

**1. Title**:
```markdown
# [Skill Title]
```
- Use Title Case
- Match skill purpose, not necessarily directory name

**2. Overview**:
```markdown
## Overview

[2-3 sentence description of what the skill does and when to use it]
```

**3. Prerequisites** (if applicable):
```markdown
## Prerequisites

**Required**:
- Prerequisite 1
- Prerequisite 2

**Recommended** (optional):
- Optional prerequisite 1
```

**4. When to Use This Skill**:
```markdown
## When to Use This Skill

- User says: "[phrase 1]"
- User says: "[phrase 2]"
- When [scenario 1]
- When [scenario 2]
```

**5. Workflow**:
```markdown
## Workflow

### Step 1: [First Action]

[Description and examples]

### Step 2: [Second Action]

[Description and examples]

[... continue for all steps]
```

**6. What You'll Get**:
```markdown
## What You'll Get

After successful execution, you will have:
- Output 1
- Output 2
- Output 3
```

**7. Next Steps**:
```markdown
## Next Steps

After completing this skill:
- Recommended action 1
- Recommended action 2
- Related skill to use next
```

#### Optional Sections (Category-Specific)

**For Visualization skills**:
- Map Types / Visualization Options
- Styling Guidelines
- Output Locations
- Performance Notes

**For Pipeline/Research skills**:
- Experiment Types / Analysis Types
- Parameters Table
- Troubleshooting
- Performance Notes

**For Enhancement skills**:
- Enhancement Phases Template
- Testing Plan
- Success Criteria

**For Editorial skills**:
- Editing Levels (Light/Standard/Heavy)
- Style Guidelines
- Quality Checklist

**For all skills (if applicable)**:
- Examples
- Troubleshooting
- Advanced Usage
- Related Skills
- Performance Notes

### Step 6: Populate Content Templates

Based on category, populate sections with appropriate guidance:

**Enhancement Skills** (Workflow pattern):
```markdown
### Step 1: Gather Context
Read relevant documentation and code

### Step 2: Plan Changes
Create implementation plan

### Step 3: Implement Changes
Make modifications following patterns

### Step 4: Test Changes
Validate with print-only → small state → full test

### Step 5: Update Documentation
Update all affected docs
```

**Pipeline Skills** (Workflow pattern):
```markdown
### Step 1: Validate Prerequisites
Check required data and configuration

### Step 2: Configure Parameters
Set command-line arguments

### Step 3: Execute Pipeline
Run with progress monitoring

### Step 4: Verify Outputs
Check results and logs

### Step 5: Handle Errors
Debug any failures
```

**Visualization Skills** (Workflow pattern):
```markdown
### Step 1: Identify Requirements
Determine what to visualize

### Step 2: Validate Data Availability
Ensure input data exists

### Step 3: Set Parameters
Configure visualization options

### Step 4: Generate Visualization
Create map/chart/diagram

### Step 5: Verify Output
Check quality and accuracy
```

**Research Skills** (Workflow pattern):
```markdown
### Step 1: Define Experiment
Research question and hypothesis

### Step 2: Select Test Cases
Choose states/parameters

### Step 3: Run Experiments
Execute test conditions

### Step 4: Collect Metrics
Gather quantitative data

### Step 5: Analyze Results
Statistical testing and visualization

### Step 6: Document Findings
Create report with results
```

**Editorial Skills** (Workflow pattern):
```markdown
### Step 1: Analyze Content
Read and assess current state

### Step 2: Identify Issues
Flag problems and opportunities

### Step 3: Apply Edits
Make corrections and improvements

### Step 4: Verify Quality
Check for errors and consistency

### Step 5: Generate Summary
Document changes made
```

### Step 7: Create Skill File

**Create directory**:
```bash
mkdir -p .claude/skills/{skill-name}/
```

**Write SKILL.md**:
- Combine YAML frontmatter + markdown content
- Use uppercase `SKILL.md` (following 23/25 pattern)
- Ensure proper formatting (no tabs, consistent indentation)

**File structure**:
```
.claude/skills/{skill-name}/
└── SKILL.md
```

### Step 8: Update Documentation

**Update CLAUDE.md**:

Add skill to appropriate phase section:
```markdown
**Available Skills (Phase N - Category Name)**:
- `/skill-name` - Brief description
- [existing skills...]
```

Update skill count:
```markdown
**Phase N Skills**: ✅ X skills (Category)
[Update total count at top]
```

Add user invocation example:
```markdown
**How to use**: Simply describe what you want to do naturally. Examples:
- "I want to [action]" → Claude offers `/skill-name`
- [existing examples...]
```

**Update ../../context/SKILLS.md** (if exists):

Add comprehensive entry with:
- Skill name and description
- When to use
- Parameters
- Examples
- Related skills

**Update ../../context/enhancements/INDEX.md** (if part of enhancement):

Mark skill creation enhancement as complete or in progress.

### Step 9: Validate Output

**Check file structure**:
- [ ] Directory created: `.claude/skills/{skill-name}/`
- [ ] File created: `.claude/skills/{skill-name}/SKILL.md`
- [ ] Filename is uppercase `SKILL.md`

**Check YAML frontmatter**:
- [ ] Valid YAML syntax (no tabs)
- [ ] `name` matches directory name
- [ ] `description` is clear and concise
- [ ] `allowed-tools` list is appropriate
- [ ] `user-invocable: true`

**Check markdown content**:
- [ ] All required sections present
- [ ] Workflow has numbered steps
- [ ] Examples are relevant
- [ ] No placeholders left unfilled

**Check naming conventions**:
- [ ] Directory name is kebab-case
- [ ] Skill name in YAML matches directory
- [ ] No typos or inconsistencies

**Check documentation updates**:
- [ ] CLAUDE.md updated with skill entry
- [ ] Skill count updated correctly
- [ ] Phase categorization is correct

### Step 10: Provide Summary

Display creation summary to user:

```
[OK] Created new skill: /skill-name

Location: .claude/skills/skill-name/SKILL.md
Category: Phase N - Category Name
Tools: Read, Edit, Bash, Glob, Grep
Status: Ready to use

Documentation Updated:
- CLAUDE.md (added to Phase N section)
- Skill count: 25 → 26

Next Steps:
- Test the skill by invoking it: /skill-name
- Refine content based on first use
- Add more examples if needed
```

## Tool Permission Guidance

When helping users select tools, provide this guidance:

**Read** - Read file contents
- Use for: All skills (required for context gathering)
- Examples: Read documentation, source code, data files

**Write** - Create new files
- Use for: Skills that generate new files (configs, reports, analysis scripts)
- Examples: Enhancement planning, report generation

**Edit** - Modify existing files
- Use for: Skills that update existing files (documentation, source code)
- Examples: Refactoring, documentation updates, code editing

**Bash** - Execute bash commands
- Use for: Skills that run scripts, build projects, execute tools
- Examples: Pipeline execution, testing, data processing

**Glob** - Find files by pattern (e.g., "**/*.py")
- Use for: Skills that search for files by name
- Examples: Finding all test files, locating configuration files

**Grep** - Search file contents by regex
- Use for: Skills that search code/data for specific patterns
- Examples: Finding function definitions, searching logs

**TodoWrite** - Create and manage task lists
- Use for: Skills with multi-step workflows that benefit from progress tracking
- Examples: Enhancements, experiments, complex refactoring

**AskUserQuestion** - Prompt user for choices during execution
- Use for: Skills that need user input to proceed (preferences, choices)
- Examples: Interactive configuration, editing with user approval

## Examples

### Example 1: Simple Visualization Skill

**User input**:
- Purpose: "Generate pie charts showing district demographic composition"
- User phrases: "create demographic pie chart", "visualize demographics"
- Category: Visualization & Documentation
- Complexity: Simple

**Generated skill**:
- Name: `create-demographic-pie-chart`
- Tools: Read, Bash, Glob
- Workflow: 4 steps (identify data → validate → generate → verify)

### Example 2: Complex Research Skill

**User input**:
- Purpose: "Test redistricting algorithm with different population tolerance values and analyze impact on compactness"
- User phrases: "test population tolerance", "parameter sweep"
- Category: Research & Analysis
- Complexity: Complex

**Generated skill**:
- Name: `parameter-sweep`
- Tools: Read, Bash, TodoWrite, Glob, Grep
- Workflow: 7 steps (define → select → run baseline → run treatments → collect → analyze → document)

### Example 3: Editorial Skill

**User input**:
- Purpose: "Edit academic papers for journal submission with focus on clarity, conciseness, and page limits"
- User phrases: "edit paper", "proofread paper", "condense paper"
- Category: Editorial
- Complexity: Moderate

**Generated skill**:
- Name: `edit-paper`
- Tools: Read, Edit, AskUserQuestion, Bash, Glob, Grep
- Workflow: 5 steps (analyze → review → condense → polish → deliver)

## Troubleshooting

**Skill name already exists**:
```
Issue: Directory .claude/skills/{name}/ already exists
Solution: Choose different name or update existing skill
```

**Invalid YAML frontmatter**:
```
Issue: YAML parsing error (tabs, indentation)
Solution: Use spaces not tabs, check indentation is 2 spaces
```

**Tool selection unclear**:
```
Issue: User unsure which tools to include
Solution: Start with category defaults, user can refine later
```

**Content too generic**:
```
Issue: Generated content is template-like
Solution: Add specific examples relevant to skill purpose
         User can refine after generation
```

**Documentation not updated**:
```
Issue: CLAUDE.md or SKILLS.md not updated
Solution: Manually update these files after skill creation
```

## Naming Conventions

**Directory names**: Use kebab-case
- `create-state-map`
- `run-experiment`
- `edit-paper`

**File names**: Use uppercase `SKILL.md`
- `.claude/skills/create-state-map/SKILL.md`
- 23 out of 25 existing skills use this convention

**YAML name field**: Must match directory exactly
```yaml
name: create-state-map  # matches directory name
```

**Markdown title**: Use Title Case
```markdown
# Create State Map
```

**Verbs to use**:
- create, generate, build (for creation)
- run, execute (for running processes)
- analyze, validate, check (for analysis)
- update, edit, refactor (for modification)
- debug, fix (for problem-solving)

## Quality Checklist

Before finalizing skill:
- [ ] YAML frontmatter is valid
- [ ] Description is clear and concise (1-3 sentences)
- [ ] Tool permissions are appropriate
- [ ] Workflow has clear numbered steps
- [ ] Examples are relevant and helpful
- [ ] "When to Use" section has 4-6 scenarios
- [ ] "What You'll Get" lists concrete outputs
- [ ] "Next Steps" suggests follow-up actions
- [ ] No Lorem Ipsum or placeholder text
- [ ] Spelling and grammar are correct
- [ ] Formatting is consistent
- [ ] Documentation files updated

## Related Skills

- `/enhancement-plan` - Plan new features (may include skill creation)
- `/enhancement-implement` - Implement planned features
- `/update-docs` - Update documentation files

## Performance Notes

**Skill creation time**:
- Without /create-skill: 30-60 minutes (manual)
- With /create-skill: 5-10 minutes (guided)
- Time savings: ~80-90%

**What takes time**:
- Gathering user input: 2-3 minutes
- Generating content: 1-2 minutes
- Updating documentation: 1-2 minutes
- Validation: 1 minute

## What You'll Get

After successful skill creation:
- **New skill file**: `.claude/skills/{skill-name}/SKILL.md`
- **Updated documentation**: CLAUDE.md with skill entry
- **Skill count updated**: Total skill count incremented
- **Category assignment**: Skill added to appropriate phase
- **Ready to use**: Skill immediately available for invocation

## Next Steps

After creating a skill:
1. **Test the skill**: Invoke it immediately to verify it works
2. **Refine content**: Update based on first use experience
3. **Add examples**: Include real-world usage examples
4. **Document in enhancement**: If part of enhancement, update enhancement file
5. **Share**: Commit to git if skill is production-ready
6. **Create related skills**: Consider complementary skills
