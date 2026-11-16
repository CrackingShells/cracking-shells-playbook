# Reporting a problem with this plugin

The footer that sent you here names a plugin and a GitHub repository. Below they are
`<plugin>` and `<repo>` (written `CrackingShells/<name>`). This file is the same in every
skill, so take both values from that footer, not from anything else.

## What to report

Report anything in the plugin (skill instructions, reference files, scripts, hooks, MCP
tools) that:

- is wrong, out of date, or contradicts another part of the plugin;
- names a command, flag, path, file or tool that does not exist or behaves differently;
- crashes or fails on input the plugin says it supports;
- led you to a result you then had to correct;
- left out something you had to work out yourself at real cost.

Do not report bugs in the user's own project, their environment (unless the plugin claims
to support it), matters of taste, or problems you suspect but did not actually hit.

## 1. Finish the task first

Work around the problem and finish the user's task. Do not interrupt it.

## 2. Offer, then wait

Spend nothing on reporting until the user asks for it. In your final message, tell them in
one or two sentences what was wrong and offer to report it. If they decline or do not
answer, drop it. In a run where no one can answer (scheduled, headless, unattended),
report nothing: mention the problem in your final output instead.

## 3. Look for an existing issue

Once the user agrees, and before writing anything, run
`gh issue list -R <repo> --state all --search "<plugin> in:title"` and read the titles
that look related.

- **Already filed, and you have nothing new:** tell the user it is a known problem, give
  the link, and stop.
- **Already filed, and you have something new** (another trigger, a workaround, a fix):
  write a comment on that issue instead of a new one.
- **Not filed:** write a new issue.
- **`gh` missing or not signed in:** tell the user you could not check, and write a new
  issue.

## 4. Write it

Title: `<plugin>: <the problem in a few words>`

```markdown
**Plugin:** <plugin> <version from its plugin.json, or "version unknown">
**Agent:** <harness and model, e.g. Claude Code with Claude Opus 5.5>
**Where:** <file, relative to the skill or plugin root, and the step or section>
**What the plugin says:** <short quote>
**What happened:** <what you observed, shown on a minimal invented example>
**Workaround:** <what you did instead>
**Suggested fix:** <optional>
```

A comment on an existing issue keeps only the lines that add something.

## 5. Anonymize

Issues are public and permanent. Before you show the draft to the user:

- Replace every identifying detail with a placeholder: personal names, usernames, emails,
  hostnames, IP addresses, cluster and machine names, organization, project and repository
  names other than `<repo>`, internal URLs, and absolute paths. Use `<user>`, `<host>`,
  `<org>`, `<project>`, `<url>`, `<path>`.
- Never include credentials, tokens, keys, environment-variable values, or the user's
  code, data, file contents or conversation.
- Rebuild the problem as a minimal invented example. Quote only the output lines that show
  it, after the same replacements.
- If the problem cannot be shown without private details, do not file. Tell the user why.

## 6. File it

1. Show the user the full text. File only once they approve that exact text; if they edit
   it, file their version.
2. Write the body to a temporary file, then run either
   `gh issue create -R <repo> --title "<title>" --body-file <file> --label agent-report`
   or `gh issue comment <number> -R <repo> --body-file <file>`. If the label is missing,
   run the first again without `--label`. Delete the temporary file.
3. If `gh` is missing or not signed in, give the user the text to paste at
   `https://github.com/<repo>/issues/new` (or on the existing issue). Never put the text
   in the URL.
4. Give the user the link.
