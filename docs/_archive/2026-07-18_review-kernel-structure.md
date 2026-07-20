## Architektururteil

**Klare Empfehlung: gezielter Mittelweg.** Vor dem WASI-Kernel-Glue sollte genau die betroffene `wasm_runtime`-Anschlusszone klein und verhaltensneutral entkoppelt werden. Danach kann die Integration weitergehen. Ein allgemeiner Umbau von `event_log`, `usb`, `durable_store` oder Rollback vorab wäre wegen der QEMU-only-Verifikation zu riskant; unverändertes Weiterbauen würde dagegen eine dritte parallele Instanziierungs-, Pointer- und Job-Lifecycle-Implementierung schaffen.

## 1. Beurteilung der großen Dateien

1. **`agent_protocol_memory.rs`: überwiegend gutartige Länge, kein klassisches God-Modul.**

   Der Kopf aggregiert allerdings viele Read-Modelle: Systemstatus, Provider, Services, Probleme und Durable Memory werden in `emit_memory_context` zusammengeführt ([agent_protocol_memory.rs:58](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:58)). Das ist breiter Read-Fan-out, aber ohne zentrale veränderliche Fachzustände; vorhanden sind lediglich Sequenzzähler ([agent_protocol_memory.rs:38](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:38)).

   Der weitaus größte Teil ist mechanische Protokollabbildung: generische Binding-Werte und Feldtabellen ([agent_protocol_memory.rs:594](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:594)), ein wiederverwendetes Makro ([agent_protocol_memory.rs:647](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:647)) und die exhaustive Abbildung von `EventBindings` ([agent_protocol_memory.rs:2178](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:2178)). Das ist ein Kandidat für Codegenerierung oder Aufteilung nach Binding-Familien, aber kein akuter Integrationsblocker.

2. **`event_log.rs`: klares God-Modul und struktureller Risikoknoten.**

   Es besitzt den globalen veränderlichen Logzustand ([event_log.rs:145](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:145)), Ringpuffer und separat verwaltete konsumierte Bindings ([event_log.rs:157](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:157)). Gleichzeitig enthält es fachliche Provider-Gates ([event_log.rs:241](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:241)), mutiert beim Konsumieren Zustand und schreibt dabei erneut Events ([event_log.rs:500](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:500)).

   Zusätzlich validiert es Module-Load-Nachweisketten ([event_log.rs:3068](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:3068)) und baut daraus das vollständige Load-Gate-Modell ([event_log.rs:4075](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:4075)). Die globale Fassade beginnt erneut mit Gate-Operationen und Hunderten `latest_*`-Zugriffen ([event_log.rs:6455](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:6455)); die Stichprobe ergab 50 Kerneldateien mit `event_log::`-Zugriffen.

   Nachbarn zeigen einen begonnenen, aber unvollständigen Schnitt: Typen liegen in `event_log_types.rs`, etwa `ModuleLoadGateBinding` ([event_log_types.rs:3018](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_types.rs:3018)) und `EventBindings` ([event_log_types.rs:3699](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_types.rs:3699)); reine Prüfungen teilweise in [event_log_module_checks.rs:10](/C:/Users/admin/Documents/raios2/seed-kernel/src/event_log_module_checks.rs:10). Im Zentrum bleiben aber Storage, Retention, Konsumpolicy, Providerpolicy, Modulpolicy und globale Query-API gekoppelt.

3. **`agent_protocol_module_load_gate_render.rs`: kohäsives, aber gefährlich überladenes Domänen-God-Modul.**

   Die Datei bleibt zwar in einer Domäne, ist aber keineswegs nur ein Renderer. Sie berechnet Status und Gründe ([agent_protocol_module_load_gate_render.rs:32](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_render.rs:32)), holt zahlreiche Zustände unmittelbar aus dem globalen Eventlog ([agent_protocol_module_load_gate_render.rs:535](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_render.rs:535)), baut das komplette Policy-Eingabemodell ([agent_protocol_module_load_gate_render.rs:4545](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_render.rs:4545)) und rendert es anschließend direkt auf das Protokoll ([agent_protocol_module_load_gate_render.rs:4670](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_render.rs:4670)). Danach folgt noch eine zweite kompakte Event-Binding-Serialisierung ([agent_protocol_module_load_gate_render.rs:4741](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_render.rs:4741)).

   Das ist hohe Fan-out-Kopplung zwischen State Acquisition, Policyprojektion und Ausgabe. Bemerkenswert ist, dass die Datei den Loader weiterhin als `module_loader_unimplemented` projiziert ([agent_protocol_module_load_gate_render.rs:4652](/C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_module_load_gate_render.rs:4652)). WASI-Glue sollte deshalb nicht über diesen alten `cap.module.load_ephemeral`-Pfad angebaut werden.

