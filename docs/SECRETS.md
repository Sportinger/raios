# Secret Handling

SeedOS must not commit provider keys or key-bearing boot artifacts.

## Local OpenAI Image

Build a local provider-default image only from the current process environment,
using a temporary ESP staging tree:

```powershell
$env:OPENAI_API_KEY = "<local key>"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-stage0.ps1 -Profile release -Image release\seedos-stage0-local-openai.img -UseTempEsp -EmbedOpenAiApiKeyFromEnv
```

`package-stage0.ps1` refuses to embed a provider key into `release\esp` or the
default `release\seedos-stage0.img`. Local OpenAI images are ignored by Git.

## Local OpenAI Boot Stick

Write a USB stick directly from the environment without creating a tracked image:

```powershell
$env:OPENAI_API_KEY = "<local key>"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\write-stage0-usb.ps1 -DiskNumber <N> -ConfirmErase "ERASE DISK <N>" -EmbedOpenAiApiKeyFromEnv
```

The USB script builds a fresh local kernel, copies it through a temporary ESP
tree, and refuses `-SkipBuild` when key embedding is requested.

## Scan Before Commit

Scan tracked files:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\scan-secrets.ps1
```

Scan the full worktree, including ignored local images but excluding `target`:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\scan-secrets.ps1 -IncludeIgnored
```

The scanner reports paths and counts only. It does not print secret values.

Scan Git history paths without printing values:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\scan-secrets.ps1 -GitHistory
```

## If A Real Key Was Committed

Rotate the provider key first. Removing it from the current tree is not enough
if it was pushed or present in Git history. After rotation, rewrite the affected
history with a repository-wide secret-removal tool and coordinate a force-push
with every clone owner.
