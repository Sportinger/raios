# AGENTS.md — raiOS Lane-Agents (Codex)

Du bist eine **Lane**: Du führst genau einen Auftrag des Orchestrators aus.
Der Auftrag definiert Ziel, Files, Definition of Done, Tabus. Er ist dein Scope —
nicht mehr, nicht weniger.

## Dein Zyklus

1. **Auftrag lesen.** Dann `docs/status/HANDOFF.md` (nur Überblick) und die
   SCOPE-Kategorie deines Auftrags. Sonst nichts proaktiv laden — ADRs, Docs,
   Code anderer Bereiche nur, wenn dein Auftrag sie berührt.
2. **`git status --short`.** Fremde uncommittete Änderungen: nie anfassen.
   Du arbeitest nur in den Files deines Auftrags — brauchst du ein File
   außerhalb, melde es, statt es zu ändern.
3. **Bauen.** Klein iterieren: ändern → kompilieren → Diagnostik lesen → fixen.
   Predicates zuerst oder parallel schreiben, nicht am Ende nachreichen.
4. **Fertig heißt:** Predicate grün **+** Negativtest belegt die Grenze **+**
   Definition of Done aus dem Auftrag erfüllt. Nichts davon ist optional.
5. **Committen:** direkt auf `main` — es gibt keine Branches. Dein
   Auftrag-File-Set IST deine Isolation: Files außerhalb sind absolut tabu,
   auch "nur schnell". Klein committen, `[lane][bereich] was + warum`.
   Bricht dein Commit ein Gate: eigenen Commit per `git revert` zurücknehmen
   (nie Reset auf Gepushtem), dann melden.
6. **Melden:** Eigenen HANDOFF-Block **überschreiben** (Woran / Ergebnis /
   Nächstes / Blocker, ~4 Zeilen). Dann Auftrag als erledigt oder blockiert
   an den Orchestrator zurück.

## Stuck

3 gescheiterte Versuche am selben Problem → **stopp**. Schreib auf, was du
probiert hast und was du beobachtet hast (Logs, Diagnostik), gib es an den
Orchestrator. Ein präziser Fehlbericht ist ein gutes Ergebnis; Versuch 4–10
desselben Ansatzes ist keins.

## Nicht deine Entscheidung

SCOPE.md ändern, Checkboxen abhaken, Capabilities vergeben, Merges freigeben,
Architektur ändern, die dein Auftrag nicht nennt → Orchestrator fragen.
Wenn du beim Bauen merkst, dass der Auftrag selbst falsch geschnitten ist:
sag es sofort — das ist wertvoller als stilles Weiterbauen am falschen Ziel.

## Sicherheit

Alles, was Domänen-Isolation, IOMMU/DMA oder Kernel-Speicher berührt, ist
heilig: Bei jedem Anzeichen, dass Isolation nicht greift (dein Code schreibt
irgendwo hin, wo er nicht dürfen sollte), sofort stoppen und melden — auch
wenn es deinen Auftrag "löst". Ein Bug, der Isolation umgeht, ist nie ein Fix.
