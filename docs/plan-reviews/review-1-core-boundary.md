# Review 1: Core Boundary, Eventloop, Snapshot, UI/Console, Service Inventory

## Kurzfazit

Die aktuelle Codebase stuetzt den langfristigen Plan als Stage-0-MVP bereits gut:
Boot, framebuffer UI, Console, Input, USB, e1000/smoltcp, Wi-Fi-Probe und
OpenAI-Direct sind in klar benannten Kernel-Modulen vorhanden. Der wichtigste
Fortschritt fuer die naechste Planphase ist aber noch nicht umgesetzt: Es gibt
noch keine explizite Core/World-Grenze, kein maschinenlesbares
`system.snapshot.v0` und kein Service-Inventar. `SystemSnapshot` ist heute eine
UI-/Console-Statuszeilen-Abstraktion, noch nicht das stabile Agent-Protokoll.

Der pragmatische naechste Schritt sollte kein dynamisches Laden sein, sondern ein
statisches Service-Modell in der laufenden Kernel-Codebase: feste Service-IDs,
Health/Last-Error, Capability-Namen und ein typed Snapshot, aus dem UI und
Console weiter ihre heutigen Zeilen ableiten koennen.

## Was im Code den Plan bereits stuetzt

- `seed-kernel/src/main.rs::_start` und `early_main` bilden eine gut lesbare
  Stage-0-Bootkette: SSE aktivieren, Heap, Serial, Memory/HHDM, Limine
  framebuffer, Provider-Konfig, Console, USB, Wi-Fi-Probe, UI, TSC, Entropy,
  Input und Netzwerk. Das ist als aktueller Monolith nachvollziehbar und
  boot-testbar.

- `seed-kernel/src/main.rs::PeriodicTasks` ist bereits die Stelle, an der eine
  spaetere Service-Registry andocken kann. Die Tasks `console`, `entropy`,
  `net`, `input`, `usb_rescan`, `provider` und `ui` sind faktisch statische
  Services mit festen Poll-Intervallen.

- `seed-kernel/src/scheduler.rs::PeriodicTask` kapselt periodische Arbeit klein
  und auditierbar. Das passt zur Phase vor echten Threads/Interrupt-getriebenen
  Services.

- `seed-kernel/src/system_status.rs::SystemSnapshot::collect` sammelt
  framebuffer, entropy, USB-xHCI, Wi-Fi, network und input an einer zentralen
  Stelle. `seed-kernel/src/ui.rs::StatusUi::render_inner` und
  `seed-kernel/src/console.rs::command_status`/`command_devices` benutzen diese
  Quelle bereits gemeinsam. Das reduziert Log-Scraping und ist der richtige
  Ansatz fuer `system.snapshot.v0`.

- Viele Module haben bereits Snapshot- oder Status-Funktionen, die in ein
  Service-Inventar aufgenommen werden koennen:
  `usb::snapshot`, `wifi::snapshot`, `net::ui_snapshot`, `net::tcp_snapshot`,
  `provider::snapshot`, `openai::snapshot`, `input::device_detail` und
  `entropy::stats`.

- `seed-kernel/src/provider.rs` ist eine kleine Provider-Fassade mit
  `AgentRequest`, `Route`, `Submitted`, `SubmitError`, `Event` und `Snapshot`.
  Das passt zur ADR-0001-Entscheidung, nicht Codex CLI in Stage-0 zu portieren,
  sondern einen nativen schmalen Provider-/Agent-Boundary zu bauen.

- `seed-kernel/src/provider_config.rs` haelt die OpenAI API key state RAM-only
  und gibt ueber `Snapshot` nur `api_key_set` aus. `console.rs` maskiert
  API-Key- und Wi-Fi-Key-Eingabe. Das ist eine gute Vorstufe fuer die in
  ADR-0002 geforderte Feldklassifikation (`public`, `local_only`, `secret`).

- `seed-kernel/src/ui.rs` ist bereits chat-first, aber nicht an Provider-Details
  hart verdrahtet genug, dass es untrennbar waere. `draw_chat`, `draw_console`
  und `draw_settings` konsumieren `ConsoleSnapshot` plus `SystemSnapshot`; das
  kann spaeter auf typed Snapshot-Felder umgestellt werden.

