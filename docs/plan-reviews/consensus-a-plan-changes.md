# Consensus A: Plan-Aenderungen

## Entscheidungsgrundlage

Gelesene Grundlage:

- `docs/plan-reviews/review-1-core-boundary.md`
- `docs/plan-reviews/review-2-agent-protocol-provider.md`
- `docs/plan-reviews/review-3-runtime-vm-persistence.md`
- `docs/ROADMAP.md`
- `docs/architecture-decisions/0003-always-on-core-and-live-rebuildable-world.md`

Massstab fuer Aufnahme:

- Eine Aenderung wird nur aufgenommen, wenn sie in mehreren Reviews auftaucht
  oder direkt aus einer harten Luecke zwischen Roadmap/ADR-0003 und aktuellem
  Plan folgt.
- Einzelvorschlaege ohne klare Sicherheits- oder Sequenzierungswirkung bleiben
  draussen.
- Ziel ist Plan-Schaerfung, nicht sofortige Code-Arbeit.

## Klare Konsenspunkte

1. **TLS/Trust ist ein Gate, kein Polish.** Der direkte Providerpfad funktioniert,
   aber `NoVerify` darf nicht Basis fuer Snapshot-Kontext, Toolverkehr,
   Recovery, Enrollment, OTA oder Persistenz werden.

2. **Vor Live-Loading braucht raiOS eine read-only Selbstbeschreibung.**
   `system.snapshot.v0`, `system.capabilities`, `system.boot_log`,
   `device.graph`, `problem.list` und `service.inventory` muessen zuerst als
   maschinenlesbarer Vertrag geplant werden.

3. **Phase 5 muss als statisches Core/Service-Modell konkretisiert werden.**
   Die aktuelle Kernel-Welt darf zunaechst monolithisch gelinkt bleiben, muss
   aber feste Service-IDs, Health, Last-Error, Replaceable/Core-Owned und
   Capability-Namen bekommen.

4. **Providerpfad, Control Plane und Recovery-Lifeline duerfen nicht vermischt
   werden.** OpenAI Direct ist ein normaler Provider-Service-Kandidat. Die
   Recovery-Lifeline aus ADR-0003 braucht einen kleineren, getrennten
   Kontrollpfad.

5. **Provider-Kontextinjektion kommt erst nach Redaction und Feldklassifikation.**
   Der Provider soll nicht automatisch lokale Systemdaten erhalten, bevor
   `public`, `local_only` und `secret` fuer Snapshot-Felder festgelegt sind.

6. **Module/Runtime/Persistenz brauchen erst Manifest-, Testreport- und
   Attestation-V0.** Ein signiertes Artefakt allein reicht nicht. Vor
   `module.load_ephemeral` oder Persistenz braucht der Plan mindestens
   `module_manifest.v0`, `raios.vm_test_report.v0` und lokale Acceptance-/
   Attestation-Regeln.

7. **Persistence und Bootmedien-Schreibpfade brauchen ein Layout-Dokument vor
   Implementierung.** A/B, `pending`, `success`, `last_good`, Safe Mode und
   verbotene Writes muessen vor Kernel- oder Packaging-Schreibcode spezifiziert
   werden.

8. **Dokumentationsdrift muss bereinigt werden.** Invarianten, Runbook und
   Roadmap duerfen nicht so wirken, als seien WebSocket-Control, SPKI-Pinning,
   Wasm Runtime, Recovery oder A/B-OTA bereits implementiert.

## Strittige Punkte / Nicht Uebernehmen

- **Keine Wasm-Runtime als naechster Blocker.** Dafuer gibt es keinen Konsens.
  Konsensfaehig ist zuerst das Datenmodell fuer Manifest, Capabilities, Health
  und Denial.

- **Kein dynamisches Service-Loading vor statischem Inventar.** ADR-0003 bleibt
  Zielbild, aber die naechste Planstufe sollte beobachtbar und denied-by-default
  sein.

- **Keine Provider-Toolcalls vor TLS, Snapshot und Redaction.** Read-only
  Snapshot-Kontext kann spaeter folgen; mutierende Tools bleiben vorher denied.

- **Keine Recovery ueber den normalen OpenAI-Chat.** Der normale Providerpfad
  darf nicht zur trusted Recovery-Basis erklaert werden.

- **Keine Kernel-Persistenz oder OTA-Schreiblogik vor Image-Layout-V0.** Die
  aktuelle Single-FAT-Stage-0-Welt bleibt ausdruecklich nicht Phase 10.

- **Keine volle QMP-/Fault-Injection als Sofortpflicht.** Konsens ist ein
  maschinenlesbarer VM-Smoke-Report als naechster Harness-Schritt; QMP kann
  danach kommen.

