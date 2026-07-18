# Scoping: Wasm-Threads in raiOS — für rustc-als-Gast

Owner-Frage (2026-07-18): „Ich will ein Threaded-System einbauen, um den
Wasm-Rust-Compiler nutzen zu können. Riesen-Arbeit beim Rewrite, aber ich
denke der richtige Weg für ein OS, das sicher ist und trotzdem selbst
Rust-Code kompilieren kann, der von AI-Agenten gebaut wurde. Bitte scopen."

Status: **Scoping-Dokument, keine Plan-Änderung.** FACTORY_PLAN bleibt
bindend, bis der Owner die Empfehlung in §7 annimmt oder verwirft.

## 1. Kern-Antwort

1. Die Richtung ist richtig — **aber es ist kein Riesen-Rewrite.** Was
   rustc braucht, sind nicht echte Mehrkern-Threads, sondern nur die
   **Thread-Spielregeln**: gemeinsamer Speicher, Atomic-Befehle, ein
   `thread-spawn`-Hostaufruf. Die kann der bestehende einkernige,
   kooperative Käfig **nachspielen** („grüne Threads": der Interpreter
   wechselt selbst reihum zwischen den Threads eines Gastes).
2. Der Kernel bleibt dabei einkernig und ohne Interrupts — genau wie heute.
   Echtes Mehrkern (SMP) bleibt Treppenstufe 5 (reine Geschwindigkeit,
   später, optional).
3. Die **erste Mauer ist nicht Threads, sondern Speicher**: Der gesamte
   Kernel-Heap ist heute 64 MiB — kleiner als die rustc-Wasm-Datei (91 MB)
   allein. Die „Bauplatz"-Gast-Klasse (Stufe 2 im FACTORY_PLAN) ist harte
   Voraussetzung, egal welche Route.
4. **Entschieden (Owner-Klarstellung 2026-07-18, nach diesem Scoping):**
   Threads-Route ist die Spur. Der Compiler wird **weder gebaut noch
   angepasst** — das existierende öffentliche Threads-Artefakt (91 MB) wird
   unverändert genutzt. Es gibt praktisch keinen threads-freien rustc-Wasm;
   der wäre nur per Fork+Patch zu haben und ist verworfen. Ein späteres
   Cloud-Nachbacken desselben Stands bleibt einzig als Herkunftsprüfung
   (Fingerabdruck-Vergleich) auf der Liste, nicht als Anpassung.
5. Sicherheit und Nachrechenbarkeit bleiben unangetastet: fester,
   deterministischer Wechsel-Takt ⇒ Doppel-Bau bleibt byte-gleich
   reproduzierbar; Hostaufrufe laufen, während alle Geschwister-Threads
   geparkt sind ⇒ keine neuen Manipulationsfenster.

## 2. Ist-Stand (gemessen im Repo, nicht geraten)

| Fakt | Beleg |
|---|---|
| Wasm-Motor = **einvendoriertes wasmi 0.31.2**, reiner Interpreter (~22k LOC), voll unter unserer Kontrolle | `seed-kernel/Cargo.toml:24`, `vendor/wasmi-0.31.2/` |
| Threads/Shared-Memory/SIMD/Multi-Memory dort **hart aus, kein Schalter** | `vendor/wasmi-0.31.2/src/engine/config.rs:416-421` |
| Kernel: **einkernig, kooperativ, keine Interrupt-Präemption**; Hauptschleife pollt | `seed-kernel/src/main.rs:308-314` |
| **Strikt ein Gast in Flug** (Single-Flight-Wächter); resumable „Pump" mit Schritt-Budget existiert bereits | `seed-kernel/src/wasm_runtime/invocation.rs:9,17,229`, `main.rs:417-440` |
| Fuel-Budgets + Host-Suspend-Mechanik vorhanden | `wasm_runtime/envelope.rs:578-624`, `wasm_runtime/suspension.rs` |
| Gast-Speicher: Vec im Kernel-Heap, Deckel **4 MiB/Gast**, Heap gesamt **64 MiB statisch** | `raios-core/src/project_runtime.rs:12`, `seed-kernel/src/main.rs:180-213` |
| **Kein WASI** (null Treffer `wasi_snapshot_preview1`); rein eigene ABI-Familien mit Import-Grant-Gate | `raios-core/src/host_import_abi_v1.rs`, `scoped_wasm_import_grant.rs` |
| Kein Pfad-Dateisystem; Persistenz = RECLOG/ARTSTOR/ProjectRevision (inhalts-adressiert) | `structured_store.rs`, `artifact_store.rs`, `project_workspace.rs` |
| Job-Kette (Gast A → Artefakt → Gast B) **noch nicht** implementiert; Artefakte wandern nur über die Stores | `agent_build_loop.rs`, FACTORY_PLAN §2 |
| rustc-als-Wasm existiert öffentlich (91 MB), Blocker = Threads | FACTORY_PLAN:26, Commit `3d164ca` |

