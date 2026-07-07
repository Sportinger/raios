# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-07 (**M7 COMPLETE — a service now survives a real restart.**
The big one is done: boot 1 accepts and saves an AI-authored module (its code +
a signed "promotion receipt"); the machine is powered down and back up; boot 2
independently re-checks the whole evidence chain from disk — re-doing the signature
check itself, never trusting a stored "already OK" flag — and only then runs it, so
the service answers again after the restart. A corrupted copy, a tampered receipt,
and safe-recovery mode are each correctly refused. All three safe disk-write
abilities (log, boot control, artifact store) are live; everything else stays
refused. Still honestly labelled dev-key, not owner-sealed. Proven end-to-end
(two-boot proof 85/85, full regression 8168/8168, plus an independent adversarial
review that found and fixed two real read bugs). raiOS is no longer "this-boot-only".
**M8 started (the emergency lifeline):** the fixed, minimal list of rescue commands
is now pinned and answered on its OWN separate path — so it can't be quietly
extended, and it touches none of the normal machinery. Two READ-ONLY rescue commands
work today: "show me the rescue command list" and NEW "show me a diagnosis snapshot"
(which parts booted, which services are alive vs broken, which are protected-core vs
replaceable) — it reads only, leaks no secrets, changes nothing. The four rescue
NEW — **die erste echte Rettungs-AKTION funktioniert:** ein schlechtes Modul lässt sich
jetzt **abschalten** (`disable_module`). Ablauf mit Sicherheitsnetz: raiOS schreibt
ZUERST einen dauerhaften Prüf-Eintrag auf die Platte und stoppt das Modul erst DANACH —
und nur, wenn der Eintrag wirklich gelang. **Kern-Dienste, die Rettungsleine selbst und
unbekannte Ziele werden strikt verweigert, BEVOR irgendetwas geschrieben oder verändert
wird** (im Test bewiesen: der Kern-Dienst `core.serial` wurde korrekt abgelehnt, nichts
angetastet). Es wird nur *entfernt*, nie etwas Neues befördert; ehrlich als Entwickler-
Schlüssel gekennzeichnet. NEU — **die zweite Rettungs-Aktion funktioniert auch:
neu-starten-in-den-letzten-guten-Zustand** (`restart_last_good`): ein abgeschaltetes
ODER abgestürztes Modul wird wieder gesund gemacht — dauerhafter Prüf-Eintrag zuerst,
dann werden die Sperren gelöst und **derselbe geprüfte Start-Weg** neu ausgeführt (er
prüft die eingebauten Modul-Bytes bei JEDEM Lauf gegen den festen Fingerabdruck — es kann
also nur das bekannte, geprüfte eingebaute Modul laufen, nie etwas Fremdes). Scheitert
der Neustart, wird ehrlich „gestoppt" gemeldet, nie fälschlich „gesund". Es bringt nur
Bekannt-Gutes zurück, befördert nichts Neues. **Damit ist der Kern von M8B fertig
(abschalten + neu-starten).** NEU (M8C-1, nur-lesen): die Diagnose zeigt jetzt auch den
DAUERHAFTEN Zustand — welche System-Kopie (A/B) die „letzte gute" ist, ob der Sicherheits-
Modus aktiv ist, ob der letzte Start als erfolgreich markiert wurde — plus eine reine
**Vorschau** „was würde ein Zurückrollen ändern" (nur anschauen, ändert nichts). Fehlt die
Information (keine Platte), wird das ehrlich als „nicht verfügbar" gemeldet, nie erfunden.
NEU (M8D-1, „prüfen aber noch nicht laden"): die Rettungsleine kann jetzt ein bereits
gespeichertes Modul per Fingerabdruck im LOKALEN Speicher finden und **die komplette
Beweiskette von Grund auf neu prüfen** (inkl. Unterschrift) — lädt es aber noch NICHT (das ist
der nächste, letzte M8-Schritt M8D-2). Sie lädt **niemals aus dem Netz** und nimmt **keine neuen
Bytes** an — nur der Fingerabdruck sucht in schon-geprüften lokalen Einträgen; ein falscher/
unbekannter Fingerabdruck wird ehrlich abgelehnt. Danach ist **M8 komplett**. Das Ausführen von
„zurückrollen" bleibt bewusst verweigert (Nicht-Ziel von M8). Und die wichtigste Absicherung von M8 ist bewiesen: selbst wenn ein laufender
Baustein WIRKLICH abstürzt, antwortet die Rettungsleine weiter — sie überlebt, weil
KI-Code ein Treibstoff-Limit hat und kooperativ läuft, noch NICHT durch echte
Hardware-Trennung (die kommt erst mit M11).).

