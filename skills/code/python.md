# Code — Python Framework Instructions

## Architecture Check

Before writing a single line, check based on task type:

**Module task:**

- Read the full module and its imports
- Check for circular import risks
- Note any module-level state or initialization
- If modifying a function signature, grep for all callers

**Test task:**

- Check `conftest.py` for existing fixtures for affected modules
- If a fixture exists → use it. Never duplicate fixture logic.
- If a fixture is missing and multiple tests need it → create it in `conftest.py`
- Follow existing test patterns in the project

**Script task:**

- Read the argument parsing and main flow
- Check for error handling and exit codes
- Verify the script is registered in any entry points or bin/ wrappers

## Targeted Test Command

Run the specific test file to confirm it fails/passes:

```bash
bin/test <tests/path/to/test_file.py>
```

## CI Failure Fix Order

If bin/ci fails:

- Lint violations → read the lint output carefully, fix the code
- Test failures → understand the root cause, fix the code not the test
- Coverage gaps → write the missing test

## Framework-Specific Hard Rules

- **Always read module imports** before modifying any module
- **Always check `conftest.py`** for existing fixtures before creating new ones
- **Never add lint exclusions** — fix the code, not the linter configuration
