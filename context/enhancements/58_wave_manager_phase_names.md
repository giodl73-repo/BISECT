# Enhancement 58 (Phase 4): Wave Manager Phase Names

**Status**: ✅ COMPLETED
**Started**: 2026-01-25
**Completed**: 2026-01-25
**Wave**: Wave 8 (WAVE-MANAGER-IMPROVEMENTS)
**Priority**: Medium
**Estimated Complexity**: Small

---

## Description

Add phase names to wave manager schema so phases can have descriptive titles like "Foundation", "Configuration", "UI Fixes" instead of just "Phase 1", "Phase 2", etc.

---

## Motivation

Currently phases are displayed as generic "Phase 1", "Phase 2" labels. Having descriptive names like "Foundation", "Configuration", "UI Fixes" makes it much easier to understand what each phase accomplishes at a glance.

**Current**: Phase 1, Phase 2, Phase 3
**After**: Phase 1: Foundation, Phase 2: Configuration, Phase 3: UI Fixes

---

## Implementation

### Tasks

1. **Update Schema** (10 min)
   - Add `name` field to phase mapping in SCHEMA.md
   - Update example wave documents with phase names
   - Document optional vs required status

2. **Update Parser** (15 min)
   - Modify `parser.py` to extract phase names from wave documents
   - Parse format: `- Phase 1: Enhancement 53 - Schema v2.0` → name = "Foundation"
   - Handle both with and without names for backward compatibility

3. **Update Frontend UX** (20 min)
   - Modify `static/index.html` to display phase names
   - Update phase rendering to show "Phase 1: Foundation" instead of "Phase 1"
   - Ensure graceful fallback if no name provided

4. **Test** (10 min)
   - Update Wave 8 document with phase names
   - Verify parser extracts names correctly
   - Verify frontend displays names correctly
   - Test backward compatibility with waves without names

5. **Document** (5 min)
   - Update SCHEMA.md with phase name field
   - Add examples to documentation

### Schema Changes

**Current phase mapping**:
```markdown
**Phases**:
- Phase 1: Enhancement 53 (✅ COMPLETED 2026-01-25)
- Phase 2: Enhancements 54, 55 (✅ COMPLETED 2026-01-25)
```

**New phase mapping with names**:
```markdown
**Phases**:
- Phase 1: Enhancement 53 - Foundation (✅ COMPLETED 2026-01-25)
- Phase 2: Enhancements 54, 55 - Configuration (✅ COMPLETED 2026-01-25)
- Phase 3: Enhancements 56, 57 - UI Fixes (✅ COMPLETED 2026-01-25)
```

**Extracted data**:
```python
{
    "id": 1,
    "name": "Foundation",  # NEW
    "enhancement_ids": [53],
    "status": "completed",
    "completed_date": "2026-01-25"
}
```

---

## Success Criteria

- [x] SCHEMA.md updated with phase name field
- [x] parser.py extracts phase names from wave documents
- [x] Frontend displays phase names (e.g., "Phase 1: Foundation")
- [x] Backward compatible with waves without phase names
- [x] Wave 8 updated with phase names as example
- [x] Ready to test

---

## Files to Modify

**Schema**:
- `tools/wave-manager/SCHEMA.md` - Add phase name field documentation

**Backend**:
- `tools/wave-manager/parser.py` - Extract phase names from markdown

**Frontend**:
- `tools/wave-manager/static/index.html` - Display phase names in UI

**Documentation**:
- `context/waves/WAVE08-wave-manager-improvements.md` - Update with phase names

---

## Testing

**Test Cases**:
1. Wave with phase names → Names displayed correctly
2. Wave without phase names → Falls back to "Phase N" only
3. Mixed wave (some phases with names, some without) → Handles gracefully
4. Phase name parsing → Correctly extracts from "Phase 1: Enh 53 - Foundation"

**Manual Testing**:
```bash
cd tools/wave-manager
python app.py
# Visit http://localhost:5104
# Expand Wave 8
# Verify phases show: "Phase 1: Foundation", "Phase 2: Configuration", etc.
```

---

## Cross-Project Application

After implementation and testing in apportionment:

1. Create guide document (like WAVE_MANAGER_LINKING_FIX.md)
2. Apply to appmanager, TCM, NHL, Performance
3. Commit changes to each repository
4. User will then update wave documents in each project with phase names

---

## Dependencies

**Prerequisites**:
- Enhancement 53 (Schema v2.0) ✅
- Enhancement 54 (Skills Integration) ✅
- Enhancement 55 (Phase Validation) ✅
- Enhancement 56 (Title Branding) ✅
- Enhancement 57 (Phase Display Fix) ✅

**Blocking Issues**: None

---

## Notes

- Phase names are optional for backward compatibility
- Parser should handle both formats gracefully
- Frontend should show "Phase N: Name" if name exists, "Phase N" if not
- This enhances readability without breaking existing functionality

---

**Enhancement 58 Summary**: Add phase names to wave manager schema for better clarity and readability.
