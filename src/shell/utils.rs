use std::collections::HashMap;
use std::env;
use std::fs;
use strsim::damerau_levenshtein;

pub fn expand_aliases(line: &str, aliases: &HashMap<String, String>) -> String {
    let mut parts = line.splitn(2, ' ');
    if let Some(first) = parts.next() {
        if let Some(replacement) = aliases.get(first) {
            if let Some(rest) = parts.next() {
                return format!("{} {}", replacement, rest);
            } else {
                return replacement.clone();
            }
        }
    }
    line.to_string()
}

pub fn suggest_commands(
    name: &str,
    builtins: &[String],
    aliases: &[String],
    include_builtins: bool,
    include_system: bool,
) -> Vec<String> {
    let mut possible = Vec::new();

    // 1. Builtins & Aliases
    if include_builtins {
        possible.extend(builtins.iter().cloned());
        possible.extend(aliases.iter().cloned());
    }

    // 3. System PATH
    if include_system {
        if let Ok(path) = env::var("PATH") {
            for dir in path.split(':') {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        if let Ok(fname) = entry.file_name().into_string() {
                            possible.push(fname);
                        }
                    }
                }
            }
        }
    }

    possible.sort();
    possible.dedup();

    let mut matches = Vec::new();

    for cmd in possible {
        let dist = damerau_levenshtein(name, &cmd);
        if dist <= 2 {
            matches.push((dist, cmd));
        }
    }

    // Sort by distance (asc), then by name (asc)
    matches.sort_by(|a, b| {
        if a.0 != b.0 {
            a.0.cmp(&b.0)
        } else {
            a.1.cmp(&b.1)
        }
    });

    matches.into_iter().map(|(_, cmd)| cmd).take(3).collect()
}

pub fn expand_braces(input: &str) -> Vec<String> {
    if let Some(start) = input.find('{') {
        let mut depth = 0;
        for (i, c) in input[start..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    let end = start + i;
                    let prefix = &input[..start];
                    let suffix = &input[end + 1..];
                    let inside = &input[start + 1..end];

                    let mut results = Vec::new();
                    for part in inside.split(',') {
                        let combined = format!("{}{}{}", prefix, part, suffix);
                        results.extend(expand_braces(&combined));
                    }
                    return results;
                }
            }
        }
    }
    vec![input.to_string()]
}

pub fn expand_paths(args: &[String], cwd: &str) -> Vec<String> {
    let mut expanded = Vec::new();

    for arg in args {
        let braced = expand_braces(arg);

        for item in braced {
            if item.contains('*') || item.contains('?') || item.contains('[') {
                let search_pattern = if std::path::Path::new(&item).is_absolute() {
                    item.clone()
                } else {
                    format!("{}/{}", cwd, item)
                };

                let mut matched_any = false;
                if let Ok(paths) = glob::glob(&search_pattern) {
                    for path in paths.flatten() {
                        matched_any = true;

                        // Preserve relative paths if the user provided one
                        if item.starts_with("./") && path.starts_with(cwd) {
                            let rel = path.strip_prefix(cwd).unwrap_or(&path).to_string_lossy();
                            let formatted = format!("./{}", rel.trim_start_matches('/'));
                            expanded.push(formatted);
                        } else {
                            expanded.push(path.to_string_lossy().to_string());
                        }
                    }
                }
                if !matched_any {
                    expanded.push(item);
                }
            } else {
                expanded.push(item);
            }
        }
    }

    expanded
}