- Host-seitig existieren erste Bausteine fuer spaetere Artefakte:
  `registry/core/src/lib.rs` verwaltet signierte Blobs/Manifeste, `modules/hello-ui`
  ist ein Platzhaltermodul, und `device-protocol/README.md` beschreibt den
  JSON-Envelope als Ziel. Diese Teile sind noch nicht mit dem Kernel verbunden,
  aber sie zeigen die Richtung.

## Abweichungen/Gaps

- Die Core/World-Grenze ist im Code noch nicht explizit. Laut ADR-0003 sollten
  UI, Console, Input, USB, Netzwerk, Wi-Fi und Provider austauschbare Services
  werden. Aktuell sind sie alle statisch in `seed-kernel` gelinkt und ueber
  globale `static`/`Mutex`-States erreichbar, z.B. `usb::STATE`,
  `net::NET_STATE`, `provider_config::STATE`, `openai::STATE` und
  `console::CONSOLE`.

- `SystemSnapshot` ist noch kein `system.snapshot.v0`. Es besteht aus
  `StatusLine { label, state, detail }` mit frei formatierten `TextBuf<128>`-
  Details. Es fehlen die ADR-0002-Felder `schema`, `os.name`, `stage`,
  `kernel_build_id`, `image_hash`, typed `network.dns`, `provider`, `problems`
  und `capabilities`.

- UI und Console konsumieren heute Statuszeilen, nicht typed Facts.
  `system_status::network_line` formatiert z.B. IP/Gateway direkt in Text;
  `usb_xhci_line` packt Controller-, Hub-, HID- und Error-Details in eine lange
  Zeile. Das ist fuer Menschen gut, fuer Agent-Protokoll und Service-Inventar
  fragil.

- Es gibt noch kein Service-Inventar mit `service_id`, `kind`, `version`,
  `health`, `last_error`, `capabilities`, `replaceable` oder `core_owned`.
  Dadurch kann ein Agent noch nicht fragen, was laeuft, was degraded ist und
  welche Aktionen erlaubt sind, ohne menschliche Console-Ausgabe zu interpretieren.

- Es gibt noch keine Capability-Tabelle im Kernel. ADR-0002 nennt
  `cap.system.snapshot.read`, `cap.system.boot_log.read`,
  `cap.system.capabilities.read`, `cap.device.graph.read`,
  `cap.problem.list.read` und `cap.module.propose`; im Code kommen diese Namen
  noch nicht vor.

- Die Agent-/Provider-Anfrage ist noch Prompt-only. `provider::AgentRequest`
  traegt `prompt`, optional `model` und `max_output`, aber keine Tool-Calls,
  keine Methodennamen wie `system.snapshot`, keine Capability-Pruefung und noch
  keine redigierte Snapshot-Kontextinjektion.

- `openai::perform_https_request` baut direkt einen Responses-API-Body aus dem
  Prompt. Der Provider bekommt noch keinen kompakten, klassifizierten
  Systemkontext. Damit bleibt der Chat fuer Diagnosefragen blind, obwohl der
  Kernel lokal schon viele Fakten kennt.

- `console.rs::write_event` erkennt Assistant-Antworten ueber den Textprefix
  `"OPENAI: "`. Das ist als MVP okay, aber als Service-/Agent-Protokoll ein
  Anti-Pattern: Event-Art und Payload sollten typed sein, nicht aus Logtext
  abgeleitet.

- `command_log` gibt nur den Console-Ring aus. Ein echtes
  `system.boot_log`/`cap.system.boot_log.read` existiert nicht; die fruehen
  Serial-Zeilen sind nicht als strukturierter Bootlog im Kernel abrufbar.

- `device-protocol/README.md` ist noch Platzhalter. Die in ADR-0002 genannten
  Dateien `agent-v0.md`, `module-manifest-v0.md` und
  `vm-test-harness-v0.md` fehlen noch.

## Risiken/Probleme

