# raiOS Fabrik-Plan — Bauen im System, ohne Werkstatt

Owner-Zielansage (bindend, 2026-07-18, präzisiert im Gespräch):

> Kern soll sein: ein Custom-Rust-Kernel, die Genesis-Schicht, und darin kann
> ich mit AI-Agenten etwas bauen (egal welche Sprache, aber Rust bietet sich
> an) und das direkt kompilieren — auf dem Gerät, ohne fremden Werkstatt-PC.
> Die Sicherheit kommt oben drauf oder wird so verwoben, dass wir von der
> Grundidee nicht abrücken müssen. Ziel-Maßstab: irgendwann ganze Spiele oder
> ein Videoschnittprogramm, gebaut und getestet von Agenten auf raiOS.

Dieser Plan ersetzt nicht `VISION_PLAN.md` (die Schleifen-Grundidee bleibt),
er **re-zentriert das Ziel**: Die On-Device-Fabrik ist die Hauptstraße, der
Liefer-Weg (Werkstatt baut, raiOS prüft/installiert) ist Übergang.

## 1. Was schon existiert und voll einzahlt (nichts davon war umsonst)

| Baustein | Status | Rolle für die Fabrik |
|---|---|---|
| Custom-Rust-Kernel + Genesis | läuft (QEMU + Surface-Boot bewiesen) | Das Haus, in dem die Fabrik steht |
| B1: Download → Install → Reboot → Rollback | bewiesen | Wie fertige Programme sicher ins System kommen und wieder raus |
| B2: KI-Quelle → inerte Dateien → Feedback (live OpenAI) | bewiesen | Wie Agenten-Aufträge und Quellcode ins System fließen — der „Job"-Eingang |
| W5-Klick (physische Freigabe) + Doppel-Bau + Nachrechnen | bewiesen | Die verwobene Sicherheit — gilt unverändert für jeden Compiler |
| B3A: Assembler-Gast baut lauffähiges Wasm im System (33/33) | bewiesen | **Der Beweis, dass die Bau-Rohrleitungen funktionieren** — Auftrag → im System bauen → prüfen → Klick → läuft |
| rlang (Mini-Sprache + Compiler, host-bewiesen) | Crate fertig, pausiert | Ersatzrad + wiederverwendbarer Encoder (siehe §4) |
| Messungen 2026-07-18 | erledigt | rustc-als-Wasm existiert (91 MB); Blocker = Threads (Bau-Schalter, kein Naturgesetz); Bau-Rezept öffentlich gefunden |

Die Maschinenform bleibt Wasm (Käfig eingebaut, byte-genau nachrechenbar,
Rust hat Wasm als offizielles Ziel) — genau das macht „Rust-Compiler im
System" überhaupt möglich.

## 2. Die Treppe zum Ziel (jede Stufe einzeln beweisbar)

1. **Bootstrap-Werkzeug (JETZT, Hauptspur):** Einmalig außerhalb (Cloud-Bau,
   kostenlos für öffentliche Projekte — der Rechner hat nur 4 GB frei) das
   öffentliche rustc-Wasm-Rezept mit umgelegtem Threads-Schalter backen.
   Ergebnis mit der wasmi-Probe messen: lädt es? RAM? Tempo?
   Danach lebt das Werkzeug im System — die Werkstatt wird nie wieder
   gebraucht (Bootstrap-Prinzip: auch das erste Linux wurde einmal von außen
   kompiliert).
2. **„Bauplatz"-Gast-Klasse:** große Speicher-Gäste (hunderte MB), begrenzte
   geprüfte Datei-Zugriffe (Quelle + Bibliotheken rein, Artefakt raus),
   Geduld-Budgets. Auf echter Hardware (Surface: mehr RAM als die 512-MB-VM).
3. **rustc als signierter Gast:** Rust-Quelle vom Agenten → Compiler-Gast baut
   → Doppel-Bau + Fingerabdruck-Nachrechnung → W5-Klick → läuft. Erste
   Grenzen ehrlich: keine Internet-Pakete (Abhängigkeiten kommen als geprüfte
   Quelle über W7), keine Makro-Programme, einkernig (langsam).
