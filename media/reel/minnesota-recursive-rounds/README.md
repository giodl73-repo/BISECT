# Minnesota recursive-round private preview

This package turns BISECT's three retained Minnesota round figures into a
15-second, sound-off explainer without changing their map semantics or claim
posture.

The package pins REEL v0.2.46 and opts into `visual_fit: contain`, preserving
each portrait figure's title and complete map extent in a 1280x720 review
artifact. The three frames remain owner-rendered BISECT inputs with explicit
five-second holds.

Render from the repository root:

```powershell
C:\src\REEL\target\release\reel.exe animatic-render `
  media\reel\minnesota-recursive-rounds\manifest.yaml `
  --asset-root . `
  --silent `
  --captions media\reel\minnesota-recursive-rounds\captions.srt `
  --output bisect-minnesota-recursive-rounds.mp4 `
  --format json
```

The accepted private-preview render is 1280x720 at 24 FPS for 15 seconds. Its
SHA-256 is
`26abf5f008f3a329254e62225e98e6458887c7e216e9fbc671796b4d3a35f00c`.
The binary is evidence, not a tracked repository artifact.