## What raiOS can actually do today

- Boots on a VM and on the bonded machine into its own graphical UI.
- You can chat with OpenAI from inside the OS over a pinned, fail-closed
  TLS connection (pin-only; not yet full certificate-chain validation).
- The system can describe itself through typed read-only commands
  (snapshot, devices, services, problems, event log).
- One built-in demo service can be loaded, hot-swapped v1<->v2, and
  rollback-previewed — all RAM-only.
- NEW (M6A-1 + M6A-2a): raiOS now has a working receiving door for outside
  code. A real Wasm program that did NOT come baked into the system can be
  sent in over the console (in small encoded pieces that get reassembled),
  checked for realness (fingerprint + parse), and held in memory as an
  inert "candidate" — while running, loading, and saving it stay firmly
  refused. Verified end-to-end with a real 4 KB program. Giving that code
  any rights is the next, gated step (M6B). Independently security-checked;
  one known limit noted: the realness-check itself isn't yet time-capped.

## Gate status

- Full verification profile: **GREEN** as of 2026-07-07 — 8,168/8,168
  checks passed in one run (report shadow-20260707-015537-28252.json,
  hash-verified) after the persistent artifact store (M7D-1). The focused
  persistence profile is now 48/48 (adds the artifact-persisted, blob-hash,
  garbage-blob, and SAFE/full-deny needles) and the audit-rollback profile is
  unchanged-green (1,709/1,709). The old "mystery" failures are explained: the
  test tooling asked for too much data at once and then misread its own
  connection loss — no bug in the OS itself.
- Working tree: the ~36,900-line backlog was committed 2026-07-04 in
  three honest commits; release binaries are no longer tracked in git.

## Current milestone

**M0 and M1 are DONE** (2026-07-05). What that means concretely:
- Test runs record exactly WHY they died (VM crash w/ exit code vs
  connection glitch); a dead VM fails in seconds, not 7 minutes (M0).
- Kernel logic lives in a `raios-core` library tested on a normal PC in
  under a second — previously every logic check needed a VM boot (M1).
- GitHub now automatically builds the kernel, runs the tests, AND boots
  the OS in a VM with 417 checks on EVERY commit (all green, ~7 min).
  A bonus: the signed-source protection proved itself by correctly
  rejecting a mis-configured build machine on the first CI attempt.

**M2 is CLOSED** (ADR 0006, provisional-overridable): the structural
disease is cured — one record model with non-divergent hashing, one
dispatch table, one command representation, one selftest runner, every
file agent-readable, zero-warning build, nine green FULL profiles. Line
count landed at ~126.5k (not the original ~20k); the optional extra
shrink (changing output vocabulary) is deferred and remains YOUR call —
say the word and it gets scheduled.

**M3 and M4 are CLOSED.** M3: raiOS performed its first real,
policy-authorized durable disk write and the hello rollback now actually
applies using that transaction as its authority record. M4 (the deepest
safety milestone so far): foreign code now runs INSIDE a real in-kernel
WebAssembly sandbox and physically cannot call anything outside its
granted functions — a module that even *imports* a forbidden function
fails to load. Four hostile-guest cases (broken bytes, memory hog,
infinite loop, crash) all end as clean evidence, never a kernel crash.
Proven: 465/465 checks incl. 49 wasm-specific ones.

**M5 is CLOSED — the rebuild is vindicated.** Adding a whole second
service (echo, which loads, runs its sandboxed wasm, reports health,
appears in the inventory, stops) cost **~1,060 lines** — a descriptor
plus a small state machine reusing everything built in M2–M4. A copy of
the old approach would have been ~19,000 lines. That number IS the proof
that the giant refactor worked: the system can now grow by services, not
by monoliths. Verified: 486/486 checks (67 echo-specific) + full profile
7,825/7,825.

