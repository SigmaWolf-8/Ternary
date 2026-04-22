# Incident Remediation: GitHub PAT Embedded in `.git/config` Origin URL

- **Date discovered:** 2026-04-22
- **Date remediated (local scrub):** 2026-04-22
- **Severity:** High — credential disclosure
- **Author of remediation record:** Replit Agent acting under task #142
- **Companion record:** `docs/incidents/INCIDENT_2026-04-22_BLAKE3_INTRODUCTION.md`

## Summary

The `origin` remote in this Repl's local `.git/config` carried a GitHub
Personal Access Token (PAT) embedded directly in the remote URL, of the form:

```
https://x-access-token:ghp_RE2C…9Su4@github.com/SigmaWolf-8/Ternary.git
```

Anyone with shell, file-read, or `git remote -v` access to this Repl could
read the live token. The token prefix is recorded here (last four characters
only) for cross-referencing the upstream revocation; the full token value is
not committed.

## Containment scope

- The leaked PAT lived **only in the live `.git/config` string**, not in any
  tracked file in committed git history. A `git grep` sweep over committed
  content for the `ghp_` and `github_pat_` prefixes returned zero hits at the
  time of remediation. **No git history rewrite was required.**
- No force-push or branch surgery was performed.
- The 41 `subrepl-*` SSH remotes and the `gitsafe-backup` remote were audited
  and confirmed credential-free (no inline `user:password@` segments).

## Actions taken (local)

1. `origin` URL reset to the bare credential-free form:
   `https://github.com/SigmaWolf-8/Ternary.git`
2. Git credential helper wired for `https://github.com` to read the
   `GITHUB_TOKEN` environment secret at request time, so no token value is
   ever written into `.git/config`, shell history, or process arguments. The
   helper config is:
   ```
   credential.https://github.com.helper = !f() { echo username=x-access-token; echo "password=${GITHUB_TOKEN}"; }; f
   credential.https://github.com.usehttppath = false
   ```
3. Verified `git credential fill` for `host=github.com` returns the helper's
   `username=x-access-token` and a non-empty password sourced from the env
   secret (length consistent with a fine-grained PAT).
4. Verified zero matches of `ghp_`, `github_pat_`, or any `x-access-token:…@`
   URL-embedded form anywhere in `.git/config`. The string `x-access-token`
   now appears exactly once — as the static username inside the helper script
   line — and is not a credential.

## Action required upstream (user-side, out of agent scope)

The leaked PAT value must be **revoked at github.com/settings/tokens** by
the owning account holder. Until that happens upstream, the leaked value
remains valid for any party that captured it from this Repl prior to the
local scrub. The local scrub stops further leakage from this environment;
it does not invalidate the previously exposed value.

## Verification

Reproducible verification commands:

```sh
# Confirm origin URL has no embedded credential
git remote get-url origin

# Confirm helper resolves from env, without printing the token value
printf 'protocol=https\nhost=github.com\n\n' | git credential fill | grep -v '^password='

# Confirm no PAT-bearing strings remain in config
grep -nE 'ghp_|github_pat_|x-access-token:[^"]+@' .git/config && echo BAD || echo CLEAN
```

Expected output of the last command: `CLEAN`.
