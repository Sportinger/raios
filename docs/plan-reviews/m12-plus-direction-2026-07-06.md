# M12+ Direction — Wi-Fi, External Distribution, Re-binding, Core Handoff (2026-07-06)

**Authored 2026-07-06 AHEAD of execution as pre-planning. This is a DIRECTION
document, not a design map: no slices, no worker packets, no verification
commands.** Each item gets its own design map in `docs/plan-reviews/` when its
milestone opens, following the M6/M7-M11 map conventions — including the
mandatory Slice 0 "map revalidation" (re-check every file:line claim against
HEAD, update and commit the map before implementation).

**ORCHESTRATOR RULE: nothing in this document is dispatchable.** No worker
packet may be derived from it. When an M12+ item becomes relevant, the only
valid action is: STOP, present the relevant OWNER DECISION block to the owner,
and only after written owner approval author a full design map in
`docs/plan-reviews/` following the M7-M11 map conventions (including Slice 0
revalidation). Milestone numbers for M12+ items are assigned by the owner when
each opens; the order list at the end is priority, not numbering.

Execution preconditions for everything here: M6 (promotion loop) CLOSED, M7
(persistence) CLOSED, plus the per-item prerequisites below. Nothing starts
early because a worker has spare capacity: the order M7 → M8 → M9 → M10 → M11
exists so every M12+ item lands on real persistence, real recovery, and a
hardened provider path instead of RAM-only current_boot state.

---

## 1. Bare-Metal Wi-Fi (Marvell 88W8897 on the bonded Surface Pro 4)

**What it is.** Real wireless on the bonded machine. Today
`seed-kernel/src/wifi.rs` only PCI-probes the Avastar 88W8897 (vendor 0x11ab,
device 0x2b38, Microsoft subsystem) and stores RAM-only SSID/passphrase with
typed validation. No firmware upload, no 802.11 stack, no WPA2, no data path;
the stored credentials do nothing yet.

**Why deliberately late.** e1000 + smoltcp already gives verified
DHCP/DNS/TCP/TLS, so Wi-Fi adds convenience on one machine, not a new
capability class. And it is, honestly, **the highest-risk driver work in the
entire plan** — above AHCI, above xHCI:

- The 88W8897 requires a proprietary firmware blob uploaded at init
  (mwifiex-class). Firmware distribution alone raises licensing and storage
  questions (blob must live in SEED_DATA or the ESP — needs M7).
- No public `no_std` Rust driver exists for this chip. The reference is the
  Linux `mwifiex` C driver (tens of thousands of lines) with partially
  documented command/event interfaces; the chip is known for firmware crashes
  and reset quirks on Surface hardware even under Linux.
- WPA2 needs a supplicant (EAPOL 4-way handshake, PBKDF2-SHA1 PSK, CCMP) — a
  second project on top of the driver.
- Verification is weak: QEMU cannot emulate this chip, so the golden-needle
  harness — the project's ground truth — cannot cover the real driver path.
  Evidence would be physical-hardware serial logs only.

**Hard prerequisites.** M7 (durable firmware blob + credential policy), M8 (a
recovery lifeline for when the driver wedges the machine), M11 preferred (slim
the kernel before adding its largest driver). A physical-hardware test
discipline (serial capture, documented power-cycle recovery) must exist before
the first firmware-upload slice.

**Honest risk assessment.** HIGH. Likely 10k+ lines plus a supplicant, no Rust
prior art, no emulated test target, silent firmware-hang failure modes. This
is the one item where "we decide not to do it" is a respectable outcome.

