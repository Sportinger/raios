# Forward-Compatibility-Audit der raiOS-Fabrik

## Urteil in einem Satz

Die heutige Architektur ist für den bewiesenen Mini-Kreislauf sinnvoll klein,
aber sie ist noch **keine verkleinerte Version einer Spiele-/NLE-Fabrik**. Vier
Grenzen sind echte Landminen: die fehlende Dateioberfläche für Toolchains, die
fehlende Unterausführung für `build.rs` und Proc-Makros, eine schnelle
Ausführungsstufe ohne Verlust des Wasm-Käfigs und der GPU-/Medien-Unterbau.

Der verbindliche Maßstab dieses Audits ist nicht „kleine Programme laufen“,
sondern: Agenten bauen und testen große Software auf raiOS selbst, über
Genesis-Jobs und ohne Werkstatt-PC. Genau so ist das Owner-Ziel formuliert
(LOCAL FACT: `docs/FACTORY_PLAN.md:5-14`). Der heutige Assembler-Beweis zeigt,
dass die Rohrleitung Auftrag → Bauen → Nachrechnen → Freigabe → Lauf echt ist;
er beweist noch nicht, dass ein allgemeiner Compiler, Cargo oder ein großes
Testsystem hindurchpasst (LOCAL FACT: `docs/FACTORY_PLAN.md:20-29`;
`docs/ROADMAP.md:229-244`).

Dieses Dokument trennt deshalb streng:

- **LOCAL FACT**: im Repository direkt belegter Ist-Stand; immer mit Datei und
  Zeile.
- **ESTIMATE**: Größen-, Zeit- oder Aufwandsabschätzung; kein Beweis.
- **ASSUMPTION-TO-VERIFY**: plausible Annahme, die durch ein benanntes
  Experiment bestätigt oder verworfen werden muss.

Die Einstufungen bedeuten:

- **UPGRADE-IN-PLACE**: eine neue Stufe oder Klasse kommt hinzu; der alte Weg
  und seine Beweise können unverändert weiterbestehen.
- **MAJOR-BUT-BOUNDED**: ein echtes neues Subsystem ist nötig, aber die
  bestehenden Vertrauens- und Beweisgrenzen können im Wesentlichen darum herum
  erhalten bleiben.
- **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR**: die Änderung berührt den
  Ausführungs- oder Vertrauenskern. Eine ADR, ein bewusstes Sicherheitsmodell
  und eine erneute Beweiskette sind Pflicht.

## Schnellübersicht

| Achse | Einstufung | Kurzurteil |
|---|---|---|
| 1. Ausführungsmaschine | **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR** | Ein schneller Interpreter ist austauschbar; native/AOT-Ausführung ohne gleichwertigen Käfig wäre ein Fundamentwechsel. |
| 2. Nebenläufigkeit und Mehrkern | **MAJOR-BUT-BOUNDED** | Viele getrennte Ein-Kern-Gäste sind beherrschbar; Shared-Memory-Threads im Gast wären eine eigene harte ADR. |
| 3. Speicherbudgets | **MAJOR-BUT-BOUNDED** | Größere Zahlen allein reichen nicht; der Bauplatz braucht isolierte, rückgewinnbare Speicherverwaltung. |
| 4. Datei- und I/O-Oberfläche | **MAJOR-BUT-BOUNDED** | Ein begrenzter virtueller Dateiraum ist ein neues Subsystem und die größte kurzfristige Toolchain-Landmine. |
| 5. Prozess- und Unterausführungsmodell | **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR** | Cargo, `build.rs` und Proc-Makros führen während des Baus weiteren Code aus; dafür existiert heute keine sichere Semantik. |
| 6. Dauerhafter Arbeitsraum | **MAJOR-BUT-BOUNDED** | Der QEMU-Proof-Store ist echt, aber klein und absichtlich nicht für physische Multi-GiB-Projekte freigegeben. |
| 7. Vertrauenshärtung | **MAJOR-BUT-BOUNDED** | Pinning und Dev-Schlüssel sind ein ehrlicher Anfang, aber kein Produktions-Vertrauensanker. |
| 8. Adressraum | **MAJOR-BUT-BOUNDED** | wasm32 bleibt nutzbar; memory64 kann später als zusätzliche Klasse kommen, ist im heutigen Motor aber aus. |
| 9. Grafik, Eingabe und Ton | **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR** | GPU-Treiber und privilegierte DMA-Ausführung berühren den Kern; Eingabe und Audio allein wären begrenzter. |
| 10. Determinismus | **MAJOR-BUT-BOUNDED** | Reproduzierbar sein muss der Bau, nicht das laufende Spiel; Threads, Tools und AOT erschweren genau diese Trennung. |
| 11. Genesis-Job und Testfabrik | **UPGRADE-IN-PLACE** | Der Mini-Job ist der richtige Anfang; der Jobgraph kann wachsen, sobald Datei-, Unterprozess- und Ressourcenmodelle existieren. |
| 12. Netz und Verteilung | **MAJOR-BUT-BOUNDED** | Die vorhandenen TCP-/Acquire-Pfade sind Test- und Spezialpfade; eine allgemeine App-Netzautorität existiert nicht. |

## 1. Ausführungsmaschine: Interpreter heute, schnelle Stufe später

| Punkt | Aussage |
|---|---|
| **Heute** | raiOS pinnt `wasmi = 0.31.2` ohne Default-Features. Der vendorte Motor bezeichnet sich selbst als Bytecode-Interpreter. Die Architekturentscheidung verbietet einen JIT im Kernel; Dienste laufen als `wasm32-unknown-unknown`. (LOCAL FACT: `seed-kernel/Cargo.toml:24`; `vendor/wasmi-0.31.2/src/engine/mod.rs:97-106`; `docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md:42-60`.) |
| **Warum bewusst gewählt** | Der Interpreter ist der Käfig: Vor dem Lauf wird das Modul geparst, und nur berechnete Host-Imports werden verlinkt. Für kleine Werkzeuge, UI-Panels und Adapter wurde Interpretertempo ausdrücklich akzeptiert. (LOCAL FACT: `docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md:44-60`.) |
| **Endziel verlangt** | Große Builds und große Laufzeitprogramme brauchen wesentlich mehr Rechenleistung. Der Fabrikplan nennt deshalb ausdrücklich eine schnelle Ausführungsstufe sowohl für Build-Tempo als auch für Spiele. (LOCAL FACT: `docs/FACTORY_PLAN.md:48-53`.) |
| **Einstufung** | **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR** für AOT/native; ein reiner wasmi-Versionssprung ist dagegen **UPGRADE-IN-PLACE**. |
| **Ehrliches Risiko** | Wenn erzeugter Maschinencode direkt mit Kernelrechten läuft, ist die heutige Aussage „nur verlinkte Imports sind erreichbar“ nicht langsamer, sondern schlicht nicht mehr wahr. |

**Geschwindigkeitsbild.** Der lokale B3-Plan schätzt den alten Interpreter für
Compilerarbeit auf grob 10- bis 100-mal langsamer als native Ausführung und
einen ersten Hello-Bau auf 100 bis 10.000 Sekunden. Das sind **ESTIMATES**, keine
raiOS-Messungen (LOCAL FACT über den Status der Schätzung:
`docs/plans/b3-plan.md:103-110`). Derselbe Plan hält eine Upstream-Aussage fest,
wonach wasmi 0.32 gegenüber 0.31 bis zu etwa fünfmal schneller sein könne. Das
ist ebenfalls **ASSUMPTION-TO-VERIFY**, ausdrücklich keine lokale Messung
(LOCAL FACT über diese Einordnung: `docs/plans/b3-plan.md:112-118`). Selbst ein
Faktor fünf löst eine Größenordnung von 10- bis 100-mal nicht garantiert.

