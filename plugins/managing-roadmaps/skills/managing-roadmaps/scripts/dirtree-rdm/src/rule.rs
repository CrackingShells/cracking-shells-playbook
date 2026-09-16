//! BNF rule identity + diagnostic data + grammar excerpt on a single enum.
//!
//! One variant per BNF rule that the validator can emit as a violation.
//! All three methods (`name`, `diagnostic`, `grammar_excerpt`) use exhaustive
//! `match`, so adding a variant forces wiring every method — drift between rule
//! identity and its diagnostic is a compile error, not a runtime bug.
//!
//! The CLI subcommand `dirtree-rdm grammar` and the validator both consult this
//! enum, so they cannot disagree about which rules exist or what their
//! grammar/diagnostic say.

/// Concrete diagnostic data attached to a `Rule`.
///
/// `expected_form` is a one-line positive example the receiver can imitate.
/// The validator only ever knows what the rule *requires*, not why a given
/// input failed to match it; cause-narrowing belongs on `Violation.hint`,
/// which only sites with site-specific evidence may populate.
#[derive(Debug, Clone, Copy)]
pub struct Diagnostic {
    pub expected_form: &'static str,
}

/// Which BNF file a rule lives in. Used for `--list` grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarSource {
    Readme,
    Leaf,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rule {
    // ── README + Leaf shared ───────────────────────────────────────────────
    H1Title,

    // ── README.md rules ────────────────────────────────────────────────────
    H2Context,
    H2Goal,
    H2Preconditions,
    H2SuccessGates,
    H2Status,
    ContextBody,
    GoalBody,
    ReferenceItem,
    GotchasBody,
    MermaidBlock,
    MermaidGraphDecl,
    MermaidNodeDecl,
    MermaidClassdefDone,
    MermaidClassdefInprogress,
    MermaidClassdefPlanned,
    MermaidClassdefAmendment,
    MermaidClassdefBlocked,
    TableHeaderNodes,
    TableHeaderAmendment,
    TableHeaderProgress,
    TableSeparator,

    // ── Both grammars (precondition/gate items appear in both) ─────────────
    CheckboxItem,
    GateItem,

    // ── leaf.bnf rules ─────────────────────────────────────────────────────
    FieldGoal,
    FieldPreconditions,
    FieldSuccessGates,
    FieldReferences,
    StepHeading,
    StepFieldGoal,
    StepFieldImplLogic,
    StepFieldDeliverables,
    StepFieldConsistency,
    StepFieldCommit,
    Step,
}

