// Lab-only catalog. Every entry names the Rust source that owns the real state.

const UI_LAB_SCENARIOS = [];

function defineScenario(id, group, title, source, patch) {
  UI_LAB_SCENARIOS.push({ id, group, title, source, patch });
}

defineScenario("genesis.dream.closed", "Genesis / Dream", "Dream · closed", "shell_host/dream.rs", { dream: true, view: "genesis", layoutT: -1, layoutGoal: -1 });
defineScenario("genesis.dream.ambient", "Genesis / Dream", "Dream · ambient", "shell_host/dream.rs", { dream: true, view: "genesis", layoutT: 0, layoutGoal: 0 });
defineScenario("genesis.dream.chat", "Genesis / Dream", "Dream · open chat", "shell_host/dream.rs", { dream: true, view: "genesis", layoutT: 1, layoutGoal: 1, dreamTab: "chat" });
defineScenario("genesis.dream.console", "Genesis / Dream", "Dream · console", "shell_host/dream.rs", { dream: true, view: "genesis", layoutT: 1, layoutGoal: 1, dreamTab: "console" });
defineScenario("genesis.dream.build", "Genesis / Dream", "Dream · build center", "shell_host/dream.rs", { dream: true, view: "genesis", layoutT: 1, layoutGoal: 1, dreamTab: "build", buildB: 72 });
defineScenario("genesis.dream.approval", "Genesis / Dream", "Dream · approval ready", "shell_host/dream.rs", { dream: true, view: "genesis", layoutT: 1, layoutGoal: 1, dreamTab: "build", buildB: 100 });
defineScenario("genesis.classic", "Genesis / Dream", "Classic Genesis reference", "shell_host/genesis.rs", { dream: false, view: "genesis" });
defineScenario("genesis.recovery", "Genesis / Dream", "Recovery context", "shell_host/recovery.rs", { dream: true, view: "genesis", layoutT: 0, layoutGoal: 0, recoveryOpen: true });
defineScenario("genesis.setup", "Genesis / Dream", "Trusted setup", "shell_host/genesis.rs", { dream: true, view: "setup", layoutT: 0, layoutGoal: 0 });

defineScenario("wifi.starting", "WiFi", "Starting WiFi", "shell_host/wifi_flow.rs::State::Starting", { dream: true, view: "wifi-starting", wifiProgress: 48 });
defineScenario("wifi.selecting", "WiFi", "WiFi networks", "shell_host/wifi_flow.rs::State::Selecting", { dream: true, view: "wifi-list" });
defineScenario("wifi.password.vault", "WiFi", "Password · Secret Vault", "shell_host/wifi_flow.rs::State::Password", { dream: true, view: "wifi-password", wifiSelected: 0, wifiPassword: "correcthorse", wifiStorage: "vault" });
defineScenario("wifi.password.legacy", "WiFi", "Password · legacy RAM", "shell_host/wifi_flow.rs::State::Password", { dream: true, view: "wifi-password", wifiSelected: 0, wifiPassword: "temporary", wifiStorage: "legacy", wifiRemember: true });
defineScenario("wifi.associating", "WiFi", "Connecting WiFi", "shell_host/wifi_flow.rs::State::Associating", { dream: true, view: "wifi-connecting", wifiProgress: 76 });
defineScenario("wifi.configured.vault", "WiFi", "Configured · Secret Vault", "shell_host/wifi_flow.rs::State::Configured", { dream: true, view: "wifi-configured", wifiSelected: 0, wifiConnected: true, wifiStorage: "vault" });
defineScenario("wifi.configured.open", "WiFi", "Configured · open network", "shell_host/wifi_flow.rs::State::Configured", { dream: true, view: "wifi-configured", wifiSelected: 2, wifiConnected: false, wifiStorage: "open" });
defineScenario("wifi.failed", "WiFi", "WiFi unavailable", "shell_host/wifi_flow.rs::State::Failed", { dream: true, view: "wifi-failed", wifiFailureReason: "wifi_device_not_detected" });

defineScenario("vault.showing", "Secret Vault", "Recovery key · shown once", "shell_host/vault_flow.rs::Mode::Showing", { dream: true, view: "vault", vaultMode: "showing" });
defineScenario("vault.confirming", "Secret Vault", "Confirm recovery key", "shell_host/vault_flow.rs::Mode::Confirming", { dream: true, view: "vault", vaultMode: "confirming", vaultMaskedLen: 80 });
defineScenario("vault.unlocking", "Secret Vault", "Unlock Secret Vault", "shell_host/vault_flow.rs::Mode::Unlocking", { dream: true, view: "vault", vaultMode: "unlocking", vaultMaskedLen: 46 });
defineScenario("vault.managing", "Secret Vault", "Manage Secret Vault", "shell_host/vault_flow.rs::Mode::Managing", { dream: true, view: "vault", vaultMode: "managing", vaultManage: "forget-provider" });
defineScenario("vault.forget.provider", "Secret Vault", "Forget provider · confirm", "shell_host/vault_flow.rs::Mode::ConfirmingForget", { dream: true, view: "vault", vaultMode: "confirming-forget", vaultForgetTarget: "provider", vaultConfirm: "forget" });
defineScenario("vault.forget.wifi", "Secret Vault", "Forget WiFi · confirm", "shell_host/vault_flow.rs::Mode::ConfirmingForget", { dream: true, view: "vault", vaultMode: "confirming-forget", vaultForgetTarget: "wifi", vaultConfirm: "cancel" });
defineScenario("vault.provider-entry", "Secret Vault", "Save provider API key", "shell_host/vault_flow.rs::Mode::ProviderEntry", { dream: true, view: "vault", vaultMode: "provider-entry", vaultMaskedLen: 28 });
defineScenario("vault.wifi-test-entry", "Secret Vault", "Contained WiFi proof", "shell_host/vault_flow.rs::Mode::WifiTestEntry", { dream: true, view: "vault", vaultMode: "wifi-test-entry", vaultMaskedLen: 18 });