**Was die schnelle Stufe mit dem Beweis macht.** Heute wird das erzeugte Wasm
zweimal gebaut, gegen eine unabhängige Kernel-Neuberechnung bytegleich geprüft
und anschließend mit `Module::new` validiert; erst ein vollständig bestandener
Lauf wird als `current_boot` behalten (LOCAL FACT:
`seed-kernel/src/agent_protocol_build_assemble.rs:623-700`;
`seed-kernel/src/agent_protocol_build_assemble.rs:747-765`). Eine sichere
schnelle Stufe müsste mindestens Folgendes ergänzen:

1. Das validierte Wasm bleibt das maßgebliche Artefakt.
2. Wasm → Maschinencode wird als eigene, hash-gebundene Transaktion behandelt.
3. Zwei frische Übersetzungen müssen bytegleich sein oder eine begründete,
   kanonische Normalisierung besitzen.
4. Der native Code läuft in einer Isolation, die dieselbe Import- und
   Speichergrenze wirklich erzwingt; Ring-0-Ausführung allein erfüllt das nicht.
5. Der Interpreter bleibt zunächst Referenz-Orakel für ausgewählte
   Differenztests.

**ASSUMPTION-TO-VERIFY:** Ein AOT-Backend kann auf der Zielhardware
reproduzierbaren Maschinencode erzeugen, ohne versteckte CPU-, Adress- oder
Zeitabhängigkeiten. Doppeltes Übersetzen beweist dabei nur Reproduzierbarkeit,
nicht die Fehlerfreiheit desselben Backends. Darum wäre „AOT erzeugt zweimal
dasselbe“ kein Ersatz für Isolation oder differenzielle Ausführung.

## 2. Nebenläufigkeit: Threads in einem Compiler sind nicht viele Gäste

| Punkt | Aussage |
|---|---|
| **Heute** | Der Kernel arbeitet in einem einzigen kooperativen Hauptloop. `PeriodicTask` ruft Funktionen nur nacheinander auf; es gibt keinen Thread-Scheduler. wasmi schaltet die Wasm-Features `threads`, Shared Memory und `memory64` aus. (LOCAL FACT: `seed-kernel/src/main.rs:306-314`; `seed-kernel/src/scheduler.rs:3-32`; `vendor/wasmi-0.31.2/src/engine/config.rs:403-423`.) |
| **Warum bewusst gewählt** | Ein serieller Loop macht Reihenfolge, Fuel-Verbrauch, Abbruch und Audit leichter nachrechenbar. Die erste Isolation sollte die Importgrenze beweisen, nicht zugleich einen SMP-Kernel und ein Shared-Memory-Modell erfinden. (LOCAL FACT: `docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md:42-60`.) |
| **Endziel verlangt** | Große Fabrikaufträge brauchen Durchsatz über mehrere Kerne. Manche Compiler können intern Threads nutzen; unabhängig davon kann die Fabrik mehrere getrennte Compile-/Testgäste parallel abarbeiten. Der Fabrikplan nennt Mehrkern-Bauen als spätere Spielmaßstab-Stufe. (LOCAL FACT: `docs/FACTORY_PLAN.md:41-53`.) |
| **Einstufung** | **MAJOR-BUT-BOUNDED** für viele voneinander isolierte Gäste auf mehreren Kernen. Shared-Memory-Threads *innerhalb* eines Gastes würden zu **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR** eskalieren. |
| **Ehrliches Risiko** | Ein „Threads an“-Schalter kann das Compiler-Wasm ladbar machen, liefert auf einem Ein-Kern-Kernel aber noch keine Parallelbeschleunigung und öffnet zugleich neue Race-, Atomics- und Determinismusfragen. |

Der Unterschied ist wichtig:

- **Threads in einem Compiler:** Ein einzelner rustc-Gast teilt Speicher zwischen
  mehreren Ausführungsfäden. Das verlangt Wasm Shared Memory, Atomics,
  Thread-Erzeugung und eine definierte Abbruchsemantik. Der heutige Motor lehnt
  Shared Memory ausdrücklich ab (LOCAL FACT:
  `vendor/wasmi-0.31.2/src/module/utils.rs:26-38`).
- **Viele parallele Gäste:** Genesis zerlegt einen Job in getrennte Einheiten,
  etwa unabhängige Crates oder Tests. Jeder Gast besitzt seinen Speicher und
  seine Imports allein. Das ist für die raiOS-Fabrik der günstigere erste Weg,
  weil Fehler und Berechtigungen nicht geteilt werden müssen.

Der gemessene rustc-Wasm-Kandidat ist 91 MiB groß und wird heute schon beim
Laden abgewiesen, weil er Shared Memory verlangt; mehr RAM oder Fuel ändert
daran nichts (LOCAL FACT: `docs/plans/b3-plan.md:249-264`). Ein threads-freier
Build würde diese *Ladebarriere* entfernen. Er würde noch keinen Mehrkern-Bau
beweisen.

**ESTIMATE: Anforderungen an einen SMP-Gäste-Scheduler.** Nötig sind
Start/Verwaltung der weiteren CPU-Kerne, pro Kern eine Run-Queue oder ein
klarer zentraler Dispatcher, Timer/Unterbrechung oder harte Fuel-Scheiben,
thread-sichere Speicher- und Store-Verwaltung, kernübergreifender Abbruch,
deterministische Audit-Sequenzen und eine Regel, welcher Kern welche
Geräte-/Netzlease besitzen darf. Der heutige `Option<ActiveBeyondEnvInvocation>`
hält genau eine aktive Beyond-Env-Ausführung im Hauptloop; er ist keine
Mehrkern-Basis (LOCAL FACT: `seed-kernel/src/main.rs:340-365`;
`seed-kernel/src/main.rs:416-443`).

## 3. Speicherbudgets: größere Zahl ist noch kein Bauplatz

| Punkt | Aussage |
|---|---|
| **Heute** | Der globale Kernel-Heap ist eine statische 64-MiB-Fläche. Buffer-Gäste sind auf 2 MiB begrenzt, der Workspace-Gast auf 4 MiB, eine Instanz, einen Speicher und 250.000 Fuel; der Assembler bekommt 1.000.000 Fuel. Der normale QEMU-Lauf gibt dem ganzen System 512 MiB. (LOCAL FACT: `seed-kernel/src/main.rs:179-187`; `seed-kernel/src/wasm_runtime/envelope.rs:4-9`; `raios-core/src/project_runtime.rs:9-15`; `seed-kernel/src/agent_protocol_build_assemble.rs:22`; `scripts/run-stage0-qemu.ps1:62-67`.) |
| **Warum bewusst gewählt** | Kleine feste Klassen begrenzen Schaden, machen Fuel-/Speicherfehler reproduzierbar und reichen für die bisher bewiesenen Mini-Gäste. Die Limits werden beim wasmi-Store wirklich angehängt, nicht nur dokumentiert. (LOCAL FACT: `seed-kernel/src/wasm_runtime/envelope.rs:175-194`; `seed-kernel/src/wasm_runtime/envelope.rs:593-623`.) |
| **Endziel verlangt** | Ein rustc-Kandidat plus Sysroot und Linker braucht eine eigene „Bauplatz“-Klasse mit Hunderten MiB und möglicherweise deutlich mehr. Der lokale Plan schätzt den Compiler-Working-Set auf 512 MiB bis 2 GiB. (LOCAL FACT für die geplante Klasse: `docs/FACTORY_PLAN.md:41-47`; ESTIMATE im Plan: `docs/plans/b3-plan.md:103-113`.) |
| **Einstufung** | **MAJOR-BUT-BOUNDED**. |
| **Ehrliches Risiko** | Wird nur `HEAP_SIZE` erhöht, kann ein großer Gast den permanenten Kern durch Fragmentierung oder OOM mitreißen; der heutige Allocation-Fehler endet im Kernel-Panic. |

