# raiOS Vision-Plan (All-over-Plan)

Stand: 2026-07-15. Owner-Dokument, deutsch. Der operative Tages-Cursor bleibt in
`docs/ROADMAP.md`; dieses Dokument ist die Landkarte darüber: von heute bis zur
Vision, in ehrlichen Stufen. Es verspricht keine Termine und keine Magie.

## 1. Die Vision in einem Satz

> Ich sage raiOS in der Genesis-Shell, was es werden soll — und raiOS baut es
> selbst: Der Agent schreibt echten Code, das System prüft, baut, signiert und
> installiert ihn, und jede Stufe bleibt beweispflichtig, eingesperrt und von
> mir physisch freigegeben.

## 2. Was NIE verhandelt wird (die Verfassung)

Diese fünf Regeln gelten auf jeder Stufe dieses Plans, sonst ist es nicht raiOS:

1. **Beweis vor Behauptung.** Nichts gilt als fertig ohne grünen Beweis-Lauf.
2. **Sandbox als Normalfall.** Programme und Dienste laufen eingesperrt (Wasm,
   begrenzte Imports); Ausnahmen (Treiber) sind eigene, einzeln freigegebene
   Projekte.
3. **Physische Freigabe.** Kein Programm läuft, keine Installation geschieht
   ohne echten Klick/Tastendruck des Owners auf dem Gerät.
4. **Nachrechnen statt Vertrauen.** Werkzeuge (auch der Compiler!) bekommen kein
   Vertrauen — ihre ERGEBNISSE werden reproduzierbar nachgeprüft
   (Doppel-Build, Byte-Identität, Fingerabdrücke, Rollback).
5. **Ehrliche Lücken.** Was fehlt, steht mit Namen in den Docs — nie hinter
   einem Fallback versteckt.

## 3. Wo wir heute stehen (die kurze ehrliche Karte)

Existiert und ist bewiesen (Details/Reports: `docs/PROJECT_STATUS.md`):

- **Genesis-Shell** (kernbesessen, unaustauschbar) mit Sicherheits-Streifen,
  F12-Notausgang, Recovery-Leine, Setup-Flüssen, deutscher Tastatur (Basis).
- **Programm-Schnellspur:** begrenzte Programm-Sprache (RUIP) → lokale
  Kompilierung → inerter Entwurf → physischer Klick → Sandbox-Lauf. Zwei echte
  Programme: Taschenrechner, Text-Editor (2026-07-14, EDITOR-1).
- **Code-Spur (W1–W6):** echter Quellcode → unveränderlich abgelegt →
  reproduzierbar zu Wasm gebaut (extern, vermessen) → signiert → Klick →
  dauerhafte Installation mit Autoload und automatischem Rollback.
- **Netz-Fundament (NET-1..8):** eng geführte net./crypto./acquire.-Fähigkeiten
  für Wasm-Dienste, F12-abbrechbar; Quarantäne-Download W7 scharfgeschaltet,
  Live-Beweis noch offen (ehrlich dokumentiert).
- **Gedächtnis/Recovery:** dauerhafte typisierte Records (M9), Recovery-Leine
  (M8), Promotion/Rollback (M6/M7).
- **Hardware:** Surface-Boot mit Framebuffer/USB/RNG; WiFi-Chip erkannt,
  Treiber-Seitenstrang begonnen; USB-Stick-Persistenz (G7) Surface-gated offen.

## 4. Die sechs Säulen bis zur Vision

Jede Säule wächst im bewährten Slice-Verfahren (Scope-Doc → Worker-Pakete →
fokussierter Beweis-Lauf → Block-Close mit full+recovery). Reihenfolge in §5.

### Säule P1 — Programme & Oberfläche
Vom Einzelprogramm zum Desktop (ohne GPU-Zwang):
- P1.1 **Programm-Persistenz:** freigegebene Programm-Entwürfe an die
  vorhandene W6-Install-Maschinerie anschließen (Editor überlebt Reboot).
  *Kleinster nächster Schritt, vom Owner bereits erfragt.*
- P1.2 **Editor-Komfort:** Cursor-Navigation (Pfeile), Umlaute/AltGr,
  mehrere Textfelder.
- P1.3 **Fenster-Desktop v1:** mehrere Programme gleichzeitig sichtbar,
  Fenster-Verwalter im Kern (Genesis bleibt Hausherr; Programme bekommen
  Flächen statt Vollbild), Task-Leiste, Programm-Wechsel per Taste.
- P1.4 **Programm-zu-Programm-Daten** (bounded, typed, owner-sichtbar) —
  erst wenn echte Programme es brauchen.

### Säule P2 — Selbstversorgung (Beschaffung)
Das System besorgt sich Code-Kandidaten selbst — in Quarantäne:
- P2.1 **W7 live beweisen:** der scharfgeschaltete Download lädt real
  (Dispatch-Bug beheben, echter Server), Kandidat bleibt inert.
- P2.2 **Download → Lauf/Install:** heruntergeladene Kandidaten durch die
  EXISTIERENDE M6/W6-Maschinerie führen (kein zweiter Weg, keine Aufweichung).
- P2.3 **Abhängigkeits-Quarantäne:** ganze Projekt-Abhängigkeiten (Crates)
  geprüft beschaffen; Registry-/Herkunfts-Vertrauen (M12-Linie).

### Säule P3 — Der Agent IM System
Vom Chat-Spielzeug zum System-Baumeister (immer über die dicke Schleuse):
- P3.1 **KI schreibt in den Workspace:** Provider-Antworten werden
  Quell-DATEIEN im W1-Workspace (Kandidaten, niemals direkt ausführbar).
