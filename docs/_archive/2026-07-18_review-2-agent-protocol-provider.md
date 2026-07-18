# Review 2: Agent-Protokoll, Provider-Pfad und Recovery

## Kurzfazit

Der aktuelle Stage-0-Stand erfuellt den direkten Provider-MVP deutlich besser
als der langfristige Agent-Plan: `ask <text>` bleibt im Gast, nutzt e1000,
DHCP/DNS, TCP, TLS 1.3, HTTPS und die OpenAI Responses API. Das ist eine starke
Basis fuer Phase 3/4.

Gleichzeitig ist der aktuelle Pfad noch kein raiOS-Agent-Protokoll. Er ist ein
fest verdrahteter Prompt-Client mit Human-Console-Ausgabe. Es fehlen
`raios.agent.v0`, `system.snapshot.v0`, Capability-Entscheidungen,
Provider-Redaction, Service-Inventar, Recovery-Lifeline und TLS-Trust. Der
Plan sollte deshalb die naechsten Schritte umordnen: erst TLS/SPKI
fail-closed, dann read-only Self-Description und Capability-Registry, danach
Tool-/Modul-Protokoll und erst spaeter Live-Loading.

## Bereits vorhandene Basis

- `seed-kernel/src/provider.rs` ist ein kleiner Provider-Boundary-Ansatz:
  `AgentRequest`, `SubmitError`, `Submitted`, `Event`, `submit()`,
  `poll()` und `snapshot()` kapseln den direkten OpenAI-Pfad hinter
  `Route::OpenAiDirect`.
- `seed-kernel/src/openai.rs` enthaelt die aktuelle Provider-State-Machine:
  `submit_request()` startet `Phase::Resolving`, `poll()` treibt DNS/TCP an,
  `perform_https_request()` fuehrt TLS/HTTPS aus, `build_request_body()`
  erstellt den Responses-Request, `read_http_response()` liest die Antwort,
  `extract_output_text()` extrahiert den ersten `output_text`.
- `seed-kernel/src/net.rs` liefert die MVP-Netzbasis: `init()` probt e1000,
  DHCP wird in `handle_dhcp_events()` verarbeitet, `resolve_hostname()` und
  `start_dns_query()` bauen den DNS-A-Record-Pfad, `tcp_connect_ipv4()`,
  `tcp_send()`, `tcp_recv()`, `tcp_abort()` und `tcp_snapshot()` stellen genau
  einen TCP-Kanal bereit.
- `seed-kernel/src/tls_io.rs` adaptiert smoltcp-TCP auf `embedded_io`:
  `KernelTcpStream::wait_for()` pollt `net::poll()` und bildet
  `TcpIoResult` auf blockierende `Read`/`Write`-Semantik ab.
- `seed-kernel/src/provider_config.rs` schuetzt die API-Key-Oberflaeche
  gegen direkte Ausgabe: `set_api_key()`, `clear_api_key()`,
  `copy_api_key()`, `api_key_set()` und `init_default_config()` halten den
  Key RAM-only oder explizit per lokalem Build-Env im Image.
- `seed-kernel/src/console.rs` hat brauchbare Nutzer- und Testoberflaechen:
  `command_provider_status()`, `command_openai_status()`,
  `command_ask()`, `submit_prompt()`, `show_setup_menu()` und
  `handle_api_key_byte()` machen Missing-Key, Busy-State und Providerstatus
  sichtbar.
- `seed-kernel/src/system_status.rs` ist ein guter Vorlaeufer fuer
  `system.snapshot.v0`: `SystemSnapshot::collect()` sammelt Framebuffer,
  Entropy, USB-xHCI, Wi-Fi, Network und Input bereits strukturiert fuer UI und
  Console.
- Die Doku-Basis ist konsistent in der Richtung: ADR 0001 verbietet Codex CLI
  im Kernel, ADR 0002
  (`docs/architecture-decisions/0002-agent-self-description-and-live-built-modules.md`)
  definiert Self-Description, Capabilities und lokale Attestation, ADR 0003
  trennt normale Agent-Service-Welt von Recovery-Agent-Lifeline.

## Abweichungen/Gaps

- `device-protocol/README.md` ist nur ein Platzhalter. Es gibt keine
  `device-protocol/agent-v0.md`, keine JSON-Schemas, keine Validatoren und
  keine dokumentierten Methoden fuer `system.snapshot`, `system.capabilities`,
  `device.graph`, `problem.list` oder Capability-Denials.