- Der normale Provider-Pfad ist noch Teil des Kernel-Monolithen. Nach ADR-0003
  soll OpenAI/Provider ein austauschbarer Service sein, nicht Core-Identitaet.
  Solange `openai.rs` direkt im Main-Loop laeuft, ist die Trennung nur
  konzeptionell.

- `openai::poll` wechselt beim verbundenen TCP-Socket in
  `perform_https_request`, und dieser Pfad blockiert synchron mit
  `KernelTcpStream::wait_for`. Dabei wird zwar `net::poll` weitergerufen, aber
  UI, Console, Input, USB-Rescan und Provider-Health laufen bis zum Ende oder
  Timeout nicht normal weiter. Das widerspricht dem Always-on-Core-Ziel, bei dem
  ein haengender Provider-Service die Kontrollflaeche nicht blockieren darf.

- TLS nutzt in `openai.rs` weiterhin `NoVerify` und loggt selbst
  "certificate verification TODO". Das ist bereits im Projektstatus bekannt,
  bleibt aber ein Boundary-Risiko: Wenn Snapshot/Agent-Context an den Provider
  geht, muss vorher klar sein, welcher Provider wirklich erreicht wurde oder
  welche Pinning-Policy gilt.

- Die aktuellen Singleton-Driver (`usb`, `net`, `e1000`, `wifi`) haben keine
  Handle-Indirection und keine versionierten State-Objekte. Spaeteres Hot-Swap
  wird schwer, wenn Clients weiter direkt globale Funktionen statt Service- oder
  Capability-Handles benutzen.

- Es gibt keine Crash-Isolation. `panic` in `main.rs` haelt den Kernel an; ein
  Fehler in UI, Provider, USB oder Netzwerk ist noch nicht als Service-Crash mit
  `last_error` und Restart-/Rollback-Entscheidung modellierbar.

- Statusdetails koennen durch feste Puffer (`TextBuf<128>`, `FixedLine`,
  `ConsoleLine`) gekuerzt werden. Fuer UI ist das akzeptabel, fuer
  maschinenlesbare Diagnose darf die kanonische Quelle nicht ein potentiell
  abgeschnittener Text sein.

- `net::dhcp_poll_enabled` gibt konstant `true` zurueck, und
  `NetUiSnapshot` enthaelt keine DNS-Server, DHCP-Phase, Failure/Timeout-State
  oder Packet-Counter. Das limitiert `system.snapshot.v0` und `problem.list`
  direkt.

## Konkrete Plan-Aenderungsvorschlaege

- Phase 5 sollte als "Static Core/Service Model" konkretisiert werden, bevor
  Phase 6 Live-Services startet. Ziel: Alle bestehenden Kernel-Komponenten
  bleiben statisch gelinkt, bekommen aber feste Service-IDs, Health-State,
  Last-Error und Capability-Namen.

- Der Plan sollte eine klare Stage-0-Core-Liste festlegen. Vorschlag:
  `core.boot`, `core.memory`, `core.serial`, `core.scheduler`,
  `core.entropy`, `core.snapshot_root`, `core.recovery_console_min`.
  Alles andere wird auch dann als Service inventarisiert, wenn es noch statisch
  im Kernel liegt: `svc.ui.framebuffer`, `svc.console`, `svc.input`,
  `drv.usb.xhci`, `drv.net.e1000`, `svc.net.ipv4`, `drv.wifi.avastar_probe`,
  `svc.provider.openai_direct`.

- `SystemSnapshot` sollte in zwei Schichten aufgeteilt werden:
  typed Facts als kanonische Quelle, daraus abgeleitete `StatusLine`s fuer UI und
  Console. Dadurch koennen bestehende Ansichten stabil bleiben, waehrend
  `system.snapshot.v0` maschinenlesbar wird.

- Das erste Agent-Protokoll sollte read-only bleiben:
  `system.snapshot`, `system.capabilities`, `system.boot_log`, `device.graph`,
  `problem.list`, `service.inventory`. `module.propose` kann danach als denied
  oder workstation-only Flow folgen. Kein `module.load_ephemeral`, bevor
  Manifest, VM-Testreport und lokale Attestation existieren.