## Konkrete Plan-Aenderungen Mit Ziel-Dokumenten / Abschnitten

### Aenderung 1: Phase 3/4 in Trust- und Context-Gates teilen

- **Ziel:** `docs/ROADMAP.md`, Phase 3 `Direct Provider Transport` und Phase 4
  `Provider Integration`.
- **Aenderung:** Phase 3 bekommt ein explizites Gate:
  `TLS verification or provider pinning fail-closed`. Phase 4 darf erst danach
  Provider-Kontext aus `system.snapshot.v0` nutzen.
- **Begruendung:** Review 1, 2 und 3 nennen TLS-Bypass als harte Luecke. Die
  Roadmap nennt HTTPS-Hardening bereits, aber noch nicht als Gate fuer Kontext,
  Tools und Recovery.

### Aenderung 2: Phase 5 als `Static Core/Service Model + Snapshot V0` schaerfen

- **Ziel:** `docs/ROADMAP.md`, Phase 5 `Core/World Boundary And Service
  Inventory`.
- **Aenderung:** Phase 5 soll ausdruecklich statische Services modellieren:
  `core.boot`, `core.memory`, `core.serial`, `core.scheduler`,
  `core.snapshot_root`, plus inventarisierte Services/Driver wie UI, Console,
  Input, USB, Network, Wi-Fi und Provider. Pflichtfelder: Service-ID, Kind,
  Health, Last-Error, Capabilities, Replaceable, Core-Owned.
- **Begruendung:** Alle drei Reviews sehen das als Bruecke zwischen heutigem
  Monolith und ADR-0003. Ohne diese Stufe ist Phase 6 Live-Services zu frueh.

### Aenderung 3: Read-only Agent-Protokoll-V0 vor mutierenden Aktionen

- **Ziel:** `device-protocol/README.md` und neue Ziel-Specs
  `agent-v0.md`, `system-snapshot-v0.md`, `capabilities-v0.md`.
- **Aenderung:** Das erste Protokoll-Gate definiert nur read-only Methoden:
  `system.snapshot`, `system.capabilities`, `system.boot_log`, `device.graph`,
  `problem.list`, `service.inventory`. Mutierende Methoden wie
  `module.load_ephemeral`, `module.persist` und `apply_config` werden als
  `capability_denied` beschrieben.
- **Begruendung:** Review 1, 2 und 3 fordern eine maschinenlesbare
  Selbstbeschreibung vor Tool- und Modulaktionen. Das passt direkt zu
  ADR-0003s Snapshot-Root und Capability Table.

### Aenderung 4: Datenklassifikation und Redaction als Voraussetzung aufnehmen

- **Ziel:** `device-protocol/system-snapshot-v0.md`, `docs/ROADMAP.md` Phase 3,
  Phase 4 und Phase 5.
- **Aenderung:** Snapshot-Felder bekommen Klassifikation
  `public`, `local_only`, `secret`; Provider-Kontext darf nur redigierte,
  explizit erlaubte Felder enthalten.
- **Begruendung:** Review 1 und 2 nennen das direkt, Review 3 bestaetigt den
  Secret-/Persistence-Gap. Ohne Klassifikation wird `system.snapshot.v0` beim
  Provider zur Leckageflaeche.

### Aenderung 5: Normaler Provider, Control Plane und Recovery getrennt planen

- **Ziel:** `docs/ROADMAP.md`, Phase 3, Phase 8 und North Star Architecture;
  optional `device-protocol/recovery-v0.md`.
- **Aenderung:** Roadmap soll drei Pfade separat benennen:
  direkter Provider-HTTPS-Promptpfad, spaetere Control Plane fuer
  Fake-Cloud/OTA/Module, und Recovery-Lifeline. Recovery bekommt nur minimale
  Methoden wie Snapshot, Restart last-good, Disable module, Rollback und Load by
  hash.
- **Begruendung:** Alle Reviews warnen vor Vermischung. ADR-0003 fordert
  ausdruecklich eine kleinere Recovery-Lifeline getrennt vom normalen Agent
  Service.

### Aenderung 6: V0-Protokoll-Gate zwischen Phase 4 und Phase 6 sichtbar machen

- **Ziel:** `docs/ROADMAP.md`, Uebergang Phase 4 -> Phase 5 -> Phase 6.
- **Aenderung:** Vor `Ephemeral Live Services` steht ein explizites Gate:
  `system.snapshot.v0 -> service.inventory.v0 -> capability policy v0 ->
  module_manifest.v0 -> vm_test_report.v0 -> local_attestation.v0 -> load
  denied by default`.
- **Begruendung:** Review 2 und 3 fordern diese Reihenfolge; Review 1 fordert
  zuerst Snapshot/Inventory. Das verhindert eine Runtime ohne Acceptance-Grenze.

