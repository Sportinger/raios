# Werkstatt-Probe: existierender Threads-rustc-Wasm unter wasmtime — GRÜN

Datum: 2026-07-18 · Zweck: FACTORY_PLAN §5 Schritt 1 (Messung statt Schätzung)
· Auftrag: Owner-Entscheid „existierendes Threads-Artefakt unverändert nutzen"

## Ergebnis in einem Satz

Der **unveränderte** öffentliche rustc-als-Wasm (Threads-Variante) kompiliert
und linkt unter wasmtime+wasi-threads echte Rust-Programme zu lauffähigem
Wasm — `hello.rs` in **1,56 s**, ein mittelgroßes Programm mit `-O` in
**1,19 s** — und die gebauten Programme laufen korrekt. Kein Fork, kein
Patch am Compiler; nur Laufzeit-Schalter, Sysroot-Ordnung und vier fehlende
Bibliotheken aus dem offiziellen wasi-sdk.

## Versionen / Herkunft (gepinnt)

| Teil | Wert |
|---|---|
| Artefakt | `oligamiq/rust_wasm` v0.3.0-release, `rustc_opt.wasm.tar.gz` 28,6 MB → **rustc_opt.wasm 91,0 MB** |
| SHA-256 | `c6dccf3e5f01631b942a0a008b9f2f5312987e7d8590f8c61024cd00687a5791` — **identisch** mit der Messung in `docs/plans/b3-plan.md` §7.1 |
| Meldet sich als | `rustc 1.83.0-dev` |
| Sysroot | `wasm32-wasip1-threads.tar.gz` → 27 rlibs + self-contained (71,1 MB entpackt) |
| Referenz-Runtime | **wasmtime 46.0.1** (x86_64-windows) — PINNEN, siehe Warnung unten |
| Nachgelieferte Libs | `libwasi-emulated-{mman,signal,process-clocks,getpid}.a` aus **wasi-sdk-33** `wasi-sysroot-33.0+m.tar.gz` |

## Messwerte

| Messpunkt | Zeit | Spitzen-RAM | OS-Threads max | Ergebnis |
|---|---|---|---|---|
| wasmtime-Übersetzung des Moduls (Cranelift, einmalig) | 25,2 s | 2 172 MB | — | `rustc_opt.cwasm` **326,7 MB** Maschinencode |
| `--version` | 0,36 s | 665 MB | 29 | `rustc 1.83.0-dev` |
| `hello.rs` `--emit=metadata` (Frontend) | 1,57 s | 669 MB | 29 | `libhello.rmeta` |
| `hello.rs` `--emit=obj` (`-Ccodegen-units=1`) | 1,33 s | 669 MB | **55** | `hello.o` 2,3 KB |
| `hello.rs` Voll-Bau inkl. Link | 1,56 s | 669 MB | 55 | **`hello.wasm` 287,4 KB** |
| `medium.rs` (Traits/Generics/HashMap/Closures) `-O`, Standard-CGUs | 1,19 s | 669 MB | **61** | `medium.wasm` 334,1 KB |
| Gebaute Programme ausgeführt | — | — | — | korrekte Ausgabe, exit=0 |

Baseline-OS-Threads von wasmtime selbst: 29. Der Sprung auf 55/61 während
Codegen ist der **direkte Beweis, dass der Gast wasi-`thread-spawn` nutzt**
(rustc-Compile-Thread + LLVM-Worker; selbst bei `codegen-units=1`).
Gast-Thread-Bedarf real beobachtet: **~26–32**.

## Durchbruch-Befund: Ein-Modul-Pipeline

Der Link-Fehler vor der Lib-Nachlieferung lautete
`rust-lld: error: unable to find library -lwasi-emulated-mman` — **aus dem
Inneren des Moduls**. Frontend, LLVM-Backend **und rust-lld stecken alle in
dem einen 91-MB-Modul**; es wird kein zweiter Prozess gestartet. Für raiOS
heißt das: die befürchtete Job-Kette (rustc-Gast → Linker-Gast) **entfällt
für die Rust-Spur** — ein einziger Bauplatz-Gast genügt. (Das separate
`llvm_opt.wasm` [93,5 MB entpackt] ist ein clang/llvm-Multitool für
C-Zwecke, für Rust-Bauten nicht nötig.)

## Rezeptur (was nötig war — vollständig)

1. wasmtime-Flags: `-W threads=y -W shared-memory=y -S threads=y`
   (v46 trennt `shared-memory` von `threads`; Vorkompilat muss mit
   denselben `-W`-Flags erzeugt sein).
2. Sysroot-Ordnung: Tar-Inhalt nach
   `sysroot/lib/rustlib/wasm32-wasip1-threads/lib/` (+`self-contained/`).
