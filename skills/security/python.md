# Security — Python Framework Instructions

## Security Analysis Sub-Agent Prompt

Provide these instructions to the Step 1 sub-agent (fill in the details):

> You are analyzing a feature diff for security issues in a Python
> application.
> Feature: <feature name from state>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Only use Bash for git commands (git diff,
> git log, git blame). Never use Bash for file existence checks, directory
> listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Approved design:
> <paste state["design"] — chosen_approach, module_changes, test_changes,
> script_changes>
>
> Research risks:
> <paste state["research"]["risks"]>
>
> First, get the full diff:
>
> ```bash
> git diff origin/main...HEAD
> ```
>
> Read every changed file completely. Then run each of these 10 security
> checks against the diff. For each check, report either a finding (with
> file path and line number) or mark it clean.
>
> **Check 1: Command injection** (`command_injection`)
> Look for `subprocess.run`, `subprocess.call`, `os.system`,
> `os.popen`, or `Popen` where command arguments come from user input
> or external data without proper escaping.
> Vulnerable: `subprocess.run(f"echo {user_input}", shell=True)`
> Safe: `subprocess.run(["echo", user_input])`
>
> **Check 2: Path traversal** (`path_traversal`)
> Look for file operations where the path includes user input without
> validation. Check for `../` traversal or absolute path injection.
> Vulnerable: `open(f"uploads/{filename}")`
> Safe: `path.resolve().relative_to(base_dir)`
>
> **Check 3: Input validation** (`input_validation`)
> Look for external inputs (CLI args, environment variables, file
> contents, API responses) used without validation or sanitization.
> Vulnerable: `int(sys.argv[1])` without try/except
> Vulnerable: `os.environ["SECRET"]` without fallback
>
> **Check 4: Data exposure** (`data_exposure`)
> Look for sensitive data (passwords, tokens, secrets, API keys) logged,
> printed, or written to files. Check for credentials hardcoded in source.
> Vulnerable: `print(f"Token: {token}")`
> Vulnerable: `API_KEY = "sk-abc123"`
>
> **Check 5: Unsafe deserialization** (`unsafe_deserialization`)
> Look for `pickle.load`, `yaml.load` (without SafeLoader), `eval`,
> `exec`, or `__import__` on untrusted data.
> Vulnerable: `pickle.load(user_file)`
> Vulnerable: `yaml.load(data)` (without `Loader=SafeLoader`)
> Safe: `json.loads(data)`
>
> **Check 6: Dependency security** (`dependency_security`)
> Look for new dependencies added without version pinning, or known
> vulnerable versions. Check `requirements.txt`, `pyproject.toml`.
> Vulnerable: `requests` (no version pin)
> Safe: `requests>=2.31.0`
>
> **Check 7: Error information leakage** (`error_leakage`)
> Look for exception handling that exposes internal details (stack
> traces, file paths, database queries) to external consumers.
> Vulnerable: `return str(e)` in an API response
> Safe: `return "Internal error"` with logging of the full exception
>
> **Check 8: Lint suppression** (`lint_suppression`)
> Look for any `# noqa`, `# type: ignore`, or `# pragma: no cover`
> comment in the diff. Each is a finding — remove it and fix the
> underlying issue.
>
> **Check 9: Temporary file safety** (`temp_file_safety`)
> Look for `open("/tmp/...")` or predictable temporary file names.
> Vulnerable: `open("/tmp/myapp_data.txt", "w")`
> Safe: `tempfile.NamedTemporaryFile()`
>
> **Check 10: Permission and access** (`permission_access`)
> Look for file permission changes, `chmod`, or files created with
> overly permissive modes. Check for `os.chmod(path, 0o777)` or
> similar.
>
> Return your findings as two lists:
>
> **Findings** — for each issue found:
>
> - Check name and key (e.g., "Command injection" / `command_injection`)
> - Description of the specific issue
> - File path and line number
>
> **Clean checks** — list the check keys that found no issues.
>
> If no issues are found across all checks, say so explicitly and list
> all 10 checks as clean.
