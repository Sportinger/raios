# raiOS Vision-Plan (All-over-Plan)

Stand: 2026-07-17 (v2 — nach Owner-Korrektur; v1 hatte fälschlich fertige
Features wie Desktop/GPU als Bauaufträge geführt). Owner-Dokument, deutsch.
Der operative Tages-Cursor bleibt in `docs/ROADMAP.md`.

## 1. Die Vision in einem Satz (korrigiert)

> raiOS ist ein maximal leichtes, kleines System, das — einmal mit dem
> Internet verbunden — **aus sich selbst heraus** Programme bauen kann,
> bis hin zu einem Desktop mit GPU-Unterstützung. Wir bauen NICHT diese
> Programme. Wir bauen die perfekte Basis: den geschlossenen
> Selbstbau-Kreislauf. Alles Weitere (wirklich alles Weitere) entsteht danach
> DURCH raiOS — auf Bare Metal oder simuliert in QEMU.

Desktop, GPU, Editor-Komfort usw. sind in diesem Plan also keine
Arbeitspakete mehr, sondern **Beispiele für spätere Früchte des Kreislaufs**.

## 2. Was NIE verhandelt wird (die Verfassung)

1. **Beweis vor Behauptung.** Nichts gilt als fertig ohne grünen Beweis-Lauf.
2. **Sandbox als Normalfall.** Gewachsenes läuft eingesperrt (Wasm, begrenzte
   Imports); privilegierte Stufen sind einzeln freigegebene Ausnahmen mit
   eigener Beweis- und Rückroll-Pflicht.
3. **Physische Freigabe.** Nichts läuft, nichts installiert sich ohne echten
   Klick/Tastendruck des Owners am Gerät.
4. **Nachrechnen statt Vertrauen.** Werkzeuge (auch der Compiler) bekommen
   kein Vertrauen — ERGEBNISSE werden reproduzierbar nachgeprüft
   (Doppel-Build, Byte-Identität, Fingerabdrücke, Rollback).
5. **Ehrliche Lücken.** Was fehlt, steht mit Namen in den Docs.

## 3. Owner-Entscheidungen vom 2026-07-15 (bindend für diesen Plan)

1. **Kern-Selbstumbau: JA, als Endstufe.** Der Kreislauf wächst stufenweise
   (Programme → Dienste → zuletzt der Kern selbst über A/B-Boot-Slots mit
   automatischem Rollback bei Fehlstart). Erst diese Endstufe macht Beispiele
   wie einen GPU-Treiber durch die Schleife baubar.
2. **Bauplatz: NUR AUF DEM GERÄT ZÄHLT.** Eine extern gesteuerte Bau-Maschine
   ist KEINE Erfüllungsstufe der Vision. „raiOS baut aus sich selbst" gilt
   erst, wenn der Compiler auf dem Gerät läuft. (Der Owner-Rechner bleibt
   während der ENTWICKLUNG unser Werkstatt-Gerüst, um raiOS selbst zu bauen —
   er zählt nur nicht als Teil des fertigen Kreislaufs.)
3. **„Basis fertig"-Kriterium: der geschlossene Kreislauf in QEMU.** Ein vom
   Agenten geschriebenes Programm durchläuft schreiben → beschaffen →
   **auf dem Gerät bauen** → nachrechnen → Owner-Freigabe → installieren →
   Reboot überleben → Rollback beweisbar, ohne Handarbeit dazwischen.
   Derselbe Beweis auf dem Surface ist der zweite Meilenstein danach.

## 4. Das Produkt: der Kreislauf (Stationen + ehrlicher Status)