- Provider-Hardening und Service-Inventar sollten als gekoppelte Milestones
  behandelt werden. TLS-Pinning/Zertifikatspruefung schuetzt den Transport;
  Feldklassifikation schuetzt die Snapshot-Inhalte. Erst danach sollte
  `provider::submit` automatisch Systemkontext an OpenAI anhaengen.

- Der Plan sollte explizit sagen: Die heutige OpenAI-Direct-Implementierung ist
  ein normaler Service-Kandidat, nicht die Recovery-Lifeline. Eine spaetere
  Recovery-Lifeline braucht einen kleineren, getrennten Kontrollpfad.

- UI/Console-Polish sollte nicht weiter eigene Statuslogik aufbauen. Neue
  Anzeigen und Befehle sollten aus `system.snapshot.v0` oder
  `service.inventory` abgeleitet werden, damit keine zweite Wahrheit entsteht.

## Naechste umsetzbare Tasks mit Dateihinweisen

1. `device-protocol/agent-v0.md` anlegen und den Envelope plus read-only
   Methoden definieren: `system.snapshot`, `system.capabilities`,
   `system.boot_log`, `device.graph`, `problem.list`, `service.inventory`.
   Dazu konkrete Beispielantworten fuer die aktuelle QEMU-e1000/xHCI-Konfiguration.

2. `seed-kernel/src/service_inventory.rs` neu einfuehren: Typen wie
   `ServiceId`, `ServiceKind`, `ServiceHealth`, `ServiceDescriptor`,
   `ServiceRuntime`, `CapabilityName`. Start nur mit statischen Deskriptoren,
   ohne dynamisches Laden.

3. `seed-kernel/src/main.rs::PeriodicTasks` so vorbereiten, dass jeder Poll-Task
   eine Service-ID hat und Health/Last-Error aktualisieren kann. Zunaechst
   reichen `Ready`, `Waiting`, `Degraded`, `Missing` und `last_error:
   Option<&'static str>`.

4. `seed-kernel/src/system_status.rs` in typed Snapshot plus Display-Zeilen
   aufteilen. Vorschlag: `SystemFacts`/`SystemSnapshotV0` sammelt OS, status,
   network, provider, services, capabilities und problems; `StatusLine` wird
   daraus fuer UI/Console gebaut.

5. `seed-kernel/src/net.rs::NetUiSnapshot` erweitern um DNS-Server,
   DHCP-Status, DHCP-Timeout/Failure und einfache RX/TX-Zaehler, soweit aus
   `e1000.rs`/smoltcp sinnvoll verfuegbar. Das ist fuer `network` und
   `problem.list` wichtiger als weitere UI-Formatierung.

6. `seed-kernel/src/console.rs` um deterministische read-only Befehle erweitern:
   `snapshot`, `services`, `caps`, optional `problems`. Diese sollten nicht die
   bestehenden Human-Zeilen parsen, sondern typed Snapshot/Inventory lesen.

7. `seed-kernel/src/ui.rs` nachziehen, sodass `draw_status_strip` und
   `draw_status_detail` weiter gleich aussehen, aber ihre Daten ueber die neue
   Snapshot-Schicht beziehen. Keine neue UI-Wahrheit neben `SystemSnapshotV0`.

8. `seed-kernel/src/provider.rs` und `seed-kernel/src/openai.rs` fuer spaetere
   Kontextinjektion vorbereiten: `AgentRequest` sollte neben `prompt` optional
   einen redigierten Snapshot-/Tool-Kontext tragen koennen. Noch nicht senden,
   bevor TLS-Verifikation/Pinning und Feldklassifikation stehen.

9. `seed-kernel/src/console.rs::write_event` und `provider.rs::Event` von
   Prefix-Erkennung auf typed Events vorbereiten, z.B. `ProviderEventKind` mit
   `AssistantText`, `Status`, `Error`. Danach muss die Chat-UI nicht mehr auf
   `"OPENAI: "` als Protokollersatz vertrauen.

10. `docs/ROADMAP.md` nachziehen: Zwischen Phase 5 und Phase 6 eine explizite
    Subphase "static service inventory and snapshot v0" aufnehmen. Das
    verhindert, dass Live-Loading begonnen wird, bevor die Beobachtungs- und
    Capability-Basis stabil ist.
