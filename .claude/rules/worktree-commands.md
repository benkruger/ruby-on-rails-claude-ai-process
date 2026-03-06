# Worktree Commands

## Use `git -C` instead of `cd && git`

Never use `cd <path> && git <command>`. Claude Code's built-in "bare
repository attacks" heuristic fires on any `cd <path> && git` compound
command, regardless of the allow list in settings.json.

Use `git -C <path> <command>` instead — it runs git in the target
directory without changing the shell's working directory, and it matches
a single permission pattern (`Bash(git -C *)`).

## Use dedicated tools instead of Bash for reads

Never use `grep`, `cat`, `head`, `tail`, or `find` via the Bash tool.
Use the Grep tool for content search, the Read tool for file contents,
and the Glob tool for file discovery. Dedicated tools never trigger
permission prompts.
