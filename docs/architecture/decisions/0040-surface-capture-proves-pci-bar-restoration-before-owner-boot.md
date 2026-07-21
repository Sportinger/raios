# 0040 — Surface capture proves PCI BAR restoration before Owner boot

Date: 2026-07-21 · Status: active

## Kontext

Der K3-Schnitt erfasst vor `usb::init()` alle PCI-Funktionen und aktiven BARs.
Die gemeinsame PCI-Routine ermittelt BAR-Groessen durch temporaeres Abschalten
von I/O- und Memory-Decoding, Schreiben von Einsen und anschliessendes
Restaurieren. `usb::init()` fragt danach XHCI-BAR0 mit derselben Routine noch
einmal gezielt ab, bevor Bus Master, MMIO und der Controller aktiviert werden.

Die erste unabhaengige Read-only-Pruefung wertete die zweite Abfrage als
Verstoss gegen eine einmalige BAR-Sizing-Grenze und den fehlenden echten
Restore-Test als Blocker. Eine zweite, neutral formulierte Pruefung stellte
fest, dass ADR 0038/0039 nur eine vollstaendige Same-Boot-Erfassung vor aktiver
Treibernutzung verlangen: Eine zweite zielgerichtete, vollstaendig
restaurierende Abfrage vor Controller-Start ist dort nicht verboten. Beide
Pruefungen lehnten jedoch einen Owner-Boot ab, solange die mutierende
Produktionslogik nur durch Regex und unabhaengige PowerShell-Modelle gedeckt ist.

## Entscheidung

1. K3 darf genau eine PCI-Gesamtenumeration vor `usb::init()` ausfuehren.
   Eine spaetere zielgerichtete `read_bar_info`-Abfrage waehrend der
   Treiberinitialisierung ist zulaessig, solange sie vor der aktiven Nutzung
   liegt und den vollstaendigen vorherigen PCI-Config-Zustand restauriert.
2. Vor dem ersten Owner-custodied Surface-Boot muss dieselbe
   `read_bar_info`-Produktionslogik ueber einen testbaren PCI-Config-Backend-Seam
   laufen. Pflichtfaelle sind I/O, Memory32, Memory64, 64-Bit-Folgeslot,
   vollstaendiger Command-/BAR-Restore und zweimaliges Sizing mit identischem
   Fakt und identischem Endzustand.
3. Source-Predicates duerfen Bootreihenfolge und Verdrahtung pruefen, ersetzen
   aber den Production-Logic-Test dieser mutierenden Grenze nicht. Der
   freestanding Build ersetzt weder diesen Test noch die spaetere
   Hardware-Evidenz.
4. Zusaetzlicher Restore-Readback in der realen Port-I/O-Routine ist moegliches
   Hardening, aber keine stillschweigende Voraussetzung dieses
   Entwicklungs-Unblockers. Ein eigener Slice muss Nutzen und Fehlersemantik
   dafuer definieren.
5. Diese Entscheidung akzeptiert weder K3 noch das Surface-Manifest. Erst der
   gruene Seam-Test, erneute unabhaengige K3-Pruefung und der reale Owner-Boot
   koennen die jeweiligen Gates schliessen.

## Alternativen & Zweitmeinungen

Die strengere Meinung verlangte, den bereits erfassten XHCI-BAR in den
USB-Treiber durchzureichen und jede zweite Sizing-Transaktion zu entfernen.
Das reduziert Konfigurationsschreibzugriffe, koppelt aber den generischen
USB-Start an den temporaeren Capture-Pfad und fuehrt eine Regel ein, die weder
ADR 0038 noch ADR 0039 festgelegt haben.

Die neutralere Meinung hielt die vorhandene Reihenfolge fuer vertragskonform:
Beide Abfragen restaurieren laut sichtbarem Code BAR und Command, und erst
danach beginnt aktive XHCI-Nutzung. Sie verlangte trotzdem einen echten Test
der Produktionsroutine, weil Build, Regex und ein getrenntes Modell die
entscheidende Restore-Grenze nicht beweisen. Wir folgen dieser kleineren,
wiederverwendbaren Loesung und behalten den ersten Einwand als dokumentiertes
Hardware-Risiko bis zu Test und Surface-Boot.

## Folgen

Der naechste Slice bleibt klein und unabhaengig von Surface-Faktenformat,
RECLOG und WLAN: Er macht die bestehende PCI-BAR-Routine injizierbar und testet
ihre exakte Transaktions- und Restore-Semantik auf dem Host. K3 kann danach
gegen diesen akzeptierten Beleg erneut geprueft werden, ohne Capture und USB
dauerhaft miteinander zu verkoppeln.

Der Seam ist auch fuer spaetere Treiber-Domains wiederverwendbar. Er beweist
keine Besonderheit des echten Surface-PCIe-Controllers; diese Restunsicherheit
bleibt bewusst beim Owner-custodied Hardwareboot.
