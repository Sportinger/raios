# HANDOFF — Wo stehen wir?

> **Format-Regel (hart):** Dieses File ist ein Fenster, kein Log. Genau EIN
> „Jetzt“-Block, EIN „Nächster Schritt“ und genau DREI „Zuletzt“-Einträge.
> Beim Update: neuen Eintrag oben einsetzen, den ältesten ERSATZLOS löschen.
> Was daraus dauerhaft wichtig ist, gehört VORHER nach `docs/status/STATUS.md`
> (Evidenz) oder in den passenden Plan unter `docs/plans/`. Pro Eintrag max.
> 4 Zeilen Text, Datei max. 60 Zeilen. Ersetzen, nie anhängen.

## Jetzt (Stand 2026-07-18)

Hauptstraße = On-Device-Fabrik (`docs/plans/plan-personal-rust-playground.md`):
Agenten bauen+testen Software direkt auf raiOS, kein Werkstatt-PC. Das
Bootstrap-Werkzeug (öffentliches rustc-als-Wasm-Threads-Artefakt, unverändert
übernommen) ist in der Werkstatt bewiesen; jetzt wird es ins System geholt.
rlang ist pausiert (Ersatzrad). Docs am 2026-07-18 neu strukturiert:
`docs/SCOPE.md` ist bindend, Historie liegt datiert in `docs/_archive/`.

## Nächster Schritt

Threads im Käfig: T1 = Atomics/Shared-Memory im vendorierten wasmi
(host-testbar), danach T2 Round-Robin-Pump + Bauplatz-Heap + WASI-Subset.
Slices und Aufwände: `docs/plans/plan-rust-kernel.md` §7.

## Zuletzt (genau 3, neueste zuerst)

### 2026-07-18 — Repository-Cleanup Phase 3 umgesetzt
Der versionierte Root enthält nur aktive Crates, INFRA und Quarantäne; das
Device-Protokoll liegt unter `docs/architecture/`. USB-Evidenz blieb erhalten,
generierte Tool-Ausgaben sind entfernt. Build, 591 Tests und QEMU quick grün;
Report: `docs/_archive/2026-07-18_repo-cleanup-phase3-report.md`.

### 2026-07-18 — Werkstatt-Probe GRÜN: rustc-als-Wasm baut und läuft
Das unveränderte Threads-Artefakt läuft unter wasmtime 46 (gepinnt): hello
1,6 s, medium `-O` 1,2 s, ~670 MB Spitze, echte Gast-Threads (~26–32),
rust-lld eingebettet — kein separater Linker-Job. Bericht:
`docs/architecture/probe-rustc-wasm-wasmtime-2026-07-18.md` (Commit 37929ba).

### 2026-07-18 — Ziel re-zentriert (Owner, bindend) + rlang pausiert
On-Device-Fabrik ist die Hauptstraße: Genesis-Job → Agenten liefern Quelle
(B2) → Bauplatz baut → Test im Käfig → W5-Klick → Install/Rollback (B1). Kein
Compiler-Fork; ein Cloud-Nachbacken dient später nur der Herkunftsprüfung.
rlang nach grünem Slice 2a committet und pausiert (Ersatzrad + Encoder).