4. **`rollback_authority_gates.rs`: fachlich kohäsiv, aber ein gefährliches God-Modul.**

   Die Funktionen gehören sämtlich zur Rollback-Autorisierung. Problematisch ist die Schichtmischung: Abhängigkeiten werden durch `use super::*` verborgen ([rollback_authority_gates.rs:1](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_authority_gates.rs:1)); Policy- und Hashketten liegen neben echten PCI/AHCI-Schreib-/Readback-Operationen ([rollback_authority_gates.rs:3651](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_authority_gates.rs:3651), [rollback_authority_gates.rs:3675](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_authority_gates.rs:3675)). Retained State wird nach Eventlog-Schreiboperationen in einen globalen Mutex übernommen ([rollback_authority_gates.rs:4255](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/rollback_authority_gates.rs:4255)); der Mutex lebt im Nachbarmodul ([runtime.rs:109](/C:/Users/admin/Documents/raios2/seed-kernel/src/hello_service/runtime.rs:109)).

   Damit sind Policy, Evidence, globaler Zustand und physische I/O in einer Einheit. Hazardös, aber nicht Teil der WASI-Anschlusszone; ein Vorabumbau wäre nicht gerechtfertigt.

5. **`module_evidence.rs`: gutartige Länge, kein God-Modul.**

   Die Datei importiert nur SHA-256 ([module_evidence.rs:1](/C:/Users/admin/Documents/raios2/seed-kernel/src/module_evidence.rs:1)), definiert Dateninputs und anschließend deterministische kanonische Hashfunktionen, beispielsweise für Manifestreferenzen ([module_evidence.rs:542](/C:/Users/admin/Documents/raios2/seed-kernel/src/module_evidence.rs:542)) und Slotreservierungen ([module_evidence.rs:4302](/C:/Users/admin/Documents/raios2/seed-kernel/src/module_evidence.rs:4302)). Es gibt weder I/O noch Mutex, Hardwarezugriff oder `unsafe`.

   Das strukturelle Problem ist stattdessen doppelte Wahrheit: `raios-core` besitzt beispielsweise dieselbe Manifest-Kanonisierung ebenfalls ([module_load_gate.rs:1480](/C:/Users/admin/Documents/raios2/crates/raios-core/src/module_load_gate.rs:1480)). Diese Duplikation sollte später in die host-testbare Core-Bibliothek konsolidiert werden; sie macht die Kerneldatei aber nicht zum God-Modul.

6. **`usb.rs`: stärkstes echtes Runtime-God-Modul der Stichprobe.**

   Ein globaler `UsbState` besitzt Snapshot und gesamten Controller ([usb.rs:158](/C:/Users/admin/Documents/raios2/seed-kernel/src/usb.rs:158), [usb.rs:275](/C:/Users/admin/Documents/raios2/seed-kernel/src/usb.rs:275)). Dazu kommen zahlreiche globale mutable DMA-/Ringpuffer ([usb.rs:905](/C:/Users/admin/Documents/raios2/seed-kernel/src/usb.rs:905)) und ein sehr breiter `XhciController`, der Rings, HID, Hubs und Mass Storage gleichzeitig hält ([usb.rs:934](/C:/Users/admin/Documents/raios2/seed-kernel/src/usb.rs:934)).

   Das Modul umfasst Controllerinitialisierung, Enumeration, Hubs, HID, MSC/SCSI sowie GPT-/Seed-Layoutprüfung. Besonders schichtwidrig: Der Mass-Storage-Probe validiert Dateisystem-/Seed-Strukturen und löst unmittelbar einen dauerhaften Diagnose-Append aus ([usb.rs:3270](/C:/Users/admin/Documents/raios2/seed-kernel/src/usb.rs:3270)); die vollständige Reclog-Frame-Erzeugung und der Schreibzugriff liegen ebenfalls im USB-Treiber ([usb.rs:3396](/C:/Users/admin/Documents/raios2/seed-kernel/src/usb.rs:3396)). Äußerer Fan-in ist relativ klein, interner Verantwortungsumfang und Unsafe-Risiko sind jedoch sehr hoch.