Der aktuelle OOM-Pfad protokolliert den Fehler und panikt anschließend (LOCAL
FACT: `seed-kernel/src/main.rs:527-530`). Für untrusted Compilerjobs ist das
kein akzeptabler Bauplatz. Der Ausbau braucht mindestens:

- eine vom kleinen permanenten Kern getrennte Seiten-/Frame-Zuteilung für
  große Gäste;
- harte Reserven für Recovery, Eingabe, Audit und Netz, die ein Baujob nie
  belegen darf;
- pro Job messbare Peak-, Commit- und Scratch-Budgets;
- vollständige Rückgabe aller Seiten nach Erfolg, Trap, Fuel-Ende und F12;
- eine OOM-Antwort als Jobfehler statt als System-Panic;
- Summenlimits über mehrere parallele Gäste.

**ASSUMPTION-TO-VERIFY:** Der threads-freie rustc-Wasm-Kandidat passt mit einem
realen Hello-Build unter eine bestimmte Peak-Grenze. Diese Grenze darf nicht
aus der 91-MiB-Dateigröße geraten werden: wasmi-Übersetzung, lineare
Gastspeicher, Sysroot, Linker und Scratch existieren gleichzeitig. Erst ein
Peak-Messlauf dimensioniert die Bauplatz-Klasse.

## 4. I/O und Dateien: die größte kurzfristige Toolchain-Landmine

| Punkt | Aussage |
|---|---|
| **Heute** | Das allgemeine Buffer-ABI erlaubt höchstens 4 KiB Eingabe und 4 KiB Ausgabe. Externe Wasm-Kandidaten sind auf 262.144 Byte begrenzt. Der Projekt-Workspace erlaubt 32 Dateien, 32 KiB pro Datei und 48 KiB Quelltext insgesamt. Dependency-Bundles sind auf 32 Dateien, 512 KiB pro Datei und insgesamt 64 Stücke zu je 24 KiB begrenzt. Projekt-Reads liefern höchstens 512 Byte, Suchen höchstens 16 Treffer. (LOCAL FACT: `seed-kernel/src/wasm_runtime/envelope.rs:4-9`; `seed-kernel/src/wasm_runtime/envelope.rs:850-927`; `seed-kernel/src/module_candidate_intake.rs:7-14`; `raios-core/src/project_workspace.rs:5-13`; `raios-core/src/project_workspace.rs:263-293`; `raios-core/src/project_dependency.rs:5-12`; `seed-kernel/src/project_query.rs:10-12`.) |
| **Warum bewusst gewählt** | Die ersten Gäste bekommen nur exakt benannte Host-Funktionen; Pfade, Symlinks, Umgebungsverzeichnisse und beliebige Geräte existieren für sie nicht. Damit ist Autorität als Importliste physisch prüfbar. (LOCAL FACT: `raios-core/src/scoped_wasm_import_grant.rs:26-99`.) |
| **Endziel verlangt** | rustc, Sysroot, Linker, Cargo-Metadaten, Quellbäume und Scratch benötigen viele benannte Dateien, wahlfreie Reads, temporäre Writes, Rename/Commit und große Artefakte. Der gemessene Kandidat besteht aus etwa 91 MiB Compiler plus mindestens einem 33–36 MiB komprimierten Ziel-Sysroot. (LOCAL FACT: `docs/plans/b3-plan.md:249-254`.) |
| **Einstufung** | **MAJOR-BUT-BOUNDED** — aber ausdrücklich ein **neues Subsystem**, nicht „Buffer auf 64 KiB stellen“. |
| **Ehrliches Risiko** | Ein zu POSIX-ähnlicher Dateibaum vergrößert die Angriffsfläche massiv; ein zu enges Handle-ABI lässt reale Toolchains erst sehr spät an tausend kleinen Operationen scheitern. |

Die heutige bekannte Host-Importliste enthält Log/Counter, drei Buffer-Imports,
begrenzte Netz-/Krypto-/Zeit-/Secret-/Acquire-Imports und sechs UI-Imports. Sie
enthält **keinen** Datei-, Verzeichnis- oder Prozessimport (LOCAL FACT:
`raios-core/src/scoped_wasm_import_grant.rs:36-99`). Zusätzlich sind pro
Grant höchstens 16 Imports zugelassen; ein künftiges Datei-ABI sollte daher
nicht tausende Einzelimporte erzeugen, sondern wenige typisierte Handle-
Operationen nutzen (LOCAL FACT: `raios-core/src/scoped_wasm_import_grant.rs:99`).
Der Projekt-Build-Status
meldet folgerichtig `generic_filesystem_exposed=false` (LOCAL FACT:
`seed-kernel/src/agent_protocol_project_build.rs:255-279`).

Zwei weitere Größenwände liegen hinter dem Intake: Die heutige dauerhafte
Projektinstallation akzeptiert höchstens 262.144 Kandidaten-Bytes, 1 MiB
Zustand und 16 Imports. Selbst die getrennte Core-Policy begrenzt ein
Kern-Executable auf 64 MiB (LOCAL FACT:
`raios-core/src/project_install.rs:21-25`;
`raios-core/src/core_policy.rs:12-16`). Diese Werte sind sinnvolle kleine
Klassen, aber kein ehrlicher Installationspfad für ein großes Spiel, eine NLE
oder später einen großen selbstgebauten Kern.

Die richtige Richtung ist kein globales Laufwerk `C:` im Gast, sondern ein
begrenzter virtueller Namensraum:

- unveränderliche Handles für exakte Source- und Sysroot-Manifeste;
- Reads nur innerhalb der hash-gebundenen Datei und Länge;
- ein job-eigener Scratch-Namensraum mit Gesamt-, Datei- und Inode-Quota;
- atomarer finaler Output-Commit;
- keine Pfadflucht, Symlinks, Geräte, Secrets oder ambienten Verzeichnisse;
- jeder geöffnete Inhalt bleibt auf Revision, Hash und Job gebunden;
- ein deterministisches Verzeichnis- und Zeitmodell für reproduzierbare Builds.

Das ist mit dem bestehenden Capability-Modell vereinbar, aber es ist trotzdem
ein VFS-/WASI-ähnliches Subsystem mit Parsern, Lebenszyklen,
Crash-Konsistenz und vielen Negativfällen. Der B3-Plan benennt genau diese
Handle-Richtung bereits (LOCAL FACT: `docs/plans/b3-plan.md:80-99`).

**ASSUMPTION-TO-VERIFY:** „Tausende Dateien“ ist für eine vollständige
Toolchain plausibel, aber im Repository nicht als gemessene Dateizahl belegt.
Lokal belegt sind nur 91 MiB Compiler und ungefähr 33–36 MiB komprimierter
Sysroot pro Ziel. Die exakte Zahl geöffneter Dateien, Operationen und Bytes muss
vor dem ABI-Entwurf getraced werden. Das Audit macht aus der plausiblen Zahl
bewusst keinen LOCAL FACT.

