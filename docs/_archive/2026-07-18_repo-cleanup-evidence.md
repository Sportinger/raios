# Repository-Cleanup — Evidenz für Phase 1

Stand: 2026-07-18, Branch `repo-cleanup`, Basis `88506d8`.

Pflicht-Stopp: Dieses Dokument sammelt nur Beweise und Urteile. Es wurde nichts
verschoben, gelöscht oder aus Git entfernt. Phase 3 beginnt erst nach dem
Owner-Review dieses Dokuments.

## Methode und Auslegungsgrenzen

- `cargo metadata --format-version 1 --locked` lief vollständig durch: 24
  Workspace-Mitglieder, 268 aufgelöste Pakete. Weil das geerbte `CARGO_HOME`
  auf das nicht vorhandene `F:\scorefollower-build\cargo` zeigte, wurde nur für
  diesen Befehl der vorhandene Benutzer-Cargo-Cache verwendet. `Cargo.lock`
  wurde dabei nicht verändert.
- Referenzen wurden mit `rg -l -F --hidden`, ohne `.git/**`, `target/**`,
  `target-tools/**`, diesen Auftrag und diese Evidenzdatei gesucht. Bei kurzen
  Namen wie `ci`, `ota`, `target` und `tools` wurden zusätzlich echte
  Pfadtreffer (`name/` beziehungsweise `name\`) geprüft, damit Wörter wie
  `pci` nicht als Referenz zählen. Archivtreffer sind unten ausdrücklich von
  Build-/CI-/Script-Referenzen getrennt.
- Der letzte Commit stammt jeweils aus
  `git log -1 --format="%ci %h %s" -- <pfad>`. In der Tabelle ist der Zeitanteil
  zur Lesbarkeit auf Datum, Hash und Betreff normalisiert. `—` heißt: Der Pfad
  ist nicht versioniert und hat deshalb keinen Git-Commit.
- Ein Root-Verzeichnis mit aktiven und generierten Inhalten wird konservativ
  nach dem aktiven Anteil beurteilt; insbesondere ist `release/` kein
  pauschal löschbares Build-Verzeichnis.

## Evidenztabelle

| Pfad | Workspace-Member? | Referenziert von? | Letzter Commit | Urteil |
|---|---|---|---|---|
| `.agents` | Nein | 0 Treffer; leer und unversioniert | — | **UNKLAR** — kein Verlauf und kein Inhalt; nicht anfassen |
| `.cargo` | Nein | `.gitignore` (`/.cargo/`); Ordner leer | — | **UNKLAR** — ignorierter lokaler Pfad, aber kein nachweisbares Build-Output |
| `.cargo-home` | Nein | `.gitignore`, `scripts/build-project-wasm.ps1`, `scripts/check-source-size.ps1`, VM-Harness | — | **GENERIERT** — Cargo-Cache (10.180 Dateien), reproduzierbar und bereits ignoriert |
| `.claude` | Nein | Claude Code lädt die enthaltene raiOS-Skilldatei automatisch; damit operative Instruktionsquelle | 2026-07-15 `418e117` chore(skill): /raios carries the standing vision mission | **INFRA** — automatisch geladene Agentensteuerung; inhaltliche Konflikte werden im Phase-3-Report dem Owner vorgelegt |
| `.git` | Nein | Repository-Metadaten | — | **INFRA** |
| `.github` | Nein | `.github/workflows/ci.yml`; aktuelle Pläne/Architektur | 2026-07-05 `943a9a0` CI vm-smoke: force LF checkout | **INFRA** |
| `.vscode` | Nein | Keine Root-Pfadreferenz; nur `settings.json` selbst | 2025-09-17 `429c73f` Enhance AI Build and Test Runbook | **VERDÄCHTIG** — 0 operative Referenzen, Commit vor 2026-06-01 |
| `ci` | Nein | Nur `ci/README.md` selbst; 0 Pfadtreffer in `.github`, Cargo oder Scripts | 2025-09-16 `e6c8be4` Remove AI Build and Test Runbook | **VERDÄCHTIG** — Platzhalter-README, 0 operative Referenzen, alter Commit |
| `docs/architecture/device-protocol` | Nein | Aktuelle ADR `docs/architecture/decisions/0002-...`; `.vscode`-Exclude; sonst historische Docs | 2026-05-23 `5b7d826` Continue recovery lifeline refactor | **UNKLAR** — 20 konkrete Spezifikationen und aktuelle ADR-Referenz, aber keine Build-/CI-/Script-Kante |
| `docs` | Nein | `AGENTS.md`, `CLAUDE.md`, `.claude/skills/raios/SKILL.md` | 2026-07-18 `88506d8` docs: make scope and active plans discoverable | **INFRA** |
| `fake-cloud` | Ja: `fake-cloud-server` | Root-`Cargo.toml`, `Cargo.lock`, `ota`, `registry` | 2026-05-19 `e002738` Harden OpenAI trust path and add MIT license | **AKTIV** — Workspace-Mitglied; Vorabverdacht widerlegt |
| `modules` | Nein | Nur `.vscode/settings.json` als Exclude; sonst begriffliche/historische „modules“-Treffer, keine Build-/CI-/Script-Pfadkante | 2025-09-16 `e6c8be4` Remove AI Build and Test Runbook | **VERDÄCHTIG** — einziges File ist `hello-ui/README.md`, 0 operative Referenzen, alter Commit |
| `ota` | Ja: `ota-tools` | Root-`Cargo.toml`; Path-Dependency von `registry-core` und `fake-cloud-server` | 2026-07-09 `012ab94` M12+ Slice 9: distribution receiver identity evidence | **AKTIV** — Vorabverdacht widerlegt |
| `raios-core` | Ja: `raios-core` | Root-`Cargo.toml`, `.github/workflows/ci.yml`, mehrere Path-Dependencies | 2026-07-18 `7787644` feat(B3A-1c): revision-bound on-device build | **AKTIV** |
| `raios-dns-parse` | Ja: `raios-dns-parse` | Root- und `raios-core/Cargo.toml`, Seed-Kernel, Wasm-Gast | 2026-07-14 `2c1967e` feat(M11-9a): extract raios-dns-parse | **AKTIV** |
| `raios-http-parse` | Ja: `raios-http-parse` | Root- und `raios-core/Cargo.toml`, Seed-Kernel, Wasm-Gäste | 2026-07-08 `ec236de` M11-7c: fix parse_content_length | **AKTIV** |
| `raios-lang` | Ja: `raios-lang` | Root-`Cargo.toml`; Path-Dependency auf `raios-wasm-ir` | 2026-07-18 `2890b63` feat(B3A-2a): rlang crate + additive typed emitter | **AKTIV** |
| `raios-w7-acquire-logic` | Ja: `raios-w7-acquire-logic` | Root-`Cargo.toml`, Seed-Kernel, `svc-net-acquire-w7` | 2026-07-14 `1fc3037` feat(NET-7): signed svc.net.acquire.w7 | **AKTIV** |
| `raios-wasm-ir` | Ja: `raios-wasm-ir` | Root-`Cargo.toml`; Path-Dependency von Kernel, rlang und Wasm-Gast | 2026-07-18 `2890b63` feat(B3A-2a): rlang crate + additive typed emitter | **AKTIV** |
| `raios-x509-spki` | Ja: `raios-x509-spki` | Root-/Core-Cargo, zwei Wasm-Gäste | 2026-07-14 `853342d` feat(NET-5 + NET-5B): opaque TLS crypto imports | **AKTIV** |
| `raios-x509-time` | Ja: `raios-x509-time` | Root-/Core-Cargo, `svc-demo-certwindow` | 2026-07-08 `18d095f` M11-6a: extract raios-x509-time | **AKTIV** |
| `registry` | Ja: `registry-tools`, `registry-core` | Root-`Cargo.toml`; Path-Dependency von `fake-cloud-server` | 2026-07-09 `c85b3a7` M12+ Slice 11: guest-verify receiver identity evidence | **AKTIV** — Vorabverdacht widerlegt |
| `release` | Nein | `.github/workflows/ci.yml`, zahlreiche Scripts und VM-Harness-Profile | 2026-07-11 `e17cf5f` ui: align pointer and boot Genesis at 1080p | **AKTIV** — gemischter Pfad: 6 versionierte Boot-Dateien plus 7 unversionierte Dateien; nicht pauschal generiert |
| `scripts` | Nein | `ci/README.md`, aktive Build-/QEMU-/Packaging-Aufrufe | 2026-07-17 `2dff098` feat(B3A-1b): on-device build proven in-VM | **INFRA** |
| `seed-kernel` | Ja: `seed-kernel` | Root-`Cargo.toml`, `.github/workflows/ci.yml`, `.gitattributes`, Scripts | 2026-07-18 `d73c5b8` feat(B3A-1c): focused proof GREEN 33/33 | **AKTIV** |
| `seed-runtime` | Nein | Nur `docs/_archive/2026-07-18_review-3-runtime-vm-persistence.md`; 0 aktuelle/operative Treffer | 2025-09-16 `e6c8be4` Remove AI Build and Test Runbook | **VERDÄCHTIG** — einziges File ist ein „to be filled out“-README, alter Commit |
| `target` | Nein | `.gitignore`, Cargo/CI-Ausgabepfade | — | **GENERIERT** — Cargo-/Test-Output, reproduzierbar und bereits ignoriert |
| `target-tools` | Nein | 0 externe Treffer; Inhalt ausschließlich Cargo-Release-Artefakte | — | **GENERIERT** — reproduzierbar, unversioniert, aber **noch nicht in `.gitignore`** |
| `tools` | Ja: `descriptor-resign`, `core-policy-sign` | Root-`Cargo.toml`, CI/Cargo und Protokolldokumente | 2026-07-10 `9e193f0` core-policy: verify owner-signed executing kernel | **AKTIV** |
| `vendor` | Nein | 15 `[patch.crates-io]`-Einträge im Root-`Cargo.toml` | 2026-07-06 `c8d6f74` M4-2: wasmi 0.31.2 vendored and pinned | **AKTIV** — direkte Build-Eingabe, kein pauschal generiertes Vendor-Output |
| `vm-harness` | Nein | `.github/workflows/ci.yml` startet `shadow-vm-smoke.ps1`; Protokolldokumente | 2026-07-18 `d73c5b8` feat(B3A-1c): focused proof GREEN 33/33 | **AKTIV** |
| `wasm-guests` | Ja: 9 Gäste | Root-`Cargo.toml`, `scripts/build-wasm-guest.ps1` | 2026-07-17 `2dff098` feat(B3A-1b): on-device build proven in-VM | **AKTIV** |
| `.gitattributes` | Nein | Historische Build-/Debug-Dokumente | 2026-07-12 `b4ef687` build-hygiene: EOL-protect seed-kernel artifacts | **INFRA** |
| `.gitignore` | Nein | Cleanup-/Hygiene-Dokumente | 2026-07-14 `82e6504` feat(NET-8): arm svc.net.acquire.w7 | **INFRA** |
| `AGENTS.md` | Nein | `CLAUDE.md`, `.claude/skills/raios/SKILL.md`, Agentendokumente | 2026-07-18 `88506d8` docs: make scope and active plans discoverable | **INFRA** |
| `Cargo.lock` | Nein | Cargo selbst, Core-Projektlogik, Status/Pläne | 2026-07-17 `2dff098` feat(B3A-1b): on-device build proven in-VM | **INFRA** |
| `Cargo.toml` | Nein | Cargo selbst, Build-/Projektlogik und Pläne | 2026-07-18 `2890b63` feat(B3A-2a): rlang crate + additive typed emitter | **INFRA** |
| `CLAUDE.md` | Nein | Agentendokumente | 2026-07-15 `a2a3efa` docs(rules): carry the standing self-build-loop mission | **INFRA** |
| `cleanup-evidence.md` | Nein | `docs/REPO_CLEANUP.md` verlangt dieses Owner-Review-Artefakt | — (neu in diesem Auftrag) | **INFRA** — wird laut Auftrag erst beim Abschluss datiert archiviert |
| `LICENSE` | Nein | Projekt- und Vendor-Metadaten | 2026-05-19 `a360559` Rename raisOS branding to raiOS | **INFRA** |
| `README.md` | Nein | Aktive Architektur-/Statusdokumente | 2026-07-18 `88506d8` docs: make scope and active plans discoverable | **INFRA** |
| `usb-diskpart-gpt.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |
| `usb-final-check-eject.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |
| `usb-format-fat32.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |
| `usb-set-esp.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |
| `usb-update-hub-kernel.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |
| `usb-update-kernel.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |
| `usb-write-gpt.log` | Nein | 0 Treffer; via `**/*.log` ignoriert | — (unversioniert; mtime 2026-05-10) | **UNKLAR** — kein Git-Verlauf; nicht ohne Inhaltsreview anfassen |

## Owner-Review: priorisierte Entscheidungen

1. **Bestätigen/streichen:** `.vscode`, `ci`, `modules`, `seed-runtime` als
   VERDÄCHTIG. Alle vier sind alt und haben keine operative Kante; `modules`
   wird lediglich von der ebenfalls verdächtigen `.vscode`-Zählerkonfiguration
   ausgeschlossen, `seed-runtime` nur in einem Archivdokument erwähnt.
2. **Entscheiden:** `docs/architecture/device-protocol` bleibt UNKLAR. Der Root-Pfad widerspricht
   zwar dem aktuellen Docs-Scope, aber die aktuelle ADR-0002 verweist auf seine
   20 konkreten V0-Spezifikationen. Ein Verschieben braucht eine explizite
   Owner-Entscheidung beziehungsweise eine gesonderte Docs-Migration.
3. **Bestätigen:** `target-tools` ist GENERIERT, aber anders als `target` und
   `.cargo-home` noch nicht ignoriert. Eine `.gitignore`-Änderung gehört erst in
   Phase 3.
4. **Entscheiden:** Tatsächlich liegen sieben Root-Dateien `usb-*.log` vor,
   nicht die im Auftrag erwarteten sechs `usb-*.txt`. Sie sind unversioniert,
   ignoriert und vom 2026-05-10; mangels Git-Historie bleiben sie UNKLAR.
5. **Entscheiden:** `release/` enthält zusätzlich zu sechs versionierten
   Boot-Dateien sieben unversionierte Dateien (vier Screenshots, zwei
   Preview-Images, `usb-write-result.txt`). Der aktive Root-Pfad darf nicht als
   Ganzes bereinigt werden; die sieben Einzeldateien benötigen einen separaten
   Owner-Entscheid.

Widerlegt sind die Vorabverdachte `fake-cloud`, `ota` und `registry`: Sie sind
Workspace-Mitglieder beziehungsweise enthalten mehrere Workspace-Mitglieder.

## Phase-1-Gate

- Positiv-Predicate: Jede beim Reportzeitpunkt vorhandene Root-Datei und jedes
  Root-Verzeichnis hat genau eine Tabellenzeile.
- Negativtest: Der Phase-1-Commit darf genau `cleanup-evidence.md` hinzufügen
  und weder Rename noch Delete enthalten.
- Nicht ausgeführt: Build, `raios-core`-Tests und QEMU-Smoke. Diese Gates sind
  laut Auftrag nach Verschiebe-Commits erforderlich; Phase 1 enthält bewusst
  keinen Verschiebe-Commit.
