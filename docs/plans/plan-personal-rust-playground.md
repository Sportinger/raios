# raiOS Fabrik-Plan — Bauen im System, ohne Werkstatt

Owner-Zielansage (bindend, 2026-07-18, präzisiert im Gespräch):

> Kern soll sein: ein Custom-Rust-Kernel, die Genesis-Schicht, und darin kann
> ich mit AI-Agenten etwas bauen (egal welche Sprache, aber Rust bietet sich
> an) und das direkt kompilieren — auf dem Gerät, ohne fremden Werkstatt-PC.
> Die Sicherheit kommt oben drauf oder wird so verwoben, dass wir von der
> Grundidee nicht abrücken müssen. Ziel-Maßstab: irgendwann ganze Spiele oder
> ein Videoschnittprogramm, gebaut und getestet von Agenten auf raiOS.

Dieser Plan ersetzt nicht `VISION_PLAN.md` (die Schleifen-Grundidee bleibt),
er **re-zentriert das Ziel**: Die On-Device-Fabrik ist die Hauptstraße, der
Liefer-Weg (Werkstatt baut, raiOS prüft/installiert) ist Übergang.

## 1. Was schon existiert und voll einzahlt (nichts davon war umsonst)

| Baustein | Status | Rolle für die Fabrik |
|---|---|---|
| Custom-Rust-Kernel + Genesis | läuft (QEMU + Surface-Boot bewiesen) | Das Haus, in dem die Fabrik steht |
| B1: Download → Install → Reboot → Rollback | bewiesen | Wie fertige Programme sicher ins System kommen und wieder raus |
| B2: KI-Quelle → inerte Dateien → Feedback (live OpenAI) | bewiesen | Wie Agenten-Aufträge und Quellcode ins System fließen — der „Job"-Eingang |
| W5-Klick (physische Freigabe) + Doppel-Bau + Nachrechnen | bewiesen | Die verwobene Sicherheit — gilt unverändert für jeden Compiler |
| B3A: Assembler-Gast baut lauffähiges Wasm im System (33/33) | bewiesen | **Der Beweis, dass die Bau-Rohrleitungen funktionieren** — Auftrag → im System bauen → prüfen → Klick → läuft |
| rlang (Mini-Sprache + Compiler, host-bewiesen) | Crate fertig, pausiert | Ersatzrad + wiederverwendbarer Encoder (siehe §4) |
| Messungen 2026-07-18 | erledigt | rustc-als-Wasm existiert (91 MB); Blocker = Threads (Bau-Schalter, kein Naturgesetz); Bau-Rezept öffentlich gefunden |
| **wasmtime-Werkstatt-Probe 2026-07-18** | **GRÜN** | Unverändertes Threads-Artefakt baut+läuft: hello 1,6 s / medium -O 1,2 s, ~670 MB, Gast-Threads real (~26–32); **Linker eingebettet → Job-Kette = 1 Gast**; Details `docs/architecture/probe-rustc-wasm-wasmtime-2026-07-18.md` |

Die Maschinenform bleibt Wasm (Käfig eingebaut, byte-genau nachrechenbar,
Rust hat Wasm als offizielles Ziel) — genau das macht „Rust-Compiler im
System" überhaupt möglich.

## 2. Die Treppe zum Ziel (jede Stufe einzeln beweisbar)

1. **Bootstrap-Werkzeug (JETZT, Hauptspur — Owner-Präzisierung 2026-07-18):**
   Das existierende öffentliche rustc-Wasm-**Threads**-Artefakt (91 MB,
   oligamiq-Rezept) **unverändert übernehmen** — kein Fork, kein Anpassen,
   kein eigener Compiler-Bau. Dafür spielt raiOS die Thread-Spielregeln im
   Käfig nach (grüne Threads, Scoping: `docs/plans/plan-rust-kernel.md`).
   Werkstatt-Probe unter wasmtime (beherrscht wasi-threads) statt wasmi:
   lädt es? Imports? Thread-Zahl? RAM? Tempo? Danach lebt das Werkzeug im
   System — die Werkstatt wird nie wieder gebraucht (Bootstrap-Prinzip:
   auch das erste Linux wurde einmal von außen kompiliert). Ein späteres
   Cloud-Nachbacken desselben Stands dient nur noch der Herkunftsprüfung
   (Fingerabdruck-Vergleich), nicht der Anpassung.