| # | Station | Status heute |
|---|---------|--------------|
| 1 | Owner fragt in Genesis („bau mir …") | Schnellspur (RUIP) existiert; B2.1b Slow-Lane-Auftrag + sichtbarer Source-Status sind der nächste Slice |
| 2 | Agent schreibt Quellcode ins System (W1-Workspace, inert) | **Key-free Grundlage bewiesen:** B2.1a legt die Antwort als inerte Source-Revision ab; B2.2a führt einen systemeigenen Preflight-Fehler über ein vierteiliges klassifiziertes Feedbackpaket in ein exaktes Kind, das danach besteht (`shadow-20260717-142445-27836.json` 654/654). Echte Provider-Anbindung und freigegebener Feedback-Export FEHLEN noch |
| 3 | Beschaffung (Abhängigkeiten, Quarantäne) | **B1 geschlossen:** W7 live, M12-Empfängeridentität, physisch freigegebener Einmallauf, W6-Install/Reboot und Rollback sind bewiesen (`shadow-20260715-021710-30228.json`, `shadow-20260715-092637-10088.json`, `shadow-20260715-132111-27848.json`); Produktions-Registry-Vertrauen später |
| 4 | **Bauen AUF DEM GERÄT** | FEHLT — heute extern-versiegelt (W4, zählt als Gerüst, nicht als Erfüllung) |
| 5 | Nachrechnen (Repro-Doppel-Build, Fingerabdrücke) | Existiert (extern bewiesen); On-Device-Variante folgt mit Station 4 |
| 6 | Physische Freigabe | Existiert, bewiesen |
| 7 | Installieren, Autoload, Reboot-Überleben | **Geschlossen (B1.2c + B1.3):** W7-Kandidaten (`shadow-20260715-132111-27848.json`) UND RUIP-Programme (Editor durch dieselbe W6-Maschinerie, ARTSTOR-gestützt, Boot-2-Autoload stellt inertes `Source::Durable` vor jedem Befehl wieder her, `shadow-20260717-114259-19696.json` 60/60); nur durable Dokument-Text ist ein späterer Slice |
| 8 | Automatisches Rollback | **Für W7-Kandidaten geschlossen (B1.2c):** exakte Inventar-Wiederherstellung + verketteter Unpromote, Zweit-Rollback verweigert, Tombstone überlebt Boot 3; alt-bewiesen für Projekt-Apps |
| 9 | ENDSTUFE: Stationen 2–8 für den KERN selbst (A/B-Slots) | Embryo vorhanden (Core-Policy-Slots A/B); Vollausbau FEHLT |

## 5. Der Weg zur fertigen Basis (Blöcke in Reihenfolge)

**B1 — Beschaffung schließen (nah):**
- B1.1 **ABGESCHLOSSEN:** W7-Download live über echte QEMU-e1000/TLS/HTTP-
  Ausführung bewiesen; Same-Boot-Retry und Negativfälle grün
  (`shadow-20260715-013325-25964.json`).
- B1.2 Heruntergeladenen Kandidaten durch die EXISTIERENDE M6/W6-Maschinerie
  führen (laufen + installieren; kein zweiter Weg) — **IN ARBEIT:**
  - B1.2a **ABGESCHLOSSEN:** W7 akzeptiert den Kandidaten nur aus dem exakt
    passenden lokalen Katalog mit sechs gastgeprüften M12-Empfängerbelegen;
    ohne sie werden die TLS-Bytes verworfen, mit ihnen bindet der Preflight den
    exakten Kandidaten und hält alle vier Folge-Gates geschlossen
    (`shadow-20260715-021710-30228.json`).
  - B1.2b **ABGESCHLOSSEN:** vollständige M6-Nachprüfung, dann der EINE
    Current-Boot-Lauf erst nach physischem Genesis-Klick (§2.3): exakte
    Beweiskette wird als inerte Vorschau unter einer Freigabe-Challenge
    eingefroren, serielle Start-/Lade-Wiederholungen bleiben verweigert,
    veraltete Vorschau wird beim Klick verweigert und erst nach frischer
    Bindung läuft der Kandidat genau einmal; der alte klicklose Dev-Lauf ist
    aus dem seriellen Protokoll entfernt
    (`shadow-20260715-090227-11004.json` 188/188,
    `shadow-20260715-092637-10088.json` 227/227).
  - B1.2c **ABGESCHLOSSEN:** derselbe bewiesene Kandidat läuft über W6-Signatur,
    zweiten physischen Install-Klick und drei verkettete RECLOG-Rahmen dauerhaft
    installiert; Boot 2 prüft beim Neustart automatisch beide Signaturen nach,
    lädt den Gast von der Persist-Disk und meldet `cross_reboot_proven=true`
    VOR jedem seriellen Befehl; Rollback stellt das exakte Inventar wieder her
    und der Tombstone überlebt Boot 3; Fehlerfälle (korrupter Blob, manipulierter
    Record, Safe-Posture) scheitern geschlossen (`shadow-20260715-115540-29456`
    244/244, `shadow-20260715-121316-27156` 271/271,
    `shadow-20260715-132111-27848` 198/198). Das veraltete m6d-rollback-Profil
    ist dabei saniert.
- B1.3 **ABGESCHLOSSEN — B1-BLOCK GESCHLOSSEN:** ein freigegebenes
  RUIP-Programm (der Editor) installiert dauerhaft über dieselbe W6-Maschinerie
  (ARTSTOR-gestützt, da 176 Byte nicht in einen 4096-Byte-RECLOG-Rahmen passen),
  Boot 2 prüft W6-Signatur + kanonische Bytes nach und stellt es als inertes
  `Source::Durable` VOR jedem Befehl wieder her (Shell nicht gestartet; ein
  frischer Klick rendert dann den Editor), Rollback-Tombstone überlebt Boot 3,
  Fehlerfälle scheitern geschlossen; der signierte Gast bleibt unverändert
  (`shadow-20260715-145046-7640` 282/282 same-boot, m6c-Regression 188/188,
  `shadow-20260717-114259-19696` 60/60 drei Boots). EHRLICHER UMFANG: persistiert
  die Programm-DEFINITION, nicht den getippten Text (späterer Dokument-Slice).

**B2 — Der Agent im System (mittel) — AKTIVER BLOCK:**
- B2.1a **ABGESCHLOSSEN:** Eine feste key-free Agentenantwort wird als inerte,
  content-adressierte Source-Revision gespeichert und über Reboot wieder exakt
  gelesen. B2.1b **NÄCHSTER SLICE:** echte `ProjectWorkspace`-Providerroute,
  Genesis-`/build`-Status und explizite `/program`-Schnellspur; live geschlossen
  erst mit Netz, lokalem Schlüssel und positivem Pin-Vertrauenspfad.
- B2.2a **ABGESCHLOSSEN, B2.2 TEILWEISE:** Der erste Beweiskreis
  Fehler → begrenztes Feedback → exaktes Kind → erneute Prüfung ist grün
  (`shadow-20260717-142445-27836.json` 654/654). Der Beweis ist bewusst nur
  Source-Preflight, kein Compiler/Test. Für den echten Agenten fehlen der
  einmalig freigegebene, redigierte Provider-Export und seine Auditbindung.
- B2.3 `/build`-Slow-Lane in Genesis: Auftrag → sichtbarer Workspace-Status →
  Freigabe am Ende; folgt auf die reale Provider-/Feedback-Grenze.

**B3 — Bauen auf dem Gerät (der harte Kern der Vision):**
- B3.0 **Forschungs-Spike zuerst, ehrlich:** rustc-als-Wasm (plausibelster
  Weg: Cranelift-Backend, WASI-Subset) auf Machbarkeit/Größe/Laufzeit
  prüfen; Ergebnis ist ein GO/NO-GO-Bericht mit Alternativen — KEIN
  Blindstart. Benannte Risiken: Speicherbedarf (heutige Gäste: 2 MiB —
  Compiler braucht Hunderte MiB → neue Gast-Klasse), Geschwindigkeit
  (Interpreter!), Datei-Import-Oberfläche.
- B3.1 Neue Gast-Klasse „Bauplatz" (großer Speicher, begrenzte, geprüfte
  Datei-Imports, Fuel-Geduld) — grants-nothing, wie immer.
