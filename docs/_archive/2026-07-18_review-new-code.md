# Unabhängiges Read-only-Review

Gesamturteil: Der neue Code hat eine gute Grundrichtung—reine `no_std`-Domänenlogik, typisierte Fehler und opt-in Engine-Erweiterungen. Für die Kernel-Integration ist der Schnitt aber noch nicht geschlossen. Besonders Ressourcenbegrenzung, FD-/Mount-Komposition und die Vertrauenskette des Reproduzierbarkeits-Gates müssen vor dem echten Build-Gast gehärtet werden.

## Priorisierte Befunde

### 1. Hoch: Zwei Strukturen können trotz Quoten unbegrenzt Kernel-Speicher wachsen lassen

Der Scheduler sammelt jeden Trace-Eintrag dauerhaft in einem unbegrenzten `Vec` ([thread_scheduler.rs:238](C:/Users/admin/Documents/raios2/crates/raios-core/src/thread_scheduler.rs:238)); jeder Wechsel erzeugt einen Eintrag ([thread_scheduler.rs:354](C:/Users/admin/Documents/raios2/crates/raios-core/src/thread_scheduler.rs:354)). Bei `10^13` Gesamt-Fuel und `10^6` Quantum sind allein Größenordnungen von zehn Millionen Switch-Ereignissen möglich ([build_guest_class.rs:109](C:/Users/admin/Documents/raios2/crates/raios-core/src/build_guest_class.rs:109)). Das ist kein fail-closed Ressourcenmodell.

Analog zählt `RamFs` nur lebende Dateien/Verzeichnisse, lässt gelöschte Nodes aber absichtlich als Tombstones stehen ([ramfs.rs:69](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/ramfs.rs:69)). Löschen leert den Slot ([ramfs.rs:591](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/ramfs.rs:591)), Neuanlage hängt dennoch immer am Vektorende an ([ramfs.rs:606](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/ramfs.rs:606)). Ein Gast kann deshalb mit wiederholtem create/unlink unterhalb der Live-File-Quote den Node-Vektor weiter wachsen lassen.

Vor Kernel-Freigabe braucht es:

- Für Traces: Zähler plus Rolling Hash und optional einen begrenzten Ringpuffer.
- Für RAMFS: eine harte Quote auf insgesamt vergebene Node-IDs/Mutationen oder sichere Slot-Wiederverwendung mit generationsbehafteten Handles.

### 2. Hoch: Das Reproduzierbarkeits-Gate vergleicht Behauptungen, nicht gebundene Inhalte

`WasiBuildOutputRun` enthält einen übergebenen Hash-String und Chunk-Metadaten ([wasi_build_output.rs:12](C:/Users/admin/Documents/raios2/crates/raios-core/src/wasi_build_output.rs:12)). Das Gate prüft lediglich, ob beide behaupteten Hashes und Chunk-Listen gleich sind ([scoped_wasi_artifact_egress.rs:56](C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_artifact_egress.rs:56)). Es rekonstruiert weder das Manifest noch verifiziert es die Chunk-Inhalte.

Die Kette vom korrekt aus `/out` berechneten `OutputManifest` zum Gate ist auch typseitig nicht unverfälschbar: Manifestfelder und gespeicherter Hash sind öffentlich ([output_manifest.rs:25](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/output_manifest.rs:25)), ebenso der angeblich nicht zum Laden berechtigende Plan samt `authorizes_load`-Boolean ([scoped_wasi_artifact_egress.rs:41](C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_artifact_egress.rs:41)).

Folge: Das Gate ist derzeit ein guter Gleichheitsprädikat, aber kein selbständiger Beweis zweier byteidentischer Builds. Es braucht einen nicht frei konstruierbaren `FrozenOutput`-Typ, dessen Hash und Chunks ausschließlich aus den tatsächlichen Bytes entstehen.

### 3. Hoch: Es fehlt eine gemeinsame FD-/Mount-Autorität

