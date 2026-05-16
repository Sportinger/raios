# Review 3: Runtime, VM-Harness und Persistence

## Kurzfazit

Der langfristige Plan ist architektonisch klarer als die aktuelle Implementierung.
Die Codebase hat eine starke Stage-0-Basis fuer Boot, UI, e1000-Netzwerk,
DHCP/DNS, direkten OpenAI-Transport, USB-HID und serielle Bedienbarkeit. Die
Host-Seite fuer Signaturen, Registry und einen Fake-Cloud-WebSocket-Stub ist
ebenfalls real und testbar.

Die live-rebuildable Runtime aus `docs/ROADMAP.md`, ADR 0002 und ADR 0003 ist
aber noch nicht in der Geraetegrenze angekommen. `seed-runtime/` und
`modules/hello-ui/` sind Platzhalter, `device-protocol/` enthaelt noch keine
Schemata, der VM-Harness ist ein einzelner Direct-OpenAI-Smoke, und
Persistence/Rollback existieren bisher nur als Plan. Der naechste sinnvolle
Plan-Schritt sollte deshalb nicht "Wasm jetzt" oder "OTA jetzt" sein, sondern:
maschinenlesbare Selbstbeschreibung, Manifest-/VM-Test-Report-Schemas,
Attestation-Record und erst danach eingeschraenktes Laden.

## Bereits vorhandene Basis

### Stage-0 als stabile Beobachtungsbasis

- `seed-kernel/src/main.rs` enthaelt die aktuelle monolithische Stage-0-Schleife:
  periodische Tasks fuer Console, Entropie, Netzwerk, Input, USB-Rescan,
  Provider und UI.
- `seed-kernel/src/system_status.rs` aggregiert bereits Statuszeilen fuer
  Framebuffer, Entropie, USB-xHCI, Wi-Fi, Netzwerk und Input. Das ist eine gute
  Quelle fuer `system.snapshot.v0`, aktuell aber noch kein Protokollobjekt.
- `seed-kernel/src/ui.rs` und `seed-kernel/src/console.rs` zeigen denselben
  Status im Framebuffer und in seriellen Befehlen (`status`, `devices`,
  `provider`, `openai`, `wifi`, `setup`, `ask <text>`).
- `seed-kernel/src/net.rs`, `seed-kernel/src/e1000.rs` und
  `seed-kernel/src/tls_io.rs` liefern eine echte Gast-Netzwerkbasis:
  e1000, DHCP, DNS, TCP und ein `embedded-io`-Stream ueber smoltcp.
- `seed-kernel/src/openai.rs` beweist den direkten Providerpfad ueber
  DNS/TCP/TLS/HTTPS zur OpenAI Responses API. Das ist ein wertvoller
  Referenz-Adapter, aber kein allgemeiner Control Channel.
- `seed-kernel/src/provider_config.rs` und `seed-kernel/src/wifi.rs` haben
  RAM-only Konfiguration fuer API-Key, SSID und WPA-Passphrase. Das ist passend
  fuer Stage-0, aber noch keine Persistenz.

### Signatur-, OTA- und Registry-Hostseite

- `ota/cli/src/lib.rs` implementiert `KeyMaterial`, `SignerCertificate` und
  `SignedBlob` mit Ed25519-Signaturen und BLAKE3-Hash des ganzen Payloads.
- Die Binaries `ota-keygen`, `ota-sign`, `ota-verify`, `mod-sign` und
  `mod-verify` sind in `ota/cli/src/bin/` vorhanden.
- `registry/core/src/lib.rs` implementiert eine lokale content-addressed
  Registry mit `blobs/`, `manifests/` und `index/`.
- `registry/cli/src/main.rs` bietet `init`, `publish` und `list`.
- `fake-cloud/server/src/lib.rs` implementiert einen WebSocket-Server mit
  JSON-Envelope, `hello` -> `hello_ack` und `ota_begin` -> `ota_ack`.
  `ota_begin` verifiziert den signed blob und kann ihn in `registry/core`
  publizieren.
