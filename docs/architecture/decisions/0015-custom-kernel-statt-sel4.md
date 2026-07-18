# ADR 0015: Custom-Kernel weiterbauen, seL4 unterschiebbar halten

## Status

Angenommen am 18.07.2026.

## Kontext

raiOS braucht einen kleinen, capability-basierten Boden. Ein Wechsel zu seL4
würde im verfügbaren Stundenbudget keinen kürzeren belastbaren Weg schaffen:
der bestehende Rust-Kernel ist bekanntes Terrain, während für den konkreten
x86_64-/Surface-Pfad weiterhin eine Beweislücke bliebe.

## Entscheidung

Der eigene Rust-Kernel wird weitergebaut. Die Genesis-Schicht erhält eine
schmale, kernel-agnostische Boden-Schnittstelle, damit seL4 später als
alternativer Unterbau eingeschoben werden kann, ohne die darüberliegenden
Capability-Verträge neu zu entwerfen.

## Konsequenzen

- Der aktuelle Custom-Kernel bleibt der Entwicklungs- und Produktpfad.
- Kernel-Interna dürfen nicht in den Genesis-Vertrag leaken.
- seL4 bleibt eine austauschbare Boden-Option, aber kein paralleles Arbeitspaket.