### Aenderung 7: VM-Harness-Report-V0 als naechsten Acceptance-Schritt planen

- **Ziel:** `docs/ROADMAP.md` Phase 9 `Shadow VM Acceptance`,
  `vm-harness/README.md`, `device-protocol/vm-test-report-v0.md`.
- **Aenderung:** Der vorhandene Smoke-Test soll als Plan-Ziel einen
  maschinenlesbaren Report erzeugen: Image-Hash, QEMU-Args-Hash,
  Hardwareprofil, Kommandos, Pruefmarker, Ergebnis und Serial-Log-Verweis.
- **Begruendung:** Review 2 und 3 koppeln Live-/Persistenzentscheidungen an
  VM-Testberichte. Das ist der kleinste konsensfaehige Schritt vor QMP,
  Fault-Injection oder Shadow-VM-Vollausbau.

### Aenderung 8: Manifest und lokale Attestation als Safety Evidence planen

- **Ziel:** `device-protocol/module-manifest-v0.md`,
  `device-protocol/vm-test-report-v0.md`, spaeter Registry-Dokumentation.
- **Aenderung:** Plan soll festhalten, dass ein ladbares Artefakt mindestens
  Manifest-Hash, Artifact-Hash, Base-Image-Hash, Test-Report-Hash,
  Capability-Grants, Risk-Level, Load-Mode und lokale Approval/Attestation
  bindet.
- **Begruendung:** Review 2 und 3 sagen beide: Signatur allein ist keine
  Acceptance. ADR-0003 verlangt VM-Reports und Rollback fuer riskante oder
  persistente Aenderungen.

### Aenderung 9: Persistence/Image-Layout-V0 vor Schreibcode

- **Ziel:** neues Plan-Dokument `docs/image-layout-v0.md`, Roadmap Phase 10
  `Persistence, Rollback, And Core Handoff`.
- **Aenderung:** Vor Erweiterung von Packaging-, USB- oder Kernel-Storage-Code
  wird ein Layout spezifiziert: Stage-0 Single-FAT als Ist-Zustand, spaeteres
  ESP-A/B oder Kernel-A/B, DATA-Partition, Pending/Success/Last-Good, Safe Mode
  und atomare Writes.
- **Begruendung:** Review 3 nennt das ausfuehrlich, Review 2 bindet
  Persistenz an Testbericht/Attestation, und ADR-0003 macht Rollback zu einem
  Kernziel. Ohne Layout ist Persistence ein Bricking-Risiko.

### Aenderung 10: Invarianten-Doku in `gewaehlt` vs `implementiert` trennen

- **Ziel:** `docs/invariant-choices.md`, `docs/AI_Build_and_Test_Runbook.md`,
  `docs/sections/00-invariants.md`.
- **Aenderung:** Checkboxes und Aussagen werden getrennt nach Architekturwahl,
  geplantem Gate und aktuell verifiziertem Stand. Direkter Provider-HTTPS wird
  als erlaubter MVP-Pfad dokumentiert; WebSocket-Control bleibt separater
  Control-Plane-Pfad.
- **Begruendung:** Review 2 und 3 nennen widerspruechliche oder missverstaendliche
  Dokumentation. Das ist ein harter Plan-Gap, weil falsche Checkboxes zu
  falscher Priorisierung fuehren.

## Reihenfolge / Prioritaet

### P0: Trust und sichere Plan-Basis

1. Phase 3/4 Trust-Gate fuer TLS-Verifikation oder Provider-Pinning.
2. Dokumentationsdrift bereinigen: Invarianten als gewaehlt vs implementiert.
3. Secrets/Testimage-Regeln im Plan klar halten: RAM-only bleibt MVP;
   eingebettete Keys sind nur lokale Testimages.

### P1: Read-only Selbstbeschreibung

1. `agent-v0`, `system-snapshot-v0`, `capabilities-v0` planen.
2. Phase 5 als statisches Core/Service-Modell konkretisieren.
3. Datenklassifikation/Redaction fuer Snapshot-Felder festlegen.

### P2: Acceptance vor Runtime

1. `module_manifest.v0`, `vm_test_report.v0` und lokale Attestation als Gate vor
   Live-Loading aufnehmen.
2. VM-Smoke-Report-V0 als ersten maschinenlesbaren Harness-Output planen.
3. `module.load_ephemeral` und Persistenz bis dahin denied-by-default halten.

### P3: Recovery und Persistence

1. Recovery-Lifeline getrennt vom normalen Providerpfad spezifizieren.
2. `image-layout-v0.md` vor Bootmedien- oder Kernel-Schreibpfaden schreiben.
3. Erst danach Roadmap-Arbeit an Rollback-Slots, Last-Good und Core-Handoff
   konkretisieren.
