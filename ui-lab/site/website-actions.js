// Website-only mock actions. They deliberately never navigate or download.

const websiteActionStatus = document.getElementById("website-action-status");
const websiteActionButtons = [...document.querySelectorAll("[data-mock-action]")];
const WEBSITE_ACTION_MESSAGES = Object.freeze({
  iso: "Mockup · Der ISO-Download ist noch nicht verbunden.",
  installer: "Mockup · Das Installer Tool ist noch nicht verbunden.",
  drivers: "Mockup · Die Treiberliste ist noch nicht verbunden.",
});
let websiteActionTimer = 0;

function runWebsiteMockAction(button) {
  for (const candidate of websiteActionButtons) candidate.removeAttribute("data-active");
  button.dataset.active = "true";
  websiteActionStatus.textContent = WEBSITE_ACTION_MESSAGES[button.dataset.mockAction];
  websiteActionStatus.dataset.visible = "true";
  window.clearTimeout(websiteActionTimer);
  websiteActionTimer = window.setTimeout(() => {
    button.removeAttribute("data-active");
    websiteActionStatus.removeAttribute("data-visible");
  }, 2400);
}

for (const button of websiteActionButtons) {
  button.addEventListener("click", () => runWebsiteMockAction(button));
}