- P3.2 **Agent-Schleife mit Beweisen:** Der Agent bekommt die Testresultate
  seiner Kandidaten zurück und iteriert — das System bleibt der Prüfer.
- P3.3 **`/build`-Slow-Lane in Genesis:** „bau mir X" erzeugt sichtbar
  einen Workspace-Auftrag (statt nur RUIP), mit Status im UI und Freigabe
  am Ende. Die Schnellspur (RUIP) bleibt für Kleines bestehen.

### Säule P4 — Der Bauplatz (Compiler-Leiter)
Owner-Frage 2026-07-15 („Compiler nachbauen unmöglich?") — Antwort: nachbauen
unmöglich und unnötig; wir spannen den ECHTEN rustc gestuft ein:
- P4.A **Versiegelter Extern-Bau (EXISTIERT):** vermessene, festgenagelte
  Toolchain auf dem Owner-Rechner; Vertrauen durch reproduzierbaren
  Doppel-Build + Fingerabdruck (W4-Beweis erbracht).
- P4.B **raiOS-gesteuerter Bauplatz:** dedizierte Build-Maschine/VM, die NUR
  baut; raiOS beauftragt, misst und verifiziert (gleiche Beweiskette,
  mehr Autonomie).
- P4.C **Compiler als Wasm-Gast IM System:** rustc (plausibelster Weg:
  Cranelift-Backend) als signierter Sandbox-Gast — braucht eine neue
  Gast-Klasse (großer Speicher, begrenzte Datei-Imports, Geduld bei der
  Geschwindigkeit). Ehrlich: Forschungsanteil; ändert NICHTS am
  Vertrauensmodell, denn auch dieser Compiler wird nur nachgerechnet.
- P4.Z **Zwischenstufe Mini-Sprache:** die vorhandene RUIP-Sprache wächst
  kontrolliert weiter (Editor war Schritt 1); kleine Programme entstehen so
  komplett IM System, ohne auf P4.C zu warten.

### Säule P5 — Hardware
- P5.1 **WiFi fertig:** Association, PORT_RELEASE, RX/TX, DHCP auf dem
  Surface (Seitenstrang läuft; Firmware-Blob bleibt als unauditiert benannt).
- P5.2 **Physische Persistenz:** G7-Stick (erst read-only Preflight), dann
  dauerhafte Datenträger-Autorität mit Owner-Siegel.
- P5.3 **GPU, bescheiden beginnen:** Modesetting + schnelles 2D zuerst
  (Desktop-Nutzen!), 3D/Beschleunigung ausdrücklich SPÄTER; Treiber wie
  WiFi als eigener, einzeln freigegebener Seitenstrang.
- P5.4 **TPM/Owner-Siegel-Hardware** (Schlüssel-Custody am Gerät).

### Säule P6 — Vertrauen & Betrieb
- P6.1 **Owner-Sealing statt Dev-Keys:** echte Schlüssel-Zeremonie; alles,
  was heute `dev_key_not_owner_sealed` trägt, wird umgezogen.
- P6.2 **Volle TLS-Wahrheit:** WebPKI-Kette + vertrauenswürdige Zeit
  (heute: Pin/SPKI, ehrlich benannt).
- P6.3 **ADR-0004-Vollausbau:** raiOS ist das Gedächtnis — typisierte Fakten
  mit Herkunft, Kontext-Broker, Provider-Redaktion (Fundament existiert).

## 5. Reihenfolge (was zuerst, und warum)

**Nah (die nächsten Blöcke):**
1. P2.1 W7-Live-Beweis (war schon der geplante nächste Block) —
   Selbstversorgung braucht diesen Anker.
2. P1.1 Programm-Persistenz (Owner-Wunsch, klein, nutzt Bestehendes).
3. P2.2 Download→Lauf/Install (schließt die Beschaffungs-Schleife).
4. P1.2 Editor-Komfort (parallelisierbar als UI-Spur).

**Mittel:**
5. P3.1+P3.2 Agent→Workspace + Beweis-Schleife (die Vision wird sichtbar:
   KI schreibt Code, System prüft).
6. P1.3 Fenster-Desktop v1 (ohne GPU).
7. P5.1 WiFi fertig; danach P5.2 Stick-Persistenz.
8. P4.B raiOS-gesteuerter Bauplatz.

**Weit (echte Großprojekte, einzeln zu entscheiden):**
9. P4.C Compiler als Wasm-Gast.
10. P5.3 GPU (Modesetting/2D zuerst).
11. P6.1 Owner-Sealing-Vollumzug + P6.2 TLS-Vollausbau.

Parallel-Regel bleibt: getrennte Schreibmengen, eine QEMU-Suite, Hardware-
Schritte Surface-gated.

## 6. Was dieser Plan bewusst NICHT verspricht

- Keine Termine (jeder Block endet erst mit grünem Beweis).
- Keine „KI schreibt fehlerfreien Code"-Magie: das System beweist
  Begrenztheit, Reproduzierbarkeit und Rücknahmefähigkeit — inhaltliche
  Korrektheit sichern Tests, Sandbox, Freigabe und Rollback.
- Kein 3D/GPU-Sprint, kein stilles Aufweichen der Verfassung (§2) — auch
  nicht, wenn es schneller ginge.

## 7. Pflege dieses Dokuments

Bei jedem Block-Close, der eine Säule bewegt, wird hier der Stufen-Status
aktualisiert (eine Zeile, mit Report-Namen). Detailstand bleibt in
`docs/PROJECT_STATUS.md`, Tages-Cursor in `docs/ROADMAP.md`.
