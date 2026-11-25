# M1 full review, lens A (harmony): one way to do one thing

Branch `feat/m1-core-foundation` at d9331cd. Scope: `crates/rdm-schema`, `crates/rdm-engine` (src and tests), a glance at
`crates/rdm-mcp`. Read-only; no code changed (`git diff --quiet` holds). Effort S/M/L. "Before main" means before M1 merges.

## Verdict

The code base is more harmonious than most at this size: zero `#[allow]` in product code, `#[must_use]` applied (55 sites;
pedantic flags only 2 missing), sentence-style test names throughout, one error enum for the store, `layout` and `tests/common`
exist and are used. What remains is four classes of drift, ranked below: record names and file suffixes still spelled in
~20 places despite `layout`; the same "this serialises" objective handled by four policies; duplicated helpers and docs that
will diverge; and conventions that nothing enforces (no `fmt`/`clippy` in CI, rdm-schema is not rustfmt-clean). None blocks M1
on correctness. H1, H2, H3 and H7 are worth doing before main because each is small and each gets more expensive once M2
adds callers.

## Findings, by consequence

### H1. Logical record names and the `.json` suffix are built in about 20 places; `layout` owns only `.rdm/` (M, before main)

`layout.rs` says "the one place a logical name becomes a path", but it covers `record_path` (name to `PathBuf`) only. The names
themselves are made elsewhere, by four different means:

| site | means |
|---|---|
| `schema/campaign.rs:37` `node_file(id)` | `format!("nodes/{id}.json")`, the intended one |
| `engine/write.rs:840` `node_path` closure, `:860` `Del` | `format!("nodes/{}.json", ..)`, in a file that imports `node_file` (`:11`) and uses it at `:52`, `:919`, `:935` |
| `engine/write.rs:846`, `:855` | `format!("_comments/{}.json")`, `format!("_amendments/{}.json")` |
| `engine/cache.rs:361`, `:491`, `:517`, `:543`, `:568` | `format!("{id}.json")` with the dir passed separately (`"nodes"` at `:357`, `:492`) |
| `engine/cache.rs:129-131` `kind_of`, `:163-` `NODES/COMMENTS/AMENDMENTS`, `engine/write.rs:1576` `effect_path_is_ours` | the three directory names re-listed three more times |
| `engine/store.rs:318` `Store::file_of` | a pure alias of `node_file` (see H8) |

Scenario: renaming `_comments` or adding a fourth record kind needs edits at `cache.rs` (3 spots), `write.rs` (3 spots) and the
path guard; the guard `effect_path_is_ours` is the one that silently refuses recovery if it is missed.
Fix: one `RecordKind { Node, Comment, Amendment }` in `layout` with `dir()`, `file_name(id)`, `logical(id)` and `from_logical(&str)`;
`NODES/COMMENTS/AMENDMENTS`, `kind_of`, the plan effects and `effect_path_is_ours` all call it. Keep `node_file` in schema as
the public name and make `RecordKind::Node.logical(id)` the engine's only spelling.

### H2. One objective, "a stored record becomes JSON", four error policies (S, before main)

| site | policy |
|---|---|
| `codec.rs:98` `to_string_pretty(..).expect("a stored record serialises to JSON")` | panic, with a reason |
| `rev.rs:29` `to_value(node).unwrap_or(Value::Null)`; `write.rs:1538`, `:1543` the same | hide the failure as `null` |
| `rev.rs:37` `to_string_pretty(&value).unwrap_or_default()` | hide the failure as the empty string |
| `write.rs:152` `to_vec(&holder).unwrap_or_default()` | hide as an empty holder record |

All four cannot fail for these types (string keys only; `rev.rs:36` even says so). The `expect` is the right policy because
it states the invariant; the others would make every node hash to the same `Rev` if the invariant ever broke, which is the
worst possible silent failure for an optimistic-concurrency token. Fix: one `pub(crate) fn json_value<T: Serialize>(&T) -> Value`
and one `json_text`, each with the single `expect`; the four sites call them. The same helper retires the `to_value` then
`.get("marks")` walk at `write.rs:1536-1544`, which reaches a typed field through JSON; `Node` has `Directory.marks` and
`Leaf.marks`, so a `Node::marks(&self) -> &[Mark]` accessor next to `id()`/`name()` is the idiomatic route.

