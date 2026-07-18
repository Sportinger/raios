#!/usr/bin/env bash
# Blockt Vordergrund-Dispatches von Agenten (codex exec / claude -p):
# sie MUESSEN mit run_in_background=true laufen (CLAUDE.md "Codex-Worker direkt").
input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // ""')
bg=$(printf '%s' "$input" | jq -r '.tool_input.run_in_background // false')
if [ "$bg" != "true" ]; then
  case "$cmd" in
    *"codex exec"*|*"claude -p"*|*"claude --print"*)
      printf '%s' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Agenten-Dispatch nur im Hintergrund: denselben Aufruf mit run_in_background=true starten (Fertig-Notification kommt automatisch, Zwischenstand ueber die Task-Output-Datei)."}}'
      ;;
  esac
fi
exit 0