- `ota/cli/tests/sign_verify.rs`,
  `registry/core/src/lib.rs` und
  `fake-cloud/server/tests/ota_roundtrip.rs` decken die Host-Roundtrips ab.

### VM- und Bare-Metal-Devflow

- `scripts/run-stage0-qemu.ps1` ist der wichtigste Windows-Runner. Er kann
  TCP-Serial, Headless, e1000, xHCI-Input und optional einen HMP-Monitor
  starten.
- `scripts/run-stage0-baremetal-vm.ps1` fixiert das bare-metal-nahe QEMU-Profil
  mit xHCI, e1000 und CPU `max`.
- `vm-harness/openai-direct-smoke.ps1` startet die VM headless, sendet
  `provider` und `ask`, wartet auf konkrete serielle Marker und prueft, dass der
  alte Host-Relay-Pfad nicht auftaucht.
- `scripts/package-stage0.ps1` baut und paketiert auf Windows ein bootbares
  FAT32-Image aus `release/esp` und `target/x86_64-seed/.../seed-kernel`.
- `scripts/write-stage0-usb.ps1` hat sinnvolle Bare-Metal-Sicherungen:
  Admin-Pruefung, explizite Loeschbestaetigung, Boot/System-Disk-Schutz und
  USB-Bus-Pruefung.
- `docs/BARE_METAL.md` beschreibt realistisch, dass Bare Metal noch
  experimentell ist und derzeit vor allem Boot, Framebuffer, xHCI-HID und
  Inventar prueft.

## Abweichungen/Gaps

### 1. Device-Protokoll ist noch Platzhalter

ADR 0002 fordert `device-protocol/agent-v0.md`,
`device-protocol/module-manifest-v0.md` und
`device-protocol/vm-test-harness-v0.md`. Aktuell enthaelt
`device-protocol/README.md` nur die Absicht fuer den JSON-Envelope.

Konsequenz: Es gibt noch keinen stabilen Vertrag fuer `system.snapshot`,
`system.capabilities`, `module.propose`, `module.test_result`,
`module.load_ephemeral`, Capability-Denials oder Redaction-Klassen.

### 2. `system.snapshot.v0` existiert nur als menschliche Statusdarstellung

`seed-kernel/src/system_status.rs` ist nah an den benoetigten Fakten, aber:

- kein JSON/Envelope,
- kein Schema-Name,
- keine Capability-Liste,
- kein `problems`-Array,
- kein Image-/Build-Hash,
- keine Klassifizierung in `public`, `local_only`, `secret`,
- kein serieller/protokollarischer Befehl `snapshot` oder `system.snapshot`.

Der Agent muesste weiterhin `status`, `devices`, `provider` und Logs parsen.
Das widerspricht ADR 0002, wonach der Agent nicht aus menschlichen Logs
ableiten soll, was das System kann.

### 3. Live-Module und Service-Runtime sind nicht implementiert

`seed-runtime/README.md` sagt selbst, dass die Wasm-Runtime und
Lifecycle-Verwaltung spaeter gefuellt werden. `modules/hello-ui/README.md`
beschreibt ein Zielmodul, aber es gibt keinen Wasm-Code, kein Manifest, keinen
Build, keine Signierung und keine Runtime-Integration.

Auch im Workspace ist `seed-runtime` kein Cargo-Crate; `Cargo.toml` enthaelt nur
`ota/cli`, `registry/cli`, `registry/core`, `fake-cloud/server` und
`seed-kernel`.

Im Kernel gibt es noch keine:

- Service-Registry,
- Service-Inventar,
- Capability-Tabelle,
- `load_service_ephemeral`,
- Health-Checks oder Crash-Records,
- Versioned State Objects,
- Migrator,
- Handle-Indirection,
- Rollback-Handle.