- `provider::submit()` akzeptiert zwar `AgentRequest { model, max_output }`,
  verwirft diese Felder aber sofort. `openai.rs` nutzt stattdessen hart
  verdrahtet `MODEL`, `max_output_tokens:128`, `API_HOST`, `API_PATH` und
  `Route::OpenAiDirect`.
- `openai::build_request_body()` sendet nur einen freien Prompt. Es gibt keine
  Einbettung eines redigierten `system.snapshot.v0`, keine Tool-Schemas, keine
  `raios.agent.v0`-Envelope und keine Korrelation von Provider-Antworten mit
  raiOS-Protokollmethoden.
- `console.rs` fuehrt Kommandos direkt aus. `ask`, `setup`, `provider`,
  `openai`, `wifi`, `status`, `devices` und `log` laufen ohne
  Capability-Check, Policy-Entscheidung oder auditierbares
  `capability_denied`-Ergebnis.
- Es gibt noch keine Capability-Registry. Die in ADR 0002 genannten Namen
  `cap.system.snapshot.read`, `cap.system.boot_log.read`,
  `cap.system.capabilities.read`, `cap.device.graph.read`,
  `cap.problem.list.read`, `cap.module.propose` und
  `cap.vm_test.report.read` existieren nicht im Code.
- `system_status::SystemSnapshot` ist strukturiert, aber nicht
  maschinenlesbar nach aussen exponiert. Die Console formatiert daraus
  Human-Text in `command_status()` und `command_devices()`; ein Agent muesste
  weiterhin Logs und UI-Zeilen interpretieren.
- Der Provider-Pfad ist noch kein Service im Sinne von ADR 0003. `main.rs`
  ruft `provider::poll()` direkt aus `PeriodicTasks::run()` auf. Es gibt kein
  Service-Inventar, keinen Health-State pro Service, keine Crash Records,
  keine Last-Good-Version und keine Handle-Indirection fuer spaetere Hot-Swaps.
- Recovery-Agent-Lifeline fehlt vollstaendig. Wenn der normale Providerpfad,
  die UI oder ein anderer Kernteil haengt oder panikt, gibt es nur den
  Panic-Halt in `main.rs::panic()` und keine minimale Recovery-Control-Plane.
- `docs/invariant-choices.md` fordert fuer Networking "TLS sessions pinned via
  SHA-256 SPKI hash" und "WebSocket overlay for all control traffic". Der
  aktuelle Runtime-Pfad ist dagegen direkte HTTPS-POSTs an OpenAI. Der Plan
  muss klar zwischen direktem Provider-Prompt-Pfad und raiOS-Control-Plane
  unterscheiden.

## Risiken/Probleme

- Kritisch: `openai::perform_https_request()` nutzt
  `embedded_tls::blocking::NoVerify` und loggt selbst
  "certificate verification TODO". SNI wird gesetzt, aber das Zertifikat wird
  nicht verifiziert und kein SPKI-Pin geprueft. Damit ist der direkte
  Provider-Pfad MITM-faehig und sollte vor ernsthafter Nutzung fail-closed
  gemacht werden.
- Hoch: `tls_io::KernelTcpStream::wait_for()` ist blockierend und spinnt mit
  `net::poll()` bis zu den Read-/Write-Timeouts. Sobald
  `openai::poll()` in `perform_https_request()` wechselt, kann die kooperative
  Stage-0-Schleife UI, Input, Recovery und weitere Provideraktionen fuer lange
  Zeit nicht sauber bedienen.
- Hoch: Secrets bleiben laenger im Speicher als noetig. `provider_config`
  nullt den gespeicherten Key bei `clear_api_key()`, aber
  `openai::perform_https_request()` kopiert ihn in `let mut key = [0u8; 256]`
  und nullt diese lokale Kopie nach dem HTTPS-Write nicht. Der explizite
  Local-Image-Pfad per `RAIOS_DEFAULT_OPENAI_API_KEY` ist nuetzlich, muss aber
  im Plan als unsicherer Testmodus markiert bleiben.
- Mittel: `provider_config::copy_api_key()` kopiert in beliebig grosse
  Zielpuffer und gibt bei zu kleinem Puffer eine stillschweigend abgeschnittene
  Laenge zurueck. Aktuell ist der Zielpuffer in `openai.rs` gleich gross, aber
  die API ist fuer zukuenftige Call-Sites riskant.
