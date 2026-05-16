# Konsens B: Plan-Aenderungen fuer SeedOS/RaiOS2

## Zusammenfassung

Die drei Reviews stimmen im Kern ueberein: Der aktuelle Stage-0-MVP ist stark,
aber der Plan darf daraus nicht ableiten, dass SeedOS schon eine
live-rebuildable Runtime besitzt. Im Code existieren Boot, UI, Console, USB-HID,
e1000/DHCP/DNS/TCP, direkter OpenAI-HTTPS-Pfad und RAM-only Setup. Was fehlt,
sind die tragenden Grenzen aus ADR 0002 und ADR 0003: maschinenlesbare
Selbstbeschreibung, Capability-Registry, Service-Inventar, Trust-/Redaction-
Regeln, VM-Test-Reports, lokale Attestation und Persistenz-/Rollback-Slots.

Meine zweite Konsensentscheidung: Der Plan sollte kurzfristig enger und
nachweisbarer werden. Vor Live-Loading, Wasm, OTA, Recovery-Agent oder
Persistenz braucht es ein V0-Protokoll-Gate und ein statisches Service-Modell
im bestehenden Kernel. Der funktionierende OpenAI-Direct-Pfad bleibt wichtig,
aber er ist ein ersetzbarer Provider-Service-Kandidat, nicht die Core-Identitaet
und nicht die spaetere Recovery-Lifeline.

Nicht geaendert werden sollte die bisherige MVP-Basis: Limine behalten,
Stage-0 klein halten, Codex CLI nicht portieren, den direkten Providerpfad nicht
wegwerfen, RAM-only Secrets fuer den MVP akzeptieren und keine grossen
Runtime-/Storage-Umbauten anfangen, bevor die Beobachtungs- und Sicherheitsbasis
steht.

## Konsens-Aenderungen

1. Phase 3/4 muessen ein Security- und Protocol-Gate bekommen.
   `seed-kernel/src/openai.rs` nutzt aktuell `NoVerify`; deshalb darf der
   Providerpfad nicht Basis fuer automatische Tools, Snapshot-Kontext oder
   persistente Aktionen werden. Erst TLS-Verifikation oder Provider/SPKI-Pinning,
   dann redigierte Kontextinjektion. `NoVerify` darf hoechstens ein expliziter
   Testmodus bleiben.

2. Vor Phase 6 muss ein V0-Protokoll-Gate stehen.
   ADR 0002 sollte praktisch vor ADR-0003-Runtime-Arbeit umgesetzt werden:
   `device-protocol/agent-v0.md`, `system-snapshot-v0`, `module-manifest-v0` und
   `vm-test-report-v0` muessen zuerst existieren. `module.load_ephemeral` und
   `module.persist` bleiben bis dahin definiert, aber denied.

3. Phase 5 sollte von "Core/World Boundary" zu "Static Service Inventory And
   Snapshot V0" konkretisiert werden.
   Der Kernel bleibt vorerst statisch gelinkt. Trotzdem sollen bestehende
   Komponenten feste IDs, Health, Last Error, Replaceable/Core-Owned-Flags und
   Capability-Namen bekommen. Naheliegende IDs:
   `core.boot`, `core.memory`, `core.serial`, `core.scheduler`,
   `core.entropy`, `core.snapshot_root`, `svc.ui.framebuffer`,
   `svc.console`, `svc.input`, `drv.usb.xhci`, `drv.net.e1000`,
   `svc.net.ipv4`, `drv.wifi.avastar_probe`, `svc.provider.openai_direct`.

4. `system_status.rs` sollte kanonische Facts liefern, nicht nur UI-Zeilen.
   `SystemSnapshot` ist heute ein guter Vorlaeufer, aber noch kein
   `system.snapshot.v0`. Die Plan-Aenderung sollte festlegen: typed Facts sind
   die Quelle; `StatusLine` fuer UI/Console wird daraus abgeleitet. Kein neuer
   UI-Status darf eine zweite Wahrheit neben dem Snapshot aufbauen.

5. Das erste Agent-Protokoll bleibt read-only.
   Startmethoden: `system.describe`, `system.snapshot`,
   `system.capabilities`, `system.boot_log`, `device.graph`, `problem.list`,
   `service.inventory`. Mutierende Methoden werden protokolliert und mit
   `capability_denied` beantwortet, bis Manifest, Testreport und Attestation
   vorhanden sind.