Der aktuelle Stage-0 ist bewusst klein, aber noch monolithisch. Das ist fuer den
MVP gut, muss im Plan aber als Vorstufe zum Service-Graph markiert bleiben.

### 4. Fake Cloud ist nur Host-seitig angebunden

`fake-cloud/server/src/lib.rs` kann WebSocket sprechen, aber Stage-0 hat keinen
WebSocket-Client zum Fake Cloud Control Plane. Der Kernel hat stattdessen den
direkten OpenAI-HTTPS-Adapter in `seed-kernel/src/openai.rs`.

Fehlende Geraete-Funktionen aus `docs/AI_Build_and_Test_Runbook.md`:

- TLS zum Control Server mit Pin,
- WebSocket-Handshake,
- `hello` vom Geraet,
- `inventory_request/response`,
- Log-Mirroring,
- `module_install/start/stop`,
- `ota_begin/chunk/commit`,
- `lockdown`,
- Reconnect/Offline-Buffering.

Fehlende Fake-Cloud-Funktionen:

- deterministische Szenarien,
- Log-Sink,
- Inventory-Flows,
- Modul-Lifecycle-Kommandos,
- Chunked OTA,
- Lockdown-Kommando,
- Test-Fixtures und Golden Transcripts.

### 5. OTA ist Signatur-Tooling, noch kein Update-System

`ota/cli` signiert und verifiziert ganze Payloads. Das ist ein guter
Trust-Baustein, aber es ist noch nicht das OTA-Modell aus Runbook/Roadmap.

Noch nicht vorhanden:

- Chunk-Manifest mit BLAKE3 pro Chunk,
- OTA-Stream ueber WebSocket,
- inaktiver A/B-Slot,
- ESP-A/B-Layout,
- DATA-Partition,
- pending/success/last-good Marker,
- kexec oder anderer Handoff,
- Rollback nach Crash oder fehlendem Success Marker,
- Safe Mode, der Module und persistente Writes blockiert.

`scripts/package-stage0.ps1` erzeugt ein einzelnes 64-MiB-FAT32-Image.
`scripts/write-stage0-usb.ps1` erzeugt eine einzelne Boot-Partition. Beides ist
fuer Stage-0 korrekt, aber nicht mit Phase 10 gleichzusetzen.

### 6. Registry-Manifest ist zu generisch fuer ADR 0002

`SignedBlob.metadata` ist optionales JSON. `mod-sign` setzt bisher nur
`module_id` und `module_version`. ADR 0002 verlangt aber frueh:

- `manifest_version`,
- `kind`,
- `target`,
- `abi`,
- `provides`,
- `requested_caps`,
- `granted_caps`,
- `risk`,
- `base_image_hash`,
- `manifest_hash`,
- `artifact_hash`,
- `test_report_hash`,
- `tests`,
- `load_mode`,
- `rollback_id`.

`registry/core/src/lib.rs` indexiert aktuell Payload-Hash, Laenge, Manifest,
Signer und logischen Namen/Version. Es bindet noch keinen VM-Test-Report, keine
lokale Approval-Entscheidung, keinen Base-Image-Hash und keinen Rollback-Pointer
an den Eintrag.

### 7. VM-Harness ist noch kein Acceptance-Harness

`vm-harness/openai-direct-smoke.ps1` ist wertvoll, aber es ist ein Smoke-Test.
Es erstellt keinen maschinenlesbaren `seedos.vm_test_report.v0` und hat keine
QMP-Orchestrierung.

Abweichungen zum Runbook:

- `scripts/run-stage0-qemu.ps1` bietet HMP per `-monitor`, aber kein QMP.
- keine Fixture-Dateien fuer Firmware, Image, Netzwerk, Ports, SPKI-Pin oder
  Szenario-Timeline,
