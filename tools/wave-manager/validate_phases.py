"""
Validate Phase Assignments

Checks that phase numbers in wave documents match the enhancement titles.
"""

from pathlib import Path
from parser import parse_wave, parse_enhancement
import re
import sys

# Force UTF-8 output encoding
if sys.stdout.encoding != 'utf-8':
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

# Paths
BASE_DIR = Path(__file__).parent.parent.parent
WAVES_DIR = BASE_DIR / 'context' / 'waves'
ENHANCEMENTS_DIR = BASE_DIR / 'context' / 'enhancements'

def extract_phase_from_title(title):
    """Extract phase number/label from enhancement title"""
    # Try "Phases X-Y" format first
    match = re.search(r'Phases? ([0-9A-Z\-]+)', title)
    if match:
        return match.group(1)
    return None

def validate_phases():
    """Validate that phase assignments are consistent"""

    print("=" * 80)
    print("PHASE ASSIGNMENT VALIDATION")
    print("=" * 80)
    print()

    # Load all waves
    waves = []
    for wave_file in sorted(WAVES_DIR.glob('WAVE*.md')):
        wave = parse_wave(wave_file)
        if wave:
            waves.append(wave)

    # Load all enhancements
    enhancements = {}
    for enh_file in sorted(ENHANCEMENTS_DIR.glob('[0-9]*.md')):
        enh = parse_enhancement(enh_file)
        if enh:
            enhancements[enh['id']] = enh

    print(f"Loaded {len(waves)} waves and {len(enhancements)} enhancements")
    print()

    # Track issues
    issues = []
    phase_assignments = {}  # wave_id -> {enh_id: phase_label}

    # Build phase assignment map from waves
    for wave in waves:
        wave_id = wave['id']
        phase_assignments[wave_id] = {}

        if wave.get('phase_mappings'):
            # Use explicit phase mappings
            for mapping in wave['phase_mappings']:
                phase_label = mapping['phase']
                for enh_id in mapping['enhancements']:
                    phase_assignments[wave_id][enh_id] = phase_label
        else:
            # No explicit mappings - enhancements are just sequential
            phase_ids = wave.get('phase_ids', [])
            for idx, enh_id in enumerate(phase_ids, 1):
                phase_assignments[wave_id][enh_id] = str(idx)

    # Validate each wave
    for wave in waves:
        wave_id = wave['id']
        wave_name = wave['name']

        print(f"\n{'-' * 80}")
        print(f"Wave {wave_id}: {wave_name}")
        print(f"{'-' * 80}")

        has_mappings = bool(wave.get('phase_mappings'))
        enhancement_ids = wave.get('phase_ids', [])

        if has_mappings:
            print(f"[OK] Has explicit phase mappings")
        else:
            print(f"[WARN] No explicit phase mappings (using sequential)")

        print(f"  Enhancements: {enhancement_ids}")
        print()

        if not enhancement_ids:
            print("  [WARN] Warning: No enhancements assigned to this wave")
            issues.append(f"Wave {wave_id}: No enhancements assigned")
            continue

        # Check each enhancement
        for enh_id in enhancement_ids:
            if enh_id not in enhancements:
                print(f"  [ERROR] Enhancement {enh_id}: File not found!")
                issues.append(f"Wave {wave_id}: Enhancement {enh_id} file not found")
                continue

            enh = enhancements[enh_id]
            title = enh['title']

            # Extract phase from title
            phase_in_title = extract_phase_from_title(title)

            # Get assigned phase from wave
            assigned_phase = phase_assignments[wave_id].get(enh_id)

            # Check consistency
            if assigned_phase and phase_in_title:
                if assigned_phase == phase_in_title:
                    print(f"  [OK] Enhancement {enh_id}: Phase {assigned_phase} (matches title)")
                else:
                    print(f"  [ERROR] Enhancement {enh_id}: Wave says Phase {assigned_phase}, title says Phase {phase_in_title}")
                    issues.append(f"Wave {wave_id}, Enhancement {enh_id}: Mismatch - wave={assigned_phase}, title={phase_in_title}")
            elif assigned_phase:
                print(f"  [WARN] Enhancement {enh_id}: Phase {assigned_phase} (not in title)")
                print(f"      Title: {title}")
            elif phase_in_title:
                print(f"  [WARN] Enhancement {enh_id}: Title has Phase {phase_in_title} but wave has no mapping")
            else:
                print(f"  [INFO] Enhancement {enh_id}: No phase info")
                print(f"      Title: {title}")

    # Summary
    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print()

    if not issues:
        print("[OK] All phase assignments are consistent!")
    else:
        print(f"[ERROR] Found {len(issues)} issues:")
        print()
        for issue in issues:
            print(f"  • {issue}")

    print()

    # Recommendations
    print("=" * 80)
    print("RECOMMENDATIONS")
    print("=" * 80)
    print()

    waves_without_mappings = [w for w in waves if not w.get('phase_mappings')]
    if waves_without_mappings:
        print(f"[WARN] {len(waves_without_mappings)} waves without explicit phase mappings:")
        for wave in waves_without_mappings:
            print(f"  • Wave {wave['id']}: {wave['name']}")
        print()
        print("  Consider adding **Phases**: field to these wave documents.")
        print("  See: context/waves/PHASE-SCHEMA.md")
    else:
        print("[OK] All waves have explicit phase mappings")

    print()

    return len(issues) == 0

if __name__ == '__main__':
    success = validate_phases()
    exit(0 if success else 1)