6. Capability-Registry und Datenklassifikation gehoeren vor Provider-Kontext.
   Fuer Felder, die den Provider verlassen koennen, braucht es
   `public`, `local_only` und `secret`. API-Key, WPA-Key und lokale
   Debugdetails duerfen nicht durch "hilfreichen Kontext" nach aussen rutschen.
   Netzwerkdaten wie IP, DNS, MAC und Hardwareinventar brauchen explizite
   Klassifikation statt ad-hoc-Senden.

7. Der VM-Harness muss zuerst Reports erzeugen, nicht sofort Fault-Injection
   koennen.
   `vm-harness/openai-direct-smoke.ps1` ist der richtige Start. Der naechste
   Schritt ist ein `seedos.vm_test_report.v0` mit Image-Hash, QEMU-Args-Hash,
   Hardwareprofil, Commands, Predicates, Ergebnis und Serial-Log-Pfad. QMP,
   Power-Faults und Golden-Screenshot-Diffs koennen spaeter folgen.

8. Registry/OTA sollen Safety-Evidence speichern, nicht nur Blobs.
   `registry/core` und `ota/cli` sind real, aber noch hostseitig. Der Plan
   sollte Acceptance Records als naechsten Host-Schritt aufnehmen:
   report hash, approval hash, manifest hash, artifact hash, base image hash,
   load mode und rollback pointer. Erst danach wird ein Gast-Loader sinnvoll.

9. Persistence braucht zuerst ein Image-/State-Layout-Dokument.
   Vor jedem Kernel-Schreibpfad muessen ESP-A/B oder Kernel-A/B, DATA-Partition,
   pending/success/last_good, Safe Mode und atomare Writes beschrieben sein.
   Die aktuelle Single-FAT-Stage-0-Welt bleibt korrekt fuer den MVP und darf
   nicht stillschweigend als Rollback-Architektur verkauft werden.

10. Recovery-Lifeline bleibt getrennt vom normalen OpenAI-Chat.
    ADR 0003 ist richtig: normale Agent-Service-Welt und Recovery-Control-Plane
    sind unterschiedliche Pfade. Der aktuelle `svc.provider.openai_direct` kann
    nicht einfach zur Recovery-Lifeline erklaert werden, weil er vom normalen
    Netzwerk-/TLS-/Providerpfad und von einem blockierenden HTTPS-Abschnitt
    abhaengt.

## Abgelehnte/verschobene Vorschlaege

- Kein Wasm- oder Gastmodul-Loader als naechster harter Blocker. Erst Schemas,
  Denial-Semantik, statisches Service-Inventar, VM-Report und lokale
  Attestation.

- Kein "OTA jetzt". Signatur-Tools und Registry sind nuetzlich, aber der Gast
  konsumiert diese Schiene nicht. OTA ohne A/B, pending/success/last_good und
  Safe Mode waere Bricking-Risiko.

- Kein WebSocket-Control-Plane-Zwang fuer den direkten Provider-MVP. Direkter
  HTTPS-Providerzugang passt zur "no dedicated custom cloud server"-Linie. Die
  Fake-Cloud/WebSocket-Control-Plane bleibt ein separater Pfad fuer OTA,
  Inventory und spaetere Modulsteuerung.

- Keine automatische Provider-Kontextinjektion vor TLS-Hardening und Redaction.
  Blindes Chatten ist begrenzt nuetzlich, aber sicherer als lokale Systemdaten
  ueber einen unverifizierten TLS-Pfad zu senden.

- Keine persistente API-Key- oder Wi-Fi-Secret-Speicherung als kurzfristiges
  Ziel. RAM-only ist fuer Stage-0 richtig. Eingebettete OpenAI-Keys bleiben ein
  lokaler Testimage-Modus und duerfen nicht in Release-, Registry- oder
  USB-Sharing-Flows geraten.

- Kein "globales Signatur-Oekosystem" als MVP-Voraussetzung. ADR 0002 ist hier
  richtig: lokale Attestation des exakt getesteten Artefakts ist wichtiger als
  ein fruehes App-Store-Modell.

- Kein Core-Generation-Handoff planen, bevor Services, Handles, State-Objekte,
  Health, Rollback und Persistenz einfach funktionieren. Das bleibt Phase 10+
  und darf nicht in Stage-0-Arbeit einsickern.