- B3.2 Compiler als signierter Wasm-Gast; Ausgabe wird wie heute per
  Doppel-Build + Fingerabdruck NACHGERECHNET (Verfassung §2.4 gilt auch hier).
- B3.3 Zwischenstufe parallel: die eigene begrenzte Programm-Sprache (RUIP)
  wächst kontrolliert weiter — kleine Programme entstehen schon heute
  komplett im System (Editor 2026-07-14 war der Beweis der Wachstumsfähigkeit).

**B4 — Basis-Abnahme:**
- Der Abnahme-Lauf nach §3.3: kompletter Kreislauf in QEMU, ohne Handarbeit.
  Erst dieser grüne Lauf erklärt die Basis für fertig.

**Dauerspuren, die die Basis „perfekt" machen (parallel, klein):**
- **Kern klein halten:** das Schrumpf-/Umzugs-Programm (Relocations in
  PC-testbare Crates) läuft weiter — „maximal leicht" ist Vision-Bestandteil.
  Der frühere layout-sensitive Verdacht war ein Host-Harness-Fehler, kein
  Kerneldefekt; der neue per-Port-Mutex verhindert dieselbe Parallelstart-Klasse.
- **UI nur für den Kreislauf:** Genesis bekommt genau die Oberflächen, die
  Aufträge/Prüfung/Freigabe brauchen — keinen Komfort auf Vorrat.
- **Vertrauen härten:** Owner-Sealing statt Dev-Keys, volle TLS-Kette +
  vertrauenswürdige Zeit, ADR-0004-Gedächtnis-Vollausbau.

## 6. Nach der Basis

- **M-Surface:** derselbe Abnahme-Lauf auf dem Surface (WiFi-Seitenstrang
  liefert die Netz-Vorbedingung; Owner-Entscheidung 2026-07-08 bleibt:
  parallel weiterbauen, gated die QEMU-Basis nicht).
- **M-Endstufe (Kern-Selbstumbau, §3.1):** der Kreislauf baut, prüft und
  installiert einen neuen KERN in den B-Slot; Fehlstart rollt automatisch
  auf A zurück. Ab hier sind Treiber (z. B. GPU) prinzipiell Schleifen-Früchte.
- **Früchte (Beispiele, KEINE Bauaufträge an uns):** Fenster-Desktop,
  GPU-Modesetting/2D→später 3D, Editor-Komfort, Medien-Apps — der Owner
  beauftragt sie dann bei raiOS, nicht bei der Werkstatt.

## 7. Was dieser Plan bewusst NICHT verspricht

- Keine Termine; jeder Block endet erst mit grünem Beweis.
- Keine „KI schreibt fehlerfreien Code"-Magie — das System beweist
  Begrenztheit, Reproduzierbarkeit, Rücknahmefähigkeit; Inhalt sichern
  Tests + Sandbox + Freigabe + Rollback.
- Desktop/GPU sind ausdrücklich KEINE Arbeitspakete dieses Plans.
- B3 trägt benannten Forschungsanteil; der Spike darf mit NO-GO enden und
  Alternativen vorschlagen (das wäre ein Ergebnis, kein Scheitern).

## 8. Pflege

Bei jedem Block-Close, der eine Station aus §4 bewegt, wird die
Status-Tabelle aktualisiert (eine Zeile, mit Report-Namen). Details:
`docs/PROJECT_STATUS.md`; Tages-Cursor: `docs/ROADMAP.md`.