`ReadOnlyFs` und jedes `RamFs` besitzen eigene `FdTable`s und zusätzliche separate Offset-Tabellen ([readonly.rs:53](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/readonly.rs:53), [ramfs.rs:74](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/ramfs.rs:74)). `WritableRoot` enthält zwei weitere unabhängige RAM-Arenen ([writable.rs:39](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/writable.rs:39)); `/sysroot` und `/src` sind dort lediglich `ReadOnly`-Routen, aus denen nicht gelesen werden kann ([writable.rs:93](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/writable.rs:93)).

Damit müsste ausgerechnet der Kernel:

- kollidierende FD-Nummern der Backends übersetzen,
- Mount-Zugehörigkeit speichern,
- Offset-Autorität festlegen,
- Cross-Mount-Regeln koordinieren,
- Read-only und writable Pfade zu einem Preview1-Prozess zusammensetzen.

Zusätzlich modelliert `FdEntry` nur ein einziges Rights-Feld ([types.rs:155](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/types.rs:155)), während das gemessene `path_open` getrennte `fs_rights_base` und `fs_rights_inheriting` trägt ([wasi_preview1_import_abi.rs:176](C:/Users/admin/Documents/raios2/crates/raios-core/src/wasi_preview1_import_abi.rs:176), [wasi_preview1_import_abi.rs:208](C:/Users/admin/Documents/raios2/crates/raios-core/src/wasi_preview1_import_abi.rs:208)). Diese Reduktion ist für einen Capability-Schnitt zu grob.

Das sollte vor der Glue-Arbeit in einem `WasiBuildInstance` zusammengeführt werden: eine FD-Tabelle, Einträge mit Mount-ID/Node/Offset/Base- und Inheriting-Rights sowie explizite Backend-Routen.

### 4. Hoch: Grant und BuildGuestClass sind noch nicht zu einer Autorisierung verbunden

Der WASI-Grant bindet aktuell Compiler-, Job- und Import-Hash sowie die deklarierte Importliste ([scoped_wasi_build_grant.rs:57](C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_build_grant.rs:57)). Mount-Manifeste, Speicherbereiche, Quoten und der Hash der `BuildGuestClassV1` fehlen. Außerdem ist `AuthorizedWasiBuildGrant` über öffentliche Felder frei konstruierbar ([scoped_wasi_build_grant.rs:65](C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_build_grant.rs:65)).

`BuildGuestClassV1` ist bislang eine kanonisch hashbare Deklaration, keine erzwungene Laufzeitgrenze. Ihre Felder sind öffentlich und `validate()` ist optional ([build_guest_class.rs:20](C:/Users/admin/Documents/raios2/crates/raios-core/src/build_guest_class.rs:20)). Die Validierung prüft vor allem Nicht-Null-Werte; Beziehungen wie `fuel_quantum <= max_total_fuel` oder `max_file_bytes <= arena_bytes` fehlen.

Für den Kernel sollte nur ein opaquer, privat konstruierter Typ wie `AuthorizedBuildJob` akzeptierbar sein, der Grant, beobachtete Modulimporte, Mount-Hashes und eine validierte Guest Class gemeinsam bindet.

### 5. Mittel: Saubere Abhängigkeitsrichtung, aber brüchige Vertragsduplikation

Positiv ist, dass `raios-wasi-preview1` keine Kernel- oder wasmi-Abhängigkeit besitzt. Allerdings werden deshalb zentrale BuildFS-Konstanten dupliziert ([output_manifest.rs:8](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/output_manifest.rs:8)), und die Random-Limit-Kompatibilität wird sogar durch Parsen einer anderen Rust-Quelldatei getestet ([build_guest_class.rs:217](C:/Users/admin/Documents/raios2/crates/raios-core/src/build_guest_class.rs:217)).

`BuildFsManifestView` hat zehn Indexmethoden ([buildfs.rs:5](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/buildfs.rs:5)); wegen der Orphan-Regeln muss der Kernel einen lokalen Newtype-Adapter zwischen Core-Manifest und WASI-View schreiben. Das ist lösbar, aber unnötige Glue-Kopplung. Ein kleines gemeinsames Contracts-Crate oder ein bewusstes `raios-wasi-preview1 -> raios-core`-Dependency wäre robuster.

