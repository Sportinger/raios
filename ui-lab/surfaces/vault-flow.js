// Mirrors seed-kernel/src/shell_host/vault_flow.rs.

const VAULT_RECOVERY_KEY_LEN = 80;
const VAULT_RECOVERY_ROW_LEN = 40;

function vaultActionRects(rect) {
  const width = Math.floor((rect.w - 52) / 2);
  const y = rect.y + rect.h - 44;
  return [
    { x: rect.x + 20, y, w: width, h: 24 },
    { x: rect.x + 32 + width, y, w: width, h: 24 },
  ];
}

function vaultManageRects(rect) {
  const x = rect.x + 20;
  const y = rect.y + 92;
  const w = rect.w - 40;
  return [
    { x, y, w, h: 24 },
    { x, y: y + 32, w, h: 24 },
    { x, y: y + 64, w, h: 24 },
  ];
}

function drawVaultMasked(x, y, length) {
  const count = Math.max(0, Math.min(VAULT_RECOVERY_KEY_LEN, length));
  drawText(x, y, "*".repeat(Math.min(count, VAULT_RECOVERY_ROW_LEN)), "TEXT_MAIN");
  if (count > VAULT_RECOVERY_ROW_LEN) {
    drawText(x, y + 16, "*".repeat(count - VAULT_RECOVERY_ROW_LEN), "TEXT_MAIN");
  }
}

function closeVault() {
  state.vaultMode = "closed";
  state.view = "genesis";
  render();
}

function drawVaultEntry(rect, kind) {
  const copy = {
    confirming: ["Confirm recovery key", "Re-enter the complete key to finish setup.", "Confirm"],
    unlocking: ["Unlock Secret Vault", "Enter your complete RR1 recovery key.", "Confirm"],
    provider: ["Save provider API key", "Stored encrypted in the unlocked Secret Vault.", "Save"],
    wifi: ["Contained WiFi Vault proof", "C1/QEMU test BSS only. No radio connection is claimed.", "Save test credential"],
  }[kind];
  drawPanel(rect, copy[0]);
  drawText(rect.x + 20, rect.y + 44, copy[1], "TEXT_MUTED");
  const field = { x: rect.x + 20, y: rect.y + 70, w: rect.w - 40, h: 52 };
  fillRect(field.x, field.y, field.w, field.h, "SURFACE_ALT");
  drawOutline(field, "HAIRLINE");
  drawVaultMasked(field.x + 10, field.y + 12, state.vaultMaskedLen);
  const [primary, cancel] = vaultActionRects(rect);
  drawButton(primary, copy[2], true, "vault-submit");
  drawButton(cancel, "Cancel", false, "vault-cancel", closeVault);
}

function drawVaultShowing(rect) {
  drawPanel(rect, "Secret Vault recovery key");
  drawText(rect.x + 20, rect.y + 44, "Write this key down. It is shown only once.", "TEXT_MUTED");
  const key = state.vaultRecoveryKey.padEnd(VAULT_RECOVERY_KEY_LEN, "X").slice(0, VAULT_RECOVERY_KEY_LEN);
  drawText(rect.x + 20, rect.y + 76, key.slice(0, VAULT_RECOVERY_ROW_LEN), "TEXT_MAIN");
  drawText(rect.x + 20, rect.y + 92, key.slice(VAULT_RECOVERY_ROW_LEN), "TEXT_MAIN");
  const [primary, cancel] = vaultActionRects(rect);
  drawButton(primary, "Space: I saved it", true, "vault-saved");
  drawButton(cancel, "Cancel", false, "vault-cancel", closeVault);
}

