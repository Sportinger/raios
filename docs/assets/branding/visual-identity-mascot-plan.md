# Visual Identity Mascot Plan

## Status

Exploratory architecture plan. This is not a current roadmap gate and should not
displace the active provider-context evidence work.

## Intent

raiOS may eventually expose a small, living visual identity for the local
machine-user-agent bond. The goal is not a decorative chatbot avatar. The goal
is a recognizable local phenotype: a visual petname for the specific raiOS
instance, shaped by real system facts, capability history, recovery evidence,
and carefully bounded randomness.

The core idea:

```text
stable bond seed -> visual genome -> constrained phenotype
real system events -> typed evolution pressure
local entropy -> individual variation and rare mutations
```

The mascot must remain a user-facing hint, not an authority. The authoritative
identity remains typed records, hashes, capability grants, audit events, and
evidence.

## Critical Assessment

### Why It Could Be Valuable

- A visual identity can make a bonded OS feel less interchangeable without
  adding a cloud account or profile system.
- A local visual petname can help the user recognize "this raiOS" in the same
  spirit as SSH randomart or identicons, while still keeping real verification
  in cryptographic records.
- Evolution can make system history visible at a glance: capabilities gained,
  repair history, provider trust posture, current problem pressure, and service
  graph maturity.
- The visual layer can become a test of the memory architecture: only typed,
  classified, evidence-bound facts should be allowed to affect it.

### Why It Is Risky

- If the visual layer reads raw chat, prompts, secrets, or unclassified memory,
  it becomes a privacy leak.
- If users infer security from the mascot, it becomes dangerous. A familiar
  shape is not proof that keys, provider trust, or policy are valid.
- If every event directly redraws the mascot, the result will feel noisy and
  arbitrary rather than alive.
- If randomness is uncontrolled, the mascot becomes meaningless decoration.
- If users can force exact rare traits, the system becomes a character
  customizer instead of an evidence-shaped phenotype.
- If persistent visual state is added before real persistence, rollback, and
  audit exist, it violates the system memory architecture.
- If the mascot becomes too human-like or emotionally pushy, it risks
  manipulating attachment instead of representing system state honestly.

## Hard Rules

```text
visual similarity is never security proof
the visual layer must not consume secrets or raw chat text
unknown fields are not visual inputs
summaries and semantic hits are locators only
all durable mutations cite typed source events and evidence
no fake persistent mascot memory before persistence and rollback exist
provider requests never receive visual state by default
```

The visual layer should have its own allowlist of coarse, visual-safe signals.
Even `local_only` data can leak if the user shares a screen, so visual inputs
must be coarser than normal local agent context.

## Comparable Patterns

- Identicons, GitHub generated avatars, Gravatar defaults, and Robohash prove
  that hashes can generate recognizable unique images. Their weakness is that
  they are mostly static and semantically shallow.
- OpenSSH visual host keys show how a cryptographic fingerprint can become a
  visual pattern for recognition. The same warning applies: visual recognition
  is probabilistic and must not replace verification.
- Petname systems map cryptographic identifiers into human-usable local names.
  raiOS can use a visual petname rather than only a text name.
- DIDs and verifiable identity systems separate identifiers from documents and
  claims. raiOS should similarly separate bond id, visual genome, and evidence.
- Dynamic NFTs show the useful pattern of stable identity plus changing
  metadata. raiOS should not need a blockchain, but the split between identity
  and evolution is relevant.
- CPPNs, NEAT-style encodings, and procedural art are good references for
  generating splines, symmetry, organic regularity, and constrained variation
  from compact genomes.

## Proposed Architecture

### Identity Layer

The stable root must be local, non-reversible, and domain separated:

```text
bond_secret = HKDF(
  device_secret || install_random || user_bond_secret || agent_public_id,
  "raios.identity.bond.v0"
)

visual_secret = HKDF(bond_secret, "raios.visual.identity.v0")
public_visual_id = BLAKE3(visual_secret || "public visual id")
```

`bond_secret` and `visual_secret` are never exported. `public_visual_id` is only
a locator and should not be treated as a proof of identity.

### Visual Genome

The genome is the stable base shape. It should define parameters, not raw pixels:

```text
schema: raios.visual_genome.v0
classification: local_only
source: raios.identity.bond.v0
fields:
  skeleton_family
  symmetry_bias
  spline_count_range
  control_point_distribution
  palette_family
  motion_family
  texture_rule
  mutation_slots
```

Hash bytes should seed a constrained generator. They should not be used directly
as uncontrolled coordinates.

### State Projection

The live visual state is a redacted projection over typed system facts:

