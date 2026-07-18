# Repository-Cleanup Phase 3 — Abschlussreport

Stand: 2026-07-18, Ausführung direkt auf `main`. In diesem Bericht bezeichnet
**Quarantäne-Root** das isolierte Root-Verzeichnis mit eigener README und
30-Tage-Löschregel.

## Ergebnis

Der versionierte Repo-Root besteht nach dem Cleanup nur aus aktiven Crates,
Build-/Projekt-Infrastruktur und dem Quarantäne-Root. `release/` enthält genau
die sechs bereits versionierten Boot-Dateien. Generierte lokale Verzeichnisse
wie `.cargo-home`, `target` und `release/vm-reports` bleiben ignoriert und sind
nicht Teil des versionierten Root-Inventars.

## Bewegungstabelle

| Ausgang | Ergebnis | Commit / Begründung |
|---|---|---|
| `.vscode` | Quarantäne-Root `.vscode` | `61e8aea`; 0 operative Referenzen, letzter Commit 2025-09-17 |
| `ci` | Quarantäne-Root `ci` | `15c8a64`; 0 operative Referenzen, letzter Commit 2025-09-16 |
| `modules` | Quarantäne-Root `modules` | `65616d6`; 0 operative Referenzen, letzter Commit 2025-09-16 |
| `seed-runtime` | Quarantäne-Root `seed-runtime` | `2d94850`; 0 operative Referenzen, letzter Commit 2025-09-16 |
| Root `device-protocol` | `docs/architecture/device-protocol/` | `082c363`; 20 Spezifikationen und alle Textverweise migriert |
| `target-tools` | aus Git und lokal entfernt; `/target-tools/` ignoriert | `02eee7a`; 297 versehentlich versionierte, reproduzierbare Buildartefakte |
| sieben `usb-*.log` und USB-Ergebnis | Quarantäne-Root `usb-logs` | `fb541d1`; Originale erhalten, bestätigte Prozedur in Bare-Metal-Doku |
| sechs Release-Bilddateien | `docs/assets/screenshots/` | `793fa33`; alle sechs SHA-256-eindeutig, keine Duplikate |
| `.agents`, `.cargo` | leere lokale Verzeichnisse entfernt | ausdrücklich erlaubte lokale Hygiene; kein versionierbarer Inhalt |
| `cleanup-evidence.md` | `docs/_archive/2026-07-18_repo-cleanup-evidence.md` | `1f435e0`; Owner-geprüfte Evidenz archiviert |
| aktiver Cleanup-Plan | datiert unter `docs/_archive/` archiviert | `175503a`; Plan abgeschlossen, Historie erhalten |

SHA-256 der verschobenen einzigartigen Assets:

| Datei | SHA-256 |
|---|---|
| `enum-console-shot.png` | `8E3E2211926296B2CE3B844961AB5DD4A5FE4DFBCDE2366D4125F9E830F65927` |
| `iommu-wifi-shot.png` | `9BADE37699547C873F8C1BCFDEED11A14FF17CFCF3569B0B23BEFCB9124E7C31` |
| `set-wifi-scan-shot.png` | `42D138DFF157E6875E2FFC3E33C63BFCFCB4F6119B59C40AD559D2A6B9F1DE31` |
| `ui-pill-detail-shot.png` | `6FCE3A7CF4A01144F97197A705F737EAAC6DFE905F3EA6FCCA8D350B7EC0AA91` |
| `raios-genesis-live-preview.img` | `87EC2AFF00B82F10DE3807E557B79CBEB0EDAC2F3677AB84794AC7881D1E9785` |
| `raios-stage0-preview.img` | `ED2D9B7409CB926CB98BCCB70454EE144E9B4B68E3F42DB6AD4AAA6C2F6DABCA` |

## `.claude`-Urteil und Diff-Vorschlag