Now active: **M6 Promotion Loop v0** — the project's first true product
moment: one AI-authored artifact travels
the whole safe loop end to end — authored, tested in the Shadow VM,
capability-granted, promoted live, and rolled back — with evidence at
every step. Split into M6A (candidate intake) → M6B (grant) → M6C
(promote) -> M6D (rollback). **M6 COMPLETE (dev-tier RAM loop closed).** A real
outside program is received over the console, checked, its identity recorded,
granted its rights, loaded, run inside the sandbox, and rolled back in RAM
through a verified undo path. It does not yet save to disk and does not claim
durable/native/owner-sealed authority. Today's signing key is a deliberate DEV
key so the loop can be built and tested; **your own key K seals it for real
later** (the sealing ceremony is the very last step).

**M7 Persistence Foundation now active — making things survive a restart.**
Done so far: the kernel reads the disk's layout + its durable log (M7A/M7B-1,
read-only); performs its **first real safe WRITE** — appending one durable
record and reading it back to confirm it, every other disk area still refused
(M7B-2); reads the boot-control area to decide which system copy (A/B) to boot and
whether to enter a safe "recovery" mode (M7C-1); and now — NEW — **safely marks
a boot as successful** and lets safe-mode switch off saving (M7C-2). Concretely:
when a boot passes its health checks, the system writes a "this copy booted OK"
record into the *spare* boot slot and reads it back to confirm — so a power cut
mid-write can only damage the spare, never the copy that's currently trusted
(crash-safe A/B switching). And if the boot-control record is missing or
damaged, the system refuses to save anything (safe recovery mode). There's also
a small offline tool for you to pre-select which copy boots next. raiOS now has
**two** of its three safe disk-write abilities live (the log, and boot control);
everything else stays refused. And NEW (M6D-2): when raiOS accepts and runs an
AI-authored module, it writes a durable "promotion receipt" into the log — a
complete, self-contained record (all the fingerprints plus the dev-key signature)
that a future boot can *independently re-check* before trusting that module again.
That is the bridge that makes a promotion survivable. Still within-boot dev-tier
(the real reboot proof is M7D). NEW (M7D-1): the third disk-write ability is live — raiOS now **stores an
AI-authored module's actual code on disk** (as a fingerprinted blob chained to its
evidence receipt). Crucially, the stored code is completely **inert** — it has zero
permission to run — until it is re-checked. So all THREE safe disk-write abilities
now exist (the log, boot control, and the artifact store). **DONE (M7D-2 — the big
one): raiOS survived an actual reboot.** Boot 1 promoted and saved a real
signed module; the machine was fully restarted; boot 2, on the same disk,
re-verified the whole evidence chain through the exact same safety gates — re-doing
the signature check itself, refusing any "trusted because it's stored" shortcut —
and only then ran it, so the service answered live after the restart. A corrupted
saved copy, a tampered receipt, and safe-recovery mode were each correctly refused.
**M7 is complete** — raiOS is no longer "this-boot-only". Still dev-key, never
owner-sealed. Next: M8 — the emergency lifeline.

## Top risk

The build loop has been producing evidence paperwork instead of new
capability (~90% of the code governs authority that is never granted).
See `docs/plan-reviews/review-4-deep-scope-code-and-process-2026-07.md`.

## Next milestones (docs/ROADMAP.md)

M6 first external AI-built service through the full safe-promotion loop
-> M7 things survive a restart (persistence + automatic fall-back to the
last good state) -> M8 emergency lifeline -> M9 real long-term memory ->
M10 stronger provider trust + a second AI provider -> M11 shrinking the
core (network parsing moves out into replaceable services) -> M12+ Wi-Fi,
downloading modules over the network, moving to new hardware.

NEW (2026-07-06): M7-M11 are fully pre-planned as step-by-step maps with
ready-made worker instructions, plus a procedure handbook
(`docs/ORCHESTRATOR_PLAYBOOK.md`). Purpose: cheaper AI agents can keep
building correctly even without an expensive orchestrator model. Every
map starts with a mandatory "check the plan against reality" step.