```text
schema: raios.visual_state_projection.v0
scope: current_boot
inputs:
  system stage
  provider trust marker
  service health classes
  problem severity counts
  capability ids and denied capability classes
  event pressure by class
  recovery/test evidence presence
omitted:
  raw boot log
  raw prompts
  raw provider responses
  secrets and sealed handles
  network topology details
  unclassified memory
```

The projection should affect coarse mood and posture: calmness, jitter,
protective shell, growth readiness, damage marks, or repair marks.

### Mutation Events

Durable evolution should be event sourced:

```json
{
  "schema": "raios.visual_mutation_event.v0",
  "id": "visual.mutation.current_boot.00000007",
  "source_event_id": "event.current_boot.00000042",
  "source_kind": "service.created",
  "source_class": "editor_tool_created",
  "evidence": ["vm_test_report:..."],
  "rng_commitment": "sha256:...",
  "visual_schema": "raios.visual_genome.v0",
  "effect": {
    "kind": "add_tool_node",
    "semantic_slot": "editor",
    "variant": "spline_plate_ring"
  },
  "classification": "public"
}
```

For Stage-0, these records must remain `current_boot` until real persistence,
rollback, and durable audit exist.

## Randomness Model

Randomness should add individuality without erasing meaning.

```text
meaning is deterministic
appearance is seeded-random
rare mutations are entropy-gated
```

For example, if many users create a text editor first, the semantic effect can
be the same while the form differs:

```text
semantic class: first editor tool
common effect: add a work/input node
variation: position, curvature, palette shift, pulse rhythm, ornament type
```

Per-event variation:

```text
event_seed = HMAC(
  visual_secret,
  canonical_event_hash || rng_commitment || visual_schema_version
)
```

The event class decides which mutation families are legal. Entropy chooses
inside that family. This keeps shared milestones legible while avoiding cloned
mascots on identical hardware or identical first workflows.

## V0 Scope

V0 should be documentation and deterministic test infrastructure only. It should
not add a persistent mascot store and should not add provider-exported visual
context.

Durable first slice:

1. Specify `raios.visual_genome.v0`,
   `raios.visual_state_projection.v0`, and
   `raios.visual_mutation_event.v0`.
2. Define the visual-safe input allowlist and classification rules.
3. Build a host-side renderer test harness over captured real protocol outputs:
   `system.snapshot`, `service.inventory`, `problem.list`, and
   `memory.recent_events`.
4. Prove deterministic behavior with fixtures:
   same seed plus same events gives same phenotype, while different
   `install_random` values produce distinct variants.
5. Only after that, consider a small framebuffer glyph fed by current-boot
   typed facts.

The renderer harness is test infrastructure. It must not pretend that in-OS
persistent visual memory exists.

## Test Requirements

- Same `visual_secret`, same schema, same canonical event stream: identical
  phenotype.
- Same event class, different `visual_secret`: same semantic mutation family,
  different geometry.
- Secret, unclassified, and raw-text fields cannot affect output.
- Unknown service ids or problem ids fall back to neutral visual classes, not
  hidden random mappings.
- Provider context export remains unaffected by visual state.
- A visual state change cites a typed source event or is labeled transient.
- Schema-version changes are explicit and can intentionally alter generation.

## Abandon Or Redesign If

- The mascot needs raw chat history to feel meaningful.
- The visual output exposes sensitive work, hardware details, network topology,
  or provider contents.
- The user starts treating the mascot as proof of security state.
- The feature starts requiring persistence before the persistence/rollback
  architecture exists.
- The UI pushes emotional attachment more strongly than system legibility.
- It delays provider trust, memory evidence, capability policy, or recovery
  architecture work.

## Open Questions

- Should the visual renderer be a replaceable service from the start, or a tiny
  framebuffer component until service loading exists?
- Which visual signals are safe when the user is screen-sharing?
- How should a user reset, hide, or re-bind the visual identity?
- Should rare mutations require explicit user-visible audit entries?
- How much accessibility control is needed for color blindness, motion
  sensitivity, and low-vision use?
- What is the minimum visual vocabulary that communicates state without turning
  the system into a game?

## Recommended Next Step

Do not implement runtime mascot behavior yet. First, write the schema spec and a
small deterministic renderer plan that consumes only captured, real
`current_boot` protocol packets. Keep all output local and non-authoritative.

## Prototype

A host-side canvas prototype exists at `docs/visual-soul-prototype.html`. It is
not raiOS runtime code. It uses synthetic visual-safe pressure values, a local
bond seed, a keyed style transform, splines, a service halo, and a simple face
to explore how a phenotype could feel generative without exposing raw memory.

Prototype constraints:

- no provider calls
- no real memory records
- no persistence
- no security authority
- no kernel or framebuffer integration
