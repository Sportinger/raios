# JSON diagnostic parser fixtures

These files are fixed examples of the `raios.cargo_diag.v0` envelope:

- `positive.json` records a successful check with exit code 0 and no errors.
- `negative-e0308.json` records the deliberately broken fixture with exit code
  1 and error E0308 at `src\lib.rs:3:24`.

They are parser fixtures, not evidence for the current commit or toolchain.
Their original commit, Rust version, and Cargo version were not captured. The
`rendered` field is host- and toolchain-dependent and must not be treated as a
stable string contract; consumers should assert the structured fields instead.
