# B1.2c kernel implementation notes

- Reused `ProjectInstallAction` unchanged for W6; its existing
  `install_envelope_sha256` field identifies the sealed W7 envelope.
- Old promotion records parse with absent W6 booleans as `false`, so they stay
  readable but cannot satisfy the new re-verification pins.
- The four scoped durable-append pins are evaluated after the existing M6
  signature/trust checks, preserving their prior first-failure reasons.
