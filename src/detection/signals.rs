//! Detection signals — lockfiles and install metadata tables.
//!
//! These constants encode the same precedence as `package-manager-detector`.
//! Order matters: earlier entries have higher priority when multiple
//! lockfiles are present in the same directory.

use std::path::Path;

/// Lockfile → agent mapping. Ordered by precedence.
pub(crate) const LOCKS: &[(&str, &str)] = &[
    ("aube-lock.yaml", "aube"),
    ("aube-workspace.yaml", "aube"),
    ("bun.lock", "bun"),
    ("bun.lockb", "bun"),
    ("deno.lock", "deno"),
    ("nub.lock", "nub"),
    ("pnpm-lock.yaml", "pnpm"),
    ("pnpm-workspace.yaml", "pnpm"),
    ("yarn.lock", "yarn"),
    ("package-lock.json", "npm"),
    ("npm-shrinkwrap.json", "npm"),
];

/// Install metadata (post-install artifacts) mapping.
/// Tuple is `(relative_path, agent_name, is_dir)`.
pub(crate) const INSTALL_METADATA: &[(&str, &str, bool)] = &[
    ("node_modules/.aube", "aube", true),
    ("node_modules/.deno", "deno", true),
    ("node_modules/.pnpm", "pnpm", true),
    ("node_modules/.yarn-state.yml", "yarn", false),
    ("node_modules/.yarn_integrity", "yarn", false),
    ("node_modules/.package-lock.json", "npm", false),
    (".pnp.cjs", "yarn", false),
    (".pnp.js", "yarn", false),
    ("bun.lock", "bun", false),
    ("bun.lockb", "bun", false),
];

pub(crate) fn is_yarn_classic_metadata(path: &Path) -> bool {
    path.ends_with(".yarn_integrity")
}
