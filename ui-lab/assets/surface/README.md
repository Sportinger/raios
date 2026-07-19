# Surface composite assets

The Website mode uses an optimized pair derived from the user-supplied
2304x4096 portrait images:

1. `surface-photo-portrait-q88.webp` — the complete photo as RGB WebP; the
   live canvas sits above it, so the photo needs no alpha channel.
2. `surface-reflection-portrait-crop.webp` — an RGBA WebP cropped to
   1102x643, containing only the glass glare above the live display.

Their paths and the measured display rectangle live in
`ui-lab/site/surface-config.js`. The screen rectangle is exactly
`x=636..1735`, `y=1001..1641`; the reflection layer extends one pixel
beyond it for clean edge coverage. The stack is deliberately fixed:

```text
reflection overlay  (top, pointer-events: none)
live raiOS canvas   (middle, exact display rectangle)
opaque surface photo (bottom)
```

The photographed panel is close to 16:9 and the canonical raiOS preview is
exactly 16:9, so the canvas is fitted without distortion and receives only
narrow black letterbox bars. The preview stays interactive and every F2
scenario can appear inside the photographed display.

Website mode measures the intro after every responsive resize and keeps the
physical display cutout exactly 64 CSS pixels below it. The alignment comes
from the configured display rectangle rather than a viewport-specific Y offset.

The optimized pair is about 1.19 MB instead of 13.79 MB (roughly 91% smaller).
Only the cropped reflection retains alpha. It is composited with CSS `screen`
blending at 72% opacity: dark pixels leave the UI intact while the photographed
highlights and room reflections are added above it.