2. **„Bauplatz"-Gast-Klasse:** große Speicher-Gäste (hunderte MB), begrenzte
   geprüfte Datei-Zugriffe (Quelle + Bibliotheken rein, Artefakt raus),
   Geduld-Budgets. Auf echter Hardware (Surface: mehr RAM als die 512-MB-VM).
3. **rustc als signierter Gast:** Rust-Quelle vom Agenten → Compiler-Gast baut
   → Doppel-Bau + Fingerabdruck-Nachrechnung → W5-Klick → läuft. Erste
   Grenzen ehrlich: keine Internet-Pakete (Abhängigkeiten kommen als geprüfte
   Quelle über W7), keine Makro-Programme, einkernig (langsam).
4. **Schnelle Ausführungsstufe:** geprüftes Wasm → schneller Maschinencode,
   auf dem Gerät, mit denselben Beweisen. Nötig für Bau-Tempo UND für große
   Programme zur Laufzeit (Spiele). Bewusste spätere ADR-Entscheidung.
5. **Parallelität + GPU:** echtes Mehrkern (SMP) und GPU-Treiber durch die
   Schleife — die letzten Stufen zu Spiel-Maßstab. Die Thread-**Spielregeln**
   (geteilter Speicher, Atomics, spawn) kommen schon in Stufe 1-3 als
   deterministisch nachgespielte grüne Threads; hier in Stufe 5 geht es nur
   noch um echtes Gleichzeitig-Tempo auf mehreren Kernen.

