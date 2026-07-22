# 0045 - Authorize one H26 Surface development test

Date: 2026-07-22 - Status: active

## Kontext

H25 proved on the Owner-custodied Surface that the post-PMK HostCmd mailbox is
live, but intentionally denied network state. H26 needs only to retain the
firmware scan TSF and AP beacon timestamp, restore the same-epoch PMK to
Associate transition, and call the accepted TSF builder. The Surface capture
cannot currently make the manifest prompt-ready because its first physical
SMBIOS entry-point access faults; normal boot now rejects that access.

ADR 0027 therefore blocks the hardware lane even though its missing CPU and
memory facts are not consumed by this WLAN-only slice. The Owner explicitly
authorized continuing to one new H26 test stick and accepts the additional
development risk. Two fresh independent read-only Codex reviews assessed the
exception without seeing each other's result; both returned
`ACCEPT_EXCEPTION` with materially matching boundaries.

## Entscheidung

1. Exactly one H26 implementation dispatch and one resulting Owner-custodied
   image/write/cold-boot/log-extraction cycle are authorized for
   `surface-pro-4@sha256:08c8d977f48f5a846edecaf31cc4d205291105dc5c821960df21621e17b36189`.
2. The hardware lane still uses `scripts/invoke-codex-lane.ps1`, binds the
   machine ID and digest, and requests only the observed fact path
   `/devices/2/identity` (`Marvell 88W8897`). A narrow explicit one-shot
   launcher gate may waive only the global `curated_context_ready` check for
   this ADR, machine, digest, fact path and H26 order. Direct `codex exec`, a
   copied manifest, or silently marking the manifest ready is forbidden.
3. The H26 product diff may only retain both timestamps for the selected live
   BSS, require them for association readiness, restore the same-epoch
   successful PMK to Associate transition, and use
   `build_associate_24ghz_with_tsf`. It may not change CPUID, SMBIOS, Limine
   memory maps, MMU, allocators, DMA, PCI discovery/BARs, device resources,
   firmware upload, USB, storage, authentication algorithms or domain grants.
4. Missing or ambiguous timestamps, selected-BSS replacement, stale epoch,
   PMK failure/timeout, duplicate completion or Associate failure must produce
   neither another Associate publication nor network state. The new builder is
   called once; the legacy builder is not used on the H26 path. Network state
   remains denied until a current-epoch successful Associate response.
5. Before image creation, focused positive and negative predicates,
   freestanding release build, unsafe-inventory check, `git diff --check`, exact
   file-scope verification and one fresh independent read-only implementation
   review must be green. The final SanDisk identity and image digest are bound
   before the physical write.
6. `surface-pro-4.v1.json` remains `curated_context_ready:false`; unknown facts
   remain unknown and may not be inferred from a public Surface SKU. Success is
   only Owner-custodied development evidence, not manifest readiness, H26/SCOPE
   closure, general Surface support, production safety, remote/TPM/cryptographic
   attestation or general Marvell compatibility.
7. This exception expires at the first of: extraction or abandonment of this
   one physical test, manifest digest drift, launcher denial, extra product
   file/surface, need for any unknown hardware fact, unsafe SMBIOS access,
   isolation concern or failed mandatory gate. A second implementation dispatch,
   image or hardware test needs a new explicit Owner decision and ADR.

## Alternativen & Zweitmeinungen

Both reviews found CPU/model/features and RAM/map facts irrelevant to the
bounded protocol-state slice, while still necessary for normal manifest
readiness. Both required the launcher, fixed manifest digest, observed Marvell
identity, no inference of unknown facts, fail-closed timestamp/epoch negatives,
one image/test only and no attestation or closure claims. There was no material
disagreement.

Waiting for a new SMBIOS capture strategy was rejected for this strand because
the current fault is already isolated and the H26 code may not consume those
facts. Directly bypassing the launcher or editing the manifest was rejected
because it would turn a reviewable exception into an unbound precedent.

## Folgen

The next H26 stick can be produced without fabricating machine facts. The cost
is a deliberately narrow governance and launcher exception plus one additional
review gate. ADR 0027 and ADR 0038 remain unchanged for every other Surface
lane and for any test after this exception expires.

## R2-Erweiterung (2026-07-22)

Der erste autorisierte H26-Dispatch lieferte einen grünen Vier-Dateien-Diff,
aber zwei vorgeschriebene unabhängige Unsafe-Reviews widersprachen sich. Eine
Review akzeptierte die unveränderten DMA-/Bounds-Invarianten; die adversariale
Review wies zwei nicht ausreichend bewiesene Grenzen nach: Scan-`CMD_DONE` ist
nicht an die erwartete Scan-Sequenz gebunden, und die ausgewählte BSS kann
theoretisch zwischen Revalidierung und Associate-Publikation beziehungsweise
Netzwerkfreigabe ersetzt werden. Deshalb wurden weder Unsafe-Baseline noch
Produktdiff akzeptiert, committed, paketiert oder auf USB geschrieben.