const vaultOutcomeTitles = {
  provisioned: "Provisioned",
  unlocked: "Unlocked",
  rejected: "Recovery rejected",
  unavailable: "Vault unavailable",
  "provider-saved": "Provider saved",
  "provider-rejected": "Provider rejected",
  "provider-unavailable": "Provider unavailable",
  "wifi-test-saved": "WiFi proof saved",
  "wifi-safe-reconnected": "SAFE WiFi reconnected",
  "wifi-test-rejected": "WiFi proof rejected",
  "wifi-test-unavailable": "WiFi proof unavailable",
  "provider-forgotten": "Provider forgotten",
  "provider-forget-rejected": "Provider forget rejected",
  "wifi-forgotten": "WiFi forgotten",
  "wifi-forget-rejected": "WiFi forget rejected",
};
Object.entries(vaultOutcomeTitles).forEach(([outcome, title]) => {
  defineScenario(`vault.outcome.${outcome}`, "Secret Vault outcomes", title, "shell_host/vault_flow.rs::Outcome", {
    dream: true,
    view: "vault",
    vaultMode: "outcome",
    vaultOutcome: outcome,
  });
});

defineScenario("personal.proof.idle", "Personal surface", "Signed shell proof · ready", "svc-personal-shell-proof/src/lib.rs", { dream: false, view: "personal", personalMode: "proof", personalInputReceived: false });
defineScenario("personal.proof.input", "Personal surface", "Signed shell proof · input", "svc-personal-shell-proof/src/lib.rs", { dream: false, view: "personal", personalMode: "proof", personalInputReceived: true });
defineScenario("personal.calculator", "Personal surface", "Calculator program", "raios-core/src/ui_program.rs::calculator_program", { dream: false, view: "personal", personalMode: "calculator", calculatorValue: 42 });
defineScenario("personal.editor", "Personal surface", "Current-boot editor", "raios-core/src/ui_program.rs::editor_program", { dream: false, view: "personal", personalMode: "editor" });

function resetScenarioState() {
  Object.assign(state, {
    scenario: "",
    dream: true,
    dreamTab: "chat",
    buildB: 34,
    approved: false,
    hoverCenter: false,
    hoverId: null,
    hoverT: 0,
    layoutT: -1,
    layoutGoal: -1,
    view: "genesis",
    pointerDown: false,
    composerFocus: false,
    composerText: "",
    recoveryOpen: false,
    keyboardLayout: "US",
    chat: [],
    chatScroll: 0,
    wifiSelected: null,
    wifiPassword: "",
    wifiRejected: false,
    wifiProgress: 0,
    wifiConnected: false,
    wifiStorage: "vault",
    wifiRemember: false,
    wifiFailureReason: "wifi_device_not_detected",
    vaultMode: "closed",
    vaultMaskedLen: 24,
    vaultManage: "forget-provider",
    vaultForgetTarget: "provider",
    vaultConfirm: "cancel",
    vaultOutcome: "provisioned",
    personalMode: null,
    personalInputReceived: false,
    calculatorValue: 42,
    editorText: "The current-boot editor is rendered through the bounded personal surface.",
    overlayAnim: 1,
  });
}

function applyScenario(id, shouldRender = true) {
  const scenario = UI_LAB_SCENARIOS.find((candidate) => candidate.id === id);
  if (!scenario) return false;
  resetScenarioState();
  Object.assign(state, scenario.patch);
  state.scenario = scenario.id;
  if (shouldRender && typeof render === "function") render();
  updateScenarioCatalog();
  return true;
}

function buildScenarioCatalog() {
  if (!labCatalog) return;
  labCatalog.replaceChildren();
  const header = document.createElement("header");
  const heading = document.createElement("h2");
  heading.textContent = "LAB · aktuelle UI-Zustände";
  const hint = document.createElement("p");
  hint.textContent = "F2 schließt · Jeder Eintrag nennt seinen Rust-Owner.";
  header.append(heading, hint);
  labCatalog.append(header);

  let group = "";
  for (const scenario of UI_LAB_SCENARIOS) {
    if (scenario.group !== group) {
      group = scenario.group;
      const title = document.createElement("h3");
      title.textContent = group;
      labCatalog.append(title);
    }
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.scenario = scenario.id;
    button.textContent = scenario.title;
    const source = document.createElement("small");
    source.textContent = scenario.source;
    button.append(source);
    button.addEventListener("click", () => applyScenario(scenario.id));
    labCatalog.append(button);
  }
  updateScenarioCatalog();
}

function updateScenarioCatalog() {
  if (!labCatalog) return;
  labCatalog.querySelectorAll("button[data-scenario]").forEach((button) => {
    button.setAttribute("aria-current", String(button.dataset.scenario === state.scenario));
  });
}

function toggleScenarioCatalog(force) {
  if (!labCatalog) return;
  const open = force === undefined ? labCatalog.hidden : force;
  labCatalog.hidden = !open;
}
