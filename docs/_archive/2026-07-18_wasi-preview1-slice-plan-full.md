# WASI-Plan — Slice-Plan für das rustc-WASIp1-Subset

## Ausgangsbefund

- Das statische Import-Subset ist noch nicht belastbar vermessen. Das Laufwerk `E:` ist in dieser Lane nicht eingebunden; deshalb nennt dieser Plan bewusst keine geratene Funktionsliste. Slice 0 erzeugt die verbindliche, typisierte Inventarliste für den gepinnten SHA-256 `c6dccf…a5791`.
- [`project_workspace.rs`](/C:/Users/admin/Documents/raios2/crates/raios-core/src/project_workspace.rs:5) ist inhaltsadressiert, aber auf 32 KiB pro Datei und 48 KiB pro Revision begrenzt. Das reicht für erste Quellen, nicht für den 71-MB-Sysroot.
- [`artifact_blob_frame.rs`](/C:/Users/admin/Documents/raios2/crates/raios-core/src/artifact_blob_frame.rs:1) kann große CAS-Blobs ablegen. Der aktuelle Readback in [`artifact_store.rs`](/C:/Users/admin/Documents/raios2/seed-kernel/src/artifact_store.rs:1086) lädt und prüft jedoch einen ganzen Frame; für Sysroot-Zugriffe braucht es gehashte Teilstücke statt eines 71-MB-Voll-Reads.
- Der bestehende Grant-Pfad kennt maximal 16 geordnete Import-Paare und hasht keine Signaturen. [`host_import_abi_v1.rs`](/C:/Users/admin/Documents/raios2/crates/raios-core/src/host_import_abi_v1.rs:14) kann außerdem nur genau einen Rückgabewert ausdrücken, nicht `proc_exit` ohne Rückgabe. WASI braucht daher eine neue, typisierte und evidence-bound Familie; `KNOWN_HOST_IMPORTS` bleibt unverändert.
- Die Doppel-Bau-Grenze existiert bereits in [`project_build.rs`](/C:/Users/admin/Documents/raios2/crates/raios-core/src/project_build.rs:208). Der dortige Vertrag beschreibt aber Cargo und einen Werkstatt-Compiler und sollte nicht stillschweigend für den einzelnen On-Device-`rustc`-Aufruf umgedeutet werden.

Shim-Kern sind die Slices 1–6; Slice 0 ist die vorgeschaltete Messung.

## Slice 0 — vollständiges Import-Inventar

**Ziel:** Ein kleines Host-Tool liest das echte `rustc_opt.wasm` und erzeugt eine kanonische Inventardatei. Erst deren Ergebnis friert den Allowlist-Vertrag der folgenden Slices ein.

**Files/Crate-Schnitt:**

- `Cargo.toml`: Workspace-Member `tools/wasm-import-inventory`
- `tools/wasm-import-inventory/Cargo.toml`
- `tools/wasm-import-inventory/src/lib.rs`
- `tools/wasm-import-inventory/src/main.rs`
- `docs/architecture/rustc-wasm-import-inventory-c6dccf3e.json`: generierte Evidenz

Das Tool verwendet `wasmparser-nostd = "=0.100.1"`; der Workspace-Patch bindet damit den vorhandenen Vendor-Baum. Pro Import werden Binärreihenfolge, Modul, Name, Import-Kind, Typindex und aufgelöste Parameter-/Result-Typen ausgegeben; Duplikate und Nicht-Funktionsimporte werden nicht ausgefiltert. Kopf der kanonischen Ausgabe: Dateilänge, SHA-256, Importanzahl und Hash über die vollständigen typisierten Importdeklarationen.

**Host-Predicates:**

- Eine Fixture mit mehreren Typen und doppeltem Import bleibt vollständig und in Binärreihenfolge erhalten.
- Wiederholte Läufe ergeben byte-identisches JSON.
- Der Lauf auf dem echten Artefakt bestätigt exakt den erwarteten SHA-256 und führt alle Module einschließlich `wasi_snapshot_preview1` und der gemessenen Threads-Oberfläche auf.
- Jeder Funktions-Typindex wird zu einer konkreten Signatur aufgelöst; kein Eintrag bleibt bei `Debug`-Text oder „unknown“.

**Negativtest:** Trunkiertes/malformes Wasm, falscher erwarteter SHA oder ein nicht auflösbarer Typindex führt zu Exitcode ≠ 0 und keiner als erfolgreich markierten Inventardatei.

