---
name: r-wave
description: Manage apportionment waves in context/waves: find active wave, show status, create next pulse, and close waves.
user-invocable: true
---

# r-wave

Use this skill to operate the apportionment wave system.

## Commands

```text
/r-wave status
/r-wave next
/r-wave close
```

## Source Of Truth

- Wave index: `context/waves/PHASES.md`
- Active wave: first row with `status: active`
- Active wave card: `context/waves/{active}/WAVE.md`
- Pulse plans: `context/waves/{active}/pulses/`
- Fork contexts: `context/waves/{active}/forks/`
- Review panels: `context/waves/{active}/panels/`

## Status Procedure

1. Read `context/waves/PHASES.md`.
2. Resolve the active wave directory.
3. Read its `WAVE.md`.
4. List pulses in order.
5. Report:
   - active wave
   - completed pulses
   - next `status: todo` pulse
   - validation gate
   - known carry-forwards

## Next Procedure

1. Resolve the next todo pulse.
2. Read the pulse completely.
3. Run the pulse's pre-implementation scout commands.
4. Implement the deliverables.
5. Check off completed pulse gates.
6. Update `WAVE.md` pulse status.
7. Run the pulse validation commands.

## Close Procedure

Close only when every pulse in `WAVE.md` is done or explicitly deferred:

1. Write `CLOSE.md`.
2. Update `context/waves/PHASES.md` status.
3. Update goal/spec docs referenced by the wave.
4. Run final validation.

## Rules

- Do not strengthen claim boundaries in close text.
- Do not mark a pulse done unless its validation ran or the blocker is written.
- Keep old numbered wave directories as history.