## 5. Prozessmodell: `build.rs`, Proc-Makros und Cargo sind heute draußen

| Punkt | Aussage |
|---|---|
| **Heute** | Es gibt keinen `spawn`-, Prozess-, Dynamic-Library- oder allgemeinen Dateisystem-Import in der vollständigen bekannten Gast-Importliste. Der B3-Plan hält fest, dass Linker-Unterprozesse, Cargo-Build-Skripte und Proc-Makros mit dem ersten Bauplatz unvereinbar sind. (LOCAL FACT: `raios-core/src/scoped_wasm_import_grant.rs:36-99`; `docs/plans/b3-plan.md:80-95`.) |
| **Warum bewusst gewählt** | Der Compiler soll Arbeiter, nicht Autorität sein. Ein Kindprozess darf nicht durch einen unkontrollierten Seitenausgang mehr Datei-, Netz- oder Secret-Rechte erhalten als sein Elternjob. (LOCAL FACT: `docs/FACTORY_PLAN.md:60-67`.) |
| **Endziel verlangt** | Reale Cargo-Projekte starten rustc und Linker mehrfach. `build.rs` wird kompiliert und während des Baus ausgeführt; Proc-Makros sind ebenfalls ausführbarer Build-Code. Große Spiele/NLEs nutzen außerdem oft Shader-, Bindings-, Asset- und C/C++-Werkzeuge. |
| **Einstufung** | **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR**. |
| **Ehrliches Risiko** | Wer „Prozess-Spawn“ wie einen normalen Import freigibt, baut einen zweiten Agenten-Shell-Zugang mitten in die Beweiskette. |

Ohne Unterausführung können folgende Klassen nicht ehrlich als unterstützt
gelten:

- jede Dependency mit einem tatsächlich benötigten `build.rs`;
- Proc-Makro-Crates und Nutzer davon, zum Beispiel verbreitete
  `derive`-Makros;
- Crates, die C/C++ bauen, Systembibliotheken suchen oder Bindings erzeugen;
- ein rustc-Weg, der `wasm-ld` nur als externen Prozess erreicht;
- Cargo als klassischer Prozess-Orchestrator;
- Buildpipelines, die Shader, Assets oder Codegeneratoren als externe Tools
  starten.

Die zwei ehrlichen Optionen sind:

1. **Scope bewusst begrenzen.** Manifestseitig `build.rs`, Proc-Makros und
   externe Tools verweigern. Das reicht für einen ersten Rust-Hello-Bau, aber
   ausdrücklich nicht für den Owner-Maßstab.
2. **Sandboxed Sub-Execution.** Genesis — nicht der Gast selbst — nimmt einen
   typisierten Kindauftrag an, wählt ein bereits geprüftes Tool-Artefakt,
   erzeugt einen neuen Gast mit abgeleiteten kleineren Handles, wartet auf ein
   hash-gebundenes Ergebnis und schreibt den Kindbeleg in den Elternjob.

Die zweite Option ist eher „Fabrikmeister startet eine abgeschlossene Maschine“
als POSIX `fork/exec`. Sie kann den Capability-Gedanken erhalten. Sie braucht
aber Regeln für Rekursionstiefe, Deadlock, Abbruch, Output, Umgebungsvariablen,
Exitstatus, dynamische Proc-Makro-Ladung und deterministische Reihenfolge.

**ASSUMPTION-TO-VERIFY:** Ein repräsentativer Rust-Projektkorpus lässt sich mit
einem kleinen Satz typisierter Unterausführungen bauen. Wenn der Trace stattdessen
breite POSIX-Prozess-, Loader- und Filesystem-Semantik fordert, ist der
Rust-Wasm-Weg für game-scale Builds wesentlich teurer als heute angenommen.

## 6. Persistenz und Workspace: echt, aber absichtlich nur Prüfmedium

| Punkt | Aussage |
|---|---|
| **Heute** | Projekt-Revisionen liegen nicht nur im RAM: `project_store` schreibt in den strukturierten C1-Store. Dieser akzeptiert aber ausschließlich einen wegwerfbaren QEMU-AHCI-Port mit fest eingefrorenen GUIDs; physische Medien werden nie als Fallback gewählt. Die Protokollantwort meldet `qemu_disposable_structured_store_only` und `physical_media_supported=false`. (LOCAL FACT: `seed-kernel/src/project_store.rs:21-25`; `seed-kernel/src/project_store.rs:216-235`; `seed-kernel/src/structured_store_c1.rs:1-5`; `seed-kernel/src/structured_store_c1.rs:746-748`; `seed-kernel/src/agent_protocol_project_query.rs:111-123`.) |
| **Heute, Build-Seite** | Die heutige Projekt-Build-Haltung ist `current_boot_ram_only`; der allgemeine Buildpfad versucht weder Gast-Compiler noch persistenten Write. (LOCAL FACT: `seed-kernel/src/agent_protocol_project_build.rs:255-291`.) |
| **Warum bewusst gewählt** | Der QEMU-Store beweist Identität, strukturierte Records, Readback und Reboot ohne versehentlich eine echte Platte zu beschreiben. Das ist korrekte Entwicklungs-Sicherheit. (LOCAL FACT: `seed-kernel/src/structured_store_c1.rs:861-944`; `docs/ROADMAP.md:353-359`.) |
| **Endziel verlangt** | Große Builds brauchen dauerhafte Multi-GiB-Quell-, Dependency-, Sysroot-, Scratch-, Cache-, Testdaten- und Artefakträume über Reboots hinweg. Ein Absturz darf nicht alle teuren Zwischenstände verlieren und keine halben Artefakte sichtbar machen. |
| **Einstufung** | **MAJOR-BUT-BOUNDED**. |
| **Ehrliches Risiko** | Wird der kleine Record-Store ohne Freiraum-, GC-, Quota- und Crash-Modell zu einem Multi-GiB-Workspace aufgeblasen, kommt die Wand erst unter echter Last und kann dann Daten oder Rollbackraum blockieren. |

Wichtig ist die Korrektur der Kurzform „RAM-only Store“: Die **Quellrevisionen**
sind auf dem wegwerfbaren QEMU-Store reboot-fähig; **Buildstatus und erzeugter
aktueller Output** sind heute RAM/current-boot. Diese Trennung steht direkt im
Code (LOCAL FACT: `seed-kernel/src/agent_protocol_project_query.rs:111-123`;
`seed-kernel/src/agent_protocol_project_build.rs:255-291`) und der Reboot-
Replay ist als beobachteter Projektbeweis dokumentiert (LOCAL FACT:
`docs/ROADMAP.md:353-359`).

Auch das heutige Recordformat ist bewusst klein: Ein strukturierter Record hat
höchstens 64 KiB Payload und 256 Datenframes (LOCAL FACT:
`raios-core/src/structured_store.rs:8-13`). Große Dateien müssen deshalb
chunked/content-addressiert bleiben; das Recordlimit einfach anzuheben wäre
keine Workspace-Architektur.

Der Ausbau kann die vorhandenen content-addressierten Blobs, Revisionen und
Transaktionen wiederverwenden. Neu nötig sind jedoch ein physischer,
owner-freigegebener Store-Backend, Multi-GiB-Geometrie, freie Bereiche,
Garbage Collection, projektbezogene Quotas, atomare Scratch-Promotion,
Verschlüsselung, Stromausfalltests und ein Recovery-reservierter Bereich, den
Buildjobs nie füllen dürfen. Das ist kein Grund, POSIX-Dateirechte zur
Autorität zu machen; der VFS aus Achse 4 kann über denselben strukturierten
Backend-Store gelegt werden.

