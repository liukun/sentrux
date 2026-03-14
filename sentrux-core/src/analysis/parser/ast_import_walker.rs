//! Generic AST-based import path extraction.
//!
//! Replaces 13 compiled language-specific text extractors with one generic
//! tree-sitter AST walker. Reads module paths directly from AST node fields
//! and children — no text re-parsing needed.
//!
//! Configuration comes from plugin.toml `[semantics.import_ast]`.
//! Two strategies:
//!   - `field_read`: read a named field/child (Python, Go, JS, C, Ruby)
//!   - `scoped_path`: concatenate scoped identifier chains (Rust, Java)

use crate::analysis::plugin::profile::ImportAstConfig;

/// Maximum recursion depth for AST walking (prevents stack overflow on malformed ASTs).
const MAX_DEPTH: usize = 64;

/// Extract import module paths from a tree-sitter import node using AST structure.
///
/// Returns raw module path strings (not yet normalized with dot→slash conversion).
/// The caller handles normalization via `normalize_module_path()`.
pub(super) fn extract_imports_from_ast(
    import_node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Vec<String> {
    match config.strategy.as_str() {
        "field_read" => extract_field_read(import_node, content, config),
        "scoped_path" => extract_scoped_path(import_node, content, config),
        _ => vec![], // Unknown strategy — caller falls back to legacy
    }
}

// ── Strategy: field_read ──────────────────────────────────────────────

/// Read module paths from a named field or child nodes of the import AST node.
/// Handles: Python (module_name), Go (path), JS (source), C (path), Ruby (arguments).
fn extract_field_read(
    import_node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Vec<String> {
    // If this is a container node (Go import_declaration), iterate child import specs
    if !config.child_import_kind.is_empty() {
        return extract_from_container(import_node, content, config);
    }

    // Try named field first
    if !config.module_path_field.is_empty() {
        if let Some(field_node) = import_node.child_by_field_name(&config.module_path_field) {
            // Filter system includes (C: <stdio.h>)
            if config.filter_system_includes && field_node.kind() == config.system_include_kind {
                return vec![];
            }
            if let Some(path) = read_path_from_node(field_node, content, config) {
                return vec![apply_transform(&path, config)];
            }
        }
    }

    // Fall back: iterate children matching module_path_node_kinds
    let mut results = Vec::new();
    if config.recursive_search {
        // Recursive: search ALL descendants (for deeply nested imports like Elixir multi-alias)
        collect_matching_descendants(import_node, content, config, &mut results);
    } else {
        // Direct children only (default — faster, no false matches)
        for i in 0..import_node.named_child_count() {
            if let Some(child) = import_node.named_child(i) {
                if config.module_path_node_kinds.iter().any(|k| k == child.kind()) {
                    if config.filter_system_includes && child.kind() == config.system_include_kind {
                        continue;
                    }
                    if let Some(path) = read_path_from_node(child, content, config) {
                        results.push(apply_transform(&path, config));
                    }
                }
            }
        }
    }
    results
}

/// Recursively collect all descendant nodes matching module_path_node_kinds.
/// Used when import paths are deeply nested (e.g., Elixir multi-alias).
///
/// Handles multi-alias expansion:
///   alias Acme.Domain.{Product, Error}
/// AST: call → arguments → dot(left: alias("Acme.Domain"), right: tuple(alias("Product"), alias("Error")))
/// Result: ["Acme.Domain.Product", "Acme.Domain.Error"] → after transform → ["acme/domain/product", "acme/domain/error"]
fn collect_matching_descendants(
    node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
    results: &mut Vec<String>,
) {
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            // Check for multi-alias container (configured via TOML, e.g., Elixir "dot")
            if !config.multi_alias_container.is_empty()
                && child.kind() == config.multi_alias_container
            {
                if let (Some(left), Some(right)) = (
                    child.child_by_field_name(&config.multi_alias_prefix_field),
                    child.child_by_field_name(&config.multi_alias_list_field),
                ) {
                    if config.multi_alias_list_kinds.iter().any(|k| k == right.kind()) {
                        if let Some(prefix) = read_path_from_node(left, content, config) {
                            // Collect each alias inside the tuple/list
                            for j in 0..right.named_child_count() {
                                if let Some(item) = right.named_child(j) {
                                    if config.module_path_node_kinds.iter().any(|k| k == item.kind()) {
                                        if let Some(name) = read_path_from_node(item, content, config) {
                                            let full = format!("{}.{}", prefix, name);
                                            results.push(apply_transform(&full, config));
                                        }
                                    }
                                }
                            }
                            continue; // Already handled — don't recurse into dot's children
                        }
                    }
                }
            }

            if config.module_path_node_kinds.iter().any(|k| k == child.kind()) {
                if let Some(path) = read_path_from_node(child, content, config) {
                    results.push(apply_transform(&path, config));
                }
            }
            // Recurse into children regardless — multi-alias may nest several levels deep
            collect_matching_descendants(child, content, config, results);
        }
    }
}

