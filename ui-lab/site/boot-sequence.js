// Website presentation only: a deliberately brief reconstruction of the
// visible Limine handoff before the Genesis framebuffer takes over.

const bootSequence = document.getElementById("boot-sequence");
const bootReplay = document.getElementById("boot-replay");
const bootLog = bootSequence.querySelector(".boot-sequence__log");
const bootParams = new URLSearchParams(location.search);
const bootReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

const BOOT_LINES = Object.freeze([
  { at: 230, text: "Loading entry: /raiOS" },
  { at: 300, text: "Loading kernel: boot():/kernel/kernel.elf" },
  { at: 370, text: "Protocol: Limine" },
  { at: 440, text: "Video mode: 1920x1080x32" },
  { at: 510, text: "Booting kernel..." },
]);

let bootTimers = [];
let bootGeneration = 0;
let bootPlayed = false;

function scheduleBoot(callback, delay) {
  const timer = window.setTimeout(callback, delay);
  bootTimers.push(timer);
  return timer;
}

function clearBootTimers() {
  for (const timer of bootTimers) window.clearTimeout(timer);
  bootTimers = [];
}

function appendBootLine(text) {
  const line = document.createElement("p");
  line.className = "boot-sequence__line";
  line.textContent = text;
  bootLog.append(line);
}

function finishBoot(immediate = false) {
  const generation = ++bootGeneration;
  clearBootTimers();
  bootSequence.dataset.state = "leaving";
  const delay = immediate || bootReducedMotion.matches ? 1 : 60;
  scheduleBoot(() => {
    if (generation !== bootGeneration) return;
    bootSequence.hidden = true;
    bootSequence.dataset.state = "idle";
    bootSequence.setAttribute("aria-hidden", "true");
    bootReplay.hidden = document.body.dataset.shellMode !== "website";
  }, delay);
}

function startBoot() {
  if (document.body.dataset.shellMode !== "website") return false;
  bootPlayed = true;
  const generation = ++bootGeneration;
  clearBootTimers();
  bootLog.replaceChildren();
  bootSequence.dataset.state = "running";
  bootSequence.hidden = false;
  bootSequence.removeAttribute("aria-hidden");
  bootSequence.setAttribute("aria-label", "Kurzer simulierter Limine-Start");
  bootReplay.hidden = true;

  const speed = bootReducedMotion.matches ? 0.25 : 1;
  for (const line of BOOT_LINES) {
    scheduleBoot(() => {
      if (generation === bootGeneration) appendBootLine(line.text);
    }, Math.round(line.at * speed));
  }
  scheduleBoot(() => {
    if (generation === bootGeneration) finishBoot();
  }, Math.round(650 * speed));
  return true;
}

bootReplay.addEventListener("click", startBoot);

document.addEventListener("raios:website-mode", (event) => {
  if (!event.detail.enabled) {
    finishBoot(true);
    bootReplay.hidden = true;
  } else if (!bootPlayed && bootParams.get("boot") !== "0") {
    startBoot();
  } else {
    bootReplay.hidden = false;
  }
});

if (document.body.dataset.shellMode === "website" && bootParams.get("boot") !== "0") {
  startBoot();
} else {
  bootSequence.hidden = true;
  bootSequence.setAttribute("aria-hidden", "true");
  bootReplay.hidden = document.body.dataset.shellMode !== "website";
}

window.__RAIOS_BOOT_SEQUENCE__ = Object.freeze({
  start: startBoot,
  skip: () => finishBoot(),
  lines: BOOT_LINES,
});