3. **Vier fehlende Bibliotheken** aus wasi-sdk-33 nach `self-contained/`
   kopieren (`libwasi-emulated-*.a`) — ohne sie scheitert der Link.
4. `--sysroot /sysroot` **explizit** übergeben (sonst Panik in
   `filesearch.rs:199`: Selbstpfad-Suche via `current_exe` läuft unter
   WASI ins Leere).
5. Arbeitsordner als **Wurzel `/`** mounten (`--dir host::/`): rustc legt
   sein Temp-Verzeichnis über `env::temp_dir()` direkt unter `/` an und
   ignoriert `TMPDIR`.

## Upstream-Befunde (Kandidaten für rubrc-Mithilfe)

1. **Kaputtes Release-Asset:** `llvm_raw.wasm.tar.gz` enthält als einzigen
   Eintrag den baumelnden Symlink `llvm-build/bin/clang -> llvm` — null
   Nutzdaten.
2. **Unvollständiges Sysroot:** das std der `wasm32-wasip1-threads`-Sysroot
   verlangt `-lwasi-emulated-mman`, aber `self-contained/` liefert keine
   der `libwasi-emulated-*.a` mit → Out-of-the-box-Link schlägt fehl.
3. Verhaltensnotizen (doku-würdig): Default-Sysroot-Suche panict unter
   WASI; `TMPDIR` wird ignoriert (Temp landet unter `/`).

## Ökosystem-Warnung (bindend für die Werkstatt)

wasmtime warnt bei jedem Lauf: **`-S threads` wird in wasmtime 47.0.0 zum
harten Fehler (Release 2026-07-20)** — wasi-threads fliegt aus der
Referenz-Runtime raus. Werkstatt-Proben daher auf **wasmtime 46.0.1
gepinnt** lassen. Strategisch bestätigt das den Kurs: raiOS beherbergt die
kleine wasi-threads-Schnittstelle selbst (grüne Threads, Scope-Dokument),
statt von Fremd-Runtimes abzuhängen; die Schnittstelle selbst ist
eingefroren und winzig (im Kern 1 Import: `wasi.thread-spawn`).

## Ableitungen für raiOS (Eichwerte für die Slices)

- **T1/T2 (Threads im Käfig):** Muss `shared memory` + Atomics + spawn
  tragen; Thread-Deckel pro Job ≥ 32 sinnvoll (beobachtet ~26–32).
- **Bauplatz-Klasse:** ~670 MB Prozess-Spitze für Kleinst-Bauten (davon
  327 MB Maschinencode-Abbild); Gast-Linearspeicher-Budget im
  Hunderte-MB-Bereich einplanen; heutiger 64-MiB-Kernel-Heap ist die
  erste Mauer (Scope-Dokument §5).
- **WASI-Subset tatsächlich benutzt:** Datei-/Pfad-Ops inkl. Verzeichnis
  anlegen (Temp unter `/`), args/env, clock, random, proc_exit,
  thread-spawn. Beschreibbare Wurzel bzw. Temp-Bereich gehört in die
  Bauplatz-Dateipolitik.
- **Stufe-4-Zielmarke (AOT):** wasmtime-Klasse-AOT bringt den Wasm-rustc
  auf nur ~2–3× langsamer als nativ (1,2–1,6 s pro Kleinst-Bau). Das ist
  die Ziellinie der schnellen Ausführungsstufe. Interpreter-Tempo bleibt
  offen, bis T1 existiert (wasmi 0.31 kann das Modul nicht laden — genau
  deshalb gibt es T1).
- **Sysroot als Store-Artefakte:** 91-MB-Compiler + 71-MB-Sysroot + 4
  wasi-sdk-Libs = vollständige Werkzeug-Lieferung für W7/W5-Einbringung.

## Ablage (Wiederverwendung, außerhalb des Repos)

`E:\raios-probe-rustc-wasm\` — Artefakte, wasmtime 46, `rustc_opt.cwasm`
(327 MB, spart die 25-s-Übersetzung), präpariertes Sysroot unter
`work\sysroot\`, Quell- und Ausgabedateien unter `work\`. C: wurde nur um
das rustup-Target `wasm32-wasip1-threads` (nightly-2024-10-15, ~40 MB)
erweitert; die Emulations-Libs kamen letztlich aus wasi-sdk-33.

## Offen (ehrlich)

- Größere Crates / mehrere Einheiten, `cargo`-Frage (Build-Orchestrierung
  ohne cargo: einzelne `rustc`-Aufrufe reichen für den Anfang), Proc-Macros
  bleiben zu (Plan-Grenze), Internet-Pakete bleiben zu (W7-Weg).
- Interpreter-Messpunkt folgt erst nach T1 (dann dieselben Quellen erneut
  bauen und gegen diese Tabelle stellen).
