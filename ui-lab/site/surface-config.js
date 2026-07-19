// Asset-only configuration for the Website mode. Paths are resolved relative
// to raios-ui-lab.html. The display rectangle below is measured directly from
// the supplied 2304x4096 display cutout; no crop or perspective is guessed.
const SURFACE_ASSET_CONFIG = Object.freeze({
  photo: "ui-lab/assets/surface/surface-photo-portrait-q88.webp",
  reflection: "ui-lab/assets/surface/surface-reflection-portrait-crop.webp",
  aspectRatio: "9 / 16",
  photoScale: 1.7,
  // The display stays 64px below the action row on regular viewports. On
  // wide layouts the gap grows with the composition so the device does not
  // appear to slide underneath the buttons.
  displayGap: 64,
  displayGapViewportPercent: 4.5,
  displayGapMax: 112,
  display: Object.freeze({
    // Measured from the screen cutout in the 2304x4096 source image:
    // x=636..1735, y=1001..1641 (1100x641, near-16:9 panel).
    left: "27.604166666667%",
    top: "24.4384765625%",
    width: "47.743055555556%",
    height: "15.6494140625%",
    radius: "0%",
  }),
  reflectionRect: Object.freeze({
    // Cropped source x=635..1736, y=1000..1642 (one pixel around display).
    left: "27.560763888889%",
    top: "24.4140625%",
    width: "47.829861111111%",
    height: "15.6982421875%",
  }),
});
