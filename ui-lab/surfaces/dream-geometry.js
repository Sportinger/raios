// ---------- Dream-Layout: exakte responsive Rust-Geometrie ----------

const DREAM_TOKENS = {
  outerMargin: 14,
  railWidth: 130,
  closedSeamGap: 20,
  closedCenterWidth: 8,
  bracketTop: 40,
  bracketBottomMargin: 18,
  bracketCenterOffset: 17,
  ambientBracketAmp: 14,
  openBracketAmp: 16,
  minCenterWidth: 240,
  maxCenterWidth: 480,
  minCenterCorridor: 132,
  composerBottomOffset: 44,
  conversationTop: 70,
  conversationComposerGap: 16,
};

const satSub = (value, amount) => Math.max(0, value - amount);

function responsiveRailWidth(logicalWidth) {
  const available = satSub(satSub(logicalWidth, DREAM_TOKENS.outerMargin * 2), DREAM_TOKENS.minCenterCorridor);
  return Math.min(DREAM_TOKENS.railWidth, Math.floor(available / 2));
}

function responsiveCenterWidth(logicalWidth, railWidth, numerator, denominator, bracketAmp) {
  const rawDesired = Math.floor(logicalWidth * numerator / denominator);
  const desired = Math.max(DREAM_TOKENS.minCenterWidth, Math.min(DREAM_TOKENS.maxCenterWidth, rawDesired));
  const corridor = satSub(satSub(logicalWidth, DREAM_TOKENS.outerMargin * 2), railWidth * 2);
  const reserve = (DREAM_TOKENS.bracketCenterOffset + bracketAmp) * 2;
  return Math.min(desired, satSub(corridor, reserve));
}

function dreamKeyframes(physicalWidth = PHYSICAL_WIDTH, physicalHeight = PHYSICAL_HEIGHT) {
  const logicalWidth = Math.floor(physicalWidth / SCALE);
  const logicalHeight = Math.floor(physicalHeight / SCALE);
  const midpoint = Math.floor(logicalWidth / 2);
  const railWidth = responsiveRailWidth(logicalWidth);
  const rightX = satSub(satSub(logicalWidth, DREAM_TOKENS.outerMargin), railWidth);
  const common = {
    logicalWidth,
    logicalHeight,
    bracketTop: Math.min(DREAM_TOKENS.bracketTop, satSub(logicalHeight, 1)),
    bracketBottom: Math.min(
      satSub(logicalHeight, 1),
      Math.max(satSub(logicalHeight, DREAM_TOKENS.bracketBottomMargin), Math.min(DREAM_TOKENS.bracketTop, satSub(logicalHeight, 1))),
    ),
    composerY: satSub(logicalHeight, DREAM_TOKENS.composerBottomOffset),
  };
  const closedCenterX = satSub(midpoint, Math.floor(DREAM_TOKENS.closedCenterWidth / 2));
  const closedLeftWidth = satSub(satSub(midpoint, DREAM_TOKENS.closedSeamGap), DREAM_TOKENS.outerMargin);
  const closedRightX = Math.min(midpoint + DREAM_TOKENS.closedSeamGap, satSub(logicalWidth, DREAM_TOKENS.outerMargin));
  const keyframe = (centerX, centerW, bracketLeft, bracketRight, amp, hook, leftX, leftW, frameRightX, rightW) => ({
    ...common,
    center: { x: centerX, w: Math.min(centerW, logicalWidth) },
    brackets: { left: bracketLeft, right: bracketRight, amp, hook },
    left: { x: leftX, w: leftW },
    right: { x: frameRightX, w: rightW },
  });
  const closed = keyframe(
    closedCenterX,
    DREAM_TOKENS.closedCenterWidth,
    satSub(midpoint, 1),
    Math.min(midpoint + 1, logicalWidth),
    7,
    0,
    Math.min(DREAM_TOKENS.outerMargin, logicalWidth),
    closedLeftWidth,
    closedRightX,
    satSub(satSub(logicalWidth, DREAM_TOKENS.outerMargin), closedRightX),
  );
  const ambientW = responsiveCenterWidth(logicalWidth, railWidth, 27, 64, DREAM_TOKENS.ambientBracketAmp);
  const ambientX = satSub(midpoint, Math.floor(ambientW / 2));
  const ambient = keyframe(
    ambientX,
    ambientW,
    satSub(ambientX, DREAM_TOKENS.bracketCenterOffset),
    ambientX + ambientW + satSub(DREAM_TOKENS.bracketCenterOffset, 2),
    DREAM_TOKENS.ambientBracketAmp,
    7,
    Math.min(DREAM_TOKENS.outerMargin, logicalWidth),
    railWidth,
    rightX,
    railWidth,
  );
  const openW = responsiveCenterWidth(logicalWidth, railWidth, 143, 320, DREAM_TOKENS.openBracketAmp);
  const openX = satSub(midpoint, Math.floor(openW / 2));
  const open = keyframe(
    openX,
    openW,
    satSub(openX, DREAM_TOKENS.bracketCenterOffset),
    openX + openW + satSub(DREAM_TOKENS.bracketCenterOffset, 2),
    DREAM_TOKENS.openBracketAmp,
    7,
    Math.min(DREAM_TOKENS.outerMargin, logicalWidth),
    railWidth,
    rightX,
    railWidth,
  );
  return { closed, ambient, open };
}

function lerp(a, b, t) { return a + (b - a) * t; }

function lerpDreamLayout(a, b, t) {
  return {
    logicalWidth: a.logicalWidth,
    logicalHeight: a.logicalHeight,
    bracketTop: a.bracketTop,
    bracketBottom: a.bracketBottom,
    composerY: a.composerY,
    center: { x: lerp(a.center.x, b.center.x, t), w: lerp(a.center.w, b.center.w, t) },
    brackets: {
      left: lerp(a.brackets.left, b.brackets.left, t),
      right: lerp(a.brackets.right, b.brackets.right, t),
      amp: lerp(a.brackets.amp, b.brackets.amp, t),
      hook: lerp(a.brackets.hook, b.brackets.hook, t),
    },
    left: { x: lerp(a.left.x, b.left.x, t), w: lerp(a.left.w, b.left.w, t) },
    right: { x: lerp(a.right.x, b.right.x, t), w: lerp(a.right.w, b.right.w, t) },
  };
}

function dreamLayoutAt(layoutT, physicalWidth = PHYSICAL_WIDTH, physicalHeight = PHYSICAL_HEIGHT) {
  const t = Math.max(-1, Math.min(1, layoutT));
  const layouts = dreamKeyframes(physicalWidth, physicalHeight);
  return t < 0
    ? lerpDreamLayout(layouts.closed, layouts.ambient, t + 1)
    : lerpDreamLayout(layouts.ambient, layouts.open, t);
}

function dreamLayout() { return dreamLayoutAt(state.layoutT, cv.width, cv.height); }

// Hintergrund einmalig vorrendern (Kernel wuerde das ebenso in einen Puffer
// backen): Bayer-Dithering zwischen 3 Dunkeltoenen, Mitte leicht heller,
// dazu ein deterministisches, sehr dezentes Sternenfeld.
