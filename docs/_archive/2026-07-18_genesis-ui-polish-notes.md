# Genesis UI polish notes

- Reused the existing Genesis palette: amber for pending install/persist, blue for workspace run, green for runnable/durable RUIP state.
- Added explicit `[INSTALL]`, `[PERSIST]`, `[RUN]`, and `[RUN + PERSIST]` labels so approval kinds differ without new layout machinery.
- Standardized visible preview hashes to a label plus the existing first-four-byte `...` form.
- Read RUIP truth from the existing `program_workspace::snapshot()` fields; no getter or authority path was added.
- Rendered durable state as installed plus `durable (survives reboot)`; RAM-only state says `current boot only`.
- Kept `context_personal_shell_rect()` untouched, preserving the approval button origin, size, center, and QMP click target.
- Added text only below the fixed approval button; setup and WiFi rectangles are unchanged.
- Made no serial, signed-shell, personal-surface, or agent-protocol changes.

## present_rect package

- Personal-focus updates now present the layout-derived personal surface and secure strip as two clipped, pitch-aware regions.
- Both logical layout rectangles are scaled to their exact physical framebuffer bounds before copying.
- Full Genesis composition, including personal-frame fallback, still uses the unchanged full-buffer `present()` path.
- `draw_personal_frame` writes only inside the personal surface; clipped text can extend only into the framebuffer edge, where pixel writes are already clipped.
- `draw_secure_strip` writes only inside the layout-derived secure strip, so no third damage region is needed.