### 6. Positiv: Engine- und Policy-Grenzen sind überwiegend diszipliniert

Die wasmi-Erweiterungen sind opt-in und typisiert: `Suspension` unterscheidet Host, Atomic und Fuel ([resumable.rs:16](C:/Users/admin/Documents/raios2/vendor/wasmi-0.31.2/src/engine/resumable.rs:16)); falsche Resume-Werte werden vor Ausführung typgeprüft ([resumable.rs:157](C:/Users/admin/Documents/raios2/vendor/wasmi-0.31.2/src/engine/resumable.rs:157)). Nicht-resumierbare Atomic-Suspensions werden in einen definierten Trap übersetzt ([mod.rs:630](C:/Users/admin/Documents/raios2/vendor/wasmi-0.31.2/src/engine/mod.rs:630)). Fuel parkt vor der nicht bezahlten Instruktion ([executor.rs:1200](C:/Users/admin/Documents/raios2/vendor/wasmi-0.31.2/src/engine/executor.rs:1200)).

Auch der Scheduler ist trotz seiner Länge eine zusammenhängende Policy-Komponente mit typisierten Zuständen und Übergangsfehlern. Eine kleine Lücke bleibt: `park_wait` verlangt nur `Runnable`, nicht „ist aktueller Thread“ ([thread_scheduler.rs:381](C:/Users/admin/Documents/raios2/crates/raios-core/src/thread_scheduler.rs:381)), während `on_quantum_end` diese Kausalität korrekt prüft ([thread_scheduler.rs:364](C:/Users/admin/Documents/raios2/crates/raios-core/src/thread_scheduler.rs:364)). Der Integrator kann die Policy daher derzeit falsch bedienen, ohne sofort abgewiesen zu werden.

## Testtiefe: ehrliches Urteil

Im Quellstand existieren 32 wasmi-Conformance-, 45 WASI-, 14 Scheduler- und 16 BuildGuestClass-Tests. Die lokale Semantikabdeckung ist gut; die End-to-End-Claims sind dagegen zu weit.

| Behauptung | Tatsächlich belegt | Urteil |
|---|---|---|
| Fuel-Suspension | Exakter Fortsetzungspunkt, unverbrauchter fehlgeschlagener Charge, Resume-Typen und Default-Verhalten ([fuel_yield.rs:165](C:/Users/admin/Documents/raios2/crates/raios-wasmi-conformance/tests/fuel_yield.rs:165)) | Tief und glaubwürdig |
| Scheduler-Policy | Round-robin, FIFO notify, Timeout-Reihenfolge, Deadlock, proc_exit, illegale Übergänge | Gute Unit-Abdeckung |
| „Replay“ | Derselbe fest codierte API-Aufrufstrom wird zweimal ausgeführt und als `Debug`-String verglichen ([thread_scheduler.rs:1036](C:/Users/admin/Documents/raios2/crates/raios-core/src/thread_scheduler.rs:1036)) | Kein echter Replay; keine serialisierte Eingabespur oder Entscheidungsverifikation |
| Kernel-Determinismus | Ein fixes Zwei-Thread-WAT mit Wait, Notify und 32 Atomic-Increments läuft zweimal ([thread_job.rs:76](C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/thread_job.rs:76), [thread_job.rs:495](C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime/thread_job.rs:495)) | Wertvoller integrierter Fixture-Test, aber kein allgemeiner Scheduling-Beweis |
| Deterministisches `/out` | Sortierung, Chunk-Grenzen und wiederholtes Freeze desselben In-Memory-Zustands ([output_manifest.rs:174](C:/Users/admin/Documents/raios2/crates/raios-wasi-preview1/src/output_manifest.rs:174)) | Belegt kanonisches Freeze, keinen Doppel-Build |
| Reproduzierbarer Build | Zwei synthetisch übergebene Hashes/Chunklisten werden verglichen ([scoped_wasi_artifact_egress.rs:146](C:/Users/admin/Documents/raios2/crates/raios-core/src/scoped_wasi_artifact_egress.rs:146)) | Belegt den Comparator, nicht die Reproduzierbarkeit eines Builds |