### H3. `StoreError::Invalid` has 12 construction sites, not three, and one hardcodes a magic file name across crates (S, before main)

Known and deferred: three constructors. New evidence: the literal `StoreError::Invalid { .. }` appears at `schema/campaign.rs:103`,
`schema/stored.rs:190` (a closure with the string `"<request>"`), `engine/codec.rs:222` (`detail: &str`), `store.rs:151`
(`detail: String`), `write.rs:43` and `:51` (`impl Into<String>`), `:270`, `:283`, `:763` (inline), `version.rs:144` and `:177`
(closures). Three different `detail` parameter types for one operation, and `REQUEST = "<request>"` (`write.rs:40`) is
duplicated as a bare literal in rdm-schema, so the sentinel `contracts.md` names is spelled twice and cannot be searched for.
Smallest fix that does not need M2's rework: `pub const REQUEST_FILE` in `rdm-schema::error` used by both crates, and make
`store::invalid(file, rule, detail: impl Into<String>)` the one engine-side constructor (replace `codec::invalid`, the two
`version.rs` closures and the three inline sites in `write.rs`). Leave the `Io`-drops-`ErrorKind` redesign to M2 as planned.

### H4. `CachedCampaign` doc text is a verbatim copy of the `Store` methods it delegates to (S, before main)

`cache.rs:626-720` (public) and `write.rs:960-1090` (`pub(crate)`) carry the same `///` blocks for `recover`, `replace`, `create`,
`move_node`, `delete_subtree`, `comment`, `batch`, `last_recovered`; `Store::open` and `CachedCampaign::open` repeat one `# Errors` paragraph.
Two copies of a contract drift: the next rule added to `replace` gets documented on one side. The public handle is the one a
consumer reads. Fix: keep the full doc on `CachedCampaign`, reduce the `pub(crate)` twins to one line plus "see
[`CachedCampaign::replace`]". Also `Store::last_recovered` is plain `pub` (`write.rs:963`) while every sibling write method is
`pub(crate)`, so the same report is reachable by two paths; make it `pub(crate)`.

### H5. Nothing enforces the conventions; rdm-schema is not rustfmt-clean (S, before main)

`cargo fmt -p rdm-engine --check`: 0 diffs. `-p rdm-schema`: 53 diffs and 33 lines over 100 columns (engine: 3). There is no
`rustfmt.toml`, `clippy.toml` or `[workspace.lints]`, and `.github/workflows/rust.yml` runs `cargo test` only (lines 47, 61).
The two crates were evidently written under different formatter settings. Fix: `cargo fmt --all`, add one `cargo fmt --check`
and one `cargo clippy --workspace --all-targets -- -D warnings` step, and a `[workspace.lints.rust]`/`clippy` table
(`unsafe_code = "forbid"`, `missing_docs` warn for the two lib crates). Do the format commit alone so blame survives.

### H6. `Version` is a tuple alias with free functions; the stored form is `String` (M, M2)

`lib.rs:12-61`: `type Version = (u16,u16,u16)`, `version_string`, `parse_version`, `is_newer`, while every stored struct holds
`schema_version: String` with the regex `^[0-9]+\.[0-9]+\.[0-9]+$` pasted six times (`stored.rs` x4, `read.rs` x2) and the
engine re-parses it (`version.rs:143 read_version`). Std idiom is a `struct Version` with `Display`, `FromStr`, `Ord`, and a
serde `String` representation; then `schema_version: Version` is validated at the type, the regex is generated once, and
`is_newer` is `>`. It matches how the id newtypes already work in this crate, so it is also the code base's own majority
pattern. Not before main (touches the on-disk contract tests); M2.

### H7. Errors: two enums have no `Display`/`Error`, and impl style varies (S, before main)