- Mittel: HTTP und JSON sind handgerollt. `read_http_response()`,
  `http_response_complete()`, `decode_chunked()`,
  `extract_json_string_after()` und `extract_output_text()` reichen fuer den
  Smoke, aber nicht fuer robuste Provider-Fehler, Streaming, mehrere
  Output-Items, grosse Antworten oder strukturierte Tool-Calls.
- Mittel: `net.rs` hat global genau einen TCP-Socket und einen DNS-Query-State.
  Das passt fuer einen Request, verhindert aber parallele Provider-,
  Recovery-, WebSocket- oder Tool-Verbindungen.
- Mittel: Datenklassifikation fehlt. ADR 0002 fordert
  `public`/`local_only`/`secret` fuer Felder, die den Provider verlassen.
  Aktuell gibt es nur ad-hoc Maskierung der API-Key-Eingabe und
  `SET`/`MISSING`-Status. Fuer `system.snapshot.v0` muss vor der ersten
  Provider-Kontextinjektion klar sein, welche Felder niemals gesendet werden.
- Niedrig bis mittel: `MODEL` ist hart in `openai.rs` kodiert. Selbst wenn der
  Wert gerade passt, gehoert Modellwahl in `provider_config` oder eine
  Provider-Policy, weil sonst jedes Modellupdate ein Kernel-Update ist.

## Konkrete Plan-Aenderungsvorschlaege

1. Phase 3/4 vorerst in zwei Gates teilen:
   - Gate 3a: direkter Providerpfad funktioniert nur mit TLS-Verifikation oder
     SPKI-Pin. `NoVerify` darf danach nur noch hinter einem eindeutig benannten
     Test-Build-Flag erreichbar sein.
   - Gate 3b: Provideranfragen koennen eine redigierte read-only
     `system.snapshot.v0` beilegen, aber noch keine mutierenden Tools
     ausfuehren.
2. `docs/invariant-choices.md` praezisieren: WebSocket bleibt die
   raiOS-Control-Plane fuer Fake-Cloud/OTA/Module, aber direkter
   Provider-HTTPS ist fuer den "no dedicated custom cloud server"-MVP erlaubt.
   Gemeinsame Pflicht bleibt: TLS fail-closed, Pin oder CA-Verification,
   strukturierte Envelope fuer raiOS-Toolverkehr.
3. ADR 0002 als kurzfristige Protokollquelle vor ADR 0003 umsetzen:
   erst `system.describe`, `system.snapshot`, `system.capabilities`,
   `system.boot_log`, `device.graph`, `problem.list`; `module.load_ephemeral`
   und `module.persist` bleiben explizit denied, bis VM-Testbericht und lokale
   Attestation existieren.
4. Capability-Modell in den Kernel einziehen, aber zuerst read-only:
   statische Cap-Liste, Datenklassifikation, lokale Policy und Auditlog. Keine
   generische Shell-/Command-Capability einfuehren.
5. Provider als ersetzbaren Service vorbereiten, ohne sofort dynamisches Laden
   zu bauen: `provider.rs` sollte Service-Metadaten, Health, Last Error,
   Endpoint, Modell, TLS-Trust-State und Capabilities melden. Das schafft die
   spaetere ADR-0003-Servicegrenze, ohne den Boot-MVP zu zerlegen.
6. Recovery-Lifeline nicht aus dem normalen OpenAI-Chat ableiten. Im Plan
   separat definieren: minimales Protokoll, minimaler Snapshot, erlaubte
   Recovery-Actions, eigener Trust-State, keine allgemeine Chat- oder
   Tool-Ausfuehrung.
7. Secrets-Plan schaerfen: RAM-only bleibt MVP, aber alle Kopien muessen
   zeroized werden; eingebettete Keys sind nur lokale Testimages; persistente
   Secrets brauchen eigene ADR/Policy vor Bare-Metal-Nutzung.

## Naechste umsetzbare Tasks mit Dateihinweisen

1. Protokoll-V0 dokumentieren:
   - `device-protocol/agent-v0.md`: Envelope `{v,t,id,ts,body}`, request,
     response, error, `capability_denied`, Methodenliste und Beispiele.
   - `device-protocol/system-snapshot-v0.md`: Felder, Datentypen,
     Datenklassifikation, Redaction-Regeln.
   - `device-protocol/capabilities-v0.md`: Cap-Namen aus ADR 0002, Risiko,
     Grant-Regeln, Audit-Level.