## 7. Trust-Härtung: Pinning und Dev-Schlüssel sind nicht das Endfundament

| Punkt | Aussage |
|---|---|
| **Heute** | Der OpenAI-Pfad akzeptiert Leaf-/SPKI-Pinning, meldet aber ausdrücklich `pin_only_no_webpki_chain_validation` und `not_validated_stage0`. Die vorhandene Zeitquelle ist `cmos_rtc_unverified`, nicht vertrauenswürdig und validiert keine Zertifikatszeit. (LOCAL FACT: `seed-kernel/src/provider_trust.rs:89-101`; `raios-core/src/scoped_provider_trust_honesty.rs:14-16`; `raios-core/src/scoped_time_authority_honesty.rs:13-24`.) |
| **Heute, Schlüssel** | Das Honesty-Record setzt `owner_sealed=false`, `dev_key_not_owner_sealed`; es gibt nur einen RAM-ephemeren Owner-Key-Kandidaten. Persistenter Owner-Key, Owner-Seal und dauerhafte Autorität bleiben verweigert. (LOCAL FACT: `seed-kernel/src/agent_protocol_honesty.rs:35-62`; `seed-kernel/src/agent_protocol_honesty.rs:600-638`; `seed-kernel/src/owner_key.rs:145-171`.) |
| **Warum bewusst gewählt** | Pinning ist klein und fail-closed; Dev-Schlüssel erlauben die Mechanik zu beweisen, ohne fälschlich Hardwarebindung zu behaupten. Providerexport verlangt zusätzlich Klassifikation, Redaction, Budget und Audit. (LOCAL FACT: `raios-core/src/scoped_provider_trust_honesty.rs:74-104`; `docs/architecture-decisions/0004-system-memory-and-agent-context.md:328-345`.) |
| **Endziel verlangt** | Vollständige Ketten-/Hostname-Prüfung gegen kontrollierte Roots, vertrauenswürdige Zeit, sichere Rotation, owner-sealed Signier-/Store-Schlüssel und klassifizierte ADR-0004-Memory-Exporte. |
| **Einstufung** | **MAJOR-BUT-BOUNDED**. |
| **Ehrliches Risiko** | Ein game-scale Build kann tausende Inputs und lange Jobs haben; ein kompromittierter Bootstrap-, Registry- oder Zeitanker vergiftet dann reproduzierbar sehr viele korrekte Belege. |

Der Härtepfad ist klar, aber nicht klein:

1. Owner-Key erzeugen und an echte Plattform-/TPM-Evidenz binden; Seal/Unseal
   samt Recovery beweisen.
2. Root-Store und Intermediate-Ketten vollständig validieren; Hostname,
   Algorithmus und Rotation in dieselbe positive Trust-Entscheidung binden.
3. Vertrauenswürdige Zeit mit Ausfall-/Rollback-Semantik einführen.
4. Toolchain, Sysroot, Dependencies und AOT-Backend als klassifizierte,
   content-addressierte Memory-/Artefakt-Records führen.
5. Providerexport nur nach den sechs ADR-0004-Gates; lokale Lesbarkeit ist
   keine Exportberechtigung (LOCAL FACT:
   `docs/architecture-decisions/0004-system-memory-and-agent-context.md:328-345`).

Eine volle TLS-Kette macht den Compiler nicht vertrauenswürdig. Sie beweist
nur, von welchem Gegenüber Bytes kamen. Doppel-Build, lokale Hashes,
Capability-Grenzen und Owner-Freigabe bleiben trotzdem nötig.

Auch das Systemgedächtnis ist absichtlich klein: Der read-only Context-Plan
setzt standardmäßig 32 Events an; dauerhafte Memory-Writes sind pro Boot auf
128 Records und 32 KiB begrenzt (LOCAL FACT:
`raios-core/src/memory_context.rs:1-10`;
`seed-kernel/src/durable_store.rs:3478-3493`). Größere Context-Budgets sind
**UPGRADE-IN-PLACE**. Eine dauerhafte, große Projekt-Memory mit Kompaktion,
Quotas, Provenienz und Recovery ist **MAJOR-BUT-BOUNDED**. Das ehrliche Risiko:
Wird nur das Limit erhöht, wird „mehr Erinnerung“ zu unprüfbarer Promptmasse
oder kann den Recovery-Store verdrängen — genau das verbietet ADR 0004 (LOCAL
FACT: `docs/architecture-decisions/0004-system-memory-and-agent-context.md:11-40`;
`docs/architecture-decisions/0004-system-memory-and-agent-context.md:155-164`).

## 8. Adressraum: wasm32 endet pro Modul bei 4 GiB

| Punkt | Aussage |
|---|---|
| **Heute** | Dienste sind `wasm32-unknown-unknown`. wasmi 0.31 deaktiviert `memory64`; seine Memory-API nennt 65.536 Seiten beziehungsweise 4 GiB als wasm32-Maximum. Die realen raiOS-Gastlimits liegen heute weit darunter bei 2 beziehungsweise 4 MiB. (LOCAL FACT: `docs/architecture-decisions/0005-bare-metal-substrate-and-wasm-isolation.md:44-49`; `vendor/wasmi-0.31.2/src/engine/config.rs:403-423`; `vendor/wasmi-0.31.2/src/memory/mod.rs:69-76`; `seed-kernel/src/wasm_runtime/envelope.rs:6-9`; `raios-core/src/project_runtime.rs:11-15`.) |
| **Warum bewusst gewählt** | wasm32 ist heute die kleine, unterstützte und gut begrenzbare Zielmaschine; 32-Bit-Pointer halten ABI und Validierung einfach. |
| **Endziel verlangt** | Große Compiler oder Programme könnten mehr als 4 GiB adressierbaren Working-Set benötigen. Große Spiele/NLEs können Assets auch streamen oder auf mehrere Dienste verteilen; sie müssen nicht automatisch alles in einen linearen Speicher legen. |
| **Einstufung** | **MAJOR-BUT-BOUNDED**: wasm32 bleibt bestehen, memory64/wasm64 käme als neue Gastklasse und ABI-Version hinzu. |
| **Ehrliches Risiko** | Wird memory64 erst nach Festschreiben vieler 32-Bit-Handle- und Pointer-ABIs bedacht, wird die spätere Koexistenz teuer; wird es jetzt ohne Messung gebaut, ist es reine Vorratsarchitektur. |

**ASSUMPTION-TO-VERIFY:** Ein rustc-Hello-Build bleibt sicher unter 4 GiB
linearer Gastmemory. Der B3-Plan schätzt 512 MiB bis 2 GiB Working-Set, aber das
ist keine Messung und sagt nichts über große Projekte aus (ESTIMATE:
`docs/plans/b3-plan.md:103-110`).

**ASSUMPTION-TO-VERIFY:** Ein game-/NLE-scale Programm braucht pro Modul mehr
als 4 GiB. Das kann wahr sein, muss aber nicht: ein Asset-/Frame-Store hinter
Handles kann große Daten halten, ohne sie dauerhaft in die lineare Memory eines
Moduls zu kopieren. Der billigste richtige Schritt ist deshalb jetzt nur:
`memory.size`, Peak und Host-Handle-Bytes messen und die ABI-Felder so
versionieren, dass eine spätere 64-Bit-Klasse danebenpasst.