7. **`durable_store.rs`: klares God-Modul.**

   Die gemeinsame Reclog-Mechanik ist eine legitime Kohäsionsachse. Die Datei enthält darüber hinaus jedoch Policy und Recordmodelle für normale Append-Aufrufe ([durable_store.rs:195](/C:/Users/admin/Documents/raios2/seed-kernel/src/durable_store.rs:195)), Install-Autorisierung inklusive physischem AHCI-Write/Readback ([durable_store.rs:1082](/C:/Users/admin/Documents/raios2/seed-kernel/src/durable_store.rs:1082)), Promotion, Recovery-Actions ([durable_store.rs:2312](/C:/Users/admin/Documents/raios2/seed-kernel/src/durable_store.rs:2312)), eingebaute Selftests und Durable Memory.

   Hardwareerkennung und Lesen des gesamten Reclogs liegen ebenfalls darin ([durable_store.rs:3293](/C:/Users/admin/Documents/raios2/seed-kernel/src/durable_store.rs:3293)). Schließlich besitzt es eine globale Memory-Write-Quota ([durable_store.rs:3509](/C:/Users/admin/Documents/raios2/seed-kernel/src/durable_store.rs:3509)) und verbindet diese mit Boot-Policy, Recordvalidierung, Planung und Media-I/O ([durable_store.rs:3624](/C:/Users/admin/Documents/raios2/seed-kernel/src/durable_store.rs:3624)). Neun andere Kernelbereiche greifen direkt auf `durable_store::` zu. Das ist sowohl breite Domänenkopplung als auch Policy+State+I/O-Mischung.

8. **`build.rs`: überwiegend gutartige Wiederholung, kein Runtime-God-Objekt.**

   `main` ist breit und führt Source-Attestation, Fixture-Kompilierung, Firmware-Einbettung sowie zahlreiche Wasm-Attestierungen zusammen ([build.rs:34](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:34), [build.rs:516](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:516)). Die Länge entsteht hauptsächlich aus ähnlich aufgebauten Artifact-Attestern, beispielsweise Echo ([build.rs:550](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:550)) und Build Assembler ([build.rs:2436](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:2436)). Fixture- und Firmware-I/O sind klar erkennbare Buildzeit-Aufgaben ([build.rs:527](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:527), [build.rs:534](/C:/Users/admin/Documents/raios2/seed-kernel/build.rs:534)).

   Das ist ein Wartungshotspot, aber kein Kernel-Laufzeitrisiko. Falls für rustc/WASI ein weiterer Attester nötig wird, sollte zuerst ein tabellengetriebener gemeinsamer Attestation-Helper entstehen, statt eine weitere große Kopie anzuhängen.

## 2. Wahrscheinlichste WASI/Threads-Anschlusszone

Der richtige Ort ist ein eigener Adapter unter `seed-kernel/src/wasm_runtime/`, nicht `event_log`, `durable_store` oder der alte Module-Load-Renderer. `wasm_runtime.rs` ist bereits die Fassade für Interpreter, Shims, Invocation und `thread_job` ([wasm_runtime.rs:47](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime.rs:47)). Im Kernel fehlt derzeit noch die Abhängigkeit auf `raios-wasi-preview1` ([Cargo.toml:11](/C:/Users/admin/Documents/raios2/seed-kernel/Cargo.toml:11)).

Die fachlichen Grenzen außerhalb des Kernels sind gesund:

- `raios-wasi-preview1` ist ausdrücklich I/O-, Kernel-, Memory- und Store-frei und erwartet spätere Adapter nach Pointervalidierung ([lib.rs:3](/C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/lib.rs:3)).
- Das Build-Grant ist typisiert und fail-closed: komplette Importliste oder nichts ([scoped_wasi_build_grant.rs:1](/C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_build_grant.rs:1), [scoped_wasi_build_grant.rs:107](/C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_build_grant.rs:107)).
- Der gemessene ABI-Vertrag enthält `proc_exit`, `wasi::thread-spawn` und exakt begrenzten Shared Memory ([wasi_preview1_import_abi.rs:243](/C:/Users/admin/Documents/raios2/crates/raios-core/src/wasi_preview1_import_abi.rs:243)).
- Thread-, Fuel- und FS-Grenzen liegen bereits im typisierten Guest-Class-Vertrag ([build_guest_class.rs:99](/C:/Users/admin/Documents/raios2/crates/raios-core/src/build_guest_class.rs:99)).