- Keine weitere UI-Politur, die eigene Statusquellen erfindet. Response-
  Wrapping und Scrolling sind sinnvoll, aber neue Statusdetails sollen aus dem
  Snapshot-/Inventory-Modell kommen.

## Konkrete Roadmap/ADR-Edits

### `docs/ROADMAP.md`

- Phase 3 umbenennen oder unterteilen:
  - Phase 3a: Direct Provider Transport With Fail-Closed TLS
  - Phase 3b: Redacted Read-Only Context Injection
  Definition of done fuer 3a: TLS certificate verification or SPKI/provider pin
  is visible in `provider::snapshot()` and the VM smoke checks a verified/pinned
  log marker.

- Phase 4 enger formulieren:
  Provider Integration bedeutet nicht nur "one response". Es muss auch klare
  Missing-Auth, Network, TLS, Provider-Error und redacted context behavior geben.
  Persistente Auth bleibt ausserhalb von Phase 4.

- Zwischen Phase 4 und Phase 6 ein explizites Gate einfuegen:
  `system.snapshot.v0 -> service.inventory.v0 -> capabilities.v0 ->
  module_manifest.v0 -> vm_test_report.v0 -> local_attestation.v0 -> load denied
  by default`.

- Phase 5 konkretisieren:
  "Core/World Boundary And Service Inventory" soll zuerst statische Services im
  bestehenden Kernel bedeuten. Keine dynamischen Module, keine Isolation, keine
  Handle-Migration als Phase-5-Pflicht.

- Phase 6 Definition of done schaerfen:
  Ein "low-risk service" darf nur nach vorhandenem Manifest, Capability-Grant,
  Auditlog, Health-State und Denial-Pfad geladen werden. Ohne diese Artefakte
  bleibt Phase 6 nicht gestartet.

- Phase 9 und 10 mit Vorbedingungen versehen:
  VM acceptance reports muessen Hashbindung enthalten. Persistenz braucht ein
  vorheriges Image-/DATA-/Rollback-Layout. Single-FAT Stage-0 bleibt als MVP
  markiert.

### `docs/architecture-decisions/0002-agent-self-description-and-live-built-modules.md`

- Ergaenzen: Erste Implementierung von ADR 0002 ist read-only ueber Console oder
  Agent-Dispatcher. Provider-Kontextinjektion kommt erst nach
  TLS-Hardening/Redaction.

- Ergaenzen: `module.load_ephemeral`, `module.persist` und `module.rollback`
  sind V0-Protokollmethoden, aber initial immer denied, solange
  `vm_test_report_hash` und `local_attestation_hash` fehlen.

- Schaerfen: Workstation-side capability artifacts sind der erste praktische
  Live-built-Typ. Guest diagnostic artifacts kommen danach. Kernel-/Driver-
  Module brauchen eine eigene ABI-/Isolation-Entscheidung.

- Feldklassifikation in `system.snapshot.v0` verpflichtend machen. Ohne
  Klassifikation darf ein Feld lokal angezeigt, aber nicht automatisch an einen
  Provider geschickt werden.

- Bei Minimal Manifest ergaenzen: `granted_caps` werden von lokaler Policy
  berechnet und duerfen nicht aus dem Manifest uebernommen werden.

### `docs/architecture-decisions/0003-always-on-core-and-live-rebuildable-world.md`

- Eine "Stage-0 Ramp" einfuegen:
  Vor echter Core/World-Trennung werden bestehende statisch gelinkte Komponenten
  als Services inventarisiert. Das ist kein Endzustand, aber die belastbare
  Bruecke zur spaeteren Runtime.

- Explizit festhalten:
  Der aktuelle OpenAI-Direct-Pfad ist normaler Agent-Service-Kandidat, keine
  Recovery-Lifeline. Recovery braucht eigenes minimales Protokoll, eigenen
  Trust-State und kleine erlaubte Aktionen.

- Blockierende Providerarbeit als Risiko nennen:
  Solange `openai::perform_https_request()` synchron laeuft und
  `tls_io::KernelTcpStream::wait_for()` die kooperative Schleife bindet, darf
  dieser Pfad nicht als always-on recovery-faehig gelten.