2. Read-only Snapshot im Kernel vorbereiten:
   - Neuer Codepfad z.B. `seed-kernel/src/agent_protocol.rs`.
   - Inputs aus `system_status::SystemSnapshot::collect()`,
     `provider::snapshot()`, `net::ui_snapshot()`, `net::tcp_snapshot()`,
     `wifi::snapshot()` und `usb::snapshot()`.
   - Erste Ausgabe als Console-Befehl in `seed-kernel/src/console.rs`, z.B.
     `snapshot`, bevor sie in Provider-Kontext oder WebSocket fliesst.
3. Capability-Registry einziehen:
   - Neuer Codepfad z.B. `seed-kernel/src/capabilities.rs`.
   - Statische Grants fuer read-only Caps.
   - `console.rs::execute()` und spaeter ein Agent-Dispatcher muessen vor
     protocolisierten Aktionen `capabilities::check()` aufrufen.
   - Mutierende Methoden wie `module.load_ephemeral`, `module.persist`,
     `apply_config` initial immer mit `capability_denied` beantworten.
4. TLS-Hardening des OpenAI-Pfads:
   - `seed-kernel/src/openai.rs::perform_https_request()` von `NoVerify` auf
     eine echte Verifikation umstellen.
   - Pragmatischer erster Schritt: eigener `TlsVerifier` fuer `api.openai.com`
     mit festem SPKI- oder Zertifikat-Pin, Trust-State in `provider::snapshot()`
     sichtbar machen.
   - `vm-harness/openai-direct-smoke.ps1` um einen Log-Marker wie
     `openai: TLS certificate verified` oder `openai: TLS SPKI pin matched`
     erweitern.
5. Secrets-Kopien begrenzen:
   - `seed-kernel/src/openai.rs::perform_https_request()` lokale `key`-Kopie
     nach dem Header-Write und vor jedem Return nullen.
   - `provider_config::copy_api_key()` in `seed-kernel/src/provider_config.rs`
     so aendern, dass zu kleine Zielpuffer einen Fehler liefern statt
     abzuschneiden.
6. Provider-Konfiguration aus dem OpenAI-Client herausziehen:
   - `seed-kernel/src/provider_config.rs` um Modell, max output, Endpoint und
     Trust-Mode erweitern.
   - `seed-kernel/src/provider.rs::submit()` darf `AgentRequest.model` und
     `AgentRequest.max_output` nicht mehr ignorieren.
   - `seed-kernel/src/openai.rs::build_request_body()` soll Werte aus der
     validierten Provider-Config verwenden.
7. Provider-Kontextinjektion read-only testen:
   - Nach Task 1/2 `openai::build_request_body()` oder einen neuen Builder so
     erweitern, dass ein redigierter `system.snapshot.v0` als Kontext
     mitgesendet werden kann.
   - Keine Secrets, keine MAC/IP-Felder ohne explizite Klassifikation senden.
8. Blockierenden HTTPS-Pfad isolieren:
   - Kurzfristig: Timeouts und UI/Console-Hinweise in
     `openai::perform_https_request()` und `tls_io::KernelTcpStream::wait_for()`
     genauer melden.
   - Mittelfristig: HTTPS/TLS in eine nicht-blockierende State-Machine oder
     einen separaten Provider-Service ueberfuehren, damit die spaetere
     Recovery-Lifeline nicht vom normalen Chat-Request blockiert wird.
9. Service-Inventar als ADR-0003-Bruecke:
   - Neuer Snapshot-Teil fuer `core`, `ui`, `console`, `input`, `usb`,
     `network`, `wifi`, `provider.openai`.
   - Zunaechst statisch, aber mit Health, Last Error und Replaceable-Flag.
   - Spaeter Basis fuer `system.snapshot.v0` und Recovery-Befehle.
10. Recovery-Lifeline spezifizieren, noch nicht gross implementieren:
    - Neues Doku-File z.B. `device-protocol/recovery-v0.md`.
    - Erlaubte Methoden: `recovery.snapshot`, `recovery.restart_service`,
      `recovery.disable_module`, `recovery.rollback_last_good`,
      `recovery.load_artifact_by_hash`.
    - Explizit getrennt von `provider.rs`/`openai.rs`; keine normale Chat-Route
      und keine allgemeinen Tools.