Nicht bewiesen sind insbesondere: realer `rustc`-Doppellauf, `wasi.thread-spawn`, kombinierte WASI-/Scheduler-/PRNG-Reihenfolge, veränderte Host-Ereignis-Timings, persistierter Replay, sowie Bindung der verglichenen Hashes an tatsächliche Chunk-Bytes.

## Empfohlene Kernel-Integrationsfläche

Die Komposition sollte so verlaufen:

```text
beobachtetes Modul
  → opaquer AuthorizedBuildJob
  → WasiBuildInstance (eine FD-/Mount-/Prozesswelt)
  → checked GuestMemory decoder/encoder
  → wasmi Linker + BuildJobRunner
  → typisiertes JobOutcome / EgressCandidate
```

Konkret:

- `raios-wasi-preview1::WasiBuildInstance`: `ProcessContext`, gemeinsame FD-Tabelle, Mount-Tabelle, RAMFS/BuildFS-Backends und gesamtes Accounting.
- `seed-kernel/src/wasm_runtime/wasi_build/guest_memory.rs`: ausschließlich Pointer-/Längen-/Alignment-Prüfung; alle Zielbereiche vor der ersten Zustands- oder Gast-Speicheränderung validieren.
- `linker.rs`: alle 30 Imports in einer Tabelle binden, erst nach erfolgreichem Grant; `path_link`/`path_readlink` müssen explizit implementiert oder definiert fail-closed abgelehnt werden.
- `runner.rs`: Engine, Store, Shared Memory, Continuations, Scheduler und `ThreadHost` koordinieren; keine Agent-Protocol-, Eventlog- oder Serial-Details in WASI-Code.
- Ausgaben nur als nicht frei konstruierbarer `FrozenOutput`; erst die Core-Gate-Entscheidung darf daraus einen Egress-Kandidaten machen.
- Negative Integrationstests für Pointer-Wrap/OOB, partielle `args_get`-Writes, Extra-Import, Rights-Inheritance, Cross-Mount-FDs, Node-Churn, Trace-Budget und gefälschte Output-Hashes.

Der Kernel besitzt bereits eine brauchbare Untermodul-Grenze in [wasm_runtime.rs:47](C:/Users/admin/Documents/raios2/seed-kernel/src/wasm_runtime.rs:47). Die großen Legacy-Dateien wie [agent_protocol_memory.rs:8665](C:/Users/admin/Documents/raios2/seed-kernel/src/agent_protocol_memory.rs:8665) oder [event_log.rs:7010](C:/Users/admin/Documents/raios2/seed-kernel/src/event_log.rs:7010) müssen dafür nicht geöffnet oder umgebaut werden.

## Klare Empfehlung

**Weiterführen; keine vorgelagerte allgemeine seed-kernel-Refaktorierung.**

Die nächste Slice sollte allerdings mit der Konsolidierung der oben beschriebenen Integrationsgrenze beginnen. Die eigentlichen Blocker liegen in den neuen WASI-/Core-Verträgen—unbegrenzter Trace/Tombstone-Wuchs, fehlende gemeinsame FD-Welt und ungebundene Grant-/Egress-Typen—nicht in den alten 3.000–8.500-Zeilen-Dateien. Eine breite Kernel-Sanierung würde Risiko und Diff-Fläche vergrößern, ohne diese Probleme zu lösen.

Streng read-only: keine Dateien geändert und keine Cargo-Tests ausgeführt, da diese Build-Artefakte schreiben würden. Die abweichende WIP-Version von `thread_job.rs` wurde ausgeschlossen; bewertet wurde der jeweilige committed Git-Blob. Commit-Vorschlag: entfällt.