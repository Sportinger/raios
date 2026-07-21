// Browser-only presentation shell. Es aendert keine raiOS-Semantik: dieselbe
// responsive Geometrie wird bis maximal 1920x1080 in das Foto komponiert.

const viewModeToggle = document.getElementById("view-mode-toggle");
const viewModeLabel = viewModeToggle.querySelector(".view-mode-toggle__label");
const websiteMode = document.getElementById("website-mode");
const websiteIntro = websiteMode.querySelector(".website-intro");
const websiteTitle = websiteIntro.querySelector("h1");
const websiteActionsRow = websiteIntro.querySelector(".website-facts");
const surfaceSection = websiteMode.querySelector(".surface-section");
const surfaceComposite = document.getElementById("surface-composite");
const surfacePhoto = document.getElementById("surface-photo");
const surfaceReflection = document.getElementById("surface-reflection");
let activeSurfaceConfig = SURFACE_ASSET_CONFIG;
let surfaceAlignmentFrame = 0;

function shouldUseWebsiteMode(searchParams, protocol) {
  const explicitMode = searchParams.get("site");
  if (explicitMode !== null) return explicitMode === "1";
  return protocol === "http:" || protocol === "https:";
}

function surfaceDisplayGapForViewport(config, viewportWidth) {
  const minimum = Number(config.displayGap) || 0;
  const preferred = viewportWidth * (Number(config.displayGapViewportPercent) || 0) / 100;
  const maximum = Number(config.displayGapMax) || minimum;
  return Math.min(maximum, Math.max(minimum, preferred));
}

function alignSurfaceDisplay() {
  if (document.body.dataset.shellMode !== "website") return;
  const displayTopRatio = Number.parseFloat(activeSurfaceConfig.display.top) / 100;
  const displayLeftRatio = Number.parseFloat(activeSurfaceConfig.display.left) / 100;
  const displayWidthRatio = Number.parseFloat(activeSurfaceConfig.display.width) / 100;
  const [aspectWidth, aspectHeight] = activeSurfaceConfig.aspectRatio
    .split("/")
    .map((value) => Number.parseFloat(value));
  const heightPerWidth = aspectHeight / aspectWidth;
  const surfaceTop = surfaceSection.getBoundingClientRect().top;
  const displayGap = surfaceDisplayGapForViewport(activeSurfaceConfig, window.innerWidth);
  // Keep the actual display cutout pinned to the action row. Both values are
  // viewport-relative, so browser zoom, scrolling and responsive reflow cancel
  // out instead of moving the photographed device underneath the buttons.
  const actionBottom = websiteActionsRow.getBoundingClientRect().bottom - surfaceTop;
  const desiredDisplayTop = actionBottom + displayGap;
  const nativePhotoWidth = PHYSICAL_WIDTH / displayWidthRatio;
  const baseWidth = Math.max(
    window.innerWidth,
    Math.min(window.innerWidth * activeSurfaceConfig.photoScale, nativePhotoWidth),
  );
  const requiredTopCoverWidth = desiredDisplayTop / (heightPerWidth * displayTopRatio);
  const responsiveWidth = Math.max(baseWidth, requiredTopCoverWidth);
  const naturalDisplayTop = responsiveWidth * heightPerWidth * displayTopRatio;
  const offsetY = Math.min(0, desiredDisplayTop - naturalDisplayTop);
  const displayCenterRatio = displayLeftRatio + displayWidthRatio / 2;
  const offsetX = -(displayCenterRatio - 0.5) * responsiveWidth;
  surfaceComposite.style.setProperty("--surface-responsive-width", `${responsiveWidth}px`);
  surfaceComposite.style.setProperty("--surface-offset-x", `${offsetX}px`);
  surfaceComposite.style.setProperty("--surface-offset-y", `${offsetY}px`);
  // Story sections start right below the display cutout, layered over the
  // remaining photo. The scrim keeps their text readable on the photo.
  const displayHeightRatio = Number.parseFloat(activeSurfaceConfig.display.height) / 100;
  const photoHeight = responsiveWidth * heightPerWidth;
  const displayBottom = offsetY + photoHeight * (displayTopRatio + displayHeightRatio);
  const photoBottom = offsetY + photoHeight;
  const storyStart = displayBottom + displayGap * 2;
  const gridPhotoOverlap = Math.min(2200, Math.max(1400, window.innerHeight * 1.4));
  websiteMode.style.setProperty("--story-overlap", `${Math.max(0, photoBottom - storyStart)}px`);
  websiteMode.style.setProperty(
    "--website-grid-start-y",
    `${photoBottom - gridPhotoOverlap}px`,
  );
}