Der Owner autorisierte daraufhin ausdrücklich genau einen zweiten
H26-Reparaturdispatch und diese ADR-Erweiterung. Dafür gilt:

1. Der neue einmalige Launcher-Token ist `ADR-0045-H26-R2`; er verwendet einen
   eigenen atomaren Claim und darf den verbrauchten ersten Claim weder löschen
   noch wieder freigeben. Maschine, Manifest-Digest und einziger Fact-Pfad
   bleiben unverändert.
2. R2 darf ausschließlich die erwartete Scan-Sequenz bis zur Antwortprüfung
   tragen und die selected-BSS-/selection-epoch-Bindung atomar bis über
   Associate-Publikation und Netzwerkfreigabe beweisen. Der vorhandene
   Vier-Dateien-Diff darf dafür nur innerhalb derselben vier Dateien repariert
   werden.
3. R2 darf weiterhin keine DMA-Spans, Pointer, Descriptoren, PCI-/BAR-Zustände,
   Ressourcen, Firmware, USB, Storage, Authentifizierungsalgorithmen oder
   Domain-Grants ändern. Entsteht dafür ein Bedarf, endet die Ausnahme.
4. Beide ursprünglichen Findings brauchen gezielte positive und negative
   Mutationen. Danach müssen alle H26-Predicates, Release-Build,
   Unsafe-Inventar und eine neue unabhängige Read-only-Review grün sein. Erst
   dann dürfen exakt die geprüften Unsafe-Hashes neu gebunden werden.
5. Die ursprüngliche Autorisierung für genau ein resultierendes
   Owner-custodied Image/Write/Cold-Boot/Extraction bleibt unverändert. R2
   autorisiert kein zweites Image und keinen zusätzlichen Hardwaretest.

## R3-Erweiterung (2026-07-22)

Die vorgeschriebene unabhängige R2-Review bestätigte die Scan-Sequenzprüfung
und die normalen Lease-Pfade, lehnte den Diff aber wegen eines zusätzlichen
Concurrent-start-Interleavings ab: Zwei Starts können die Vorprüfung bestehen;
der verlierende Start kann danach den Snapshot des bereits installierten Jobs
auf `Failed` setzen und dessen Lease hängen lassen. Außerdem waren die neuen
Race-Predicates für diesen Punkt nur Quelltextmutationen statt eines
ausführbaren Zustandsmodells.

Der Owner hat für notwendige, weiterhin eng im H26-Ziel liegende Reparatur- und
Review-Schritte automatische Fortsetzung autorisiert. Deshalb gilt genau ein
weiterer Token `ADR-0045-H26-R3` mit eigenem atomaren Claim. R3 darf nur:

1. Lease-Akquisition, `CONNECTION`-Job-Publikation und Snapshot-Übergang so
   atomar koppeln, dass ein konkurrierender Start den Gewinner weder
   überschreiben noch dessen Lease stranden kann;
2. für Concurrent-start, konkurrierende Auswahl und alle terminalen
   Lease-Freigaben ein ausführbares deterministisches Zustands-/Interleaving-
   Modell plus negative Mutationen ergänzen;
3. den vorhandenen Vier-Dateien-Diff innerhalb derselben Grenzen reparieren.

R1 und R2 bleiben verbraucht und unveränderlich. Alle übrigen R2-Grenzen und
die Autorisierung für nur ein resultierendes Image/Hardwaretest bleiben
unverändert. Eine weitere Ablehnung am selben Ziel beendet diese Strategie;
danach wird nicht automatisch eine vierte Variante gestartet.

## R3-Dispatch-Recovery-Erweiterung (2026-07-22)

Der mit Commit `bfa9fef` (`bfa9fef535d160f210c3cb1bf5726015c008993e`)
ausgeführte R3-Launcher verbrauchte
`target/state/adr0045-h26-r3.claim` mit SHA-256
`b32ef3b8e56f3eba19e846160cf84ffe7c014b6933126d8e766c7d1c03bcf11d`.
Der Fehlernachweis `target/lanes/h26-r3-race-repair.stdout.log` hat SHA-256
`4d8bf26aecef9ff08ef78485691f81909a6b03dd762cb8510492c26bb083e7c8`
und enthält exakt `accepted=false`, `child_started=false` und
`codex_child_start_failed`. Windows wies den als native ausführbare Datei
übergebenen PowerShell-Shim vor dem Child-Start ab. Es gibt keinen
R3-Workerbericht `target/lanes/h26-r3-race-repair-report.md`. Damit ist der
R3-Claim verbraucht, aber weder ein R3-Produktlauf noch eine neue
Produktvariante entstanden.