impl Rule {
    /// Every variant, in stable display order. Exhaustive-match elsewhere
    /// guarantees this list stays in sync with the enum.
    pub const ALL: &'static [Rule] = &[
        Rule::H1Title,
        Rule::H2Context,
        Rule::H2Goal,
        Rule::H2Preconditions,
        Rule::H2SuccessGates,
        Rule::H2Status,
        Rule::ContextBody,
        Rule::GoalBody,
        Rule::ReferenceItem,
        Rule::GotchasBody,
        Rule::MermaidBlock,
        Rule::MermaidGraphDecl,
        Rule::MermaidNodeDecl,
        Rule::MermaidClassdefDone,
        Rule::MermaidClassdefInprogress,
        Rule::MermaidClassdefPlanned,
        Rule::MermaidClassdefAmendment,
        Rule::MermaidClassdefBlocked,
        Rule::TableHeaderNodes,
        Rule::TableHeaderAmendment,
        Rule::TableHeaderProgress,
        Rule::TableSeparator,
        Rule::CheckboxItem,
        Rule::GateItem,
        Rule::FieldGoal,
        Rule::FieldPreconditions,
        Rule::FieldSuccessGates,
        Rule::FieldReferences,
        Rule::StepHeading,
        Rule::StepFieldGoal,
        Rule::StepFieldImplLogic,
        Rule::StepFieldDeliverables,
        Rule::StepFieldConsistency,
        Rule::StepFieldCommit,
        Rule::Step,
    ];

    /// Kebab-case rule name as it appears in BNF and in CLI flags
    /// (`--rule <name>`).
    pub fn name(&self) -> &'static str {
        match self {
            Rule::H1Title => "h1-title",
            Rule::H2Context => "h2-context",
            Rule::H2Goal => "h2-goal",
            Rule::H2Preconditions => "h2-preconditions",
            Rule::H2SuccessGates => "h2-success-gates",
            Rule::H2Status => "h2-status",
            Rule::ContextBody => "context-body",
            Rule::GoalBody => "goal-body",
            Rule::ReferenceItem => "reference-item",
            Rule::GotchasBody => "gotchas-body",
            Rule::MermaidBlock => "mermaid-block",
            Rule::MermaidGraphDecl => "mermaid-graph-decl",
            Rule::MermaidNodeDecl => "mermaid-node-decl",
            Rule::MermaidClassdefDone => "mermaid-classdef-done",
            Rule::MermaidClassdefInprogress => "mermaid-classdef-inprogress",
            Rule::MermaidClassdefPlanned => "mermaid-classdef-planned",
            Rule::MermaidClassdefAmendment => "mermaid-classdef-amendment",
            Rule::MermaidClassdefBlocked => "mermaid-classdef-blocked",
            Rule::TableHeaderNodes => "table-header-nodes",
            Rule::TableHeaderAmendment => "table-header-amendment",
            Rule::TableHeaderProgress => "table-header-progress",
            Rule::TableSeparator => "table-separator",
            Rule::CheckboxItem => "checkbox-item",
            Rule::GateItem => "gate-item",
            Rule::FieldGoal => "field-goal",
            Rule::FieldPreconditions => "field-preconditions",
            Rule::FieldSuccessGates => "field-success-gates",
            Rule::FieldReferences => "field-references",
            Rule::StepHeading => "step-heading",
            Rule::StepFieldGoal => "step-field-goal",
            Rule::StepFieldImplLogic => "step-field-impl-logic",
            Rule::StepFieldDeliverables => "step-field-deliverables",
            Rule::StepFieldConsistency => "step-field-consistency",
            Rule::StepFieldCommit => "step-field-commit",
            Rule::Step => "step",
        }
    }

    /// Resolve a rule name (kebab-case) back to its variant.
    pub fn from_name(name: &str) -> Option<Rule> {
        Self::ALL.iter().find(|r| r.name() == name).copied()
    }

    /// Which grammar file owns this rule.
    pub fn source(&self) -> GrammarSource {
        match self {
            Rule::H1Title | Rule::CheckboxItem | Rule::GateItem => GrammarSource::Shared,
            Rule::H2Context
            | Rule::H2Goal
            | Rule::H2Preconditions
            | Rule::H2SuccessGates
            | Rule::H2Status
            | Rule::ContextBody
            | Rule::GoalBody
            | Rule::ReferenceItem
            | Rule::GotchasBody
            | Rule::MermaidBlock
            | Rule::MermaidGraphDecl
            | Rule::MermaidNodeDecl
            | Rule::MermaidClassdefDone
            | Rule::MermaidClassdefInprogress
            | Rule::MermaidClassdefPlanned
            | Rule::MermaidClassdefAmendment
            | Rule::MermaidClassdefBlocked
            | Rule::TableHeaderNodes
            | Rule::TableHeaderAmendment
            | Rule::TableHeaderProgress
            | Rule::TableSeparator => GrammarSource::Readme,
            Rule::FieldGoal
            | Rule::FieldPreconditions
            | Rule::FieldSuccessGates
            | Rule::FieldReferences
            | Rule::StepHeading
            | Rule::StepFieldGoal
            | Rule::StepFieldImplLogic
            | Rule::StepFieldDeliverables
            | Rule::StepFieldConsistency
            | Rule::StepFieldCommit
            | Rule::Step => GrammarSource::Leaf,
        }
    }

    /// Positive example for the rule, used in the violation render's
    /// `expected form:` slot. Exhaustive — every variant must have one.
    pub fn diagnostic(&self) -> Diagnostic {
        match self {
            Rule::H1Title => Diagnostic {
                expected_form: "# <Title>",
            },
            Rule::H2Context => Diagnostic {
                expected_form: "## Context",
            },
            Rule::H2Goal => Diagnostic {
                expected_form: "## Goal",
            },
            Rule::H2Preconditions => Diagnostic {
                expected_form: "## Pre-conditions",
            },
            Rule::H2SuccessGates => Diagnostic {
                expected_form: "## Success Gates",
            },
            Rule::H2Status => Diagnostic {
                expected_form: "## Status",
            },
            Rule::ContextBody => Diagnostic {
                expected_form: "<one or more non-blank lines under `## Context`>",
            },
            Rule::GoalBody => Diagnostic {
                expected_form: "<single-line goal statement>",
            },
            Rule::ReferenceItem => Diagnostic {
                expected_form: "- [R01 <Name>](<path>) — <one-line summary>",
            },
            Rule::GotchasBody => Diagnostic {
                expected_form: "<one or more non-blank lines under `## Gotchas`>",
            },
            Rule::MermaidBlock => Diagnostic {
                expected_form: "```mermaid / graph TD / <node-decls> / classDef done … / … / ```",
            },
            Rule::MermaidGraphDecl => Diagnostic {
                expected_form: "graph TD",
            },
            Rule::MermaidNodeDecl => Diagnostic {
                expected_form: "    n1[<Title>]:::planned   (status ∈ done|inprogress|planned|amendment|blocked)",
            },
            Rule::MermaidClassdefDone => Diagnostic {
                expected_form: "    classDef done       fill:#166534,color:#bbf7d0",
            },
            Rule::MermaidClassdefInprogress => Diagnostic {
                expected_form: "    classDef inprogress fill:#854d0e,color:#fef08a",
            },
            Rule::MermaidClassdefPlanned => Diagnostic {
                expected_form: "    classDef planned    fill:#374151,color:#e5e7eb",
            },
            Rule::MermaidClassdefAmendment => Diagnostic {
                expected_form: "    classDef amendment  fill:#1e3a5f,color:#bfdbfe",
            },
            Rule::MermaidClassdefBlocked => Diagnostic {
                expected_form: "    classDef blocked    fill:#7f1d1d,color:#fecaca",
            },
            Rule::TableHeaderNodes => Diagnostic {
                expected_form: "| Node | Type | Status |",
            },
            Rule::TableHeaderAmendment => Diagnostic {
                expected_form: "| ID | Date | Source | Nodes Added | Rationale |",
            },
            Rule::TableHeaderProgress => Diagnostic {
                expected_form: "| Node | Branch | Commits | Notes |",
            },
            Rule::TableSeparator => Diagnostic {
                expected_form: "|:-----|:-----|:-------|",
            },
            Rule::CheckboxItem => Diagnostic {
                expected_form: "- [ ] <precondition>   (initial; use `- [x] <precondition>` once satisfied)",
            },
            Rule::GateItem => Diagnostic {
                expected_form: "- ⬜ <gate description>   (initial; use `- ✅ <gate description>` once passed)",
            },
            Rule::FieldGoal => Diagnostic {
                expected_form: "**Goal**: <one-line statement of what this leaf accomplishes>",
            },
            Rule::FieldPreconditions => Diagnostic {
                expected_form: "**Pre-conditions**: / - [ ] <precondition>   (use `- [x]` once satisfied)",
            },
            Rule::FieldSuccessGates => Diagnostic {
                expected_form: "**Success Gates**: / - ⬜ <gate>   (use `- ✅` once passed)",
            },
            Rule::FieldReferences => Diagnostic {
                expected_form: "**References**: R01, R02",
            },
            Rule::StepHeading => Diagnostic {
                expected_form: "## Step <N>: <step title>",
            },
            Rule::StepFieldGoal => Diagnostic {
                expected_form: "**Goal**: <what this step achieves>",
            },
            Rule::StepFieldImplLogic => Diagnostic {
                expected_form: "**Implementation Logic**: (header alone on its line) / <one or more body lines, blank lines allowed>",
            },
            Rule::StepFieldDeliverables => Diagnostic {
                expected_form: "**Deliverables**: <files/symbols this step produces>",
            },
            Rule::StepFieldConsistency => Diagnostic {
                expected_form: "**Consistency Checks**: <command> (expected: PASS|FAIL)",
            },
            Rule::StepFieldCommit => Diagnostic {
                expected_form: "**Commit**: `<type>(<scope>): <summary>`   (type ∈ feat|fix|test|docs|chore|refactor|style|perf|ci|build|revert)",
            },
            Rule::Step => Diagnostic {
                expected_form: "(constraint: 1 ≤ step count ≤ 5; numbering starts at 1)",
            },
        }
    }

    /// BNF excerpt for this rule, suitable for `dirtree-rdm grammar --rule <name>`.
    /// Includes the rule's production and (where useful) directly-referenced
    /// sub-rules. Exhaustive — every variant must have an excerpt.
    pub fn grammar_excerpt(&self) -> &'static str {
        match self {
            Rule::H1Title => "<h1-title> ::= /^# .+/ EOL",
            Rule::H2Context => "<h2-context> ::= /^## Context$/ EOL",
            Rule::H2Goal => "<h2-goal> ::= /^## Goal$/ EOL",
            Rule::H2Preconditions => "<h2-preconditions> ::= /^## Pre-conditions$/ EOL",
            Rule::H2SuccessGates => "<h2-success-gates> ::= /^## Success Gates$/ EOL",
            Rule::H2Status => "<h2-status> ::= /^## Status$/ EOL",
            Rule::ContextBody => {
                "<context-body> ::= <non-blank-line>+\n<non-blank-line> ::= /^.+/ EOL"
            }
            Rule::GoalBody => {
                "<goal-body> ::= <non-blank-line>\n<non-blank-line> ::= /^.+/ EOL"
            }
            Rule::ReferenceItem => "<reference-item> ::= /^\\- \\[R\\d{2} .+\\]\\(.+\\) — .+/ EOL",
            Rule::GotchasBody => "<section-gotchas> ::= <h2-gotchas> BLANK_LINE* <non-blank-line>+",
            Rule::MermaidBlock => {
                "<mermaid-block> ::= MERMAID_OPEN EOL\n                     <mermaid-graph-decl>\n                     <mermaid-node-decl>*\n                     <mermaid-classdef-done>\n                     <mermaid-classdef-inprogress>\n                     <mermaid-classdef-planned>\n                     <mermaid-classdef-amendment>\n                     <mermaid-classdef-blocked>\n                     MERMAID_CLOSE EOL\nMERMAID_OPEN  ::= /^```mermaid$/\nMERMAID_CLOSE ::= /^```$/"
            }
            Rule::MermaidGraphDecl => "<mermaid-graph-decl> ::= /^graph TD$/ EOL",
            Rule::MermaidNodeDecl => {
                "<mermaid-node-decl> ::= /^    [a-z][a-z0-9_-]*\\[.+\\]:::(done|inprogress|planned|amendment|blocked)$/ EOL"
            }
            Rule::MermaidClassdefDone => {
                "<mermaid-classdef-done> ::= /^    classDef done +fill:#166534,color:#bbf7d0$/ EOL"
            }
            Rule::MermaidClassdefInprogress => {
                "<mermaid-classdef-inprogress> ::= /^    classDef inprogress +fill:#854d0e,color:#fef08a$/ EOL"
            }
            Rule::MermaidClassdefPlanned => {
                "<mermaid-classdef-planned> ::= /^    classDef planned +fill:#374151,color:#e5e7eb$/ EOL"
            }
            Rule::MermaidClassdefAmendment => {
                "<mermaid-classdef-amendment> ::= /^    classDef amendment +fill:#1e3a5f,color:#bfdbfe$/ EOL"
            }
            Rule::MermaidClassdefBlocked => {
                "<mermaid-classdef-blocked> ::= /^    classDef blocked +fill:#7f1d1d,color:#fecaca$/ EOL"
            }
            Rule::TableHeaderNodes => {
                "<table-header-nodes> ::= /^\\| Node +\\| Type +\\| Status +\\|$/ EOL"
            }
            Rule::TableHeaderAmendment => {
                "<table-header-amendment> ::= /^\\| ID +\\| Date +\\| Source +\\| Nodes Added +\\| Rationale +\\|$/ EOL"
            }
            Rule::TableHeaderProgress => {
                "<table-header-progress> ::= /^\\| Node +\\| Branch +\\| Commits +\\| Notes +\\|$/ EOL"
            }
            Rule::TableSeparator => "<table-separator> ::= /^\\|[-: |]+\\|$/ EOL",
            Rule::CheckboxItem => "<checkbox-item> ::= /^\\- \\[[ x]\\] .+/ EOL",
            Rule::GateItem => "<gate-item> ::= /^\\- (✅|⬜) .+/ EOL",
            Rule::FieldGoal => "<field-goal> ::= /^\\*\\*Goal\\*\\*: .+/ EOL",
            Rule::FieldPreconditions => {
                "<field-preconditions> ::= /^\\*\\*Pre-conditions\\*\\*:$/ EOL\n                          <checkbox-item>+\n<checkbox-item> ::= /^\\- \\[[ x]\\] .+/ EOL"
            }
            Rule::FieldSuccessGates => {
                "<field-success-gates> ::= /^\\*\\*Success Gates\\*\\*:$/ EOL\n                          <gate-item>+\n<gate-item> ::= /^\\- (✅|⬜) .+/ EOL"
            }
            Rule::FieldReferences => "<field-references> ::= /^\\*\\*References\\*\\*: .+/ EOL",
            Rule::StepHeading => {
                "<step-heading> ::= /^## Step [1-5]: .+/ EOL\n# CONSTRAINT: step numbers must be sequential starting at 1"
            }
            Rule::StepFieldGoal => "<step-field-goal> ::= /^\\*\\*Goal\\*\\*: .+/ EOL",
            Rule::StepFieldImplLogic => {
                "<step-field-impl-logic> ::= /^\\*\\*Implementation Logic\\*\\*:$/ EOL\n                             <impl-body-line>+\n# CONSTRAINT: at least one <impl-body-line> must be non-blank (validator-enforced)\n<impl-body-line> ::= <non-blank-line> | BLANK_LINE\n<non-blank-line> ::= /^.+/ EOL"
            }
            Rule::StepFieldDeliverables => {
                "<step-field-deliverables> ::= /^\\*\\*Deliverables\\*\\*: .+/ EOL"
            }
            Rule::StepFieldConsistency => {
                "<step-field-consistency> ::= /^\\*\\*Consistency Checks\\*\\*: .+\\(expected: (PASS|FAIL)\\)$/ EOL"
            }
            Rule::StepFieldCommit => {
                "<step-field-commit> ::= /^\\*\\*Commit\\*\\*: `(feat|fix|test|docs|chore|refactor|style|perf|ci|build|revert)\\([a-z][a-z0-9_-]*\\): .+`$/ EOL"
            }
            Rule::Step => {
                "<step> ::= <step-heading>\n           BLANK_LINE*\n           <step-field>+\n# CONSTRAINT: step count must be in range [1, 5]"
            }
        }
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ── search helpers ─────────────────────────────────────────────────────────

