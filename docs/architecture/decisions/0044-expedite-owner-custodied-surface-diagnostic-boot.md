# 0044 - Expedite the Owner-custodied Surface diagnostic boot

Date: 2026-07-22 - Status: active

## Kontext

ADR 0043 waehlt fuer K3 eine produktionsnahe dreiteilige Reparatur mit breiter
CPUID-Abdeckung, eigenem testbaren RECLOG-Automaten und umfangreichen
Mutationsmatrizen. Das ist sicher, verlaengert aber den physischen Diagnose-Loop,
obwohl ADR 0038 nur einen Owner-custodied Entwicklungs-Capture verlangt.

Der Owner priorisiert jetzt ausdruecklich einen schnellen naechsten Stick-Test
und akzeptiert fuer diesen einmaligen Diagnose-Boot mehr verbleibendes Risiko.
Die Entscheidung lockert Verifikation und Vollstaendigkeitsbreite, nicht die
Grenzen, deren Verletzung fremden Speicher, PCI-Zustand oder den falschen
Datentraeger beschaedigen koennte.

## Entscheidung

1. K3 wird nach dem akzeptierten PCI-Result-Slice als ein Fast-Track-Slice ueber
   den fuenf geerbten Dirty-Dateien repariert. Eine weitere Modulaufteilung ist
   fuer diesen Boot nicht erforderlich.
2. Nicht verhandelbar bleiben: SMBIOS-Reads muessen mit checked arithmetic,
   einem 1-MiB-Cap, kanonischen HHDM-Adressen und vollstaendiger Einbettung in
   einen erlaubten Limine-Memory-map-Eintrag autorisiert sein; PCI verwendet
   ausschliesslich die fail-closed Capture-Enumeration; nach einem begonnenen
   unsicheren RECLOG-Write/Readback/Reparse-Fehler darf kein spaeterer Frame die
   partielle Serie fortsetzen; der physische Stick wird nur nach expliziter
   finaler Datentraegernummer geschrieben.
3. Fuer diesen Entwicklungs-Capture genuegen SMBIOS-2-Count/Type-127 und
   eindeutige Type-17-Handles sowie CPUID Leaf 0/1, alle bounded Leaf-7-Subleafs,
   Extended-Maximum, `0x80000001` und die Brand-Leaves. Weitere Extended-Leaves,
   globale SMBIOS-Handle-Eindeutigkeit und ein eigener RECLOG-Modul-Seam werden
   als Hardening geparkt, sofern der Extractor keinen konkreten fehlenden Fakt
   meldet.
4. Das Fast-Track-Gate ist: bestehender PCI-Predicate gruen, ein fokussierter
   K3-Predicate mit echten Build-/Unsafe-Gates gruen, freestanding Release-Build,
   `unsafe-inventory.py --check`, `git diff --check` und eine unabhaengige
   Read-only-Abnahme ohne Owner-Boot-Blocker. Eine zweite Implementierungsreview
   und breite Fehler-/Mutationsmatrix sind fuer diesen Boot nicht erforderlich.
5. Ein physischer Fehl-Capture schliesst keine SCOPE-Checkbox. Er liefert nur
   neue Logs fuer den naechsten Loop. Manifest-Readiness, H26 und jede
   Produktions-/Attestationsbehauptung bleiben bis Extraktion und separater
   Verifikation offen.

## Alternativen & Zweitmeinungen

ADR 0043 dokumentiert die zwei unabhaengigen Meinungen und bleibt die
vollstaendige Zielhaertung. Ihre einhelligen Grenzen gegen unautorisierten
Speicher, gemischte PCI-Fakten und fortgesetzte partielle RECLOG-Serien werden
beibehalten. Geparkt wird nur Beweisbreite, die fuer die erste verwertbare
Owner-Diagnose nicht unmittelbar erforderlich ist.

Unveraendert nach ADR 0043 in drei Slices weiterzuarbeiten wurde verworfen,
weil der Owner die langsamere Risikoabsenkung gegen einen frueheren
Hardwarebefund getauscht hat. Ganz ohne Memory-/PCI-/Datentraegergrenzen zu
booten wurde verworfen, weil das nicht nur Diagnosequalitaet, sondern reale
Fremdzustaende gefaehrdet.

## Folgen

Das naechste Image wird frueher verfuegbar und kann auf ungewoehnlicher Firmware
fail-closed abbrechen oder spaeter einen Nachcapture benoetigen. Die Ergebnisse
sind bewusst Entwicklungsevidenz unter Owner-Custody. Nach dem Hardware-Loop
kann die in ADR 0043 geparkte Haertung anhand realer Luecken priorisiert werden.
