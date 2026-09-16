# Gap Analysis: agent_plugin_nest — hub mode is only half implemented

## Problem Statement

The `marketplace` spec key suppresses the two local marketplace files and nothing else. Every other
part of the generator that needs to know *which* marketplace a plugin belongs to still derives that
name from fields the documentation tells a hub-bound spec to delete — so following the documented
path produces install instructions naming a marketplace that does not exist, with no error anywhere.
Separately, the example spec that the regeneration guard measures is not hub-mode, so the guard will
fail the moment colgrep-mcp relinquishes its marketplace.

Reported by the coordinator of `nest_migration` in colgrep-mcp after consuming the reshaped
generator; every claim below was verified against this repository's own code before this report was
written.

## Evidence

| # | Claim | Verified at |
|:--|:------|:------------|
| 1 | Hub mode gates only `merge_marketplace` | `spawn_plugin.py:612` — `if not spec.get("marketplace"):` wraps lines 614-616 alone |
| 2 | `claude_marketplace.name` still feeds the generated dev README | `spawn_plugin.py:599` computes `market`, passed to `dev_readme()` at `:601` |
| 3 | Both name fields still feed the install snippet | `spawn_plugin.py:744-745`, each falling back to `f"{spec['name']}-marketplace"` |
| 4 | The docs tell a hub-bound spec to drop both fields | `references/manifests.md:191-193` — "a spec bound for a hub carries no `claude_marketplace` or `codex.marketplace_name` section at all" |
| 5 | `dev/README.md` is written through `put()`, so `--force` overwrites it | `spawn_plugin.py:601` |
| 6 | The guard exempts that file as hand-maintained | `evals/test_regeneration.py` — `ALLOWED_DIVERGENCE["dev/README.md"] = None` |
| 7 | `install_snippet` derives its slug from the product repo | `spawn_plugin.py:742-743` — `slug` from `spec["repository"]` |
| 8 | The `plugin.json` guard entry is now inert | colgrep-mcp's manifest carries `extensions`, so the subset check passes trivially |

Consequence of 1-4 together: a spec that follows the documentation emits
`claude plugin install colgrep-mcp-dev@colgrep-mcp-marketplace` into `dev/README.md`. That
marketplace does not exist. Nothing warns.

Consequence of 5-6: `spawn --force` regenerates the very file the guard documents as deliberately
divergent. It destroyed real content in colgrep-mcp (the dev plugin's eval-case location, the
skill-creator attribution for the progressive-disclosure layout, and the CONTRIBUTING framing),
caught only on diff review.

Consequence of 7: in hub mode the marketplace lives in the hub, so `marketplace add <product-slug>`
is backwards. Note the `marketplace` key accepts **any truthy value**, so the hub's own slug can be
carried in it — `"marketplace": "CrackingShells/Nest"` already works as a sentinel today and needs
no new field.

## Root Cause

Hub mode was specified as "suppress the marketplace files" and implemented exactly that literally.
The campaign never asked what *else* consumes marketplace identity, so the two consumers that are
not marketplace files were missed — and the documentation then described the narrow implementation
as though it were the whole feature. The `dev/README.md` exposure is older: the guard's exemption
records that the file diverges without protecting it, which reads as safety but is only bookkeeping.

## Impact Assessment

- **Scope**: the example spec, `install_snippet`, `dev_readme`'s protection under `--force`,
  `references/manifests.md`'s hub-mode paragraph, one stale guard entry, one new trap.
- **Not in scope**: `init_spec`'s multi-plugin blindness, already documented as a known gap.
- **Dependencies**: must land before colgrep-mcp's `relinquish_marketplace`, which is what turns
  finding 1 from latent into a red guard. Independent of `verify/end_to_end`, but belongs in the
  same push.
- **A related trap with no code fix**: once a repo declares hub mode its marketplace files are never
  regenerated, so a generator fix to their contents can never reach an already-generated repo. The
  `authentication: "NONE"` fix is the live instance — colgrep-mcp's file had to be repaired by hand.
  Documentation only.

## Chosen Resolution (A1, approved 2026-09-16)

The maintainer rejected the first proposed fix — sniffing whether the `marketplace` key's value
looked like an `owner/repo` slug — on the grounds that this is a *skill*, not only a CLI: the LLM
using it can ask the human for information the human already knows, rather than the generator
inferring it. That is the better resolution, and the reason it is better is visible in finding 2
and 3 above: the only two consumers of marketplace identity are `dev_readme()` and
`install_snippet`, both **documentation outputs**. No manifest that a loader reads needs it. So the
generator was never the right place to derive that identity.

The fields therefore stay, reframed: `claude_marketplace.name` and `codex.marketplace_name` are the
*recorded answer* to a question the skill asks. Recording it rather than prompting at generation
time is what keeps `spawn` reproducible — an interactively supplied name would make the regeneration
guard meaningless, since the guard re-runs `spawn` and compares output.

Two consequences for the fix:

- The `marketplace` key gains an explicit shape carrying the hub repository, and `load_spec`
  **fails** when hub mode is declared with no hub recorded, naming the key to add. A load-time error
  is how a human gets asked. Falling back to `spec["repository"]` is the wrong answer by
  construction, and a hedged "cannot determine the hub" line in generated prose is worse than
  refusing to generate it.
- `SKILL.md` gains the question in its spec-writing step, stated as a principle rather than a
  special case: the generator reads recorded answers and never infers human-facing identity.

Findings 5 and 6 (`dev/README.md` clobbered under `--force`) and finding 1 (the example spec not
being hub-mode) are untouched by this reframing — no amount of asking protects a file from `--force`
or makes a fixture hub-mode. They remain steps 1 and 3.
