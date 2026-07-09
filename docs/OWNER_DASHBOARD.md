# Owner Dashboard

One page, plain language, updated every session (rule: AGENTS.md,
"Capability Definition Of Done"). Hard cap: ~30 content lines.

Updated: 2026-07-09.

Current capability: the default local boot image is refreshed for the next
Surface Pro 4 owner-key capture attempt. `release\raios-stage0.img` now contains
the `ownerkey` console command and the current
`system.honesty_report.owner_key_provisioning` path.

Owner-key behavior today: RAM boot automatically creates a secret, RAM-only
`current_boot` owner-key candidate from entropy and exposes only a stable handle
plus `sha256:` fingerprint. Persistent install is only a policy target
(`generate_hardware_bound_owner_key_on_persistent_install`); no persistent key,
owner seal, load authority, or durable-write authority is granted until real
TPM/hardware seal-unseal evidence exists.

Latest focused proof: `m12-distribution-provenance`
`shadow-20260709-115747-6148.json` passed 252/252 against the exact default
image, not a temp image. Image SHA-256:
`96c2d84b85a2831533a4312660df01d5930b1d96a14b2ee6e36f93ec2e9a4268`.

Gate status: latest full profile remains green at
`shadow-20260708-150428-34396.json` 7867/7867. The focused M12 report above is
the current slice evidence per aggressive-fast cadence.

Next owner action: write the refreshed `release\raios-stage0.img` to USB, boot
the Surface Pro 4, run `ownerkey` or `system.honesty_report`, and capture the
real TPM2 ACPI/interface fields. If CRB/TIS is present, the next code slice is a
read-only TPM register-status probe; authority stays fail-closed.