Im Kernel ist die Zone dagegen **gelb, nicht rot**:

- Positiv: Der bestehende Envelope-Pfad autorisiert Imports vor der Instanziierung, definiert nur gewährte Hostfunktionen, prüft anschließend die tatsächlichen Modulimports und ruft erst dann `instantiate` auf ([envelope.rs:313](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/envelope.rs:313), [envelope.rs:377](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/envelope.rs:377), [envelope.rs:394](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/envelope.rs:394)).
- Hazard: Personal Shell besitzt eine eigene parallele Grant-/Linker-/Instantiation-Pipeline ([personal_shell.rs:295](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/personal_shell.rs:295), [personal_shell.rs:357](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/personal_shell.rs:357)).
- Hazard: Pointer- und Memory-Export-Prüfungen sind bereits mehrfach lokal implementiert, etwa in Envelope ([envelope.rs:807](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/envelope.rs:807), [envelope.rs:861](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/envelope.rs:861)) und Personal Shell ([personal_shell.rs:766](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/personal_shell.rs:766)). Dreißig WASI-Imports würden diese Streuung massiv verstärken.
- Hazard: Die bestehende langlebige Invocation wird nur an einer Input-Tick-Grenze gepumpt ([main.rs:419](/C:/Users/admin/Documents/raios2/seed-kernel/src/main.rs:419)) und blockiert währenddessen Provider-Polling ([main.rs:470](/C:/Users/admin/Documents/raios2/seed-kernel/src/main.rs:470)). Zusätzlich existiert eine globale Wasm-Ausführungssperre ([invocation.rs:229](/C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/invocation.rs:229)). Ein Buildjob darf nicht unbesehen an diesen UI-getriebenen Lebenszyklus gehängt werden.

Die saubere Zielstruktur wäre daher ein dediziertes `wasm_runtime::wasi_build_job` mit per-`Store` gehaltenem WASI-/Threadzustand, ergänzt um einen gemeinsamen `guest_memory`-Adapter und einen expliziten, periodisch gepumpten Job-Lifecycle. Grant, exakte Importsignaturen und Shared-Memory-Limits müssen vollständig geprüft sein, bevor irgendeine Instanziierung erfolgt.

## 3. Entscheidung und notwendige Vorarbeit

**Gezielter Mittelweg: eine kleine Anschlussstellen-Refaktorierung vor dem Kernel-Glue, dann Integration fortsetzen.**

Diese Vorarbeit sollte ausschließlich umfassen:

- zentrale, host-testbare Range-/Overflow-/Iovec-Prüfung plus dünnen `wasmi::Memory`-Adapter;
- einen wiederverwendbaren Job-Lifecycle aus der bisherigen Thread-Selftest-Pumpe, ohne UI-Input als Taktgeber;
- einen einzigen typisierten Link-/Importplan für Grant, Implementierungsverfügbarkeit und tatsächliche Modulimports;
- einen eigenen WASI-Build-Adapter, der keine direkten Abhängigkeiten auf `event_log`, `durable_store`, `usb` oder den alten Module-Load-Renderer erhält.

Als Abnahmegrenzen sollten mindestens gelten: bestehender Threads-QEMU-Selftest unverändert grün, Import außerhalb des gemessenen Vertrags instanziiert nie, OOB-/Overflow-Pointer verändert weder Guest- noch Kernelzustand, Thread 49 wird abgewiesen und `proc_exit` beendet den gesamten Job deterministisch.

Ein großer Vorabumbau der identifizierten God-Module wäre mangels Hosttests unverhältnismäßig regressionsgefährlich. Ganz ohne Anschlussstellenarbeit weiterzumachen würde dagegen die aktuell noch beherrschbare `wasm_runtime`-Dopplung in eine dauerhafte Architektur verwandeln.

Prüfstatus: ausschließlich statische Read-only-Analyse; keine Builds oder QEMU-Läufe. Während der Analyse erschienen fremde Änderungen an `thread_job.wat` und `wasm_runtime/thread_job.rs`; sie wurden nicht verändert und nicht als stabiler Architekturstand bewertet. Commit-Vorschlag entfällt, da keinerlei Dateien geändert wurden.