function syncWebsiteRendererResolution() {
  if (document.body.dataset.shellMode !== "website") return false;
  const displayRect = cv.getBoundingClientRect();
  const resolution = rendererResolutionForDisplay(displayRect.width);
  const changed = setRendererResolution(resolution.width, resolution.height);
  if (changed && typeof render === "function") render();
  if (typeof syncPointerFromViewport === "function") syncPointerFromViewport();
  return changed;
}

function scheduleSurfaceAlignment() {
  cancelAnimationFrame(surfaceAlignmentFrame);
  surfaceAlignmentFrame = requestAnimationFrame(() => {
    alignSurfaceDisplay();
    syncWebsiteRendererResolution();
  });
}

function setSurfaceAsset(image, source) {
  if (!source) {
    image.hidden = true;
    image.removeAttribute("src");
    return false;
  }
  image.src = source;
  image.hidden = false;
  return true;
}

function configureSurfaceComposite(config) {
  activeSurfaceConfig = config;
  const display = config.display;
  surfaceComposite.style.setProperty("--surface-aspect", config.aspectRatio);
  surfaceComposite.style.setProperty("--display-left", display.left);
  surfaceComposite.style.setProperty("--display-top", display.top);
  surfaceComposite.style.setProperty("--display-width", display.width);
  surfaceComposite.style.setProperty("--display-height", display.height);
  surfaceComposite.style.setProperty("--display-radius", display.radius);
  surfaceComposite.style.setProperty("--reflection-left", config.reflectionRect.left);
  surfaceComposite.style.setProperty("--reflection-top", config.reflectionRect.top);
  surfaceComposite.style.setProperty("--reflection-width", config.reflectionRect.width);
  surfaceComposite.style.setProperty("--reflection-height", config.reflectionRect.height);
  const hasPhoto = setSurfaceAsset(surfacePhoto, config.photo);
  const hasReflection = setSurfaceAsset(surfaceReflection, config.reflection);
  surfaceComposite.dataset.assetsReady = String(hasPhoto && hasReflection);
  scheduleSurfaceAlignment();
}

function setWebsiteMode(enabled, syncUrl = false) {
  document.body.dataset.shellMode = enabled ? "website" : "ui";
  viewModeToggle.setAttribute("aria-pressed", String(enabled));
  viewModeLabel.textContent = enabled ? "UI Lab" : "Website";
  viewModeToggle.setAttribute(
    "aria-label",
    enabled ? "Zur vollständigen UI-Vorschau wechseln" : "Zur raiOS-Website-Vorschau wechseln",
  );
  if (enabled) {
    toggleScenarioCatalog(false);
    state.pointer = null;
    if (typeof present === "function") present();
    alignSurfaceDisplay();
    scheduleSurfaceAlignment();
  } else {
    if (setRendererResolution(PHYSICAL_WIDTH, PHYSICAL_HEIGHT) && typeof render === "function") {
      render();
    }
    cv.focus();
  }
  if (syncUrl) {
    const url = new URL(location.href);
    if (enabled) url.searchParams.set("site", "1");
    else if (location.protocol === "http:" || location.protocol === "https:") {
      url.searchParams.set("site", "0");
    } else {
      url.searchParams.delete("site");
    }
    history.replaceState(null, "", url);
  }
  document.dispatchEvent(new CustomEvent("raios:website-mode", {
    detail: { enabled },
  }));
}

configureSurfaceComposite(SURFACE_ASSET_CONFIG);
const websiteParams = new URLSearchParams(location.search);
const hostedWebsite = location.protocol === "http:" || location.protocol === "https:";
if (websiteParams.get("chrome") === "0" || (hostedWebsite && websiteParams.get("chrome") !== "1")) {
  document.body.classList.add("lab-chrome-hidden");
}
setWebsiteMode(shouldUseWebsiteMode(websiteParams, location.protocol));

viewModeToggle.addEventListener("click", () => {
  setWebsiteMode(document.body.dataset.shellMode !== "website", true);
});

window.addEventListener("resize", scheduleSurfaceAlignment);
if (typeof ResizeObserver === "function") {
  const surfaceAlignmentObserver = new ResizeObserver(scheduleSurfaceAlignment);
  surfaceAlignmentObserver.observe(websiteIntro);
  surfaceAlignmentObserver.observe(websiteActionsRow);
  surfaceAlignmentObserver.observe(surfaceComposite);
}
if (document.fonts && document.fonts.ready) {
  document.fonts.ready.then(scheduleSurfaceAlignment);
}

window.__RAIOS_SURFACE_COMPOSITE__ = {
  configure: configureSurfaceComposite,
  setWebsiteMode,
  align: alignSurfaceDisplay,
  syncRendererResolution: syncWebsiteRendererResolution,
};
