/// BNF grammar embedded at compile time.
/// These constants are referenced by validate.rs error messages and can be
/// printed by any future `dirtree-rdm grammar` subcommand.
#[allow(dead_code)]
/// These strings are the canonical grammar specs; the validator references
/// them for error messages that cite production names.
pub const README_BNF: &str = include_str!("../grammar/readme.bnf");
pub const LEAF_BNF: &str = include_str!("../grammar/leaf.bnf");
