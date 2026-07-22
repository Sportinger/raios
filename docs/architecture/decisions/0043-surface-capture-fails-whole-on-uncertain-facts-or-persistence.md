# 0043 - Surface capture fails whole on uncertain facts or persistence

Date: 2026-07-22 - Status: active

## Kontext

Der ungepruefte K3-Slice soll in einem Limine-Boot CPUID-, SMBIOS-, Memory-map-
und PCI-Fakten erfassen und danach als Surface Fact Wire V1 in USB-RECLOG
schreiben. Zwei frische unabhaengige Read-only-Pruefungen lehnten den
Owner-Boot ab: physische SMBIOS-Intervalle waren nicht an die Limine-Memory-map
gebunden, SMBIOS- und CPUID-Vollstaendigkeit war unzureichend, PCI-Probe-Fehler
wurden wie legitime Funktionen ohne BARs dargestellt und der RECLOG-Pfad war
nur durch Regex und ein unabhaengiges PowerShell-Modell geprueft.

Die lokale Limine-0.5.0-API dokumentiert `SmbiosResponse::entry_32/entry_64`
ausdruecklich als physische Adressen. Die vorhandene Richtung physisch plus
HHDM ist daher richtig; die fehlende Autorisierung des ganzen Intervalls ist
der Fehler. Der Unsafe-Baseline-Check ist ausserdem rot, weil das alte
Predicate nur `--summary` ausfuehrt.

## Entscheidung

1. Surface Fact Wire V1, Core-Modell und Extractor bleiben unveraendert. K3
   publiziert nur einen vollstaendigen Snapshot; ein nicht beweisbarer Fakt,
   Parserfehler oder Persistenzfehler verwirft beziehungsweise vergiftet den
   gesamten Capture-Pfad. Es gibt keine Teilvollstaendigkeits-Fakten.
2. Eine private physische Lesegrenze uebersetzt genau einmal ueber HHDM und
   erzeugt erst nach allen Nachweisen einen Slice. Jedes halboffene Intervall
   muss mit checked arithmetic, `usize`/`isize`-Grenzen und kanonischem Start
   sowie Endbyte vollstaendig in genau einem Limine-Eintrag liegen. Zulaessig
   sind nur `RESERVED`, `ACPI_RECLAIMABLE`, `ACPI_NVS` und
   `BOOTLOADER_RECLAIMABLE`; alle anderen oder unbekannten Typen sind verboten.
3. Deklarierte SMBIOS-Entry-Laengen muessen zwischen ihrem Versionsminimum und
   256 Byte liegen. Tabellen sind auf 1 MiB, einzelne Strukturen inklusive
   Strings auf 64 KiB und die Anzahl auf 4096 begrenzt. SMBIOS 2 muss seinen
   deklarierten Structure Count exakt erfuellen und Type 127 als letzte
   Struktur enthalten. SMBIOS 3 wird innerhalb der deklarierten Maximalgroesse
   bis zum genau einmaligen Type 127 geparst; Bytes hinter diesem Terminator
   werden nicht als Strukturen oder notwendiges Nullpadding interpretiert.
   Struktur-Handles sind global eindeutig, Type-17-Fakten werden nach Handle
   sortiert und reservierte Extended-Size-Bits werden abgelehnt.
4. CPUID bleibt rohe Hardware-Evidenz und behauptet keine OS-benutzbaren
   Features. Erfasst werden kanonisch Leaf 0, Leaf 1, alle von Leaf 7.0 EAX
   angekuendigten Subleafs bis zu einem Policy-Cap von 32, Extended-Maximum,
   `0x80000001`, Brand-Leaves `0x80000002..4`, `0x80000007` und
   `0x80000008`, jeweils nur wenn angekuendigt. Ein hoeheres Leaf-7-Maximum
   verwirft den Capture statt still abzuschneiden. Ein spaeterer Resolver muss
   `resolved-feature-set` auf CPUID-advertised Hardwaremerkmale begrenzen;
   OS-usable AVX/AMX wuerde zusaetzliche XCR0-Evidenz brauchen.
5. `pci.rs` erhaelt neben den unveraenderten Treiber-APIs eine
   capture-spezifische `Result`-Enumeration. Nur `Empty` und `Unassigned`
   duerfen aktive BAR-Fakten auslassen. `Unavailable`, `Invalid`, All-ones,
   Identitaets-, Header- oder Metadatenwechsel verwerfen die Gesamterfassung.
   Identitaet, Klasse, Revision, IRQ und relevanter Header werden vor und nach
   der bestehenden reichen BAR-Probe gebunden; K3 liest danach keine PCI-Felder
   erneut. BDFs und BARs sind streng kanonisch geordnet.