- keine Golden-Transcript-Diffs,
- keine Framebuffer-Hashing-Utilities,
- keine QMP-Fault-Injection fuer Power, Netzwerk, Clock oder Storage,
- keine Snapshot-/Ephemeral-Mode-Sicherung fuer alle Tests,
- keine Report-Bindung an Image-Hash, Artifact-Hash, QEMU-Args-Hash und
  Hardwareprofil.

### 8. Persistence und Secrets sind absichtlich RAM-only

Das ist fuer Stage-0 gut dokumentiert, bleibt aber ein harter Gap fuer
Roadmap-Phasen 4, 9 und 10:

- OpenAI-Key ist RAM-only oder in einem lokalen Testimage eingebettet
  (`scripts/package-stage0.ps1 -EmbedOpenAiApiKeyFromEnv`).
- Wi-Fi-SSID und WPA-Passphrase sind RAM-only in `seed-kernel/src/wifi.rs`.
- Kein DATA-Volume, kein KV-Store, keine Device-ID, kein Device-Key, kein
  Token, kein Enrollment und kein Secret-Rotation-Modell.

### 9. TLS/Pinning ist widerspruechlich dokumentiert

`docs/invariant-choices.md` spricht von TLS mit SPKI-Pin und WebSocket-Control.
`seed-kernel/src/openai.rs` nutzt aber `embedded_tls::blocking::NoVerify` und
loggt den Certificate-Verification-TODO. `docs/PROJECT_STATUS.md` nennt diesen
Gap korrekt.

Zusaetzlich sollte der Plan sauber trennen:

- Provider-TLS fuer `api.openai.com` braucht CA-Verifikation oder
  Provider-Pinning.
- Fake-Cloud-Control-TLS braucht einen eigenen Server-/SPKI-Pin.
- Artifact-Signing ueber Offline-Root/Online-Signer ist ein anderer Trust-Pfad
  und sollte nicht stillschweigend mit TLS-Serveridentitaet vermischt werden.

### 10. Bare-Metal-Devflow hat keinen Rueckkanal fuer reproduzierbare Reports

`docs/BARE_METAL.md` und die USB-Skripte sind pragmatisch. Was fuer ADR 0002/0003
fehlt, ist ein maschinenlesbarer Bare-Metal-Beobachtungsbericht:

- aktueller Hardware-Snapshot,
- PCI/USB/NIC/Wi-Fi Fakten,
- Input-Status,
- relevante serielle Marker,
- Snapshot-Preconditions fuer einen VM-Repro,
- Verweis auf Image-Hash und Boot-Konfiguration.

Ohne diesen Rueckkanal kann der VM-Harness Bare-Metal-Fehler spaeter schlecht
nachstellen.

## Risiken/Probleme

- **Falsche Sequenzierung:** Wenn jetzt direkt eine Wasm- oder Service-Runtime
  gebaut wird, ohne `system.snapshot.v0`, Manifest v0 und VM-Test-Report v0,
  entsteht eine Runtime ohne belastbare Sicherheits- und Acceptance-Grenze.
- **Scheinsicherheit durch Host-Tools:** Signatur, Registry und Fake Cloud sind
  testbar, aber das Geraet konsumiert diese Schiene noch nicht. Ein erfolgreicher
  `fake-cloud-server`-Test beweist daher noch keine OTA- oder Runtime-Faehigkeit
  im SeedOS-Gast.
- **TLS-Sicherheitsluecke:** Der direkte Providerpfad funktioniert, aber
  `NoVerify` darf nicht zur Basis fuer Enrollment, OTA oder persistente
  Agent-Aktionen werden.
- **Persistenz-Bricking-Risiko:** Sobald der Kernel in Bootmedien schreibt,
  braucht es vorher A/B-, Pending-, Success- und Last-Good-Regeln. Die aktuelle
  Single-FAT-Image-Welt hat keinen Schutz gegen halb geschriebene Updates.
