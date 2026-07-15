# B3.0-Spike (Stufe 1): Kann ein Compiler als Wasm-Gast auf raiOS laufen?

Stand: 2026-07-15. Owner-Entscheidung vom selben Tag: der B3.0-Forschungs-Spike
aus `docs/VISION_PLAN.md` §5 wird VORGEZOGEN und läuft als Seitenstrang parallel
zu B1/B2 (analog zum WiFi-Seitenstrang; er gated nichts). Dieses Dokument ist
Stufe 1: Schreibtisch-Recherche — Repo-Fakten plus Stand der Technik. Stufe 2
(Messungen am Werkstatt-PC) ist unten konkret benannt. Das endgültige
GO/NO-GO fällt erst nach Stufe 2.

## 1. Die Frage und der Maßstab

Vision (§1): raiOS baut Programme **auf dem Gerät**. Der plausibelste Weg laut
Plan: rustc als signierter Wasm-Gast (B3.2), in einer neuen Gast-Klasse
„Bauplatz" (B3.1). Dieser Spike prüft: Ist das prinzipiell machbar, wie groß
ist der Abstand zur heutigen Sandbox, und was sind die Alternativen?

## 2. Was unsere Sandbox heute wirklich kann (Repo-Fakten, verifiziert)

Quelle: Quellcode-Durchsicht 2026-07-15 (Pfade:Zeilen im Repo-Stand von heute).

- **Laufzeit:** wasmi 0.31.2, vendored, `no_std`, reiner Interpreter ohne JIT
  (`seed-kernel/Cargo.toml:23`, `vendor/wasmi-0.31.2/`). Ausführung global
  seriell — ein Gast zur Zeit (`wasm_runtime/invocation.rs:229`).
- **Speicher pro Gast:** 2 MiB (Demo/Shell/Beyond-Env) bzw. 4 MiB (Workspace)
  hart über `StoreLimits` (`wasm_runtime/envelope.rs:8`,
  `raios-core/src/project_runtime.rs:12`). Ein Compiler braucht Hunderte MiB.
- **Kernel-Heap gesamt: 64 MiB** statische Arena (`seed-kernel/src/main.rs:179`)
  — DAS ist die bindende Obergrenze, nicht die 512 MiB der VM. Jede
  Gast-Memory wird aus dieser Arena alloziert.
- **Fuel:** übliche Budgets 10k–1M Instruktionen (`wasm_runtime/artifacts.rs`).
  Ein Compiler braucht Milliarden. Positiv: `call_resumable` existiert bereits
  (`invocation.rs:565`) — lange Läufe in Fuel-Scheiben mit Wiederaufnahme sind
  architektonisch schon angelegt.
- **Import-Oberfläche:** grants-nothing; es existiert KEIN einziger
  Datei-Import (kein open/read/write/seek). Ein-/Ausgabe sind 4-KiB-Puffer
  (`envelope.rs:6-7`); unbekannte Imports werden vor Instanziierung
  abgewiesen (`envelope.rs:394`). Ein `wasm32-wasi`-Binary würde heute an
  `fd_read`/`path_open` sofort scheitern.
- **Kandidaten-Größe:** selbstgebaute Installationen sind auf 256 KiB gedeckelt
  (`raios-core/src/project_install.rs:18`). Ein rustc-Wasm-Modul liegt
  vermutlich bei Dutzenden bis Hunderten MiB (plus Sysroot).
- **Wasm-Features (wasmi-Default):** bulk-memory und funcref-Tabellen ja;
  threads, memory64, multi-memory, simd, tail-call NEIN
  (`vendor/wasmi-0.31.2/src/engine/config.rs`). Einzelspeicher 32-bit →
  maximal 4 GiB linear möglich, was für kleine Crates reichte.

## 3. Stand der Technik draußen (Existenzbeweise)

- **rubrc** (github.com/oligamiq/rubrc): der KOMPLETTE rustc mit LLVM-Backend,
  kompiliert nach WASI, läuft im Browser und erzeugt `wasm32-wasip1`- und
  `x86_64-linux-musl`-Binaries. Status: Work-in-progress, pausiert, ohne
  Threads „very slow", braucht ein virtuelles Dateisystem-Shim. **Beweis: es
  geht prinzipiell.** Größen-/Speicherzahlen unveröffentlicht → Stufe 2 misst.
- **cg_clif** (Cranelift-Backend für rustc, bjorn3): wird inzwischen über
  rustup als experimentelles Backend verteilt — der im Vision-Plan genannte
  Cranelift-Weg ist real und gepflegt.
- **TinyCC als Wasm** (github.com/lupyuen/tcc-riscv32-wasm u.a.): ein winziger,
  selbst-hostender C-Compiler läuft bewiesen als Wasm-Modul mit ROM-FS —
  Speicherbedarf einstellige MiB, Modulgröße unter 1 MiB.
- **Interpreter-Tempo:** wasmi zählt zu den schnellsten Interpretern, liegt
  aber für rechenlastige Läufe grob ein bis zwei Größenordnungen unter
  nativ/JIT; wasmi 0.32 brachte bis zu 5× gegenüber unserer 0.31er-Engine
  (wasmi-labs Benchmark/Blog). Grobe ehrliche Erwartung: ein Mini-Crate, das
  nativ unter einer Sekunde kompiliert, braucht interpretiert Minuten; echte
  Crates eher Stunden. Die Vision verlangt geschlossen, nicht schnell — aber
  der Owner soll die Zahl kennen, BEVOR gebaut wird.

## 4. Die fünf Lücken zwischen heute und „Bauplatz" (jede einzeln lösbar, keine prinzipiell)

