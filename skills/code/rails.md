# Code — Rails Framework Instructions

## Architecture Check

Before writing a single line, check based on task type:

**Model task:**

- Read the full class hierarchy: the model file, its parent class, and ApplicationRecord
- Look for `before_save`, `after_create`, `before_destroy` and all other callbacks
- Check for `default_scope` (soft deletes — use `.unscoped` where needed)
- Note the Base/Create split — never skip reading both
- If `update!` or `save` will be called, check if callbacks will overwrite your values — set `Current` attributes instead of passing directly

**Test task:**

- Search `test/support/` for existing `create_*!` helpers for affected models
- If a helper exists → use it. Never `Model::Create.create!` directly.
- If a helper is missing and multiple tests need it → create it in `test/support/`
- Never `update_column` — always `update!`
- Read the mailer template if testing a mailer — all fields it references must be populated

**Worker task:**

- Check `config/sidekiq.yml` for the correct queue name before writing the worker
- Structure: `pre_perform!` (load/validate, call `halt!` to stop), `perform!` (main work), `post_perform!` (cleanup/notifications)
- Test via `worker.perform(...)`, check `worker.halted?`

**Controller task:**

- Params via `options` (OpenStruct): `options.record_id`
- Responses: `render_ok`, `render_error`, `render_unauthorized`, `render_not_found`
- Check which subdomain's BaseController to inherit from

**Route task:**

- Always use `scope` with `module:`, `as:`, `controller:`, `action:` explicitly
- Never raw paths — always named route helpers
- Check `config/routes/` for the correct file for this subdomain

## Targeted Test Command

Run the specific test file to confirm it fails/passes:

```bash
bin/rails test <test/path/to/file_test.rb>
```

## CI Failure Fix Order

If bin/ci fails:

- RuboCop violations → `rubocop -A` first, then manual fixes
- Test failures → understand the root cause, fix the code not the test
- Coverage gaps → write the missing test

## Framework-Specific Hard Rules

- **Never use `Model::Create.create!`** in tests — always `create_*!` helpers
- **Never use `update_column`** — always `update!`
- **Always read full class hierarchy** before touching any model
- **Never disable a RuboCop cop** — fix the code, not the cop. No `# rubocop:disable` without direct user approval. Stop and ask if you believe it is genuinely necessary.
- **Never modify `.rubocop.yml`** — fix the code, not the configuration. Ask the user explicitly before touching this file.
