# T.14 spectral camera-tour private preview

This package tours BISECT's owner-authored Laplacian-to-ordering figure without
moving graph or partition semantics into REEL.

The package pins REEL v0.2.47 and combines `visual_fit: contain` with a still
`camera_track`. The first and final states preserve the complete wide figure;
the intermediate states focus the adjacency graph, Laplacian/Fiedler
calculation, and stable spectral ordering. Keyframe centers remain inside the
reachable crop range so motion review measures no clamped stalls.

Render from the repository root:

```powershell
C:\src\REEL\target\release\reel.exe animatic-render `
  media\reel\t14-spectral-camera-tour\manifest.yaml `
  --asset-root . `
  --silent `
  --captions media\reel\t14-spectral-camera-tour\captions.srt `
  --output bisect-t14-spectral-camera-tour.mp4 `
  --format json
```

The accepted private-preview render is 1280x720 at 24 FPS for 15 seconds. Its
SHA-256 is
`0ee8f0b8889c2de64bfa6f7f9bd0d264685a184bce2d8229c5e25d7ec03e6b63`.
The binary is evidence, not a tracked repository artifact.

Lower-third captions can overlap the figure's bottom annotations. The graph,
matrix, and ordering geometry remain legible in this capability proof, but a
caption-safe picture region is separate follow-up work before publication use.