- **Secret-Leak-Risiko im Devflow:** Lokale Images mit eingebettetem
  `OPENAI_API_KEY` sind praktisch fuer den Smoke-Test, muessen aber klar
  ausserhalb jedes Registry-/Release-/USB-Sharing-Flows bleiben.
- **Monolithischer Pfad verfestigt sich:** Neue Features koennen leicht in
  `seed-kernel/src/main.rs`, `console.rs` oder `openai.rs` landen. Das wuerde
  die spaetere Core/World-Grenze aus ADR 0003 erschweren.
- **Harness ohne Attestation:** Ohne JSON-Report, Hashbindung und Fixture kann
  ein spaeteres `module.load_ephemeral` nicht pruefen, ob exakt das Artefakt
  unter exakt dem relevanten Profil getestet wurde.

## Konkrete Plan-Aenderungsvorschlaege

### 1. Roadmap zwischen Phase 4 und 6 um einen V0-Protokoll-Gate ergaenzen

Vor "Ephemeral Live Services" sollte ein explizites Gate stehen:

```text
system.snapshot.v0 -> module_manifest.v0 -> vm_test_report.v0
-> local_attestation.v0 -> load still denied by default
```

Das entspricht ADR 0002 und reduziert das Risiko, dass Live-Loading ohne
Acceptance-Modell entsteht.

### 2. Wasm-Runtime nicht als naechsten harten Blocker behandeln

`seed-runtime/` sollte zuerst die Datenmodelle und Lifecycle-Zustaende
modellieren, nicht sofort eine vollstaendige Wasm-Engine in Stage-0 bringen.
Der erste "live-built module"-Schritt sollte gemaess ADR 0002 ein
workstation-side capability artifact oder ein proposal-only Modul sein.

Guest-Wasm und `modules/hello-ui` sollten erst folgen, wenn Manifest,
Capabilities, Test-Report und Denial-Semantik existieren.

### 3. Control Plane und Providerpfad trennen

Der direkte OpenAI-Pfad in `seed-kernel/src/openai.rs` sollte als
Provider-Adapter betrachtet werden. Der Fake-Cloud-WebSocket sollte als
separater Control Channel geplant werden:

- `control.hello`,
- `system.snapshot`,
- `log.append`,
- `module.propose`,
- `module.load_ephemeral` mit erwarteter Denial-Antwort,
- spaeter `ota_begin/chunk/commit`.

Das verhindert, dass Chat/Provider-Transport, Recovery-Lifeline und OTA-Control
untrennbar vermischt werden.

### 4. Trust-Modell in zwei Achsen aufteilen

Plan und Dokumentation sollten getrennt benennen:

- TLS-Serveridentitaet fuer Provider und Fake Cloud,
- Artefakt-Signatur ueber Offline-Root/Online-Signer,
- lokale Attestation ueber Image-Hash, Manifest-Hash, Artifact-Hash,
  VM-Test-Report-Hash und Approval.

Ein Artefakt darf nicht nur wegen einer Signatur ladbar werden. Eine Signatur
beweist Herkunft/Integritaet; die lokale Attestation beweist, dass dieses
Artefakt in diesem Kontext akzeptiert wurde.

### 5. Registry um Acceptance Records ergaenzen statt nur Packages speichern

Die Registry sollte neben `blobs/`, `manifests/` und `index/` ein Konzept fuer
Test-/Approval-Records bekommen, zunaechst hostseitig:

```text
reports/<report_hash>.json
approvals/<attestation_hash>.json
index/<namespace>/<name>/<tag>.json -> payload + manifest + report + approval
```

Damit wird die Registry zur Safety-Evidence-Ablage und nicht nur zum lokalen
Blob-Store.

### 6. Image-/Persistence-Layout vor Schreibcode spezifizieren

Vor jedem Kernel-Schreibpfad sollte ein Layout-Dokument stehen:

- ESP-A und ESP-B oder Kernel-A/B innerhalb ESP,
- DATA-Partition,
- `pending`, `success`, `last_good`,
- `/data/SAFE`,
- welche Writes in Safe Mode verboten sind,
- welche Writes atomar sein muessen.