## 9. Grafik/GPU, Eingabe und Ton

| Punkt | Aussage |
|---|---|
| **Heute** | Ausgabe ist ein von Limine übergebener 32-Bit-Framebuffer mit CPU-gezeichnetem Backbuffer. Eingabe unterstützt USB-HID-Tastatur und Pointer; zusätzlich existiert ein PS/2-Tastatur-Fallback. Das deklarative UI-Programm ist auf 16 KiB und 64 Widgets begrenzt, ein Frame auf 16 KiB und 256 Zeichenbefehle. In der vollständigen Kernel-Modulliste gibt es kein GPU- oder Audio-/Sound-Treibermodul. (LOCAL FACT: `seed-kernel/src/framebuffer.rs:39-74`; `seed-kernel/src/usb.rs:432-457`; `seed-kernel/src/input.rs:32-43`; `raios-core/src/ui_program.rs:15-27`; `raios-core/src/ui_frame.rs:11-17`; `seed-kernel/src/main.rs:1-136`.) |
| **Warum bewusst gewählt** | Framebuffer und HID reichen für Genesis, physische Freigabe, Recovery und sichtbare Beweise. GPU, Audio und Komfort hätten den geschlossenen Selbstbau-Kreislauf nicht früher bewiesen. (LOCAL FACT: `docs/VISION_PLAN.md:7-17`; `docs/VISION_PLAN.md:161-167`.) |
| **Endziel verlangt** | Spiele/NLEs brauchen beschleunigte 2D/3D-/Compute-Ausführung, Display-Synchronisation, große GPU-Ressourcen, niedrige Eingabelatenz, Gamepads/weitere Geräte, Audio Ein-/Ausgabe, Medienuhren und wahrscheinlich Codec-/DMA-Pfade. |
| **Einstufung** | Insgesamt **HARD-RESEARCH / POSSIBLE-ONE-WAY-DOOR** wegen GPU/DMA und privilegiertem Treibercode. Weitere Eingabegeräte und ein begrenzter Audiotreiber wären jeweils **MAJOR-BUT-BOUNDED**. |
| **Ehrliches Risiko** | Ein GPU-Treiber ist nicht bloß „schneller zeichnen“: Er kontrolliert DMA, MMIO, Firmware, Interrupts und große geteilte Speicher und kann bei falscher Isolation den Wasm-Käfig umgehen. |

Die Vision ordnet GPU bewusst erst nach dem selbstumbaufähigen Kern mit
A/B-Slots ein; dann soll der Treiber *durch* die Schleife entstehen, nicht als
Werkstatt-Komfortprojekt (LOCAL FACT: `docs/VISION_PLAN.md:150-159`). Das ist
konsequent, bedeutet aber: Der Owner-Maßstab „Spiel/NLE läuft“ liegt hinter
einer privilegierten Treiberstufe, nicht nur hinter rustc.

Für diese Stufe braucht raiOS voraussichtlich:

- eine genaue Ziel-GPU statt allgemeiner Hardwareabdeckung;
- IOMMU-/DMA-Einschränkung und getrennte GPU-Adressräume;
- Kern- oder Hardware-abgesicherte Command-Submission statt beliebigem MMIO
  aus einem App-Gast;
- Ressourcen-/VRAM-Quotas, Fence-/Timeout-/Reset-Verhalten und Recovery;
- eine versionierte Grafik-/Compute-ABI, damit Spiele nicht den Treiber direkt
  besitzen;
- für NLE zusätzlich Audio-/Video-Zeitbasen und Frame-/Buffer-Handles;
- A/B-Kernel-/Treiber-Rollback, weil ein kaputter Displaytreiber die sichtbare
  Freigabeoberfläche zerstören kann.

**ASSUMPTION-TO-VERIFY:** Die Ziel-GPU kann mit einem kleinen, raiOS-spezifischen
Treiberumfang sinnvoll genutzt werden. „Nur eine Maschine“ reduziert die
Treiberbreite, nicht die inhärente Komplexität eines modernen GPU-Command- und
Memory-Managers.

## 10. Determinismus: der Bau muss reproduzierbar sein, nicht das Spiel

| Punkt | Aussage |
|---|---|
| **Heute** | Der B3A-Bau führt zwei frische Gastläufe aus, verlangt bytegleiche Outputs, vergleicht mit der Kernel-Neuberechnung und erlaubt beim erzeugten Modul null Imports und genau einen Entry Point. (LOCAL FACT: `seed-kernel/src/agent_protocol_build_assemble.rs:623-700`; `seed-kernel/src/agent_protocol_build_assemble.rs:747-765`.) |
| **Warum bewusst gewählt** | Der Compiler wird nicht als Autorität vertraut; sein Ergebnis wird reproduzierbar nachgerechnet und bleibt bis zur physischen Freigabe inert. (LOCAL FACT: `docs/FACTORY_PLAN.md:60-67`; `docs/VISION_PLAN.md:21-30`.) |
| **Endziel verlangt** | Reale Apps verwenden Zeit, RNG, Netz, Threads, GPU und nutzerabhängige Eingaben. Diese Laufzeit darf nondeterministisch sein. Der Build muss trotzdem aus eingefrorenen Inputs ein reproduzierbares Artefakt oder eine klar definierte reproduzierbare Normalform erzeugen. |
| **Einstufung** | **MAJOR-BUT-BOUNDED**. |
| **Ehrliches Risiko** | Wenn raiOS Byte-Identität versehentlich auch vom laufenden Spiel verlangt, wird das Endziel unbrauchbar; wenn es sie beim Build zu früh aufgibt, verliert die Fabrik ihren zentralen Sicherheitsbeleg. |

Die tragfähige Trennung lautet:

- **Build-Determinismus:** Source, Dependency-Manifest, Sysroot, Compiler,
  Flags, Environment, Dateiordnung, Seed und Zeitbasis sind eingefroren. Zwei
  Builds müssen dasselbe Artefakt liefern oder einen benannten
  Nondeterminismus-Fehler.
- **Test-Wiederholbarkeit:** Tests bekommen aufgezeichnete Eingaben, Seeds,
  virtuelle Zeit und Ressourcenbudgets. Sie dürfen mehrere gültige Ergebnisse
  besitzen, wenn die Toleranz ausdrücklich Teil des Tests ist.
- **App-Laufzeit:** Nutzerinput, Netzwerk, RNG, Threads und GPU dürfen echt
  nondeterministisch sein. Autorität bleibt durch Imports/Handles begrenzt;
  Nondeterminismus ist keine zusätzliche Berechtigung.

Wo es wirklich spannt:

- Parallel rustc/Cargo kann Datei- und Linkreihenfolgen verändern.
- `build.rs` und Proc-Makros können Zeit, Environment, Dateireihenfolge oder
  Zufall lesen.
- AOT kann Adressen, CPU-Features und Layout in den Output einmischen.
- GPU-Ergebnisse sind zwischen Treiber-/Hardwareständen nicht immer
  bitidentisch.

Diese Fälle brauchen nicht „alles deterministisch“, sondern eine klare
Build-Grenze: Nondeterministische Generatoren werden verweigert, ihre Inputs
virtualisiert oder ihre Outputs kanonisiert. Für GPU-Tests sind semantische
Toleranzen möglich; für das installierte Binärartefakt bleibt der Hash exakt.

## 11. Genesis-Job und Testfabrik: richtige Rohrleitung, noch kein allgemeiner Jobgraph