Außen-Stand: Das öffentliche Rezept ([oligamiq/rubrc](https://github.com/oligamiq/rubrc),
[rust_wasm](https://github.com/oligamiq/rust_wasm)) läuft im Browser **mit**
Threads (WASI-Threads-Shim + SharedArrayBuffer); die threads-freie Variante
ist die ungeprüftere. [wasi-threads](https://github.com/WebAssembly/wasi-threads)
ist ein eingefrorenes Legacy-Protokoll mit winziger Oberfläche (im Kern ein
Import: `thread-spawn`), von wasmtime/WAMR/Wasmer/toywasm unterstützt.
Neuere wasmi-Versionen (bis [1.0](https://wasmi-labs.github.io/blog/posts/wasmi-v1.0/))
haben die Threads-Erweiterung **nicht** — ein wasmi-Upgrade allein bringt
also nichts; der Eigenbau am Vendor-Stand ist der realistische Weg.

## 3. Die entscheidende Umdeutung: drei Ausbaustufen von „Threads"

| Stufe | Was es ist | Kostet | Braucht rustc das? |
|---|---|---|---|
| **T-A: Grüne Threads im Käfig** | Interpreter wechselt deterministisch reihum zwischen N Instanzen EINES Gastes, die sich einen Speicher teilen. Kernel bleibt wie heute. | moderat, additiv, host-testbar | **Ja — das reicht.** rustc merkt den Unterschied nicht. |
| T-B: Präemptive Kernel-Threads (einkernig) | Timer-Interrupt, Kontextwechsel, Kernel-Stacks | groß, bringt Nichtdeterminismus | Nein |
| T-C: Echtes Mehrkern (SMP) | AP-Boot, per-CPU-Zustand, Locks überall | sehr groß („der Rewrite") | Nein — nur für Tempo. Bleibt Treppenstufe 5. |

Der gefürchtete Rewrite ist T-C. T-A ist ein Anbau, kein Umbau — und die
Schnittstelle, die T-A den Gästen gibt (`thread-spawn`, Atomics), bleibt
gültig, wenn T-C irgendwann echtes Parallel-Tempo nachliefert. Gäste müssen
dann nicht geändert werden.

## 4. Was für T-A konkret zu bauen ist

**T1 — wasmi-Vendor-Patch (host-testbar, ohne QEMU beweisbar):**
- Validator: `shared`-Flag auf Memory akzeptieren, Atomics-Opcodes zulassen
  (`config.rs`-Schalter + Validierung).
- Executor: die Atomic-Befehlsfamilie (Load/Store/RMW/CmpXchg für
  i32/i64-Breiten, `atomic.fence`) — im einkernigen Interpreter sind das
  gewöhnliche Speicheroperationen mit Bound-Checks, mechanische Arbeit.
- `memory.atomic.wait32/64` → als Host-Suspend austreten (Mechanik
  existiert: `suspension.rs`); `memory.atomic.notify` → Aufweck-Zähler.
- Mehrere Instanzen desselben Moduls im selben Store an **eine** geteilte
  Memory binden (wasmi kann importierte Memories teilen; Globals bleiben
  korrekt pro Instanz).

**T2 — Thread-Scheduler im Kernel-Glue:**
- Den bestehenden Single-Flight-Pump (`ActiveBeyondEnvInvocation::pump`,
  Schritt-Budget 11.250) zu einem **festen Round-Robin über N resumable
  Invocations innerhalb EINES Jobs** verallgemeinern. Der
  Single-Flight-Wächter bleibt auf Job-Ebene unverändert.
- `wasi.thread-spawn`-Hostimport: neue Instanz anlegen, `wasi_thread_start`
  als resumable Call einreihen, deterministische TID-Vergabe.
- Wait/Notify an den Scheduler koppeln; Deckel pro Job (z. B. max 8
  Threads); Fuel bleibt EIN Topf pro Job.
- Die Stellen, die „genau eine aktive Invocation" annehmen (`main.rs:349`,
  Busy-Checks in `envelope.rs:138,310,554`), auf „genau ein aktiver Job"
  umformulieren.

**Invarianten (bindend für die Umsetzung):**
1. Wechsel nur an festen Fuel-Quanten, Aufweck-Reihenfolge deterministisch
   ⇒ zwei Läufe desselben Jobs sind byte-gleich (Doppel-Bau-Beweis bleibt).
2. Hostaufrufe laufen, während alle Geschwister-Threads geparkt sind ⇒ kein
   Time-of-Check/Time-of-Use-Fenster auf Gast-Speicher. (Diese Invariante
   fällt erst bei T-C — dort dann Copy-in-Muster; im ADR festhalten.)
3. Geteilter Speicher existiert nur **innerhalb eines Jobs**; Import-Grants,
   Inertheit bis W5-Klick, signierte Installation: alles unverändert.

## 5. Was so oder so nötig ist (Threads-Frage ändert daran nichts)

| Mauer | Stand | Einordnung |
|---|---|---|
| **Speicher**: 64-MiB-Heap < 91-MB-Modul; 4-MiB-Gast-Deckel vs. Hunderte MB Bedarf | `main.rs:180-213` | **Erste echte Mauer.** Bauplatz-Klasse = Heap auf Limine-Memmap umstellen + große Gast-Speicher; QEMU-RAM für Bauplatz-Profile erhöhen; Surface-RAM (4–16 GB je Modell) messen |
| **WASI-Subset + Datei-Sicht**: rustc erwartet `wasi_snapshot_preview1` (fd/path/args/env/clock/random/exit) | kein WASI im Baum | Shim auf die geprüfte Bauplatz-Datei-Politik: Sysroot nur-lesen, Quelle rein, Artefakt raus, tmp im RAM. Größter Einzelposten der rustc-Spur |
| ~~**Job-Kette** für den Linker~~ | **ENTFÄLLT (Probe 2026-07-18):** rust-lld ist **im Modul eingebettet**, kein zweiter Prozess — ein Bauplatz-Gast genügt | Beleg: `docs/architecture/probe-rustc-wasm-wasmtime-2026-07-18.md` |
| **Sysroot**: vorkompilierte Standard-Bibliothek als Artefakte im Store | — | Teil des Cloud-Bootstrap-Outputs |
| **Ehrliche Grenzen bleiben**: keine Internet-Pakete, keine Proc-Macros, einkernig-langsam | FACTORY_PLAN:46-47 | unverändert; Tempo kommt aus Stufe 4 (AOT), nicht aus Threads |

## 6. Routenvergleich

| | Route A: threads-freier Fork (bisheriger Plan §5) | Route B: Threads nachbauen (dieses Scoping) |
|---|---|---|
| rustc-Artefakt | Fork + Schalter, **unbewiesene** Variante des Rezepts | Stock-Rezept, im Browser **nachweislich laufend** |
| Pflege | Fork-Tretmühle bei jedem Toolchain-Update | einmalige Arbeit in **unserem** Vendor-Code |
| OS-Arbeit | keine (für Threads) | T1+T2 (~2–3 Wochen Kadenz) |
| Zukunftswert | keiner | Thread-API für alle künftigen Gäste (Spiele, NLE); SMP später = reines Tempo-Upgrade ohne Gast-Änderung |
| Risiko | Einzelthread-Betrieb von rustc kaum erprobt | Upstream-Wackler beim Threads-Target (s. §8) |

**Entscheidung (Owner 2026-07-18): Route B. Route A verworfen — kein Fork,
kein Anpassen des Compilers.**

## 7. Spur + Aufwand

Reihenfolge:

1. **ERLEDIGT — GRÜN (2026-07-18):** Werkstatt-Probe unter wasmtime 46:
   Voll-Bau hello 1,56 s / medium `-O` 1,19 s, ~670 MB Spitze, Gast-Threads
   real ~26–32, gebaute Programme laufen. Linker eingebettet, Sysroot-Lücke
   mit 4 wasi-sdk-Libs geschlossen. Bericht:
   `docs/architecture/probe-rustc-wasm-wasmtime-2026-07-18.md`.
2. **T1-Slice** (wasmi-Patch, host-Tests): ~3–5 Codex-Pakete, 1–2 Wochen.
   Parallel-tauglich zur UI-Spur, QEMU-frei.
3. **T2-Slice** (Scheduler-Glue): ~2–3 Pakete, ~1 Woche.
4. **Bauplatz/Heap** (~2–4 Pakete) und **WASI+Datei-Sicht** (~4–6 Pakete,
   2–3 Wochen) — routenunabhängig, können vor/neben T1 starten.
5. **Sysroot als Store-Artefakte** (Job-Kette entfällt — Linker eingebettet,
   Probe 07-18), dann erster `hello.rs`-Bau in QEMU als W5-Beweis.

Grobsumme bis „erstes Rust-Programm auf raiOS kompiliert (langsam, aber
echt)": **~6–10 Wochen** bei aktueller Kadenz; davon Threads-spezifisch nur
~2–3 Wochen. Stufe 4 (AOT-Tempo) und Stufe 5 (SMP/GPU) unverändert danach.

## 8. Risiken (ehrlich)

- **Fremd-Artefakt-Herkunft („trusting trust"):** Das Binary stammt von
  einem Dritten. **Ausführen ist käfig-sicher** (nur genehmigte Importe,
  kein Netz, Ergebnis inert bis W5-Klick — selbst ein bösartiges Binary
  kommt nicht aus dem Käfig; Restrisiko sind Fehler in unseren eigenen
  Käfigstäben, dem kleinen vendorierten Interpreter). Aber ein vergifteter
  Compiler könnte theoretisch Hintertüren in seine **Kompilate** einbauen.
  Gegenmittel ohne Anpassen: Rezept ist öffentlich → denselben Stand später
  selbst nachbacken und Fingerabdruck vergleichen (Herkunftsprüfung);
  Quellen-Preflight (B2) und Inertheit bis zum Klick stehen ohnehin davor.
  Der Compiler bleibt „Arbeiter, keine Autorität" (FACTORY_PLAN §3).
- **Upstream wackelt:** `wasm32-wasip1-threads` hat aktuell offene
  Bruch-Meldungen ([#146721](https://github.com/rust-lang/rust/issues/146721),
  [#146843](https://github.com/rust-lang/rust/issues/146843)) — Toolchain im
  Bootstrap pinnen. [rubrc](https://github.com/oligamiq/rubrc) selbst ist
  „work in progress".
- **wasi-threads ist Legacy** ([zurückgezogen 2023](https://github.com/WebAssembly/wasi-threads/issues/48)
  zugunsten shared-everything-threads) — für WASIp1-Ziele aber weiterhin
  DER Mechanismus; unsere Oberfläche ist klein genug für spätere Migration.
- **Tempo:** interpretiertes rustc wird langsam sein (Minuten für
  Hello-World möglich). Das ist Stufe-4-Arbeit, kein Threads-Problem — im
  Erwartungsmanagement klar halten.
- **RAM auf Zielgerät:** Surface-Ausbau (4/8/16 GB) bestimmt die reale
  Bauplatz-Obergrenze; in Probe-Schritt 2 mitmessen.
- **wasmi-Patch-Tiefe:** Atomics im Interpreter sind mechanisch, aber der
  Validator-Teil braucht Sorgfalt; deshalb host-testbar als eigener Slice
  mit voller Testabdeckung, bevor irgendetwas in QEMU muss.

## 9. Quellen

- [oligamiq/rubrc — Rust compiler in the browser](https://github.com/oligamiq/rubrc), [Demo](https://oligamiq.github.io/rubrc/), [oligamiq/rust_wasm](https://github.com/oligamiq/rust_wasm)
- [wasm32-wasip1-threads — rustc platform support](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1-threads.html)
- [Bytecode Alliance: Announcing wasi-threads](https://bytecodealliance.org/articles/wasi-threads), [wasi-threads Spec/Repo](https://github.com/WebAssembly/wasi-threads), [Zukunft des Proposals](https://github.com/WebAssembly/wasi-threads/issues/48)
- [wasmtime-wasi-threads (experimentell)](https://docs.rs/wasmtime-wasi-threads)
- [Wasmi 1.0 Release-Post — Threads nicht enthalten](https://wasmi-labs.github.io/blog/posts/wasmi-v1.0/)
- Rust-Threads-Target-Bugs: [#146721](https://github.com/rust-lang/rust/issues/146721), [#146843](https://github.com/rust-lang/rust/issues/146843)
