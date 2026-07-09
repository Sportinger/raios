# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: Surface owner-key capture now shows the next TPM register
raiOS would read. `ownerkey` and `system.honesty_report.owner_key_provisioning`
report whether a read-only TPM status-register plan exists, the register kind,
physical address, width, and reason.

Owner-key behavior today: RAM boot creates a secret, RAM-only `current_boot`
owner-key candidate from entropy and exposes only handle + `sha256:`
fingerprint. Persistent install remains policy-only; no persistent key, owner
seal, load authority, or durable-write authority is granted.

Latest focused proof: `m12-distribution-provenance`
`shadow-20260709-120614-8340.json` passed 253/253 against the exact default
image, not a temp image. Image SHA-256:
`564087d277d029bdc84efe481137d9edc38d07868be23fdf024925c8177772c3`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. This slice used the focused M12
profile per aggressive-fast cadence.

Next owner action: boot this refreshed USB on the Surface Pro 4, run `ownerkey`
or `system.honesty_report`, and capture TPM2 ACPI/interface/status-plan fields.
If the plan is available, the next code slice is the actual read-only volatile
TPM status-register read; authority stays fail-closed.