6. Vor dem ersten RECLOG-Write werden Slice-Reihenfolge, Completion-final,
   alle Wire-Encodings, Gesamt-Kapazitaet und alle Null-Zielsektoren geprueft.
   Der Produktionspfad und ein Host-Fake benutzen denselben kleinen
   RECLOG-Zustandsautomaten. Jeder Record nutzt FUA, exakten Readback, aeusseren
   RECLOG-Reparse und inneren Wire-Reparse aus den zurueckgelesenen Bytes; erst
   dann darf der Append-Punkt fortschreiten.
7. Ab dem ersten Write-Versuch vergiftet jeder Write-, Readback- oder
   Reparse-Fehler den gemeinsamen Append-Punkt fuer den Rest des Boots. Damit
   kann weder Completion noch ein spaeterer H25-Frame einen partiellen Prefix
   fortsetzen. Reine Preflight-Fehler vor dem ersten Write lassen den H25-Pfad
   verfuegbar.
8. Die Reparatur wird wegen der Fuenf-Dateien-Grenze in drei sofort zu
   sichernde Slices geteilt: (a) capture-spezifische PCI-Result-Enumeration,
   (b) echter bounded SMBIOS/CPUID/Ordering-Capture, (c) testbarer
   RECLOG-Zustandsautomat plus Boot-Verdrahtung und Unsafe-Baseline. Jeder Slice
   braucht Produktionslogiktests, Mutation-Negative und unabhaengige
   Read-only-Abnahme, bevor er exakt committed und gepusht wird.
9. Das finale Gate umfasst den bestehenden PCI-Predicate, echte Rust-Fixtures
   und Fake-Backends, den freestanding Release-Build, Extractor-Selftest,
   `unsafe-inventory.py --check`, `git diff --check` fuer alle Dateien und zwei
   frische K3-Abnahmen. Erst danach darf der Owner ein Capture-Image schreiben.

## Alternativen & Zweitmeinungen

Beide Meinungen empfahlen fail-whole-capture mit unveraendertem Wire V1 und
eine separate PCI-Result-Grenze. Beide bestaetigten anhand der lokalen API die
physische SMBIOS-Pointerdomaene. Eine Meinung erlaubte ein Intervall ueber
mehrere benachbarte erlaubte Memory-map-Eintraege; die andere verlangte genau
einen Eintrag. Wir waehlen genau einen Eintrag: ein zusaetzlicher sicherer
Fehlabbruch ist fuer den einmaligen Owner-Capture akzeptabel, waehrend
Zusammenfuegung mehr Beweislogik und Ueberlappungsregeln benoetigt.

Eine Meinung verlangte Nullpadding hinter SMBIOS-3-Type-127; die andere wies
darauf hin, dass die Tabellenlaenge dort nur ein Maximum ist. Wir interpretieren
nach Type 127 keine weiteren Strukturen und verlangen kein unbelegtes Padding.
SMBIOS 2 bleibt durch exakte Tabellenlaenge und Count gebunden.

Beim Persistenztest wollte eine Meinung den generischen Kern in `usb.rs`
halten, die andere eine kleine neue `surface_fact_reclog.rs`. Wir waehlen die
kleine Modulgrenze, weil nur so Produktionsautomat und Host-Fake denselben
Rust-Code ausfuehren, ohne den gesamten xHCI-Treiber in einen Host-Harness zu
ziehen. Das Modul darf keine zweite Wire- oder RECLOG-Semantik erfinden.

Wire V2 mit expliziten Teil-/Probe-Fehlerfakten und ein separates UEFI-Tool
wurden verworfen. Beides erweitert Modell, Codec, Extractor und Vertrauen,
obwohl fail-closed Abbruch fuer den H26-Unblocker genuegt.

## Folgen

Der naechste Hardwareboot kann keine scheinbar vollstaendige Evidenz aus
unautorisiertem Speicher, gemischten PCI-Zeitpunkten oder einer fortgesetzten
partiellen RECLOG-Serie erzeugen. Die strengeren Grenzen koennen auf echter
Firmware sicher fehlschlagen und einen weiteren Capture-Build erfordern; das
ist der bewusste Preis fuer verwertbare Maschinenfakten.

Limine-/Firmware-Vertrauen, echtes PCI-Port-I/O, Controller-FUA und
Power-loss-Verhalten bleiben Hardwareevidenz. Der Capture bleibt gemaess ADR
0038 Owner-custodied Entwicklungsevidenz und keine Remote-Attestation. Das
Surface-Manifest und H26 bleiben bis Extraktion, Checker, Review und
Digest-Pinning blockiert.