/// Handle container import nodes (Go import_declaration with multiple import_spec children).
fn extract_from_container(
    container_node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Vec<String> {
    let mut results = Vec::new();

    // Look for container list node (Go: import_spec_list)
    for i in 0..container_node.named_child_count() {
        if let Some(child) = container_node.named_child(i) {
            // Check if this child is the list or the spec itself
            if child.kind() == config.child_import_kind.as_str() {
                // Direct child matches spec kind
                results.extend(extract_field_read_single(child, content, config));
            } else {
                // Check grandchildren (the list contains the specs)
                for j in 0..child.named_child_count() {
                    if let Some(gc) = child.named_child(j) {
                        if gc.kind() == config.child_import_kind.as_str() {
                            results.extend(extract_field_read_single(gc, content, config));
                        }
                    }
                }
            }
        }
    }
    results
}

/// Extract a single path from an import spec node.
fn extract_field_read_single(
    node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Vec<String> {
    if !config.module_path_field.is_empty() {
        if let Some(field_node) = node.child_by_field_name(&config.module_path_field) {
            if config.filter_system_includes && field_node.kind() == config.system_include_kind {
                return vec![];
            }
            if let Some(path) = read_path_from_node(field_node, content, config) {
                return vec![apply_transform(&path, config)];
            }
        }
    }
    vec![]
}

/// Read the module path text from a node, optionally unwrapping string literals.
fn read_path_from_node(
    node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Option<String> {
    // Handle Python relative imports
    if !config.relative_import_kind.is_empty() && node.kind() == config.relative_import_kind {
        return read_relative_import(node, content, config);
    }

    // If string_content_kind is set, unwrap the string literal
    if !config.string_content_kind.is_empty() {
        return find_string_content(node, content, &config.string_content_kind);
    }

    // Read node text directly
    node.utf8_text(content).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Read module path from a Python relative import node.
/// Counts dots from import_prefix and appends the module path.
fn read_relative_import(
    node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Option<String> {
    let mut dots = String::new();
    let mut module_path = String::new();

    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            if !config.import_prefix_kind.is_empty() && child.kind() == config.import_prefix_kind {
                // Count dots in the prefix
                if let Ok(text) = child.utf8_text(content) {
                    dots = text.to_string();
                }
            } else {
                // The module path after the dots
                if let Ok(text) = child.utf8_text(content) {
                    module_path = text.trim().to_string();
                }
            }
        }
    }

    if dots.is_empty() && module_path.is_empty() {
        // Fallback: read whole node text
        return node.utf8_text(content).ok().map(|s| s.trim().to_string());
    }

    let combined = format!("{}{}", dots, module_path);
    if combined.is_empty() { None } else { Some(combined) }
}

/// Find a child node of the given kind and read its text (unwrap string literals).
fn find_string_content(
    node: tree_sitter::Node,
    content: &[u8],
    kind: &str,
) -> Option<String> {
    // Direct child
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == kind {
                return child.utf8_text(content).ok().map(|s| s.to_string());
            }
            // One level deeper (string → string_fragment)
            for j in 0..child.child_count() {
                if let Some(gc) = child.child(j) {
                    if gc.kind() == kind {
                        return gc.utf8_text(content).ok().map(|s| s.to_string());
                    }
                }
            }
        }
    }
    None
}

// ── Strategy: scoped_path ─────────────────────────────────────────────

/// Extract module paths from scoped identifier chains (Rust, Java).
/// Handles use_list branching (Rust `use a::{b, c}`).
fn extract_scoped_path(
    import_node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
) -> Vec<String> {
    // Module declaration (e.g., Rust `mod foo;`) — just read the name field.
    // Configured via mod_declaration_kind in plugin TOML.
    if !config.mod_declaration_kind.is_empty() && import_node.kind() == config.mod_declaration_kind {
        if let Some(name) = import_node.child_by_field_name("name") {
            if let Ok(text) = name.utf8_text(content) {
                let t = text.trim().to_string();
                if !t.is_empty() {
                    return vec![t];
                }
            }
        }
        return vec![];
    }

    // Find the argument/path child of the import declaration
    let arg_node = import_node.child_by_field_name("argument")
        .or_else(|| {
            // Fall back: find first child matching scoped_path_kinds
            (0..import_node.named_child_count()).find_map(|i| {
                import_node.named_child(i).filter(|c| {
                    config.scoped_path_kinds.iter().any(|k| k == c.kind())
                })
            })
        });

    match arg_node {
        Some(node) => collect_scoped_paths(node, content, config, 0),
        None => {
            // Last resort: read the whole node text (minus keywords)
            import_node.utf8_text(content)
                .ok()
                .map(|t| vec![t.trim().to_string()])
                .unwrap_or_default()
        }
    }
}

/// Recursively collect paths from a scoped path node.
/// At use_list nodes, branches into multiple paths (one per child).
fn collect_scoped_paths(
    node: tree_sitter::Node,
    content: &[u8],
    config: &ImportAstConfig,
    depth: usize,
) -> Vec<String> {
    if depth >= MAX_DEPTH {
        return vec![];
    }

    let kind = node.kind();

    // If this is a use_list, branch into each child
    if !config.use_list_kind.is_empty() && kind == config.use_list_kind {
        let mut results = Vec::new();
        for i in 0..node.named_child_count() {
            if let Some(child) = node.named_child(i) {
                results.extend(collect_scoped_paths(child, content, config, depth + 1));
            }
        }
        return results;
    }

    // If this is a scoped use list (path + list), combine prefix with each list item.
    // Matches any scoped_path_kind that has both "path" and "list" fields.
    if config.scoped_path_kinds.iter().any(|k| k == kind) && node.child_by_field_name("list").is_some() {
        let prefix = node.child_by_field_name("path")
            .and_then(|p| read_scoped_text(p, content))
            .unwrap_or_default();

        if let Some(list) = node.child_by_field_name("list") {
            let mut results = Vec::new();
            for i in 0..list.named_child_count() {
                if let Some(child) = list.named_child(i) {
                    let child_paths = collect_scoped_paths(child, content, config, depth + 1);
                    for cp in child_paths {
                        if prefix.is_empty() {
                            results.push(cp);
                        } else {
                            results.push(format!("{}{}{}", prefix, config.path_separator, cp));
                        }
                    }
                }
            }
            // If all children were types (returned empty), emit just the prefix
            // as the module dependency (the crate/module is what matters, not types).
            if results.is_empty() && !prefix.is_empty() {
                results.push(prefix);
            }
            // Deduplicate: if some children resolved to prefix and others to prefix::submod,
            // keep the submodule paths but ensure prefix itself is included if any types found.
            return results;
        }

        // No list child — just the path
        if !prefix.is_empty() {
            return vec![prefix];
        }
    }

    // If this is a scoped identifier, read its full text
    if config.scoped_path_kinds.iter().any(|k| k == kind) {
        if let Some(text) = read_scoped_text(node, content) {
            return vec![text];
        }
    }

    // Leaf identifier — read text directly.
    // Leaf kinds are configured via leaf_identifier_kinds in TOML.
    // Fallback: "identifier" if leaf_identifier_kinds is empty.
    let is_leaf = if config.leaf_identifier_kinds.is_empty() {
        kind == "identifier"
    } else {
        config.leaf_identifier_kinds.iter().any(|k| k == kind)
    };
    if is_leaf {
        if let Ok(text) = node.utf8_text(content) {
            let t = text.trim();
            if !t.is_empty() {
                // If configured, skip uppercase identifiers in use_list
                // (they're type imports — parent module is the real dependency).
                if config.skip_type_imports_in_use_list {
                    let first_char = t.chars().next().unwrap_or('a');
                    if first_char.is_uppercase() && !config.use_list_kind.is_empty() {
                        if let Some(parent) = node.parent() {
                            if parent.kind() == config.use_list_kind {
                                return vec![]; // Type import — skip
                            }
                        }
                    }
                }
                return vec![t.to_string()];
            }
        }
    }

    // Fallback: read the node text
    if let Ok(text) = node.utf8_text(content) {
        let t = text.trim().to_string();
        if !t.is_empty() {
            return vec![t];
        }
    }
    vec![]
}

/// Read the full text of a scoped identifier/path node.
fn read_scoped_text(node: tree_sitter::Node, content: &[u8]) -> Option<String> {
    node.utf8_text(content).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

// ── Post-processing transforms ────────────────────────────────────────

/// Apply module name transform (e.g., Elixir PascalCase→snake_case).
fn apply_transform(path: &str, config: &ImportAstConfig) -> String {
    match config.module_name_transform.as_str() {
        "pascal_to_snake" => super::lang_extractors::pascal_to_snake_path(path),
        _ => path.to_string(),
    }
}
