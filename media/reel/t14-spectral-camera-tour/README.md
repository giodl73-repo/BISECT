# T.14 spectral camera-tour private preview

This package tours BISECT's owner-authored Laplacian-to-ordering figure without
moving graph or partition semantics into REEL.

The package pins REEL v0.2.48 and combines `visual_fit: contain`, a still
`camera_track`, and `reserve-caption-band` picture composition. The first and
final states preserve the complete wide figure; the intermediate states focus
the adjacency graph, Laplacian/Fiedler calculation, and stable spectral
ordering. Keyframe centers remain inside the reachable crop range so motion
review measures no clamped stalls.

Render from the repository root:

```powershell
C:\src\REEL\target\release\reel.exe animatic-render `
  media\reel\t14-spectral-camera-tour\manifest.yaml `
  --asset-root . `
  --silent `
  --captions media\reel\t14-spectral-camera-tour\captions.srt `
  --caption-picture-layout reserve-caption-band `
  --output bisect-t14-spectral-camera-tour.mp4 `
  --format json
```

The accepted private-preview render is 1280x720 at 24 FPS for 15 seconds. Its
SHA-256 is
`db1ff062a9fc3d4ad508b6f4bc5e6c168a9fad46d0dcb9d1345bf3dd5797028a`.
The binary is evidence, not a tracked repository artifact.

REEL fits the picture into a 1280x520 region and pads it to the 1280x720
delivery canvas before applying captions. Caption-layout evidence confirms that
the caption region beginning at y=520 does not intersect the picture region.
Captions remain private-review dialogue rather than publication evidence.