`.claude` ist **INFRA**: Claude Code lädt die raiOS-Skilldatei automatisch,
also wirkt sie operativ. Die Skilldatei wurde nicht verändert. Sie widerspricht
der aktuellen Instruktionslage in vier Punkten:

1. Sie bindet noch `docs/VISION_PLAN.md` und B1–B4; aktuell bindend sind
   `docs/SCOPE.md` und der offene Checkbox-Loop.
2. Sie verweist auf alte Pfade wie `docs/OWNER_DASHBOARD.md`, `docs/ROADMAP.md`,
   `docs/PROJECT_STATUS.md` und `docs/plan-reviews/`; die aktuellen Einstiege
   sind `docs/status/HANDOFF.md`, `docs/status/STATUS.md` und `docs/plans/`.
3. Sie verlangt einen nicht vorhandenen `codex-worker`-Skill; `CLAUDE.md`
   beschreibt stattdessen direkte, eng gescopte Lane-Dispatches.
4. Sie verlangt Updates mehrerer alter Statusdateien; aktuell ist HANDOFF das
   kurze Zustandsfenster, dauerhafte Evidenz gehört in STATUS, Pläne oder ADRs.

Vorschlag, vom Owner zu entscheiden und danach in einem eigenen Commit
umzusetzen:

```diff
- description: advance the self-build-loop vision (docs/VISION_PLAN.md)
+ description: orchestrate the open raiOS scope from docs/SCOPE.md and HANDOFF

- THE MISSION: implement docs/VISION_PLAN.md completely; work B1 -> B4
+ THE MISSION: close docs/SCOPE.md checkboxes in dependency order; owner steering wins

- Read VISION_PLAN, OWNER_DASHBOARD, ROADMAP and plan-reviews
+ Read docs/status/HANDOFF.md, then the selected docs/SCOPE.md category and relevant docs/plans entry

- Dispatch through the codex-worker skill; update OWNER_DASHBOARD/ROADMAP/PROJECT_STATUS
+ Dispatch narrow lanes per AGENTS.md/CLAUDE.md; update HANDOFF and durable STATUS/ADR evidence only when required
```

Empfehlung: `CLAUDE.md` und `docs/SCOPE.md` gewinnen als Instruktionsquellen;
die Skilldatei wird zu einem kurzen Einstieg, der diese Quellen nicht dupliziert.

## Verifikation

- CI-konformer Host-Build: `cargo build --locked -p raios-core` grün.
- Core-Gate: `cargo test --locked -p raios-core`, 591/591 grün.
- QEMU `quick` grün: Reports `shadow-20260718-152303-11664.json`,
  `shadow-20260718-152851-22208.json` und
  `shadow-20260718-153516-22908.json` samt SHA-256-Sidecars.
- Negativpredicate: keine alte Root-Protokolldirektorie; jeder Treffer auf
  `device-protocol` zeigt auf `docs/architecture/device-protocol`.
- Negativpredicate: keine Quarantäne-Pfadreferenz außerhalb des
  Quarantäne-Roots.
- `release/` hat im Index exakt sechs Dateien.

Der pauschale Root-Befehl `cargo build --locked` ist für diesen gemischten
Workspace kein valides Gate: Er baut reine `wasm32`-Gäste für den Windows-Host
und scheitert erwartbar an `core::arch::wasm32` sowie Panic-Unwinding. Nach
Owner-Freigabe wurde deshalb dieselbe Aufteilung wie in CI verwendet:
`raios-core` auf dem Host, Seed-Kernel/Target-Pfade im QEMU-Smoke.

## Offene Punkte

1. Owner-Entscheid zum obigen Skill-Diff; keine automatische Umschreibung.
2. Fremde, nicht vom Cleanup stammende Arbeitskopie-Änderungen an `CLAUDE.md`
   sowie `.claude/hooks/` und `.claude/settings.json` bleiben unberührt.
3. Der vorbestehende Stash `P4-4b2 partial` bleibt unberührt.
4. Kein Push und kein History-Rewrite wurden ausgeführt.