Erst danach sollten `scripts/package-stage0.ps1`,
`scripts/write-stage0-usb.ps1` und spaeter Kernel-Storage-Code erweitert werden.

### 7. VM-Harness inkrementell zu Reports ausbauen

Der naechste Harness-Schritt muss noch nicht volle QMP-Fault-Injection sein.
Ein guter Zwischenstand waere:

- serieller Smoke erzeugt JSON-Report,
- Report enthaelt Image-Hash, QEMU-Args-Hash, Hardware-Profil, Commands,
  Predicates und Ergebnis,
- Report wird unter `release/reports/` abgelegt,
- `fake-cloud/server` oder `registry/core` kann diesen Report als Evidence
  referenzieren.

Danach kann QMP statt HMP ergaenzt werden.

### 8. Bare-Metal als Beobachtungsquelle in den Harness zurueckfuehren

Bare-Metal-Devflow sollte nicht direkt Runtime-Loading freischalten. Er sollte
zuerst reproduzierbare Fakten liefern:

- neuer serieller Befehl `snapshot` oder `inventory`,
- Host-Skript, das den TCP-/Datei-Serial-Log in einen Bare-Metal-Report
  ueberfuehrt,
- Mapping von Bare-Metal-Problem zu VM-Fixture-Preconditions.

## Naechste umsetzbare Tasks mit Dateihinweisen

1. **Protokoll-Schemas anlegen**

   Dateien:
   `device-protocol/agent-v0.md`,
   `device-protocol/module-manifest-v0.md`,
   `device-protocol/vm-test-report-v0.md`,
   `device-protocol/README.md`.

   Inhalt:
   JSON-Envelope, Request/Response/Error, `system.snapshot.v0`,
   Capability-Namen, Capability-Denial, Redaction-Klassen, `module_manifest.v0`
   und `seedos.vm_test_report.v0`.

2. **Read-only Snapshot aus bestehendem Statusmodell ableiten**

   Dateien:
   `seed-kernel/src/system_status.rs`,
   `seed-kernel/src/console.rs`,
   optional `seed-kernel/src/provider.rs`.

   Task:
   Einen seriellen Befehl `snapshot` oder `system.snapshot` hinzufuegen, der aus
   den vorhandenen Statusquellen ein begrenztes, secret-redacted JSON-aehnliches
   Objekt ausgibt. Kein Live-Loading, keine Persistenz, keine Secrets.

3. **Provider-Kontext erst nach Snapshot-Gate erweitern**

   Dateien:
   `seed-kernel/src/openai.rs`,
   `seed-kernel/src/provider.rs`,
   `seed-kernel/src/system_status.rs`.

   Task:
   Sobald `system.snapshot.v0` stabil ist, dem direkten Provider-Request eine
   kompakte, redacted Snapshot-Zusammenfassung beilegen. Nicht vorher weitere
   Tool-Aktionen an den Provider haengen.

4. **VM-Smoke in maschinenlesbaren Report ueberfuehren**

   Dateien:
   `vm-harness/openai-direct-smoke.ps1`,
   `vm-harness/README.md`,
   optional `release/reports/.gitkeep` oder ein dokumentierter Output-Pfad.

   Task:
   Neben den bestehenden seriellen Marker-Pruefungen einen
   `seedos.vm_test_report.v0` schreiben: Image-Hash, QEMU-Args-Hash,
   Hardwareprofil `qemu-e1000-usb-xhci-v0`, Commands, Predicates, Result,
   Serial-Log-Pfad.

5. **Harness-Fixture-V0 dokumentieren**

   Dateien:
   `vm-harness/README.md`,
   optional `vm-harness/fixtures/openai-direct-smoke.json`.

   Task:
   Erstes Fixture-Format definieren. Noch ohne volle QMP-Fault-Injection, aber
   mit Image, Serial-Port, Net-Mode, USB-Profil, erwarteten seriellen Markern
   und Timeout.

