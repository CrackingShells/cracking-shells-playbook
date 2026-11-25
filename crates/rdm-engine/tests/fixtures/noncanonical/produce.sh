#!/bin/sh
# Produces every non-canonical fixture from a hand-written canonical one, by the real
# transformation it stands for. Run from this directory. Each fixture is read back by the test
# `noncanonical_valid_input_reads_and_reencodes_canonically`, which names the canonical file it
# must re-encode to. Tools: jq 1.7, perl 5, git 2.
set -eu
C=../canonical

# An editor or formatter that sorts keys (a JSON "sort keys" command, `jq -S`).
jq -S . $C/leaf.json > reordered_keys_leaf.json

# An editor set to four spaces (`jq --indent 4`).
jq --indent 4 . $C/directory.json > four_space_indent_directory.json

# A Windows checkout with autocrlf, or `unix2dos`: every LF becomes CRLF.
perl -pe 's/\n/\r\n/' $C/comment.json > crlf_comment.json

# An editor that does not add the final newline: drop the last byte.
perl -pe 'chomp if eof' $C/leaf_paused.json > no_trailing_newline_leaf_paused.json

# A tool that writes ASCII only (`jq -a`): non-ASCII becomes \uXXXX, an emoji a surrogate pair.
jq -a . $C/comment_escapes.json > unicode_escapes_comment.json

# Another writer that gives an absent optional field an explicit null (the stored schema says a
# reader treats null as absent): jq sets keys, so they land at the end of their objects.
jq '.status_reason = null | .steps[0].commit_sha = null | .steps[0].done_at = null | .references[1].target = null | .history[0].override = null' \
  $C/leaf.json > explicit_nulls_leaf.json

# An editor that writes a UTF-8 byte order mark: Windows PowerShell 5 (`Out-File -Encoding utf8`,
# `Set-Content -Encoding UTF8`) and older Notepad. EF BB BF in front of the canonical bytes.
printf '\357\273\277' | cat - $C/leaf.json > bom_leaf.json

# A formatter that sorts keys (`jq -S`) over a leaf that carries unknown keys at two levels: the
# unknown keys come out sorted too. The expectation is the canonical leaf with the same unknown
# keys added in that sorted order (an unknown object value is sorted by jq as well: a before b).
jq '.x_zeta = {"b": 1, "a": 2} | .x_alpha = [1, 2] | .steps[0].x_note = "n"' $C/leaf.json \
  | jq -S . > sorted_unknown_keys_leaf.json
jq '.x_alpha = [1, 2] | .x_zeta = {"a": 2, "b": 1} | .steps[0].x_note = "n"' $C/leaf.json \
  > sorted_unknown_keys_leaf.expected.json

# Two branches edit one node file and git merges them: one retypes the title by hand (no space
# after the colon), the other appends a progress note. `base` is the canonical leaf; each side
# is a change to its own lines, so the merge is clean and keeps the hand-typed spacing.
T=$(mktemp -d)
cp $C/leaf.json "$T/base.json"
sed 's/^  "title": "M1 Core foundation",$/  "title":"M1 Core foundation, renamed",/' $C/leaf.json > "$T/ours.json"
perl -pe 's/^      "first note"$/      "first note",\n      "second note"/' $C/leaf.json > "$T/theirs.json"
git merge-file -p "$T/ours.json" "$T/base.json" "$T/theirs.json" > merge_leaf.json
# What the merge must read as: both edits, canonically written.
sed 's/^  "title": "M1 Core foundation",$/  "title": "M1 Core foundation, renamed",/' $C/leaf.json \
  | perl -pe 's/^      "first note"$/      "first note",\n      "second note"/' > merge_leaf.expected.json
rm -r "$T"