4. **Schnelle Ausführungsstufe:** geprüftes Wasm → schneller Maschinencode,
   auf dem Gerät, mit denselben Beweisen. Nötig für Bau-Tempo UND für große
   Programme zur Laufzeit (Spiele). Bewusste spätere ADR-Entscheidung.
5. **Parallelität + GPU:** Mehrkern-Bauen und GPU-Treiber durch die Schleife —
   die letzten Stufen zu Spiel-Maßstab. Threads sind nicht „undenkbar",
   sondern teuer (Determinismus + Käfig); sie kommen, wenn Stufe 1-4 stehen.

Job-Bild des Owners = genau diese Kette: Genesis-Auftrag („baue X") →
Agenten liefern Quelle (B2) → Bauplatz baut (Stufe 2-3) → Tests im Käfig →
W5-Klick → installieren/zurücknehmen (B1). Die Stationen existieren; nur der
Produzent wächst.

## 3. Sicherheit: verwoben, nicht verhandelbar

Für jeden Compiler (rlang heute, rustc morgen) gilt unverändert:
Ausgabe wird doppelt gebaut und unabhängig nachgerechnet, bleibt inert bis
zum physischen Klick, läuft im Import-geprüften Käfig, installiert nur
signiert mit Rollback. Der Compiler ist ein Arbeiter, keine Autorität.
Von der Grundidee wird nicht abgerückt — sie ist der Grund, warum man einer
Agenten-Fabrik je vertrauen kann.

## 4. rlang — ehrliche Einordnung (Owner-Frage: „wo brauchen wir das?")

- **Was es geleistet hat:** Es bewies mit dem Assembler die kompletten
  Bau-Rohrleitungen, durch die später rustc fließt — dieselben Routen,
  Freigaben, Beweise. Diese Arbeit zahlt 1:1 ein.
- **Was bleibt:** Der typisierte Wasm-Encoder aus rlang ist genereller
  Werkzeugkasten (jeder Compiler kann ihn nutzen); die Crate ist ein
  Ersatzrad, falls der Bootstrap länger klemmt, und ein Werkzeug für winzige
  Systemaufgaben.
- **Entscheidung:** rlang ist **pausiert** (nach Slice 2a, committet und
  grün). Es ist NICHT auf dem kritischen Pfad zum Fabrik-Ziel. Weiterbau nur,
  wenn ein konkreter Bedarf entsteht — kein Selbstzweck.

## 5. Nächste Schritte (Reihenfolge)

1. **Cloud-Bootstrap-Bau** des threads-freien rustc-Wasm (Rezept gefunden:
   `oligamiq/rust_wasm`-Workflows; Fork + Schalter + CI). Braucht das Go des
   Owners (läuft über seinen GitHub-Account, öffentlich, kostenlos).
2. **wasmi-Probe** auf das Ergebnis (lädt es? Import-Liste? Übersetzungszeit/
   RAM) — Werkstatt-seitig, Stunden.
3. Bei Erfolg: **Bauplatz-Scoping** mit den gemessenen Budgets (statt
   geratenen) + erster Hello-World-Rust-Bau im System als W5-Beweis.
4. Bei Misserfolg: exakte Wand dokumentieren; Alternativen (eigener Beitrag
   zum Upstream-Rezept; rlang-Weiterbau als Zwischennutzen) neu bewerten.
5. Platz-Frage GELÖST (2026-07-18): Laufwerk E: hat ~400 GB frei — der
   VM-Scratch wird dorthin umgelenkt (C: bleibt entlastet). Kein OS-Wechsel:
   die Werkstatt bleibt Windows (alle Beweise/Skripte laufen hier); für einen
   etwaigen lokalen Linux-Bau existiert bereits ein Ubuntu in Windows (WSL),
   von hier steuerbar und mit E:-Platz — das ist der Rückfall, falls der
   Cloud-Bau klemmt. Das separate Arch-Dual-Boot wird NICHT genutzt (dort
   könnte der Agent nicht mithelfen).

Ehrlich benannt: Schritt 1-2 können auch scheitern oder Wochen an
Upstream-Feinheiten hängen. Jedes Ergebnis wird gemessen berichtet, nie
geraten.