**OWNER DECISION (before the milestone opens).**
- Option A: full 88W8897 driver + WPA2 supplicant. True to the Tamagotchi bond
  (the machine's own radio). Highest cost/risk, weakest verification.
- Option B: USB-Ethernet adapter with a CDC-ECM (or RNDIS) class driver.
  CDC-ECM is a simple documented USB class; xHCI already exists; frames plug
  into smoltcp essentially where e1000 does; QEMU can emulate a USB network
  device, so golden needles CAN cover it. A fraction of Option A's effort.
  Cost: a dongle on the Surface — wired-over-USB, not wireless.
- Option C: B now, A later as a research lane that may be abandoned.
**Recommendation: Option C.** B delivers "the bonded machine gets real network
without QEMU" cheaply and verifiably; A stays honest as high-risk research
instead of blocking the roadmap.

**Design-map author investigates first.** PCIe-mwifiex vs SDIO attachment on
Surface Pro 4; where the firmware blob legally comes from and how it is
hashed/attested; whether smoltcp's interface abstraction takes an 802.11 data
path unchanged; for B: which dongle chipset, and QEMU USB-net emulation
fidelity for the harness.

---

## 2. External Artifact Distribution ("agent downloads things")

**What it is.** Today every artifact is repo-local: build-time signed and
embedded, or (after M6) candidate bytes injected via the serial harness. This
item makes raiOS fetch artifacts over the network: the parked `ota/`
(BLAKE3+Ed25519 sign/verify CLI), `registry/` (content-addressed store with
non-authorizing evidence records), and `fake-cloud/` (WebSocket
verify-and-publish server) become a real distribution lane, plus a device-side
download client, which today exists NOWHERE.

**Why deliberately late.** ADR 0005 §4 parks this lane explicitly: "No
resumption of the ota/registry/fake-cloud lane without a new ADR." Correctly
so — until M6's promotion loop and M7's persistence are proven, a download
path has nothing safe to feed into.

**The non-negotiable framing: download = candidate intake, NEVER install.** A
downloaded artifact enters exactly the M6 pipeline: inert bytes → hash →
Shadow-VM report bound to that exact hash → local attestation → computed grant
→ promotion transaction → rollback plan. The network gets zero new authority.
A distribution signature proves provenance ("who published this"), never
load-worthiness — that stays local, per ADR 0002.

**Hard prerequisites.** M6 CLOSED (candidates need somewhere to go), M7 CLOSED
(downloaded bytes + evidence need durable storage; a download that vanishes on
reboot is theater), M10 CLOSED (WebPKI + trusted time; pin-only TLS is too
brittle for a distribution endpoint), M11 preferred (the download client
should be a Wasm service on the narrow tcp host imports, not more
kernel-resident internet parsing).

**A NEW ADR is REQUIRED (STOP: owner must approve). It must decide:**
1. Transport: HTTPS pull from a static content-addressed registry
   (recommended — simplest, cacheable, stateless) vs the fake-cloud WebSocket
   push model vs both.
2. Signing authority: how Ed25519/BLAKE3 distribution keys relate to ADR 0002
   local-first attestation. Recommended stance: distribution signature =
   provenance evidence field only; local attestation remains the sole
   authority. Publishing-key custody and rotation must be specified.
3. How a download becomes an M6 candidate: which component fetches (kernel vs
   Wasm service), byte-size bounds, where inert bytes stage (M7D SEED_DATA
   area), and the typed events emitted (`current_boot` until promoted).
4. Revive vs rewrite of the parked ~3.7k-line lane: frozen since 2026-05,
   predating the record model — expect evidence-format reconciliation with
   raios-core.
5. Fail-closed rules: no download without a prior owner-approved manifest
   request; quota/rate bounds; typed denials for everything else.

**Honest risk assessment.** MEDIUM mechanically (plumbing into an existing
gate chain), HIGH conceptually — the largest attack-surface expansion in the
plan. The mitigations already exist; the risk is a shortcut that lets network
bytes skip a gate. The design map must make every skip physically impossible,
not policy-forbidden.

**OWNER DECISIONS.** Approve the new ADR (mandatory); transport (recommend
HTTPS pull); first registry host (recommend the local workstation over LAN
before any public endpoint).

**Design-map author investigates first.** Parked-crate state vs the current
record model; whether fake-cloud is worth reviving at all vs a static-file
registry; how M7D's artifact store stages candidates; how M6A's intake surface
bounds byte sources.

---

## 3. Re-binding to New Hardware (the Tamagotchi model)

**What it is.** README: raiOS doesn't port — it re-binds, building a fresh
instance on new hardware while carrying forward policies, modules, history.
This item defines the mechanism: what travels from machine A to B, what
re-derives on B.

**Carries over (identity):** typed memory records with provenance and
classification (M9), capability policies and grant/denial history, descriptors
and manifests, promoted artifact hashes + evidence chains, the audit/rollback
ledger, owner profile. **Re-derives (body):** drivers and hardware bring-up
(new machine = new PCI IDs; only the new box gets support), calibration, and —
owner decision below — possibly attestation keys. **Never carries:** secrets
(RAM-only rule; provider keys are re-entered on B, never exported), and live
promoted status — every carried artifact re-enters through the M7D
re-promotion gate on B (evidence re-verified before anything runs; ADR 0003
binds VM reports to hardware profile, so some reports must re-run).

**Why deliberately late.** Meaningless before there is anything durable to
carry: M7 and M9 ARE the export surface. An earlier export format would be a
parallel fake persistence layer, which the standing rules forbid.

**Hard prerequisites.** M7 and M9 CLOSED. M10 helpful if the bundle ever moves
over a network instead of physical media.

**Honest risk assessment.** MEDIUM. Mechanically an export/import of
already-typed records — the hard work happens in M7/M9. Real risks: (a)
classification leaks — export MUST apply public/local_only/secret gates
fail-closed, treating machine B as untrusted until proven; (b) evidence
laundering — imported evidence gets `foreign_machine` provenance and never
reads as verified-on-B until re-verified.

**OWNER DECISIONS.** (1) Keys transfer or re-derive? Recommend: re-derive on
B, retaining A's public key as a provenance root — a stolen bundle then grants
nothing on new hardware. (2) Transfer medium: physical USB (recommended first)
vs network. (3) Bond moves vs two live instances — recommend bond moves: one
machine, one bond is the model.

**Design-map author investigates first.** M9's record export shape; whether
the M7 ledger supports a consistent snapshot-for-export; descriptor re-sign
flow with a fresh key; a typed hardware-profile record so "earned on other
hardware" is a field, not prose.

---

## 4. Core-Generation Handoff + Native Service Graph (ADR 0003 long-term)

**What it is.** Two coupled research items. (a) Handoff: updating the
permanent survival core without a visible reboot — load core N+1, freeze
services, snapshot roots, migrate core metadata, switch dispatch, keep N for
rollback. (b) Native service graph: ADR 0003's end state — services in
separate protection domains with versioned state migrators, superseding
interpreter-speed Wasm where performance demands it. ADR 0005 defers (not
rejects) this: entered "only after the Wasm service world demonstrably works."

**Why deliberately late.** These reshape the trust base itself. Everything
else fails safely because the core stays fixed; here the thing guaranteeing
rollback is the thing being replaced. Only M8's lifeline and M7C's last-good
slot survive a botched handoff — both must be battle-tested first.

**Hard prerequisites (evidence, not dates).** ALL of: M7C A/B boot control has
survived real failed-update fallbacks on hardware, not only QEMU; M8's
lifeline has recovered a genuinely wedged system at least once in anger;
multiple externally-authored services have run full promote/rollback cycles
across reboots (M7D) for an extended period; and a concrete pain point exists
that service hot-swap cannot solve (a survival-core bug forcing visible
reboots, or a measured Wasm performance wall on a service that cannot stay
kernel-native).

**Honest risk assessment. RESEARCH-GRADE — not normal milestone work.** No
slice plan, effort estimate, or completion promise is honest at this distance.
The native graph needs a protection-domain mechanism (ring transitions or
per-service address spaces) that does not exist in the kernel at all; handoff
needs a frozen versioned core-metadata ABI that constrains the core forever.
Expect one or more new ADRs plus at least one throwaway prototype branch that
is explicitly allowed to fail.

**OWNER DECISIONS.** Whether to open it at all — a raiOS that reboots for core
updates is a legitimate end state (SAFE mode + A/B slots may be enough
forever); native-graph isolation mechanism (new ADR); how much
reboot-avoidance is actually worth.

**Design-map author investigates first.** Which core state needs migrating
(memory map, capability table, service handles, event ring); whether Limine
constraints allow a second resident kernel image; the minimum frozen N→N+1
ABI; real Wasm performance measurements to test whether the native graph is
needed at all.

---

## Dependency Graph and Recommended Order

```text
M6 Promotion Loop v0  (OPEN — closes first; gate for everything below)
 |
 v
M7 Persistence (M7A layout -> M7B record store -> M7C boot control -> M7D artifact store)
 |----------------+----------------+
 v                v                v
M8 Recovery      M9 Durable       M10 Provider Trust
Lifeline         Memory/Context   Hardening & Adapters
(full scope      (ADR 0004 D,     (WebPKI, time, 2nd provider)
 needs M7C/D)     needs M7B)       |
 |                |                v
 |                |               M11 Kernel Slimming (TLS/HTTP out of kernel)
 |    +-----------+                |
 v    v                            v
[M12+ items]
 3. Re-binding             <- M7 + M9 (+M10 if networked transfer)
 2. External distribution  <- M6 + M7 + M10 (+M11 preferred) + NEW ADR
 1. Wi-Fi / USB-Ethernet   <- M7 + M8 (+M11 preferred); B cheap, A research
 4. Core handoff + native graph <- all of the above proven on hardware; research-grade
```

**Recommended M12+ order.**
1. **USB-Ethernet (Wi-Fi Option B)** — smallest, QEMU-verifiable, gives the
   bonded machine a real network life; may interleave soon after M8 if the
   owner wants hardware progress early.
2. **External distribution** — the owner's core story, highest product value
   once M10 hardens the transport; gated on its new ADR.
3. **Re-binding** — the M9 record world should be lived-in before an export
   format is frozen.
4. **88W8897 Wi-Fi (Option A)** — optional research lane, may run parallel to
   2/3 if the owner accepts the risk, may be abandoned without shame.
5. **Core handoff / native graph** — last, evidence-gated, research-grade.

**Standing STOP-tripwires for all M12+ work:** any new ADR (items 2 and 4
require one; Wi-Fi Option A firmware distribution likely does too), any
trust-model change, destructive disk operations, anything touching
`release/raios-stage0.img`, any secret leaving RAM, and any physical-hardware
test procedure lacking a documented power-cycle recovery path — all stop the
orchestrator and go to the owner first.
