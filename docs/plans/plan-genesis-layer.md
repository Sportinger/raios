# raiOS Genesis-Plan — Steckdose und Sichtfenster

Owner-Vormerkung (2026-07-18, Gespräch mit dem Orchestrator). Status:
**Vormerkung** — dieser Plan ordnet zwei künftige Ausbaustufen der
Genesis-Schicht ein; er ändert keinen SCOPE-Text und aktiviert erst nach den
Fabrik-Meilensteinen (hello.rs-Doppelbau, W5-Fabrikbeweis). Die zugehörigen
Kästchen stehen in `docs/scope/02-genesis-layer.md`.

## 1. Die Hardware-Steckdose (einmal insgesamt, nie pro Hersteller)

Kernaussage des Gesprächs: Die Kernel-Erweiterung für echten Hardware-Zugriff
von Domänen ist **genau einmal** nötig und herstellerneutral, weil sie nichts
Gerätespezifisches weiß:

- PCIe-Register-Fenster (BAR) als verleihbare, widerrufbare Autorität
- IOMMU-Schranke pro Domäne (DMA nur in gewährte Regionen; Voraussetzung:
  Maschine hat IOMMU, Surface: Intel VT-d)
- Interrupt-Weiterleitung als Grant
- Kill-Switch pro Domäne (existiert als Primitiv-Ziel in Scope 02)

Das ist wortgleich die „Capability granularity"-Gruppe in Scope 02 — dieser
Plan hält nur die Reihenfolge-Entscheidung fest: Die Steckdose kommt **nach**
dem Fabrikbeweis und **vor** GPU-Arbeit (`docs/plans/plan-drivers-hardware.md`),
weil jeder Treiber-Port sie voraussetzt. Danach gilt: Der Kernel liefert die
Steckdose, der Agent baut das Gerät — kein Geräte-Port erfordert je wieder
Kernel-Änderungen.

## 2. System- und Vertrauens-Monitor („Task-Manager")

Owner-Wunsch: Genesis soll wie ein Task-Manager zeigen können, was läuft und
was es verbraucht — in Echtzeit. Architektur-Befund: raiOS führt diese
Buchhaltung **bereits als Nebenprodukt der Sicherheit** (jede Ressource geht
durchs Verleih-Nadelöhr); es fehlt nur das Sichtfenster.

| Anzeige | Quelle | Stand |
|---|---|---|
| Laufende Domänen/Gäste | Kernel-Liste | existiert |
| RAM pro Domäne (Seiten, Decke, Wachstums-Spur) | Bauplatz-Speicherzähler (B1G) | im Bau |
| CPU-Zeit pro Domäne | Scheduler-Uhr | kleine Ergänzung nötig |
| Gehaltene Autoritäten pro Domäne | Import-Grants | existiert |
| Wer hat wann erlaubt (Provenienz, Signatur) | Vertrauens-Store | existiert |
| „Task beenden" | Kill-Switch-Primitiv | Scope 02 |

Die letzten drei Zeilen machen daraus einen **Vertrauens-Manager** — „wer darf
was, auf wessen Erlaubnis" kann ein klassisches OS prinzipbedingt nicht zeigen.

Arbeitsteilung (bindende Vision, VISION_PLAN): Die Genesis-Schicht liefert die
Messwerte nur als abfragbare, lesbare Fakten-Facetten; die Anzeige selbst ist
eine **Frucht der Schleife** — ein Programm, das ein Agent auf raiOS baut, mit
einer verliehenen Lese-Autorität „darf Systemzustand sehen". Ein Vorläufer
existiert in der Shell (TRUST/PROBLEMS/BUILD-Leisten).