- Live-Rebuild-Primitives mit V0-Vorstufen verbinden:
  `load_service_ephemeral` setzt mindestens Service-ID, Manifest,
  Capability-Grants, VM-Testreport, Healthcheck und Audit Record voraus.

## Reihenfolge fuer Umsetzung

1. TLS-Hardening fuer `svc.provider.openai_direct`.
   Ziel: `NoVerify` aus dem normalen Pfad entfernen, Trust-State sichtbar
   machen, Smoke-Test-Marker ergaenzen.

2. Protokoll-Dokumente anlegen.
   `device-protocol/agent-v0.md`, `system-snapshot-v0`, `capabilities-v0`,
   `module-manifest-v0`, `vm-test-report-v0`. Erst Beispiele fuer die aktuelle
   QEMU-e1000/xHCI-Lage.

3. Typed Snapshot aus vorhandenen Quellen bauen.
   Inputs: `system_status.rs`, `net.rs`, `usb.rs`, `wifi.rs`, `provider.rs`,
   `openai.rs`, `input.rs`, `entropy.rs`. Ausgabe zunaechst ueber seriellen
   Befehl `snapshot` oder `system.snapshot`.

4. Statische Capability-Registry und Service-Inventar einfuehren.
   Read-only Caps zuerst. Mutierende Methoden liefern
   `capability_denied`. UI und Console nutzen weiter abgeleitete Human-Zeilen.

5. VM-Smoke zu `seedos.vm_test_report.v0` erweitern.
   Noch ohne QMP-Pflicht. Report unter einem klaren Output-Pfad, mit Hashbindung
   und Predicate-Ergebnis.

6. Provider-Kontextinjektion nur fuer redigierte Snapshot-Zusammenfassung.
   Keine Secrets, keine unklassifizierten lokalen Details, keine mutierenden
   Tools.

7. Hostseitig Manifest und Attestation typisieren.
   `ota/cli`, `registry/core` und `registry/cli` sollen Safety-Evidence
   sichtbar machen, bevor der Gast etwas daraus laedt.

8. Image-/Persistence-Layout spezifizieren.
   Erst Dokument, dann Skripterweiterung, erst danach Kernel-Schreibpfade.

9. Ephemeral Loading vorbereiten.
   Erst wenn Service-Inventar, Capability-Grants, VM-Report, Manifest,
   Attestation und Audit existieren. Anfangs read-only diagnostic/helper, kein
   MMIO-/Interrupt-Driver.

10. Recovery-Control-Plane spezifizieren.
    Separates `recovery-v0` mit minimalen Aktionen:
    snapshot, crashed services, restart last-good, disable module, rollback,
    load recovery artifact by hash. Kein allgemeiner Chat.

## Offene Fragen

- TLS-Trust fuer OpenAI: CA-Verifikation im Kernel, SPKI-Pin, Zertifikat-Pin
  oder eine kleine provider-spezifische Pin-Policy? Pinning ist kleiner, aber
  rotationsanfaellig.

- Welche Snapshot-Felder duerfen einen Provider verlassen? IP/DNS, PCI/USB-
  Inventar, Wi-Fi-Chip, Bootlog und Error-Texte brauchen konkrete
  Klassifikation.

- Wie wird `kernel_build_id` oder `image_hash` im laufenden Stage-0
  zuverlaessig bestimmt, ohne das Windows-Paketierungsmodell zu verkomplizieren?

- Welche Service-IDs sind Core-owned und welche nur statisch gelinkte Services?
  Diese Liste muss klein bleiben, sonst wird ADR 0003 Wunschdenken.

- Welches Format bekommt der erste Gast-Diagnostic-Artifact-Typ: native Rust ABI,
  Wasm oder ein noch kleineres bytecode-/message-basiertes Modell?

- Welche Aktionen brauchen direkte Nutzerbestaetigung auch dann, wenn ein
  VM-Testreport vorliegt? Mindestens Persistenz, Hardware-Schreibzugriffe,
  Secret-Speicherung und Recovery-Rollback.

- Wie bildet ein Bare-Metal-Run reproduzierbare Preconditions fuer den
  VM-Harness ab, wenn die reale Hardware nicht voll in QEMU nachgestellt werden
  kann?

- Wo liegt der Auditlog in Stage-0 vor Persistenz? Ein RAM-only Ring reicht fuer
  den Anfang, aber Attestation und Persistenz brauchen spaeter ein dauerhaftes
  Format.