`order::KeyError` (no `Display`, no `std::error::Error`) and `NotInVersion` (neither) are public error types in the same
crates where `IdError`, `NameError`, `MintError`, `LockError`, `StoreError` implement both. A caller cannot `?` a `KeyError` into
`Box<dyn Error>`. Style also differs: `ids.rs`/`error.rs` `use std::fmt` and write `fmt::Formatter`; `lock.rs:30` and `ids.rs:41`
spell `std::fmt::` in full. `StoreError::NotFound` prints `{target:?}` (Debug) in `Display` (`error.rs:253`), the only variant
that leaks Debug into a user message. `MintError` message hardcodes "32" instead of `MAX_COLLISIONS`. `LockError` has a `source()`
override while `StoreError::Io` stores `message: String` (the known ErrorKind deferral). Fix: add the two impls, use `fmt::` imports,
format `RecordRef` readably, interpolate the constant.

### H8. Same-purpose items present twice (S, before main)

- `Store::file_of(id)` (`store.rs:318`) is `node_file(id)` and nothing else; used by tests only (7 sites). Delete it; tests call `node_file`.
  `tests/common/campaign.rs:44 file_of` is a third copy that reimplements the format string instead of calling `node_file`; make it
  `node_file`. The same file defines `ALPHABET` (line 23) = `ID_ALPHABET` and `SCHEMA_VERSION` (21) = `version_string(FORMAT_VERSION)`:
  tests that restate the constants cannot catch the constants changing, and cannot be found when they do.
- `read::Projection` and `read::LevelSource` are defined and tested (schema/tests) and used nowhere in either crate; `READ_MODEL_VERSION`
  has one use, a test. They are M4 scaffolding by design: **keep, deliberate**, but note they are the only public items with no
  production consumer. `diff::is_state` and `Campaign::is_validated` do have consumers.
- Parsing ids: product code calls the inherent `NodeId::parse` everywhere (7 sites) and never `str::parse::<NodeId>()`; `FromStr`
  and `TryFrom<String>` are implemented on every id type by the macro. `FromStr` is idiomatic: keep. `TryFrom<String>` has no
  consumer in either crate and `Name` lacks it, so the type family is not uniform: drop it or add it to `Name` (S, M2).

### H9. Test helpers duplicated inside rdm-engine/tests (S, before main)

`tests/common` was the fix, but these remain, byte for byte or nearly:

| helper | copies |
|---|---|
| `rev_of(store, n)` | `store.rs:871`, `crash_prefix.rs:85` (identical) |
| `delete_op(store, n)` | `store.rs:905`, `crash_prefix.rs:89` |
| `diagnostic_keys(store)` | `crash_prefix.rs:297`, `crash_kill.rs:522` (identical but for a path-qualified `Rule`) |
| `at_prefix(pre_dir, log, k, into)` | `store.rs:2393`, `crash_prefix.rs:530` (signatures differ: one returns `PathBuf`) |
| `leaf_of` | `cache.rs:37` (takes `&Store`), `store.rs:403` (takes `&mut CachedCampaign`) |
| `base()` | `common/campaign.rs:186`, `crash_prefix.rs:40`, `crash_kill.rs:52`: three different fixtures under one name; each local one shadows the shared one |
| `holder_child` | `fs.rs:109`, `lock.rs:71`: different bodies, same name |

Fix: move the first four into `common/campaign.rs`; rename the three `base()` to what they model (`hot_dirs`, `spare_and_deep`).
The local fixtures also use `let n = nid; dir_node(n(1), ..)` while common offers `dir_n(1, ..)`: use `dir_n`/`leaf_n`.
rdm-schema/tests/common (`fixture`, `round_trip`, `inject_unknown`) shares nothing with the engine's: **keep, deliberate**, the two
helper sets address different needs (JSON fixtures versus on-disk campaigns); a shared dev-only crate would cost more than it saves.

### H10. `#[non_exhaustive]` on 2 of 40 public enums, with no stated policy (S, before main)