Job-Bild des Owners = genau diese Kette: Genesis-Auftrag („baue X") →
Agenten liefern Quelle (B2) → Bauplatz baut (Stufe 2-3) → Tests im Käfig →
W5-Klick → installieren/zurücknehmen (B1). Die Stationen existieren; nur der
Produzent wächst.

## 3. Sicherheit: verwoben, nicht verhandelbar

Für jeden Compiler (rlang heute, rustc morgen) gilt unverändert:
Ausgabe wird doppelt gebaut und unabhängig nachgerechnet, bleibt inert bis
zum physischen Klick, läuft im Import-geprüften Käfig, installiert nur
signiert mit Rollback. Der Compiler ist ein Arbeiter, keine Autorität.
Von der Grundidee wird nicht abgerückt — sie ist der Grund, warum man einer
Agenten-Fabrik je vertrauen kann.

## 4. rlang — ehrliche Einordnung (Owner-Frage: „wo brauchen wir das?")

- **Was es geleistet hat:** Es bewies mit dem Assembler die kompletten
  Bau-Rohrleitungen, durch die später rustc fließt — dieselben Routen,
  Freigaben, Beweise. Diese Arbeit zahlt 1:1 ein.
- **Was bleibt:** Der typisierte Wasm-Encoder aus rlang ist genereller
  Werkzeugkasten (jeder Compiler kann ihn nutzen); die Crate ist ein
  Ersatzrad, falls der Bootstrap länger klemmt, und ein Werkzeug für winzige
  Systemaufgaben.
- **Entscheidung:** rlang ist **pausiert** (nach Slice 2a, committet und
  grün). Es ist NICHT auf dem kritischen Pfad zum Fabrik-Ziel. Weiterbau nur,
  wenn ein konkreter Bedarf entsteht — kein Selbstzweck.

## 5. Nächste Schritte (Reihenfolge)

1. **ERLEDIGT — GRÜN (2026-07-18):** wasmtime-Probe auf das vorhandene
   Threads-Artefakt: baut und läuft (hello 1,6 s / medium `-O` 1,2 s,
   ~670 MB Spitze, Gast-Threads real ~26–32). Linker-Antwort: **rust-lld
   ist im Modul eingebettet** — die Rust-Spur braucht keine Job-Kette.
   Werkstatt-Referenz auf wasmtime 46.0.1 gepinnt (47 entfernt
   wasi-threads). Voller Bericht: `docs/architecture/probe-rustc-wasm-wasmtime-2026-07-18.md`.
2. **Threads im Käfig** (T1: Atomics/Shared-Memory im vendorierten wasmi,
   host-testbar; T2: Round-Robin-Pump über Thread-Instanzen) + **Bauplatz/
   Heap** + **WASI-Subset** — Slices und Aufwände in
   `docs/plans/plan-rust-kernel.md` §7.
3. Bei grünen Proben: **Bauplatz-Scoping** mit den gemessenen Budgets (statt
   geratenen) + erster Hello-World-Rust-Bau im System als W5-Beweis.
4. Bei Misserfolg: exakte Wand dokumentieren; Alternativen (eigener Beitrag
   zum Upstream-Rezept; rlang-Weiterbau als Zwischennutzen) neu bewerten.
   Der frühere Plan „threads-freien Fork backen" ist **verworfen**
   (Owner 2026-07-18: kein Fork, kein Anpassen des Compilers).
5. Platz-Frage GELÖST (2026-07-18): Laufwerk E: hat ~400 GB frei — der
   VM-Scratch wird dorthin umgelenkt (C: bleibt entlastet). Kein OS-Wechsel:
   die Werkstatt bleibt Windows (alle Beweise/Skripte laufen hier); für einen
   etwaigen lokalen Linux-Bau existiert bereits ein Ubuntu in Windows (WSL),
   von hier steuerbar und mit E:-Platz — das ist der Rückfall, falls der
   Cloud-Bau klemmt. Das separate Arch-Dual-Boot wird NICHT genutzt (dort
   könnte der Agent nicht mithelfen).

Ehrlich benannt: Schritt 1-2 können auch scheitern oder Wochen an
Upstream-Feinheiten hängen. Jedes Ergebnis wird gemessen berichtet, nie
geraten.

## 6. WASI preview1 subset — slice plan (planned 2026-07-18, xhigh second opinion)

Full report with per-slice predicates:
`docs/_archive/2026-07-18_wasi-preview1-slice-plan-full.md`. Key code findings
behind the cut: project workspace caps at 32 KiB/file (too small for the 71-MB
sysroot → chunked BuildFS needed), the existing import-grant path carries max
16 untyped pairs and cannot express `proc_exit` (no-return) → WASI becomes a
NEW typed family, not an extension.

| # | Slice | Where | Size |
|---|---|---|---|
| 0 | Import inventory tool: canonical typed JSON of every import of the pinned `rustc_opt.wasm` (measure, don't guess). Worker builds tool + fixtures; orchestrator runs it against `E:\raios-probe-rustc-wasm\` (lanes can't see E:) and commits the evidence. | `tools/wasm-import-inventory` | S |
| 1 | Typed grant family `raios.wasi_build_imports.v1`: binds compiler SHA, job manifest, mount manifests, ranges, quotas, full linker list. Fail-closed before instantiation. | `crates/raios-core` | M |
| 2 | `raios-wasi-preview1` core: types/errno, path resolve (no escape), fd table (0-2 std, 3 = `/` preopen, lowest-free from 4). no_std, dependency-free, raw pointers stay outside. | new crate | M |
| 3 | Read-only `/sysroot` + `/src`: BuildFS manifest v1, 64-KiB CAS chunks, range reads without materializing 71 MB. | shim + core | L |
| 4 | RAM-tmp + root scratch children (rustc creates temp under `/`), `/out` arena; freeze `/out` to sorted manifest; only two byte-identical double-build manifests produce an egress plan. | shim + core | L |
| 5 | args/env from job manifest; logical clock = job fuel counter, realtime = fixed epoch + logical; random = specified PRNG seeded from job-manifest hash; `proc_exit` as HostEffect. | shim | M |
| 6 | Thin kernel glue behind the grant gate; `ThreadHost` trait handed to T2 (spawn interface only). All domain logic must be host-green first; kernel build + QEMU smoke = orchestrator. | seed-kernel | L |

Owner/ADR questions raised (with recommendations, undecided): BuildFS format
(rec: chunk-CAS), guest realtime epoch (rec: fixed 2000-01-01), root-tmp
policy (rec: any quota'd root child), egress buffering (rec: RAM until proven
too big), T2 contract (≥32 threads, first `proc_exit` wins), build receipt v2
(rec: new version, don't reinterpret cargo receipts). Honest limits: the plan
measures the static import surface only — dynamic preview1 edge semantics get
recalibrated against real rustc runs after T1/T2 land.
