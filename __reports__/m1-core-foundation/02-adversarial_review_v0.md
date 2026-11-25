# M1 adversarial review (d6), v0

Verdict: the cornerstone guards catch the bugs they name, wherever a mutation was run, and the before-main findings of
every review are fixed on `feat/m1-core-foundation`. Two cornerstone guards (I7, I12) have no recorded mutation:
Eliott ruled on 2026-11-25 that the mutation checks run so far are enough and asked for an open review instead.

## How the review was run

The leaf planned one fresh verifier injecting each guard's named mutation. It ran as a sequence instead, each step by
fresh Sonnet agents briefed to assume something was wrong:

| Round | Agents | Report | Outcome |
|:--|:--|:--|:--|
| Per-leaf verifiers | one per risky leaf | the leaf files' "verifier's fix round" gates | fixes landed per leaf |
| Sustainability review | 2 reviewers (src, tests) | `02-sustainability_review_{src,tests}_v0.md` | S3, S4, S5, S7, S8, S11, T1-T5, T7, T10 fixed (merges 162de9b, a0ae710) |
| Full review | 2 reviewers (harmony, safety) | `03-full_review_{harmony,safety}_v0.md` | before-main items fixed (merge 813e42f), verified by a fresh agent |

Eliott (2026-11-25): "I think you have already run the mutation checks enough. I am more interested in the open hunt
for full safety", which replaced a second mutation pass with the full review's two lenses.

## Cornerstone guards: mutation and result

| Invariant | Guard | Mutation | Red? | Where recorded |
|:--|:--|:--|:--|:--|
| I3 | `every_write_prefix_loads_and_recovers_to_pre_or_post` | node files removed before the amendment is written | yes | `crash_test.md` gate |
| I3 | `killed_workers_lose_no_acknowledged_write` | lock released between the revision check and the write | yes | `crash_test.md` gate |
| I4 | `stale_replace_changes_nothing` (two-row table) | a write judges the touched files against the cache, not the disk | yes | `02-sustainability_review_tests_v0.md`, seen-red script |
| I7 | `move_writes_one_file_and_keeps_ids` | none recorded | not shown | gap, covered by Eliott's ruling |
| I10 | `cached_read_equals_cold_read` | revalidation by directory mtime alone; signature ignoring identity, size or mtime | yes | `hot_cache.md` gate; seen-red script |
| I10 | `write_through_visible_to_cold_reader_on_return` | buffered write | yes | `hot_cache.md` gate |
| I12 | `unknown_keys_survive_in_place_at_every_level` | none recorded | not shown | gap; the safety review read it as sound, and found a separate I12 breach (sS6, integers beyond 64 bits), deferred to M2 |
| Links | `reverse_index_equals_brute_force_scan` | reverse-index update on delete removed | 3 of 3 | `03-full_review_safety_v0.md` |
| Version | `newer_file_refused_before_parsing` | decoder parsing the whole struct first | yes, per the leaf's step | `codec_and_versions.md` step 1, in the implementer's hand-back only |
| C28 | `requests_breaking_a_rule_are_refused`, step and directory rows | step-reference check commented out; directory check skipped | yes | fix round 66372fc |
| S2 (store.md) | `create_on_a_volume_without_hard_links_publishes_and_still_refuses_a_taken_name` | fallback removed; fallback without the absent check | yes (re-run by the verifier) | fix round 80380bc |

## Leaf gates against what was built; relaxed rules

The per-leaf verifiers and the two full-review lenses checked each leaf's gates as written. Rules found relaxed and
fixed: step references unchecked (sS1); the hard-link fallback promised by `store.md` but missing (sS2); four silent
serialisation fallbacks (H2); opening a missing root creating a directory tree. None remain open before main.

## Fixtures and seams

- Fixtures come from real producers: hand-written canonical files, the store's own writes and their prefixes, and
  real-actor transformations (CRLF checkout, `git merge-file`, `jq -S`, `touch -r`, rename-over, `git checkout`). The
  test sustainability round culled the cache rows shaped by mutants rather than by actors.
- Seams: `rev` is computed once (codec helpers) and compared by store and cache; `RecordingFs` and `RealFs` share one
  `Fs` trait and one effect-apply routine; `StoreError` and its constructors live in rdm-schema and both crates use
  them.

## Deferred, with milestone

Carried in `04-knowledge_transfer_v0.md`: H6, H11-H14; sS3-sS9 except the items fixed; S1, S2, S6, S9, S10, S12, S13;
T8, T9; the four nits of the fix-round verifier.

## CI evidence

- Push run at 813e42f, every job green (ubuntu, macOS, Windows, MSRV 1.88, lint):
  https://github.com/CrackingShells/cracking-shells-playbook/actions/runs/45618278562
- Soak run at 813e42f (`RDM_SOAK_ITERS=1000`, `PROPTEST_CASES=10000`): ubuntu, macOS, MSRV 1.88 and lint green;
  Windows cancelled at the job's 45-minute cap with `reverse_index_equals_brute_force_scan` still running. The
  kill harness passed on Windows at full count (100 s).
  https://github.com/CrackingShells/cracking-shells-playbook/actions/runs/45621003595
- Cause of the Windows cancel: a test helper read the store once per candidate node (fixed in a73b07b, 37 %
  faster), on a store that stats every file of the campaign, history included, on every read.
- Soak run at 825ea5d, after the helper fix, every job green. Job durations, build included: ubuntu 18 min 37 s,
  macOS 17 min 56 s, Windows 33 min 6 s, MSRV 1.88 10 min 27 s.
  https://github.com/CrackingShells/cracking-shells-playbook/actions/runs/45648611166
- **Thin margin:** Windows finishes 12 minutes under its cap, and the cost behind it is the store's read path, not
  the test. Eliott ruled on 2026-11-26 that the read path is redesigned in its own PR, measured first:
  `__reports__/rdm_read_path_efficiency/`.
- CRLF probe: a Windows checkout with `core.autocrlf=true` did rewrite committed JSON (the schema drift guard went red
  on Windows at d9331cd). `.gitattributes` now pins `eol=lf` on the test fixtures and the committed schemas, and the
  CI step `git ls-files --eol` prints the fixtures' line ends on every run.