Die bestehende Owner-Autorisierung für automatische, notwendige und eng im
H26-Ziel liegende R3-Reparatur- und Review-Schritte deckt genau eine Recovery
dieses bewiesenen Infrastrukturfehlers vor Child-Start. Dafür und nur dafür
werden folgende drei neuen, fest benannten Elemente autorisiert:

- Token `ADR-0045-H26-R3-DISPATCH-RECOVERY-1`;
- Claim `target/state/adr0045-h26-r3-dispatch-recovery-1.claim`;
- Schema `raios.adr0045_h26_r3_dispatch_recovery_1_claim.v1`.

Der Claim wird genau einmal atomar mit `CreateNew` erzeugt. R1, R2 und R3
bleiben bytegenau unveränderlich und verbraucht; Löschen, Erstatten,
Freigeben, Umbenennen oder Wiederverwenden eines alten Claims ist verboten.
Der Recovery-Pfad akzeptiert ausschließlich den neuen Token, Claim-Pfad und
das neue Schema. Alte Token, Claim-Pfade und Schemas dürfen nur als exakt
hashgebundene Beweise gelesen, nie als Recovery-Autorität interpretiert
werden.

Die Recovery dispatcht ausschließlich die unveränderten Bytes von
`target/lanes/h26-r3-race-repair-order.md` mit SHA-256
`0ef25b8ce5fdefe15790ec83ff7803c3597b4f9933008fc27674367f18172585`.
Sie bindet unverändert:

- Maschine `surface-pro-4` und Manifest SHA-256
  `08c8d977f48f5a846edecaf31cc4d205291105dc5c821960df21621e17b36189`;
- ausschließlich den Fakt `/devices/2/identity = Marvell 88W8897` und die
  Sandbox `workspace-write`;
- `seed-kernel/src/wifi.rs` mit Prehash
  `690aa68efaa835fa1df59cfd7316472828e438976e68238e021b6b9c0496f91e`;
- `seed-kernel/src/marvell_wifi_pcie.rs` mit Prehash
  `d53f1eeedd66fe529d2ad55ab6f821e731135971b17c4b6f584f1105c68c5595`;
- `scripts/test-marvell-connection-telemetry.ps1` mit Prehash
  `fddb8474d46d53b802a5e93b9780b4343a305400c20bdf5026d380608174c31f`;
- `scripts/test-wifi-ephemeral-physical.ps1` mit Prehash
  `f9e818260b369f17b984af06ba86cc795adca14415228265609eddcea228ac65`.

Vor Claim-Erzeugung gelten in dieser Reihenfolge folgende harte Gates:

1. Der korrigierte Launcher ist committed. Sein vollständiger Git-Commit und
   der SHA-256 der exakten Launcher-Datei sind in der Recovery-Autorität
   festgelegt. Eine frische unabhängige Read-only-Review muss sowohl die
   Korrektur als auch die nachfolgende Windows-Regression akzeptiert haben.
   Für diese Recovery sind das ausschließlich Commit
   `4c77bdaf03a42ab0e543ca389e1310d7bcf5baf2` und Launcher-Datei-SHA-256
   `2124404d8a7616767d87f1260b8f5fe61d6afb6c73419022567337e483ad700d`
   (SHA-256 der Git-Blob-Bytes dieses Pfads in genau diesem Commit).
2. Ein produktionsnaher Windows-Test durchläuft denselben Resolver und
   Startpfad mit einem `.ps1`-Shim. Er beweist native Host-Auflösung,
   `UseShellExecute=false`, exakte Argumente und exakte stdin-Bytes bei genau
   einem gestarteten Child. Weder `.ps1` noch `.cmd` dürfen direkt als
   `ProcessStartInfo.FileName` verwendet werden. Ein nicht auflösbarer oder
   nicht unterstützter Startplan muss vor Claim-Erzeugung ablehnen.
3. Der Windows-Startplan wird vor Claim-Erzeugung vollständig aufgelöst und
   festgelegt: native Host-Datei, gegebenenfalls Shim, deren kanonische Pfade
   und SHA-256, strukturierte Argumente, stdin-Hash und genau ein Child. Ein
   späteres erneutes Auflösen nach Pfad ist verboten.
   Der einzige autorisierte Resolver-Eingang ist
   `C:\Users\admin\AppData\Roaming\npm\codex.ps1` mit SHA-256
   `0c149db80ed0bf442c810146b0ad0163b74982fe4542d673f56c354d7b8229cb`;
   der einzige native Host ist
   `C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe` mit SHA-256
   `7600ffe12da441fe89d035b13801e8e91d064bc544a27b19a5cf49f6ab8b18f5`.
4. R3-Claim, Fehlerlog, R3-Auftrag, alle vier Produkt-Prehashes, der
   korrigierte Launcher-Commit samt Datei-Hash, Maschine, Manifest, Fakt,
   Sandbox und der aufgelöste Windows-Startplan werden gegen die oben
   festgelegten exakten Werte validiert. Jeder Mismatch lehnt vor dem Claim ab.