/// Whether a rule's name or grammar excerpt contains `needle` as a
/// case-insensitive literal substring.
pub fn rule_matches_substring(rule: Rule, needle: &str) -> bool {
    let needle = needle.to_lowercase();
    rule.name().to_lowercase().contains(&needle)
        || rule.grammar_excerpt().to_lowercase().contains(&needle)
}

/// Whether a rule's name or grammar excerpt matches `pattern` as a regex
/// (already compiled).
pub fn rule_matches_regex(rule: Rule, pattern: &regex::Regex) -> bool {
    pattern.is_match(rule.name()) || pattern.is_match(rule.grammar_excerpt())
}

/// Token-overlap fallback: score by how many distinct query terms appear
/// (case-insensitive substring) in the rule's name or grammar excerpt.
/// Returns rules with non-zero score, sorted by score descending, then by
/// rule name ascending. Empty result iff no rule shares any token with the
/// query.
pub fn rules_by_token_overlap(query: &str, limit: usize) -> Vec<(Rule, usize)> {
    let terms: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();
    if terms.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(Rule, usize)> = Rule::ALL
        .iter()
        .copied()
        .map(|rule| {
            let name_lc = rule.name().to_lowercase();
            let body_lc = rule.grammar_excerpt().to_lowercase();
            let score = terms
                .iter()
                .filter(|t| name_lc.contains(t.as_str()) || body_lc.contains(t.as_str()))
                .count();
            (rule, score)
        })
        .filter(|(_, score)| *score > 0)
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name().cmp(b.0.name())));
    scored.truncate(limit);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_have_unique_kebab_names() {
        let mut names: Vec<&str> = Rule::ALL.iter().map(|r| r.name()).collect();
        names.sort();
        let len_before = names.len();
        names.dedup();
        assert_eq!(len_before, names.len(), "duplicate rule names in Rule::ALL");
    }

    #[test]
    fn all_rules_roundtrip_through_from_name() {
        for &rule in Rule::ALL {
            assert_eq!(Rule::from_name(rule.name()), Some(rule), "roundtrip failed for {}", rule.name());
        }
    }

    #[test]
    fn unknown_name_returns_none() {
        assert_eq!(Rule::from_name("does-not-exist"), None);
    }

    #[test]
    fn diagnostics_and_excerpts_are_non_empty() {
        // Compile-time exhaustiveness guarantees coverage; this guards against
        // accidentally wiring an empty string.
        for &rule in Rule::ALL {
            let d = rule.diagnostic();
            assert!(!d.expected_form.is_empty(), "empty expected_form for {}", rule.name());
            assert!(!rule.grammar_excerpt().is_empty(), "empty grammar_excerpt for {}", rule.name());
        }
    }

    #[test]
    fn token_overlap_finds_consistency_for_concept_query() {
        let results = rules_by_token_overlap("test pass fail expected", 5);
        assert!(
            results.iter().any(|(r, _)| *r == Rule::StepFieldConsistency),
            "step-field-consistency should appear among token-overlap matches for concept query; got {:?}",
            results.iter().map(|(r, s)| (r.name(), *s)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn token_overlap_empty_for_no_overlap() {
        let results = rules_by_token_overlap("xyzqzz", 5);
        assert!(results.is_empty(), "no overlap should yield empty result; got {:?}", results.iter().map(|(r, _)| r.name()).collect::<Vec<_>>());
    }
}
