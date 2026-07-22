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