function drawVaultManage(rect) {
  drawPanel(rect, "Manage Secret Vault");
  drawText(rect.x + 20, rect.y + 44, "Provider credential: saved", "TEXT_MAIN");
  drawText(rect.x + 20, rect.y + 62, "WiFi credential: saved", "TEXT_MAIN");
  const actions = vaultManageRects(rect);
  drawButton(actions[0], "Forget provider credential", state.vaultManage === "forget-provider", "vault-forget-provider");
  drawButton(actions[1], "Forget WiFi credential", state.vaultManage === "forget-wifi", "vault-forget-wifi");
  drawButton(actions[2], "Close", state.vaultManage === "close", "vault-manage-close", closeVault);
}

function drawVaultForget(rect) {
  const provider = state.vaultForgetTarget === "provider";
  drawPanel(rect, provider ? "Forget provider credential?" : "Forget WiFi credential?");
  drawText(
    rect.x + 20,
    rect.y + 54,
    provider
      ? "Future provider use will be denied until a new key is saved."
      : "Future WiFi use will be denied until a new credential is saved.",
    "APP_RED",
  );
  drawText(rect.x + 20, rect.y + 76, "This action requires this second explicit confirmation.", "TEXT_MUTED");
  const [cancel, forget] = vaultActionRects(rect);
  drawButton(cancel, "Cancel", state.vaultConfirm === "cancel", "vault-forget-cancel");
  drawButton(forget, provider ? "Forget provider" : "Forget WiFi", state.vaultConfirm === "forget", "vault-forget-confirm");
}

const VAULT_OUTCOMES = {
  provisioned: ["Recovery key confirmed. Vault is ready.", "APP_GREEN"],
  unlocked: ["Secret Vault is unlocked.", "APP_GREEN"],
  rejected: ["Recovery key rejected. No access was granted.", "APP_RED"],
  unavailable: ["Secret Vault is not available for this boot.", "APP_RED"],
  "provider-saved": ["Provider API key saved in Secret Vault.", "APP_GREEN"],
  "provider-rejected": ["Provider API key was not saved.", "APP_RED"],
  "provider-unavailable": ["Unlock Secret Vault before saving an API key.", "APP_RED"],
  "wifi-test-saved": ["Contained WiFi credential saved in Secret Vault.", "APP_GREEN"],
  "wifi-safe-reconnected": ["SAFE WiFi reconnect was explicitly authorized.", "APP_GREEN"],
  "wifi-test-rejected": ["Contained WiFi Vault proof was denied.", "APP_RED"],
  "wifi-test-unavailable": ["Contained WiFi proof is unavailable here.", "APP_RED"],
  "provider-forgotten": ["Provider credential forgotten.", "APP_GREEN"],
  "provider-forget-rejected": ["Provider credential was not forgotten.", "APP_RED"],
  "wifi-forgotten": ["WiFi credential forgotten.", "APP_GREEN"],
  "wifi-forget-rejected": ["WiFi credential was not forgotten.", "APP_RED"],
};

function drawVaultOutcome(rect) {
  drawPanel(rect, "Secret Vault");
  const [message, color] = VAULT_OUTCOMES[state.vaultOutcome] || VAULT_OUTCOMES.unavailable;
  drawText(rect.x + 20, rect.y + 64, message, color);
  const [primary] = vaultActionRects(rect);
  drawButton(primary, "Close", true, "vault-outcome-close", closeVault);
}

function drawVaultFlow() {
  fillRect(L.personal.x, L.personal.y, L.personal.w, L.personal.h, "SCRIM");
  hits.push({ ...L.personal, action: () => {} });
  const rect = L.overlay;
  switch (state.vaultMode) {
    case "showing": drawVaultShowing(rect); break;
    case "confirming": drawVaultEntry(rect, "confirming"); break;
    case "unlocking": drawVaultEntry(rect, "unlocking"); break;
    case "managing": drawVaultManage(rect); break;
    case "confirming-forget": drawVaultForget(rect); break;
    case "provider-entry": drawVaultEntry(rect, "provider"); break;
    case "wifi-test-entry": drawVaultEntry(rect, "wifi"); break;
    case "outcome": drawVaultOutcome(rect); break;
  }
}