5. Alle dateibasierten Eingaben und aufgelösten Startdateien werden vor der
   Validierung einmal geöffnet und bis zum erfolgreichen Dispatch-Handoff mit
   nicht schreibbaren und nicht löschbaren Handles gegen Austausch geleast;
   Hashes und Start stammen aus diesen Handles, nicht aus erneut geöffneten
   Pfaden. Die Produkt-Leases werden erst nach erfolgreichem Child-Start und
   vollständiger stdin-Übergabe für die autorisierte R3-Bearbeitung gelöst.
6. Erst nach diesen Gates wird der neue Claim erzeugt und daraus höchstens ein
   Child gestartet. Der Claim bindet alle validierten Werte einschließlich
   altem Claim-, Fehlerlog-, Auftrags- und Produkt-Hash, vollständigem
   Launcher-Commit und -Datei-Hash sowie einem Digest des Startplans.

Jedes Scheitern nach Erzeugung des Recovery-Claims lässt ihn dauerhaft
verbraucht; es gibt weder Claim-Refund noch einen zweiten Infrastrukturversuch.
Ein zweiter Infrastrukturfehler sowie jede Produkt- oder Review-Ablehnung
blockieren H26 ohne automatische weitere Variante. Ein Mismatch vor der
Claim-Erzeugung autorisiert weder Austausch noch Reparatur des Beweises,
sondern lehnt diese Recovery ab. Insbesondere ist keine Recovery bei
verändertem Claim, Fehlerlog, Auftrag, Produkt-Prehash, Launcher,
Startplan, Maschine, Manifest, Fakt oder Sandbox und kein zweiter Versuch
autorisiert.

Diese Entscheidung ist keine neue Produktvariante und kein R4. Sie erweitert
weder Scope noch DoD, Hardwareannahmen, Fakt-Autorität, Produktdateien,
Image-Anzahl oder Hardwaretest. Sie schafft keinen generischen Retryparameter,
keinen Claim-Override und kein wiederverwendbares Recovery-Schema. Nach einem
erfolgreichen Child-Start gelten unverändert Auftrag und Ergebnisregeln von R3
sowie sämtliche bisherigen Grenzen dieser ADR.

Beide frischen unabhängigen Read-only-Meinungen empfahlen diese eng gebundene
Recovery und lehnten eine Wiederverwendung des R3-Claims ab. Opinion A forderte
dafür eine explizite Owner-Autorisierung. Opinion B bewertete die bereits für
notwendige enge H26-Reparatur- und Review-Schritte erteilte automatische
Owner-Autorisierung als ausreichend. Die Entscheidung folgt Opinion B in
dieser Zuständigkeitsfrage: Die vorhandene Owner-Autorisierung deckt genau
diese einmalige Infrastruktur-Recovery und nichts darüber hinaus.

## R3-Abschluss (2026-07-22)

Der Recovery-Dispatch startete den R3-Worker erfolgreich. Die vier fokussierten
Dateien, beide ausführbaren Interleavingmodelle, 63 Marvell-Tests, 16
DMA-Safety-Tests, rustfmt, Unsafe-Selftest und der Root-freestanding-Release-
Build waren grün. Die vorgeschriebene unabhängige Read-only-Abnahme lehnte den
Diff dennoch ab: Zwei Starts aus einem vorhandenen Ready-Zustand können beide
einen besitzlosen, später veralteten `replace_ready`-Entschluss tragen. Nach
Publikation oder erfolgreichem Abschluss des Gewinners kann der Verlierer
dadurch dessen Hardware quieszen und gegebenenfalls Snapshot, Data-Link und
Netzbindung löschen. Beide neuen Modelle starten nur aus Idle und beweisen
diese Ready-Replacement-Grenze nicht.

Zwei anschließende frische, voneinander unabhängige Read-only-Prüfungen
bestätigten denselben minimalen Trace und untersagten den Hardware-Write. Eine
Ausgabe war im Kopf widersprüchlich als `REFUTE` bezeichnet, ihr Befund und
ihre Freigabeentscheidung bestätigten das Race jedoch vollständig; es besteht
kein materieller Dissens.

Damit ist die in der R3-Erweiterung festgelegte Stop-Bedingung erreicht. H26
ist beim Owner blockiert; der Vier-Dateien-Diff bleibt uncommitted und darf
nicht paketiert oder auf USB geschrieben werden. Es gibt keine automatische
R4-Variante, keinen weiteren Claim und keinen weiteren Hardwaretest. Eine
Fortsetzung benötigt eine neue ausdrückliche Owner-Entscheidung mit geänderter
Strategie oder engerer Scoping-Grenze.
