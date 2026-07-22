# 0042 - Bound PCI BAR probes to validated header windows

Date: 2026-07-22 - Status: active

## Kontext

ADR 0040 verlangt vor dem Surface-Capture-Boot einen Test derselben
BAR-Sizing-Produktionslogik. Zwei Implementierungsstrategien erreichten grüne
Hosttests, wurden aber unabhängig abgelehnt: BAR-Gültigkeit und Slotverbrauch
waren gekoppelt, ein Memory64-Folgeslot konnte außerhalb des Header-BAR-Fensters
liegen, verschwundene Geräte und ungültige Bereiche waren nicht durchgehend
fail-closed, und ein Fake konnte die konkrete x86-`outw`-Verdrahtung nicht
beweisen. Nach der Stuck-Regel wird der alte Seam nicht weiter inkrementell
gepatcht, sondern durch eine neue begrenzte Probe-Grenze ersetzt.

## Entscheidung

1. Jede Probe beginnt mit verfügbarer Funktionsidentität und einem validierten
   Header-BAR-Fenster: Typ 0 hat sechs, Typ 1 zwei, Typ 2 einen Slot. Unbekannte
   Header und All-ones-Identität/-Header/-Command/-Leader werden vor Mutation
   abgelehnt. Ein Memory64-Folgeslot darf nur bei `index + 1 < bar_count`
   gelesen oder geschrieben werden.
2. Eine mutationsfreie Klassifikation bestimmt den deklarierten Slotverbrauch
   unabhängig von der späteren BAR-Akzeptanz. Ein Memory64-Leader verbraucht
   immer zwei Slots, auch wenn er unzugewiesen, abgeschnitten oder ungültig ist;
   ein abgeschnittener Leader autorisiert keinen Folgeslotzugriff. Direkte
   Abfragen eines Folgeslots dürfen ihn nicht als eigenen BAR vermessen.
3. Die private Produktionsgrenze liefert ein reiches Ergebnis mit
   `consumed_slots` und einer Disposition wie usable, empty/unassigned,
   unavailable oder invalid. Die öffentliche
   `read_bar_info(address, index) -> Option<PciBar>`-Signatur bleibt für diesen
   Slice bestehen und bildet ausschließlich usable auf `Some` ab. `None` ist
   fail-closed: Enumeration veröffentlicht keinen BAR; XHCI startet nicht.
4. Nur klassifizierte, innerhalb des Fensters liegende I/O-, Memory32- und
   Memory64-Leader beginnen die Sizing-Transaktion. Nach der ersten Mutation
   gibt es einen gemeinsamen Restore-Pfad: Memory64 high vor low, dann das
   exakte Command-Wort zuletzt. Allgemeine Runtime-Synchronisation und
   zusätzlicher Restore-Readback bleiben außerhalb dieses Slices.
5. Ein BAR ist nur usable, wenn Encoding und Maske unterstützt und kanonisch
   sind, die Größe ungleich null und eine Zweierpotenz ist, die Basis ungleich
   null und größen-ausgerichtet ist und `base + size - 1` ohne Überlauf in der
   jeweiligen I/O-/32-/64-Bit-Adressbreite liegt. All-ones- oder geänderte
   Identität nach einer Transaktion kann niemals einen usable BAR ergeben;
   soweit das Gerät noch antwortet, wird der Snapshot trotzdem restauriert.
6. Hosttests verwenden dieselbe Probe-/Restore-Produktionslogik mit Fake-Config.
   Zusätzlich erzeugt der Predicate mit dem gepinnten Toolchain Assembly oder
   Objektcode eines retained Wrappers um den konkreten x86-Transportpfad, ohne
   privilegiertes I/O auszuführen. Er muss CF8 als dword und CFC/CFE als word
   beweisen und bei einer dword-Datenport-Mutation rot werden. Ein bloßer
   Source-Regex oder Trait-Fake genügt nicht.
7. Pflichtnegative sind Header-Endslot-Memory64 für Typ 0/1/2, direkte
   Follower-Abfrage, All-ones vor und während der Probe, nicht zusammenhängende
   Masken, Fehlalignment und Bereichsüberlauf für jede Breite. Jede Ablehnung
   nach Mutation muss den beobachtbaren Fake-Zustand exakt restaurieren. Der
   Predicate nutzt `git -C $RepoRoot` und prüft auch seinen eigenen Inhalt,
   solange die Datei noch untracked ist.

## Alternativen & Zweitmeinungen

Zwei frische neutrale Read-only-Meinungen empfahlen übereinstimmend die
zweistufige, Header-begrenzte Klassifikation, unabhängigen Slotverbrauch und
einen emitted-code-Beleg des realen Word-Writes. Meinung A bevorzugte einen
privaten reichen Outcome mit unveränderter öffentlicher `Option`-API, weil das
den Slice auf zwei Dateien begrenzt. Meinung B bevorzugte ein öffentliches
`Result`, um unavailable und malformed an alle Caller weiterzugeben, außerdem
eine abschließende Identitätsklassifikation.

Wir übernehmen die abschließende Identitätsklassifikation, behalten aber die
öffentliche `Option`-API. Für den aktuellen Boot ist `None` an beiden Call-Sites
fail-closed; ein `Result` würde USB-Call-Sites im bereits fremd veränderten
K3-Slice anfassen und die Reviewfläche ohne zusätzlichen Bootschutz vergrößern.
Eine spätere typisierte Fehlertelemetrie ist ein eigener Slice.

Eine typisierte Capability, die Folgeslotadressen nur konstruktiv erzeugen kann,
wurde als stärker, aber für die ausschließlich privaten Probe-Einstiege und den
serialisierten Pre-Driver-Boot als unnötig großer Umbau verworfen.

## Folgen

Ein neuer konservativer Zwei-Dateien-Worker darf den bisherigen Kandidaten
durch diese Grenze ersetzen. Akzeptanz verlangt den vollständigen Runtime-
Predicate, Mutation-Negative für den konkreten Transportbeleg und zwei neue
unabhängige Read-only-Reviews. Erst danach darf K3 erneut geprüft werden.

Hosttests beweisen weiterhin nicht das reale Verhalten des Surface-Chipsatzes,
Geräteverschwinden während echter Port-I/O oder Restore ohne Readback. Diese
Restunsicherheit bleibt beim owner-custodied Cold Boot; sie erlaubt keinen
vorzeitigen Stick-Boot.