| Punkt | Aussage |
|---|---|
| **Heute** | B3A baut ein winziges Wasm im System, validiert es und führt nach einem Klick genau eine Funktion aus, die 42 zurückgibt. `rlang` ist absichtlich loop-frei, hat eine feste Funktion, höchstens 32 immutable Bindings und keine Calls, Imports, Memory, Strings, Rekursion oder I/O; es ist pausiert und nicht der kritische Toolchainpfad. Die aktuelle Buildsession hält höchstens acht Receipts im RAM. (LOCAL FACT: `docs/ROADMAP.md:229-248`; `docs/plans/b3a2-plan.md:3-19`; `docs/plans/b3a2-plan.md:69-74`; `docs/FACTORY_PLAN.md:69-80`; `seed-kernel/src/project_build.rs:20-25`.) |
| **Warum bewusst gewählt** | Diese Minimalform beweist die komplette Sicherheitsrohrleitung ohne vorgetäuschte Rust-/Cargo-Unterstützung. |
| **Endziel verlangt** | Ein Genesis-Jobgraph muss beschaffen, viele Buildschritte planen, mehrere Testarten ausführen, Logs/Artefakte bündeln, fehlerhafte Kinder an den Agenten zurückmelden, fortsetzen und schließlich W5/W6/Rollback benutzen. |
| **Einstufung** | **UPGRADE-IN-PLACE** für den Jobgraph selbst; seine Arbeiter hängen jedoch an den Major-/Hard-Grenzen 1 bis 6 und 9. |
| **Ehrliches Risiko** | Predicate-Zahlen oder ein Return-42-Test können leicht wie Fabrikfortschritt aussehen, während reale Build- und Testarten noch keinen ausführbaren Pfad besitzen. |

Die Jobsteuerung kann additiv wachsen: jeder Schritt bekommt eingefrorene
Inputs, abgeleitete Handles, Ressourcenbudgets, Status, Artefakthashes und einen
Abbruchpfad. Der bestehende W5-/W6-/Rollback-Weg bleibt das Ende der Kette. Was
nicht ehrlich wäre: Cargo-Ausgaben als Text zu simulieren, `build.rs` still zu
überspringen oder einen Host-PC-Build als on-device zu etikettieren. Die Vision
zählt nur den vollständigen QEMU-Kreislauf ohne Werkstatt als Basis-Abnahme
(LOCAL FACT: `docs/VISION_PLAN.md:38-47`; `docs/VISION_PLAN.md:120-137`).

## 12. Netz und Verteilung: bewiesene Rohre, noch keine allgemeine App-Autorität

| Punkt | Aussage |
|---|---|
| **Heute** | Das Host-ABI kennt vier TCP-Operationen. Der tatsächlich verdrahtete Gastpfad heißt jedoch `test.fixture.net_shims`; sein Grant-Probe setzt die produktive Policy auf `false`. Pro Fixture-Lauf gelten 4 KiB pro Call, 32 KiB Senden und 320 KiB Empfangen. Auch der Acquire-Linker ist ausdrücklich nur ein Testmodul; die lokale signierte Distribution Registry hält höchstens acht Einträge und vier Chunks. (LOCAL FACT: `raios-core/src/host_import_abi_v1.rs:62-79`; `seed-kernel/src/wasm_runtime/invocation.rs:313-332`; `seed-kernel/src/wasm_runtime/net_shims.rs:24-29`; `seed-kernel/src/wasm_runtime/net_shims.rs:267-313`; `seed-kernel/src/wasm_runtime/acquire_shims.rs:12-17`; `raios-core/src/distribution_registry.rs:20-26`.) |
| **Warum bewusst gewählt** | Netz ist Außenwelt-Autorität. Der erste Beweis brauchte Timeouts, Kill, Quotas und pin-/identitätsgebundene Beschaffung, nicht beliebige Sockets für beliebige Gäste. |
| **Endziel verlangt** | Die Fabrik braucht skalierbare, signierte und hash-gebundene Beschaffung für Toolchains, Sysroots und Dependencies. Der heutige Zielweg dafür ist geprüfte Quelle über W7, nicht Internetzugang des Compilers (LOCAL FACT: `docs/FACTORY_PLAN.md:41-47`). **ASSUMPTION-TO-VERIFY:** Einzelne spätere Spiele/NLEs brauchen zusätzlich produktive Socket-, Streaming- oder Kollaborations-Leases; das ist nicht für jede Ziel-App zwingend. |
| **Einstufung** | **MAJOR-BUT-BOUNDED** für produktive, ziel- und quota-gebundene Netz-Leases; **UPGRADE-IN-PLACE** für größere Registry-/Chunk-Klassen. |
| **Ehrliches Risiko** | Eine allgemeine Socket-Freigabe würde reproduzierbare Buildinputs, Secret-Grenzen und Exfiltrationsschutz gleichzeitig aufweichen; der Test-Fixture beweist diese Produktionspolitik ausdrücklich noch nicht. |

Der Fabrikweg sollte W7-artig bleiben: Der Job fordert einen benannten Inhalt
an, der Host prüft Quelle, Pin/Identität, erwarteten Hash, Länge und Quota und
liefert anschließend einen unveränderlichen Store-Handle. Das ist etwas
anderes als „der Compiler darf ins Internet“. Produktive App-Netzwerke können
später eine getrennte Lease-Klasse mit Zielmenge, Protokoll, Byte-/Zeitbudget,
Audit und Kill erhalten; sie dürfen nicht still durch die Build-Sandbox erben.

## Rangliste der größten realen Landminen

### 1. Dateioberfläche für rustc, Sysroot, Linker und große Projekte

**Warum Rang 1:** Der heute bewiesene 4-KiB-Kanal und 48-KiB-Workspace haben
fast keine semantische Überlappung mit einer Toolchain, die viele Dateien,
Metadaten, Scratch und atomare Outputs erwartet. Das ist kein Kapazitätsregler,
sondern ein neues Subsystem.

**Billigstes Experiment jetzt:** Auf der Werkstattseite je einen
`no_std`-Hello-Build und einen kleinen repräsentativen Rust-Build vollständig
tracen. Aufzeichnen: jeder Open/Stat/Read/Write/Rename, Pfad, Reihenfolge,
eindeutige Datei, gelesene/geschriebene Bytes und Peak-Scratch. Danach dieselben
Reads gegen ein eingefrorenes Manifest replayen. Ergebnis ist eine gemessene
Mindest-ABI; kein raiOS-VFS-Code ist dafür nötig.

**Abbruchkriterium:** Wenn schon Hello breite Symlink-, Locking-,
Memory-Mapping- oder ambient-directory-Semantik zwingend braucht, darf der
geplante „kleine Handle-Namensraum“ nicht weiter als nahezu fertig bezeichnet
werden.

### 2. Sandboxed Sub-Execution für Cargo, `build.rs` und Proc-Makros

**Warum Rang 2:** Ohne sie baut raiOS nur einen bewusst eingeschränkten Teil der
Rust-Welt. Mit einem unkontrollierten `spawn` würde es dagegen die eigene
Sicherheitsverfassung umgehen.

**Billigstes Experiment jetzt:** Einen Corpus aus repräsentativen
Game-/NLE-nahen Rust-Projekten per Cargo-Metadaten untersuchen: Anzahl der
Crates, Build-Skripte, Proc-Makros, nativen Toolaufrufe und dynamischen
Bibliotheken. Zusätzlich je ein minimales `build.rs`- und Proc-Makro-Beispiel
tracen und den exakten Eltern-/Kindvertrag notieren. Danach nur einen
Genesis-Subjob-Prototyp planen: festes signiertes Kind, abgeleitete Read-Handles,
ein Output-Handle, kein allgemeines Spawn.