**Größe:** S  
**Abhängigkeiten:** Zugriff auf das Artefakt unter `E:\raios-probe-rustc-wasm\`; in der aktuellen Lane fehlt dieses Laufwerk.

## Slice 1 — getypte WASI-Build-Familie und Grant-Vertrag

**Ziel:** Das gemessene Importinventar wird als eigener Build-Service-Vertrag eingefroren. Gleichzeitig entsteht der Ressourcen-Grant, der Compiler, Sysroot, Quellen, RAM-Quoten und Output-Bereich an genau ein Job-Manifest bindet.

**Files/Crate-Schnitt:**

- `crates/raios-core/src/wasi_preview1_import_abi.rs`
- `crates/raios-core/src/scoped_wasi_build_grant.rs`
- `crates/raios-core/src/lib.rs`
- Testfixture mit Inventar-Hash aus Slice 0

Neue ABI-ID, etwa `raios.wasi_build_imports.v1`. Eine Importdeklaration enthält `module`, `name`, `kind`, `params` und `results`; damit sind auch `proc_exit`, eine eventuell importierte Shared Memory und `wasi.thread-spawn` korrekt beschreibbar. Der Grant bindet zusätzlich Compiler-SHA, Job-Manifest-SHA, Mount-Manifest-SHAs, erlaubte Blob-/Chunk-Ranges, Rechte, tmp/out-Quoten sowie die vollständige Linker-Implementierungsliste.

**Host-Predicates:**

- Nur gepinntes Compilerartefakt + exakt gemessene typisierte Imports + identische Linker-Liste + gültige Range-Grants werden autorisiert.
- ABI-Hash und Entscheidung sind unabhängig von HashMap- oder Eingabereihenfolge byte-stabil.
- Bestehende `raios.host_imports.v1`-Tests bleiben unverändert grün; WASI erscheint nicht in `KNOWN_HOST_IMPORTS`.
- `wasi.thread-spawn` wird als Schnittstelle deklariert, aber noch nicht als Scheduler implementiert.

**Negativtest:** Ein zusätzlicher Import, falsche Signatur, vertauschte Importreihenfolge, falscher Compiler-SHA, Range-Overflow oder fehlender Output-Grant verweigert den gesamten Job vor Instanziierung; es wird kein Teil-Grant ausgegeben.

**Größe:** M  
**Abhängigkeiten:** Slice 0.

## Slice 2 — `raios-wasi-preview1`: Pfadauflösung und FD-Tabelle

**Ziel:** Reine, kernelunabhängige Preview1-Kernlogik für Pfade, Rights, Dateideskriptoren und ABI-Datentypen.

**Files/Crate-Schnitt:**

- `Cargo.toml`: neuer Workspace-Member
- `crates/raios-wasi-preview1/Cargo.toml`
- `crates/raios-wasi-preview1/src/{lib,types,errno,path,fd_table}.rs`

Die Crate bleibt ohne externe Dependencies und außerhalb der Tests `no_std`; `alloc` ist zulässig. Raw-Wasm-Pointer bleiben außerhalb: Die API arbeitet mit geprüften Slices, Handles und Requests, sodass Pointerprüfung und Fachlogik getrennt testbar sind.

**Host-Predicates:**

- FD-Belegung ist fest: `0/1/2` für Standardströme, `3` als Root-Preopen, dynamische FDs ab `4` immer nach der kleinsten freien Nummer.
- `fd_prestat_get`/`fd_prestat_dir_name` veröffentlichen ausschließlich `/`.
- Pfadnormalisierung behandelt `.`, wiederholte Trenner und relative Basen deterministisch; ein Aufstieg über die Capability-Wurzel ist unmöglich.
- Angeforderte Rights werden stets mit Mount- und FD-Rights geschnitten; Rights können durch `path_open` nie wachsen.
- FD-Limit, Cookie- und Seek-Arithmetik haben geprüfte Overflow-Grenzen.

**Negativtest:** `../../escape`, ein unbekannter FD, Rights-Eskalation und FD-Erschöpfung liefern feste Preview1-Errnos, ohne FD oder Backend-Zugriff anzulegen.

**Größe:** M  
**Abhängigkeiten:** Slice 1 für ABI/Errno-Vertrag; unabhängig von T1/T2.

## Slice 3 — schreibgeschützte `/sysroot`- und `/src`-Sicht

**Ziel:** Inhaltsadressierte, range-gelesene Build-Eingänge ohne Pfad-Dateisystem oder ambienten Store-Zugriff.

**Files/Crate-Schnitt:**

- `crates/raios-wasi-preview1/src/{buildfs,readonly,dir}.rs`
- `crates/raios-core/src/buildfs_manifest.rs`
- `crates/raios-core/src/lib.rs`

`BuildFsManifest v1` enthält kanonisch sortierte Verzeichnisse und Dateien. Jede Datei besitzt Gesamtlänge/-hash und geordnete, vorzugsweise 64-KiB-CAS-Chunks; die Shim-Crate sieht nur validierte `ReadGrant { blob, offset, len }`-Handles. Quellen können aus verifizierten Workspace-Blobs adaptiert werden, der Sysroot aus einem eigenen großen BuildFS-Paket.

**Host-Predicates:**

- Gleiche Manifestbytes ergeben gleiche Pfade, Inodes, Filestats, Directory-Cookies und Lesebytes.
- Verzeichnisreihenfolge ist strikt nach kanonischen UTF-8-Pfadbytes, nicht Backend-/Einfügereihenfolge.
- `path_open`, Read/Pread, Seek/Tell, Filestat, Readdir und die exakt in Slice 0 gemessenen Read-Ops funktionieren über einen Mock-Range-Reader.
- Ein 71-MB-Testmanifest kann kleine Bereiche lesen, ohne das Paket vollständig zu materialisieren.
- Eingangszeiten und Metadaten sind feste ABI-Werte; Host-Dateizeiten werden nie sichtbar.

**Negativtest:** Schreiben, Erzeugen, Umbenennen oder Löschen unter `/sysroot` bzw. `/src` liefert `ERRNO_ROFS`. Ungegrantete Chunks, Hashfehler und Range-Überläufe führen fail-closed zu `ERRNO_IO` beziehungsweise `ERRNO_NOTCAPABLE`, ohne Ausweich-Lookup im globalen Store.

**Größe:** L  
**Abhängigkeiten:** Slice 2; Sysroot-Packer/Import darf später folgen, muss aber exakt dieses Manifest erzeugen.

## Slice 4 — RAM-tmp, Root-Tempanlage, `/out` und Output-Freeze

**Ziel:** Eine vollständig flüchtige Schreibsicht für rustc einschließlich Temp-Anlage direkt unter `/`; einzig `/out` kann nach erfolgreichem Doppel-Bau zu einem inerten Artefakt werden.

**Files/Crate-Schnitt:**

- `crates/raios-wasi-preview1/src/{ramfs,writable,output_manifest}.rs`
- `crates/raios-core/src/wasi_build_output.rs`
- `crates/raios-core/src/scoped_wasi_artifact_egress.rs`
- `crates/raios-core/src/lib.rs`

Die Root-Ansicht ist ein Composite: reservierte Mounts plus neue flüchtige Root-Kinder. Ein `path_create_directory` für beispielsweise `/rustcXXXX` legt einen RAM-tmp-Knoten an; `/tmp` nutzt denselben Arena-Typ, `/out` eine getrennte Arena. Der WASI-Aufruf schreibt niemals direkt auf ARTSTOR.

**Host-Predicates:**

- Dateien und Verzeichnisse können unter `/tmp`, `/out` und als neue Kinder direkt unter `/` erstellt, zufällig beschrieben, gesucht, umbenannt und gelöscht werden.
- Reservierte Namen `/sysroot`, `/src`, `/out`, `/tmp` können weder überschrieben noch verschattet werden.
- Speicher-, Datei-, Directory- und Gesamtquoten greifen vor Mutation; fehlgeschlagene Aufrufe hinterlassen Zustand und FD-Tabelle unverändert.
- Ein erfolgreicher Run friert `/out` als sortiertes Manifest mit Pfad, Länge und Chunk-Hashes ein; tmp und Root-Scratch fehlen garantiert.
- Nur zwei byte-identische Output-Manifeste erzeugen einen Egress-Plan. Die Run-Nummer ist kein Teil von Pfaden, PRNG-Seed oder Manifest.

**Negativtest:** Bereits ein abweichendes Outputbyte, ein zusätzliches Outputfile, Quotenüberschreitung, Nonzero-Exit oder Trap ergibt `build_outputs_not_reproducible` beziehungsweise keinen Egress-Plan; kein Persist-Callback wird aufgerufen.

**Größe:** L  
**Abhängigkeiten:** Slices 2–3 und bestehender Doppel-Bau-Vergleich; tatsächliche ARTSTOR-Schreibanbindung erst Slice 6.

## Slice 5 — args/env, deterministische Uhr, Random und `proc_exit`

**Ziel:** Alle nicht-dateibezogenen, durch Slice 0 bestätigten Preview1-Imports erhalten eine vollständig manifestgebundene Antwort.

**Files/Crate-Schnitt:**

- `crates/raios-wasi-preview1/src/{process,determinism,stdio}.rs`
- bei gemessenem Bedarf zusätzlich `poll.rs`

Args und Env sind geordnete Bytefolgen aus dem kanonischen Job-Manifest; es gibt keinen Zugriff auf Kernel-/Host-Environment. `proc_exit` wird als `HostEffect::Exit(code)` zurückgegeben, nicht als Host-Prozessabbruch. `poll_oneoff`, `sched_yield` oder `clock_res_get` werden nur aufgenommen, falls Slice 0 sie tatsächlich findet.

**Host-Predicates:**

- Zwei Kontexte aus demselben Job-Manifest und derselben Fuel-/Aufrufspur liefern identische args/env-, Clock-, Random-, stdout- und stderr-Bytes.
- `monotonic_ns = total_job_fuel_used`; `realtime_ns = 946684800000000000 + monotonic_ns` und readonly-Filestat-Zeiten verwenden denselben festen Epoch-Vertrag.
- Random-Seed ist `SHA256("raios.wasi.random.v1" || canonical_job_manifest_sha256)`; beide Doppel-Bauten starten denselben spezifizierten xoshiro256**-Stream mit fester Little-Endian-Ausgabe.
- Andere Input-Manifeste erzeugen andere Random-Streams. Audit-/Run-ID und reale Zeit sind ausdrücklich nicht Teil des Seeds.
- `proc_exit(code)` bewahrt den exakten Code und verhindert weitere normale Guest-Calls.

**Negativtest:** Unbekannte Clock-ID, Pointer-/Längen-Overflow, übergroßer Random-Request oder Zugriff auf einen nicht deklarierten Env-Wert liefert das festgelegte Errno ohne Teilantwort und ohne Fortschalten des PRNG-Zustands.

**Größe:** M  
**Abhängigkeiten:** Slices 1–2; T2 liefert später den jobweiten Fuel-Zähler und die processweite Exit-Wirkung.

## Slice 6 — dünnes Kernel-Glue, Store-Adapter und T2-Handoff

**Ziel:** Die getestete Crate wird exakt hinter dem Grant-Gate an wasmi gebunden; Kernelcode übersetzt nur Caller-Memory, Capability-Handles und HostEffects.

**Files/Crate-Schnitt:**

- `seed-kernel/Cargo.toml`
- `seed-kernel/src/wasm_runtime/mod.rs`
- `seed-kernel/src/wasm_runtime/wasi_preview1.rs`
- `seed-kernel/src/wasm_runtime/wasi_build_storage.rs`
- schmale Ergänzungen in `seed-kernel/src/artifact_store.rs` und `seed-kernel/src/project_workspace.rs`
- T2-eigene Scheduler-Dateien sind ausdrücklich nicht Bestandteil dieses Pakets

`wasi_preview1.rs` registriert nur die in Slice 1 autorisierte, typisierte Linker-Oberfläche. Ein injizierter `ThreadHost` stellt `spawn(start_arg) -> i32` bereit; TID-Vergabe, Round-Robin, Wait/Notify, Threaddeckel und jobweiter Exit bleiben vollständig T2. Blob-Adapter erhalten ausschließlich vorgeprüfte Chunk-Handles und Ranges, niemals freie Store-Offsets oder globale Lookup-Rechte.

**Host-Predicates:**

- Mock-Guest-Memory beweist für jeden gemessenen Import Bounds-, Overflow- und Aliasprüfung, bevor ein Backend aufgerufen wird.
- Das Importinventar, der Grant und die tatsächlich registrierten Linker-Definitionen sind exakt gleich; zusätzliche oder fehlende Definitionen verhindern Instanziierung.
- Read-Adapter prüfen Chunk-Frame und Payload-Hash; sie können nur die im Job-Grant enthaltenen Chunks lesen.
- Nach Doppel-Bau-Gleichheit schreibt der Egress-Adapter ausschließlich das kanonische Output-Bundle in den konkret gewährten ARTSTOR-Span und erzeugt eine `authorizes_load=false`-Referenz.
- Nonzero-Exit, Trap, Grantfehler oder Store-Readbackfehler erzeugt keinen persistenten Output.

**Negativtest:** Ein Wasm mit einem zusätzlichen Preview1-Import, falscher Signatur, Out-of-bounds-Gastspeicher oder ARTSTOR-Range außerhalb des Grants wird vor Wirkung abgewiesen. Besonders: kein Fallback-Linker, kein ambienter Store-Lookup und kein Teil-Persist.

**Größe:** L  
**Abhängigkeiten:** Slices 1–5, T1 für Shared Memory/Atomics, T2 für den `ThreadHost`, Bauplatz-Heap und importierte Compiler-/Sysroot-Artefakte. Codex-Worker können `seed-kernel` nicht bauen; deshalb müssen alle Fachpredicates vorher in Host-Crates grün sein, während `cargo check -p seed-kernel` und QEMU-Rustc-Smoke einem buildfähigen Integrations-Runner gehören.

## Design-Empfehlungen

**Virtuelles Layout.** `/sysroot` und `/src` sind immutable CAS-Mounts, `/out` ist eine getrennte RAM-Arena, `/tmp` eine wegwerfbare RAM-Arena. `/` ist ein Composite-Preopen: reservierte Mounts sind unveränderlich, neu erzeugte Root-Kinder werden automatisch als tmp klassifiziert, damit rustcs beobachtete Temp-Anlage direkt unter `/` funktioniert. Nichts außer dem nach Doppel-Bau eingefrorenen `/out` kann den Bauplatz verlassen.

**FD-Tabellen-Modell.** `0/1/2` sind deterministisch stdin/stdout/stderr, `3` ist der einzige Preopen `/`, ab `4` wird stets der kleinste freie FD vergeben. Jeder Eintrag trägt Node-ID, Offset, Rights und FD-Flags; Rights sind die Schnittmenge aus Mount, Eltern-FD und angefragten Rights. Directory-Cookies sind Indizes in einer kanonisch sortierten Snapshot-Liste, nicht Backend-Cursor.

**Determinismus.** Die logische Zeit stammt aus dem jobweiten Fuel-Zähler; Realtime ist eine feste Epoch plus derselben logischen Zeit. Random ist ein spezifizierter PRNG-Stream aus dem kanonischen Job-Manifest-Hash, während Verzeichnisordnung, Inodes, FD-Vergabe, tmp-Node-IDs und Metadaten ausschließlich aus kanonischer Reihenfolge beziehungsweise deterministischer Mutationsfolge entstehen. Der gesamte Errno-Vertrag wird als numerische Preview1-Goldentabelle getestet; Hostfehlertexte oder Gerätetiming dürfen nie die Guest-Antwort verändern.

**Crate-Schnitt.** Die neue Crate heißt `raios-wasi-preview1`, ist dependency-frei und außerhalb von Tests `no_std`. Sie nimmt validierte Manifeste, geprüfte Guest-Slices und schmale Traits wie `ReadAtGrant`, `RamPages` und `ThreadHost`; sie kennt weder AHCI noch Structured Store noch wasmi. Kernel-Glue beschränkt sich damit auf Pointerprüfung, Trait-Adapter und `func_wrap`.

**Grant-Gate.** Empfehlung ist eine neue Familie `raios.wasi_build_imports.v1`, keine Erweiterung von `raios.host_imports.v1`. Sie bindet die vollständigen typisierten Imports an das gepinnte Compilerartefakt und denselben Job-Manifest-Hash wie alle Datei-/Range-Grants; eine deklarierte Funktion ohne konkrete Linker-Implementierung ist nicht grantbar. `wasi.thread-spawn` gehört in diesen Vertrag, seine Wirkung aber ausschließlich in T2.

**Fehlerphilosophie.** Unbekannte Imports oder Signaturen scheitern vor Instanziierung, nicht mit einem Laufzeit-`ENOSYS`. Ungültiger FD → `BADF`, Guest-Memory-Fehler → `FAULT`, ungültige Flags/Clock → `INVAL`, Pfadflucht/fehlendes Recht → `NOTCAPABLE`, RO-Mutation → `ROFS`, Mount-übergreifendes Rename → `XDEV`, Quoten → `NOSPC`/`FBIG`/`MFILE`; fehlende Nodes und Typfehler verwenden `NOENT`, `NOTDIR`, `ISDIR`, `EXIST`, `NOTEMPTY`. Verifizierungs- oder Backendfehler innerhalb eines bereits gewährten Objekts werden auf ein festes `IO` reduziert und geben keine Store-Topologie preis.

## Offene Owner-/ADR-Fragen

- **BuildFS-Format:** 64-KiB-CAS-Chunks plus versioniertes, sortiertes Manifest übernehmen oder ein gepacktes Image mit Index? Empfehlung: Chunk-CAS, weil Range-Prüfung, Dedup und Heap-Nutzung klarer sind.
- **Realtime-Vertrag:** Feste Epoch 2000 wie oben oder ein manifestgebundener `SOURCE_DATE_EPOCH`? Empfehlung: feste Guest-Epoch; reale/auditierbare Zeit bleibt außerhalb des Compiler-Gastes.
- **Root-tmp-Politik:** Beliebige neue Root-Namen RAM-only erlauben oder auf beobachtete rustc-Präfixe beschränken? Empfehlung: beliebige neue, quotierte Root-Kinder; Präfixfilter wäre toolchain-fragil.
- **Egress-Zeitpunkt:** Outputbytes vollständig bis zum Doppel-Bau im RAM halten oder je Run in einen inert-only Scratch-Store schreiben? Empfehlung für den ersten Beweis: RAM; Scratch-Persist erst bei gemessenem RAM-Druck und mit eigener Lösch-/GC-Policy.
- **T2-Vertrag:** Threaddeckel muss die gemessenen etwa 26–32 Gastthreads tragen; außerdem ist festzulegen, dass `proc_exit` den gesamten Job beendet und welche Exit-Anforderung bei Konkurrenz gewinnt. Empfehlung: mindestens 32 Threads, deterministische TID-Reihenfolge und der erste im Round-Robin beobachtete `proc_exit` gewinnt.
- **Build-Evidenzversion:** Bestehenden Cargo/Werkstatt-Receipt erweitern oder einen On-Device-Receipt v2 einführen? Empfehlung: neuer Receipt v2 für einzelne `rustc`-Aufrufe; v1-Semantik nicht umdeuten.

## Ehrliche Risiken

- Slice 0 misst die statische Link-Oberfläche, nicht die Semantik aller dynamisch erreichbaren Fehlerpfade. Die Probe beweist weder große Crates noch inkrementelle Builds, Response-Files, ungewöhnliche Ausgaben oder sämtliche libc-Pfadfolgen.
- Das beobachtete „Temp direkt unter `/`“ verrät keine vollständigen Create/Rename/Delete-Sequenzen. Root-tmp und Preview1-Randsemantik müssen deshalb gegen echte rustc-Proben nach T1/T2 nachgeeicht werden.
- Gefälschte Zeit kann rustc-interne Zeitmessung, Freshness-Prüfung oder Timeouts beeinflussen. Fuel-basierte Fortschaltung verhindert eine stehende Uhr, ersetzt aber nicht den End-to-End-Test mit größeren Kompilationen.
- Deterministisches `random_get` ist absichtlich vorhersehbar. Der Bauplatz darf damit keine Schlüssel, Tokens oder andere Sicherheitsgeheimnisse erzeugen.
- Der aktuelle Artifact-Readback materialisiert ganze Frames. Ohne Chunk-CAS würde der Sysroot zusätzlich zu Compiler, Guest Memory und wasmi-Zustand den Kernel-Heap unnötig belasten.
- Die gegenwärtigen Workspace-Grenzen reichen nur für kleine Quellen; größere Einzeldateien oder source-only Crate-Bundles brauchen später eigene Bauplatz-Quoten. Das darf nicht durch Lockerung globaler Workspace-Grenzen „nebenbei“ gelöst werden.
- Preview1-Importlisten können sich bei jedem Compilerartefakt ändern. Jeder Toolchain-Update-SHA braucht ein neues Slice-0-Inventar und eine neue bzw. explizit versionierte ABI-Allowlist.
- Der erste echte rustc-Lauf bleibt gemeinsam von Bauplatz-Heap, T1 und T2 abhängig. Der WASI-Kern kann vollständig host-grün sein, ohne dass damit Shared Memory, 32 Threads oder die ungefähr 670 MB Laufzeitspitze bereits bewiesen wären.