Only `Rule` and `StoreError` carry it. The enums a consumer matches exhaustively and that M2-M4 will extend are unmarked:
`Current`, `LockError`, `IdError`, `NameError`, `KeyError`, `MintError`, `Durability`, `Pending`, `IntentEffect`, `Op`, `OpResult`,
`Position`, `RecordRef`. The wire enums (`Status`, `ActorKind`, `Expected`, ...) are closed by format version: **keep, deliberate**.
Fix: add it to the error enums and the engine's operational enums, and write the one-line rule (closed by format = exhaustive; closed
by implementation = `non_exhaustive`) in `lib.rs`. Do it now: adding it later is a breaking change for any in-tree consumer.

### H11. Newtype family is three hand-written shapes (M, M2)

`checked_string!` (NodeId, CriterionId, StepId, CommentId, AmendmentId, OpId), then `ItemId` and `Name` hand-written with the same
`FromStr`/`Display`/`Serialize`/`Deserialize`/`JsonSchema` bodies (about 90 duplicated lines, `ids.rs:238-405`), then `opaque_string!`.
Visible differences: `Name` has no `is_valid` and no `TryFrom<String>`, `ItemId` has no `is_valid`, `Name` mixes its error
tag (`NAME_FORMAT_TAG`) with the id tag; `ItemId::parse` reports `WrongPrefix { expected: "c_" }` for an `s_`-or-`c_` choice, a
misleading message. Decoders classify a failure by string prefix (`MALFORMED_ID_TAG`): fragile, but documented and tested; **keep,
deliberate** until serde offers typed custom errors, and note the tag must move with the message text. Fix: extend the macro with
an optional error-tag argument so `Name` uses it, leave `ItemId` hand-written (it is an enum) but add `is_valid`.
`check_prefixed` also scans `ID_ALPHABET.contains(c)` linearly per character; harmless at id length, mentioned only because
`ids.rs` is the hot path of every cold open (S, M2).

### H12. Ownership and path signatures (S, M2)

Roots are `impl Into<PathBuf>` in `Store::open`, `CachedCampaign::open`, `RecordingFs::new/over`, but `&Path` in `WriterLock::acquire`,
`revalidate`, `intent::*` and every `layout` function. API Guidelines (C-GENERIC) and std lean to `impl AsRef<Path>` for a path that
is only read, `impl Into<PathBuf>` only where it is stored; the two `open`s do store it, so they are right and the `&Path` helpers are
right: **keep, deliberate**. The one genuine mismatch is `Entries::put(name: String, ..)` vs the `&str` taken everywhere else at
`cache.rs:91`, negligible. ULIDs travel as `&str`/`String` (`layout::intent_record(root, ulid: &str)`): known, M2.

### H13. Doc-comment shape and small idioms (S, M2)

- `# Errors` sections: present on `Store`/`CachedCampaign` and the schema crate, absent on 21 `Result` functions (13 on `fs.rs`'s
  trait impls, 3 each in `version.rs`, `intent.rs`, one each `lock.rs`, `codec.rs`). A trait method documented once on the trait is
  fine (`Fs`); the free functions `intent::begin/complete/pending` and `WriterLock::acquire` are public and lack it.
- Public enum variants and struct fields are mostly undocumented in `stored.rs`, `ops.rs`, `read.rs` (the stored-schema doc is the
  documentation); acceptable while the design docs are normative, but then `missing_docs` cannot be enabled (H5): pick one.
- Product-code idioms clippy points at: `rev.rs:12,22` (`format!` into a `String`, use `write!`), `cache.rs:463-464` (`for_each` for side
  effects), `cache.rs:562` and `write.rs:640` (unused `self`), `campaign.rs:185,231,385` (redundant closures), `write.rs:838`
  (`let text = |n: &Node| encode(n)` beside direct `encode(c)`), `order.rs:70` (`midpoint` by hand; no overflow possible with 32 digits).
- `match` versus combinators: both are used for the same `Option` chains (`links.rs:58` `and_then(parse ... .ok())` vs `write.rs:589`
  `if let Ok`), no majority; leave it.