**Abbruchkriterium:** Wenn der repräsentative Corpus überwiegend beliebige
Hostprozesse, dynamisches Laden und breite Environment-/Filesystem-Semantik
braucht, ist „Cargo-kompatibel“ eine eigene Langzeitplattform und kein B3-
Nachsatz.

### 3. Schnelle Ausführungsstufe ohne Verlust des Käfigs

**Warum Rang 3:** Der Interpreter ist für große Builds und Spiele voraussichtlich
zu langsam; ein naiver Wechsel auf nativen Code würde aber die stärkste
Sicherheitsgarantie entfernen.

**Billigstes Experiment jetzt:** Dasselbe feste Compute-Wasm und — sobald
ladbar — denselben rustc-Hello-Build hostseitig unter wasmi 0.31, einer separat
geprüften neueren wasmi-Version und einem AOT-Motor messen. Festhalten:
Load-/Übersetzungszeit, Peak-RAM, Laufzeit, Output, Imports und Abbruch. AOT
zweimal frisch erzeugen und die nativen Bytes vergleichen. Parallel eine
einseitige Threat-Liste schreiben: welche Speicher-/Syscall-Grenze hält den
Maschinencode wirklich fest?

**Abbruchkriterium:** Wenn die schnellste Variante nur als unisolierter nativer
Ring-0-Code sinnvoll schnell ist, ist sie keine Optimierung des heutigen Wegs,
sondern eine neue Ausführungsarchitektur mit ADR- und Re-Proof-Pflicht.

### 4. GPU-/Medien-Unterbau

**Warum Rang 4:** Ohne GPU und Audio können Agenten vielleicht große Programme
bauen, aber das Owner-Beispiel Spiel/NLE nicht real ausführen und testen. Die
GPU bringt zugleich DMA- und Recovery-Risiken in den privilegierten Unterbau.

**Billigstes Experiment jetzt:** Zuerst zwei getrennte Fakten gewinnen:

1. Read-only die exakte Ziel-GPU samt PCI-ID, BARs, Firmware- und IOMMU-
   Voraussetzungen inventarisieren und gegen den kleinsten realistischen
   Treiberpfad abgleichen.
2. In QEMU mit einer einfachen virtuellen GPU nur den künftigen Vertrag messen:
   ein begrenztes Buffer-Handle, eine Command-Submission, ein Fence, ein
   Timeout/Reset und ein sichtbarer Frame. Das beweist nicht die physische GPU,
   findet aber früh heraus, ob raiOS überhaupt ein tragfähiges GPU-Capability-
   und Recovery-Modell hat.

**Abbruchkriterium:** Ein Bildschirm-Clear ohne DMA-Isolation, Reset und
Recovery gilt nicht als GPU-Fundament und darf nicht zur Entwarnung benutzt
werden.

## Reversibel oder Fundamentwechsel?

### Genuin reversibel beziehungsweise parallel ergänzbar

- **4-KiB-Buffer-ABI:** behalten für kleine Gäste; große Gäste erhalten
  zusätzliche immutable Datei-/Stream-Handles.
- **262-KiB-Intake-Cap:** behalten für kleine Direktkandidaten; große
  Toolchains kommen chunked/content-addressiert aus dem Store.
- **2-/4-MiB-Gastklassen:** behalten; `Bauplatz` wird eine eigene, gemessene
  Klasse.
- **64-MiB-Heap und 512-MiB-QEMU-Profil:** als kleine Normalprofile behalten;
  große Forschung/Abnahme bekommt ein ausdrücklich benanntes Ressourcenprofil.
- **Assembler und rlang:** sie bleiben Beweiswerkzeug/Ersatzrad; rustc muss sie
  nicht ablösen oder rückwärtskompatibel erweitern.
- **wasm32:** bleibt Standard für kleine Dienste; memory64 kann später daneben
  existieren.
- **Pin-only TLS:** kann fail-closed als zusätzlicher enger Trustmodus bleiben,
  während volle Kette und Zeit hinzukommen.
- **Wegwerfbarer QEMU-Store:** bleibt Testbackend; ein physischer
  Multi-GiB-Backend kann dieselben strukturierten Records bedienen.
- **Framebuffer, USB HID und PS/2:** bleiben Recovery-Fallback, auch wenn GPU,
  weitere Eingabe und Audio hinzukommen.
- **Mehrere getrennte Ein-Kern-Gäste:** können auf einem künftigen SMP-
  Scheduler laufen, ohne Shared Memory einzuführen.
- **Kleine Registry-, Chunk- und Netzquotas:** können als kleine Klassen
  bestehen bleiben; größere signierte Beschaffung und App-Netz-Leases kommen
  getrennt hinzu.

### Zwingt zu einer Fundamententscheidung und erneuten Beweisen

- **Unisoliertes JIT/AOT/native:** Sobald Maschinencode außerhalb einer
  gleichwertigen Sandbox läuft, muss die Capability-Garantie neu begründet
  werden.
- **Shared-Memory-Threads im Gast:** Speicher-, Atomics-, Scheduling-, Kill-
  und Auditsemantik werden Teil der Sicherheitsgrenze.
- **Allgemeiner Prozess-Spawn oder dynamisches Build-Code-Laden:** Das ist eine
  neue Autoritätsdelegation, nicht nur eine Komfortfunktion.
- **Privilegierte GPU-/DMA-Treiber aus der Schleife:** IOMMU, Command-
  Validierung, A/B-Kern-/Treiber-Rollback und Recovery werden
  sicherheitskritisch.
- **Beliebiges POSIX-Dateisystem im Gast:** Nicht zwingend eine Einbahnstraße,
  aber ein vermeidbarer Fundamentwechsel. Der bounded Handle-/Namespace-Weg
  hält die bestehende Capability-Idee wesentlich besser intakt.

## Schlussfolgerung für die Reihenfolge

Die heutige Kleinheit ist überwiegend reversibel, **solange sie als Klassen und
enge ABIs behandelt wird**. Nicht reversibel wäre, aus Zeitdruck die schnellen
und kompatiblen Wege mit breiter Autorität zu bauen: nativer Ring-0-Code,
allgemeines Spawn, Shared Memory ohne Modell oder GPU-DMA ohne Isolation.

Die sachlich günstigste Reihenfolge ist deshalb:

1. threads-freies rustc-Wasm laden und Peak/Tempo messen;
2. aus echten Dateitraces das kleinste Bauplatz-Datei-ABI bestimmen;
3. einen no-dependency/no-build-script Rust-Hello-Bau im Bauplatz schließen;
4. repräsentativen Cargo-Corpus messen und erst dann Sub-Execution entwerfen;
5. Interpreter-Upgrade gegen AOT messen, bevor die Ausführungs-ADR fällt;
6. SMP zunächst für getrennte Gäste, Shared-Memory-Threads nur bei gemessenem
   Bedarf;
7. GPU/Audio erst durch den geschlossenen Loop, aber ihre Verträge und
   billigsten Read-only/QEMU-Experimente früh genug durchführen, dass die
   Landmine nicht erst nach Fertigstellung der Compilerfabrik sichtbar wird.

Damit bleibt die Vision unangetastet, aber die Reihenfolge ist ehrlich: Der
erste große Wall ist nicht „rustc irgendwie laden“, sondern rustc mit echten
Dateien, Kindern, Speicher, Tempo und derselben Beweisqualität betreiben.