1. **Datei-Oberfläche:** neue geprüfte Import-Familie (WASI-Subset oder eigene
   `build.*`-ABI): read-only Quellbaum + Sysroot, read-write Scratch,
   alles gehasht und quotiert. Heute: null Datei-Imports.
2. **Speicher:** Gast-Limit von 2–4 MiB auf Hunderte MiB heben UND die
   64-MiB-Kernel-Arena vergrößern (Allocator-Arbeit; QEMU-Profil ggf. auf
   mehr RAM). Wasm32 selbst trägt bis 4 GiB.
3. **Fuel-Geduld:** Budgets um 3–5 Größenordnungen höher, als Scheiben über
   das vorhandene `call_resumable` — dann blockiert ein langer Bau auch nicht
   den Ein-Gast-zur-Zeit-Betrieb.
4. **Tempo:** wasmi-Upgrade 0.31 → aktuell (Register-Engine) als erster
   billiger Hebel; alles Weitere erst nach Messung.
5. **Größe/Beschaffung:** rustc-Modul + Sysroot (zusammen vermutlich einige
   hundert MiB) müssen durch W7-Beschaffung und auf den Datenträger — die
   256-KiB-Kandidaten-Grenze gilt für diese neue Artefakt-Klasse nicht mehr.

## 5. Wege im Vergleich

- **Weg A — rustc als Wasm-Gast (Plan-Weg, B3.2):** durch rubrc als machbar
  belegt; teuerster Weg (alle fünf Lücken), aber der einzige, der „beliebige
  Rust-Programme auf dem Gerät" liefert. LLVM- vs. Cranelift-Backend
  entscheidet Stufe 2 (Cranelift vermutlich kleiner/schneller im Interpreter).
- **Weg B — eigener Wasm→nativ-Übersetzer im Kernel (JIT/AOT):** würde alles
  beschleunigen, ist aber ein großes neues Vertrauens-Bauteil (selbst-
  modifizierender Code widerspricht der Sandbox-Verfassung) — NICHT empfohlen
  vor der Endstufe.
- **Weg C — RUIP wächst weiter (B3.3, läuft schon):** erfüllt „Programme
  entstehen im System" früh, aber nie „beliebige Programme". Bleibt die
  parallele Absicherung, ersetzt A nicht.
- **Weg D — Mini-Compiler zuerst (neuer Vorschlag):** einen winzigen
  bewiesenen Compiler (TinyCC-Klasse, wenige MiB) als ERSTEN echten
  On-Device-Compiler durch die Schleife führen. Braucht dieselben Lücken
  1–3, aber in kleiner Dosis (16–64 MiB statt Hunderten). Beweist Station 4
  der Schleife komplett, BEVOR der rustc-Brocken angefasst wird, und
  de-riskt Weg A fast vollständig.

## 6. Vorläufiges Urteil (Stufe 1)

**GO unter Messvorbehalt.** Es gibt keinen prinzipiellen Blocker: der komplette
rustc läuft nachweislich als Wasm/WASI (rubrc), unsere Architektur hat mit
`call_resumable`, StoreLimits und der Import-Subset-Prüfung bereits die
richtigen Ansatzpunkte, und alle fünf Lücken sind Ingenieursarbeit, keine
Forschung. Empfohlene Reihenfolge, wenn B3 dran ist: **Lücken 1–3 in kleiner
Dosis für Weg D → Station-4-Beweis mit Mini-Compiler → dann Weg A skalieren.**
Das endgültige GO/NO-GO für Weg A fällt nach Stufe 2.

## 7. Stufe 2: die Messungen (Werkstatt-PC, stört keine laufende Arbeit)

1. rustc-als-WASI-Build (rubrc-Artefakt oder eigener Build, LLVM und — falls
   verfügbar — Cranelift-Variante) unter wasmtime UND unter wasmi (aktuelle
   Version, Interpreter wie bei uns) auf dem Host laufen lassen:
   Hello-World-Crate und ein kleines No-std-Crate kompilieren.
2. Messen: Peak-Speicher, Wanduhr-Zeit (wasmi vs. wasmtime vs. nativ),
   Modulgröße rustc.wasm, Sysroot-Größe, tatsächlich benötigte
   WASI-Import-Liste (die definiert unsere Lücke 1 exakt).
3. Dasselbe für einen TinyCC-Klasse-Kandidaten (Weg D) — erwartbar um
   Größenordnungen kleiner; bestätigt die kleine Dosis der Lücken 1–3.
4. Ergebnis: GO/NO-GO-Nachtrag hier im Dokument mit den echten Zahlen;
   erst danach darf B3.1 („Bauplatz"-Gast-Klasse) zugeschnitten werden.

## 8. Was dieser Bericht NICHT behauptet

- Keine Zahlen aus eigener Messung — alle Größen-/Tempo-Angaben sind
  Literaturwerte oder benannte Schätzungen; Stufe 2 ersetzt sie.
- Kein Zeitplan; der Spike gated B1/B2 nicht und wird nicht von ihnen gated.
- Weg D ist ein Vorschlag an den Owner, keine beschlossene Planänderung —
  `docs/VISION_PLAN.md` bleibt unverändert, bis Stufe 2 vorliegt.

Quellen (extern): github.com/oligamiq/rubrc; bytecodealliance.org
(„Wasmtime and Cranelift in 2023", cg_clif via rustup);
github.com/lupyuen/tcc-riscv32-wasm; wasmi-labs.github.io (v0.32-Engine);
internals.rust-lang.org/t/running-rustc-on-wasm/16198.