6. **Fake Cloud um read-only Control Messages erweitern**

   Dateien:
   `fake-cloud/server/src/lib.rs`,
   `fake-cloud/server/tests/ota_roundtrip.rs`,
   neuer Test z.B. `fake-cloud/server/tests/control_messages.rs`.

   Task:
   `inventory_request`, `log_append` und `module_propose` als deterministische
   V0-Nachrichten implementieren. `module_propose` darf zunaechst nur
   `accepted_for_test` oder `denied_missing_vm_report` antworten.

7. **Manifest-Modell typisieren**

   Dateien:
   `ota/cli/src/lib.rs`,
   `ota/cli/src/bin/mod-sign.rs`,
   `ota/cli/src/bin/mod-verify.rs`,
   `registry/core/src/lib.rs`.

   Task:
   Neben `SignedBlob` einen typed `ModuleManifestV0` einfuehren. `mod-sign`
   soll mindestens `manifest_version`, `module_id`, `module_version`, `risk`,
   `requested_caps`, `load_mode`, `base_image_hash` und optional
   `test_report_hash` schreiben. Registry-Index soll diese Felder sichtbar
   machen.

8. **Acceptance-/Attestation-Record hostseitig modellieren**

   Dateien:
   `registry/core/src/lib.rs`,
   `registry/README.md`,
   optional `registry/cli/src/main.rs`.

   Task:
   Einen `LocalAttestationV0` fuer Manifest-Hash, Artifact-Hash,
   Base-Image-Hash, VM-Test-Report-Hash, Approval-Quelle, Load-Mode und
   Rollback-Pointer definieren. Noch nicht vom Kernel konsumieren.

9. **Image-Layout-V0 spezifizieren**

   Dateien:
   neuer Plan/Spec z.B. `docs/image-layout-v0.md`,
   spaeter `scripts/package-stage0.ps1`,
   `scripts/write-stage0-usb.ps1`,
   `scripts/make-fat32-image.py`.

   Task:
   Single-FAT-Stage-0 explizit vom spaeteren ESP/DATA/A-B-Layout trennen.
   Erst nach der Spec optional ein Package-Flag fuer DATA-Skeleton einfuehren.

10. **`seed-runtime` zuerst als Lifecycle-Modell scaffolden**

    Dateien:
    `seed-runtime/Cargo.toml`,
    `seed-runtime/src/lib.rs`,
    Workspace `Cargo.toml`,
    `seed-runtime/README.md`.

    Task:
    Noch keine echte Wasm-Engine. Zuerst Service-Zustaende, Manifest-Referenz,
    Capability-Grants, Health-State und Denial-Gruende als host-testbare
    Datenmodelle. Danach kann entschieden werden, was davon in Stage-0,
    Stage-1 oder nur hostseitig gebraucht wird.

11. **Bare-Metal-Report-V0 vorbereiten**

    Dateien:
    `docs/BARE_METAL.md`,
    `scripts/run-stage0-baremetal-vm.ps1`,
    optional neues Skript `vm-harness/capture-serial-report.ps1`.

    Task:
    Definieren, wie ein Boot-Log oder TCP-Serial-Run in einen Report mit
    Hardwareprofil, PCI/USB-Fakten, Input-Status und Snapshot-Preconditions
    ueberfuehrt wird.

12. **Dokumentations-Drift bereinigen**

    Dateien:
    `docs/AI_Build_and_Test_Runbook.md`,
    `docs/invariant-choices.md`,
    `docs/sections/00-invariants.md`.

    Task:
    Checkboxes klar in "Invariant gewaehlt" vs. "implementiert/verifiziert"
    trennen. Aktuell koennen `[x]` bei Wasm Runtime, A/B-OTA, SPKI-WebSocket und
    Recovery so gelesen werden, als waeren sie implementiert.