- `Campaign::diagnostics` returns `&[Diagnostic]` and `Store::diagnostics` returns `Vec<Diagnostic>` (it merges two sources): **keep,
  deliberate**, but the doc of the second should say it allocates.

### H14. Naming drift (S, M2)

`Written`/`Deleted`/`Commented` result types versus `CreateOp/ReplaceOp/RelocateOp/DeleteOp/CommentOp` and methods
`create/replace/move_node/delete_subtree/comment`: the op verb differs from the method in two of five (`Relocate`/`move_node`,
`Delete`/`delete_subtree`); "move" is the known M2 item. `fault` (`validate.rs:20`), `diag` (`campaign.rs:69`), `node_fault`,
`request_fault`, `invalid` all build an error value from `(rule, detail)`; one noun, "fault", would do. Test files: `crash_kill`/
`crash_prefix` are named by technique, the others by module; fine. Test-function names are uniformly sentence-style: **keep**.

### H15. rdm-mcp (spike): what to delete now

Nothing depends on it, and it depends on neither rdm crate (zero references to `rdm_schema`/`rdm_engine`). It is labelled
"Throwaway MCP Apps feasibility stub" (`Cargo.toml` description), has its own `build.rs`, a 280K committed `view/dist`, fixtures
and a Python freezer, and every `cargo test --workspace` in CI builds `rmcp`, `tokio` and `jsonschema` for it. It also supplies 4
of the 6 unused-`async` and the cast lints in the pedantic run, noise for the other two crates. Recommendation: do not delete before
main if M4 still wants its view and tests as a reference; but either move it out of `members = ["crates/*"]` (add to `exclude`, like
the `dirtree-rdm` entries) or delete it the day M4 starts. If it must stay in the workspace, exclude it from the CI test matrix: the
cost is paid by every other change (S, before main, only if CI time matters).

## Clippy pedantic, counts by class (signal only)

Whole workspace, all targets, about 110 warnings in total. By class: `# Errors` missing 21; `Map::default()` clearer 17 (tests);
function too long 17 (14 in tests; product: `Campaign::open` 198, `write.rs:1088` 132); `async` without await 4 (rdm-mcp);
`format!` from iterator 3; unused `self` 3; `map().unwrap_or()` on Result 3; items after statements 3; `for_each` 4;
single-pattern `match` 2; identical arms 2; redundant closure 8; casts 3; `#[must_use]` missing 2; other single hits 12. Product code
of the two crates accounts for about 40 of these, nearly all in the classes named in H13. No finding was raised from a lint alone.

## Keep, deliberate (differences that are justified)

- Zero `#[allow]` in product code; `#![allow(dead_code)]` in both `tests/common/mod.rs` (standard for a shared test module).
- `.expect(..)` with a stated reason at `ids.rs:30`, `:64`, `write.rs:140`, `codec.rs:98`: invariants, not input handling. (Contrast H2.)
- `let _ = remove_file` for scratch cleanup (`fs.rs:464`, `:491`, `publish.rs:42`, `lock.rs:128`) and `let _ = self.refresh()` after a failed
  write (`write.rs:403`): best-effort on an error path that already returns the primary error, and commented. Lens B owns whether the
  `refresh` failure should be logged.
- `Rule::code()` hand-written next to `serde(rename_all)`: two sources of one string, but `tests/campaign.rs:50 rule_codes_are_their_serde_names`
  pins them.
- The two test-helper sets across crates (H9). The two `diagnostics` return types (H13).
- `impl Into<PathBuf>` where the path is stored, `&Path` where it is only read (H12).

## Not checked

Did not read `write.rs` end to end (1718 lines) or `tests/store.rs` (3015); idiom review there is by grep and clippy only. Did not
compare `diff.rs`'s `to_value` use against H2 beyond noting it. Consumers were counted with a bypassed identifier grep, not colgrep
(a batch of 45 names in one pass; the colgrep index was not built). Did not run the test suite. rdm-mcp was only skimmed for H15.
