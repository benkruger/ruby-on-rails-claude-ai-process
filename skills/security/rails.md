# Security — Rails Framework Instructions

## Security Analysis Sub-Agent Prompt

Provide these instructions to the Step 1 sub-agent (fill in the details):

> You are analyzing a feature diff for security issues in a Ruby on Rails
> application.
> Feature: <feature name from state>
>
> **Tool rules:** Use Glob and Read tools for all file and directory checks.
> Use Grep for searching code. Only use Bash for git commands (git diff,
> git log, git blame). Never use Bash for file existence checks, directory
> listings, or reading file contents (`test -f`, `ls`, `cat`, etc.).
>
> Approved design:
> <paste state["design"] — chosen_approach, schema_changes, model_changes,
> controller_changes, worker_changes, route_changes>
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
> **Check 1: Authorization gaps** (`authorization_gaps`)
> Look for controller actions added or modified in the diff that have no
> `before_action` authentication/authorization filter. Check whether
> existing auth filters are skipped for new actions.
> Vulnerable pattern: a new action with no auth filter covering it
> Vulnerable pattern: `skip_before_action :require_login, only: [:new_action]`
>
> **Check 2: Unscoped record access** (`unscoped_access`)
> Look for record lookups that use `params[:id]` (or similar) without
> scoping to the current user, account, or tenant.
> Vulnerable: `Record::Base.find(params[:id])`
> Safe: `current_account.records.find(params[:id])`
>
> **Check 3: Mass assignment** (`mass_assignment`)
> Look for `params.permit!` (permits everything), overly broad `permit`
> calls that include admin-only or internal fields, or params hashes
> passed directly to `create!`/`update!` without `permit`.
> Vulnerable: `Record::Create.create!(params[:record])`
> Vulnerable: `params.require(:record).permit!`
> Vulnerable: `permit(:role, :admin, :internal_flag)` when user-facing
>
> **Check 4: SQL injection** (`sql_injection`)
> Look for string interpolation or concatenation inside `where`, `order`,
> `select`, `group`, `having`, `pluck`, `execute`, `find_by_sql`, or
> `ActiveRecord::Base.connection` calls.
> Vulnerable: `where("name = '#{params[:name]}'")`
> Vulnerable: `order("#{params[:sort]} #{params[:dir]}")`
> Safe: `where(name: params[:name])`
> Safe: `where("name = ?", params[:name])`
>
> **Check 5: Data exposure** (`data_exposure`)
> Look for sensitive fields (password, token, secret, ssn, credit card,
> api\_key) included in `as_json`, `to_json`, serializer attributes,
> or API responses. Check for PII logged via `Rails.logger` or `puts`.
> Check for credentials or secrets hardcoded in source.
> Vulnerable: `render json: user.as_json`
> Vulnerable: `Rails.logger.info("User #{user.email} token: #{user.token}")`
>
> **Check 6: CSRF bypass** (`csrf_bypass`)
> Look for `skip_before_action :verify_authenticity_token`. This is only
> acceptable on API-only controllers that use token auth instead of
> cookies. If the controller serves browser requests with cookie auth,
> this is a finding.
> Vulnerable: `skip_before_action :verify_authenticity_token` on a
> controller that uses session/cookie auth
>
> **Check 7: Open redirects** (`open_redirects`)
> Look for `redirect_to` where the URL comes from user input (params,
> form fields, headers). Attackers use this for phishing.
> Vulnerable: `redirect_to params[:return_url]`
> Vulnerable: `redirect_to request.referer`
> Safe: `redirect_to root_path`
> Safe: `redirect_to params[:return_url], allow_other_host: false`
>
> **Check 8: RuboCop disables** (`rubocop_disables`)
> Look for any `# rubocop:disable` comment anywhere in the diff. This is
> an automatic finding — no judgment needed. Every disable must be
> removed and the underlying code fixed.
>
> **Check 9: Auth test coverage** (`auth_test_coverage`)
> If the diff adds a `before_action` auth check or an authorization
> gate, look for a corresponding test that verifies the unauthorized
> or forbidden case. A new auth check without a test for the reject
> path is a finding.
>
> **Check 10: Route exposure** (`route_exposure`)
> Look for new routes (in `config/routes.rb` or `config/routes/*.rb`)
> that point to controller actions. Read the target controller and
> verify it has an auth filter covering the routed action. A route to
> an unprotected action is a finding.
>
> Return your findings as two lists:
>
> **Findings** — for each issue found:
>
> - Check name and key (e.g., "Authorization gaps" / `authorization_gaps`)
> - Description of the specific issue
> - File path and line number
>
> **Clean checks** — list the check keys that found no issues.
>
> If no issues are found across all checks, say so explicitly and list
> all 10 checks as clean.
