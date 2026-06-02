use crate::models::Keybind;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Context for parsing Hyprland Lua configurations
pub struct HyprLuaContext {
    pub variables: HashMap<String, String>,
    pub processed_files: HashSet<PathBuf>,
    pub base_dir: PathBuf,
}

impl HyprLuaContext {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            variables: HashMap::new(),
            processed_files: HashSet::new(),
            base_dir,
        }
    }
}

/// Recursively parses Hyprland Lua configuration
pub fn parse_hyprland_lua_recursive(
    path: PathBuf,
    ctx: &mut HyprLuaContext,
    binds: &mut Vec<Keybind>,
) {
    let canonical = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(_) => {
            // Try appending .lua
            let with_ext = path.with_extension("lua");
            match fs::canonicalize(&with_ext) {
                Ok(p) => p,
                Err(_) => return,
            }
        }
    };

    if ctx.processed_files.contains(&canonical) {
        return;
    }
    ctx.processed_files.insert(canonical.clone());

    let content = match fs::read_to_string(&canonical) {
        Ok(c) => c,
        Err(_) => return,
    };

    parse_lua_content(&content, &canonical, ctx, binds);
}

/// Parses the content of a Lua file
fn parse_lua_content(
    content: &str,
    file_path: &Path,
    ctx: &mut HyprLuaContext,
    binds: &mut Vec<Keybind>,
) {
    let file_dir = file_path.parent().unwrap_or(&ctx.base_dir).to_path_buf();

    // Strip block comments --[[ ... ]]
    let content = strip_block_comments(content);

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        i += 1;

        // Skip empty lines and single-line comments
        if line.is_empty() || line.starts_with("--") {
            continue;
        }

        // Strip inline comments: `code -- comment` → `code`
        let line = strip_line_comment(line);

        // Handle local variable assignment: local var = "value"
        if let Some(caps) = regex_local_var().captures(line) {
            let name = caps[1].to_string();
            let raw_value = caps[2].trim().to_string();
            let value = eval_lua_string_expr(&raw_value, ctx);
            ctx.variables.insert(name, value);
            continue;
        }

        // Handle require("path")
        if let Some(caps) = regex_require().captures(line) {
            let req_path = &caps[1];
            let resolved = resolve_require_path(req_path, &file_dir, &ctx.base_dir);
            parse_hyprland_lua_recursive(resolved, ctx, binds);
            continue;
        }

        // Handle for i = start, end do ... end (collect block)
        if let Some(caps) = regex_for_loop().captures(line) {
            let var_name = caps[1].to_string();
            let start: i64 = caps[2].parse().unwrap_or(0);
            let end: i64 = caps[3].parse().unwrap_or(0);

            // Collect loop body until matching "end"
            let mut body_lines = Vec::new();
            let mut depth = 1;
            while i < lines.len() && depth > 0 {
                let l = lines[i].trim();
                if is_block_opener(l) {
                    depth += 1;
                }
                if l == "end" || l.starts_with("end ") || l.starts_with("end)") {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                body_lines.push(lines[i]);
                i += 1;
            }

            let body = body_lines.join("\n");
            for val in start..=end {
                let expanded = body.replace(
                    &format!(" {} ", var_name),
                    &format!(" {} ", val),
                );
                // Also replace variable in string concatenation contexts and standalone
                let expanded = expand_loop_var(&expanded, &var_name, val);

                // Temporarily set loop variable
                let old_val = ctx.variables.get(&var_name).cloned();
                ctx.variables.insert(var_name.clone(), val.to_string());
                parse_lua_content(&expanded, file_path, ctx, binds);
                // Restore
                match old_val {
                    Some(v) => ctx.variables.insert(var_name.clone(), v),
                    None => ctx.variables.remove(&var_name),
                };
            }
            continue;
        }

        // Handle hl.bind(...) - may span multiple lines
        if line.contains("hl.bind(") {
            let mut full_line = line.to_string();
            // Collect continuation lines if parens aren't balanced
            while !parens_balanced(&full_line) && i < lines.len() {
                full_line.push(' ');
                full_line.push_str(lines[i].trim());
                i += 1;
            }
            if let Some(kb) = parse_hl_bind(&full_line, ctx) {
                binds.push(kb);
            }
        }
    }
}

/// Parses a single hl.bind(...) call into a Keybind
fn parse_hl_bind(line: &str, ctx: &HyprLuaContext) -> Option<Keybind> {
    // Extract the arguments inside hl.bind(...)
    let bind_start = line.find("hl.bind(")?;
    let args_start = bind_start + "hl.bind(".len();
    let args_str = extract_balanced_parens(&line[args_start..])?;

    // Split into top-level arguments (respecting nested parens/braces/strings)
    let args = split_top_level_args(&args_str);
    if args.len() < 2 {
        return None;
    }

    // First arg: key combination string
    let keys_raw = eval_lua_string_expr(args[0].trim(), ctx);

    // Second arg: dispatcher (we show it as command)
    let command_raw = args[1].trim().to_string();
    let command = simplify_dispatcher(&command_raw, ctx);

    // Third arg (optional): options table {description = "...", ...}
    let description = if args.len() >= 3 {
        extract_description(args[2].trim())
    } else {
        None
    };

    // Parse key combination: "SUPER + SHIFT + Return" -> mods=[SUPER, SHIFT], key=Return
    let (mods, key) = parse_key_combo(&keys_raw);

    if key.is_empty() {
        return None;
    }

    Some(Keybind {
        mods,
        key,
        command,
        description,
    })
}

/// Parses key combination string like "SUPER + SHIFT + Q" or "mainMod .. \" + Q\""
fn parse_key_combo(keys: &str) -> (Vec<String>, String) {
    let parts: Vec<String> = keys
        .split('+')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return (vec![], String::new());
    }

    let key = parts.last().unwrap().clone();
    let mods: Vec<String> = parts[..parts.len() - 1].to_vec();

    (mods, key)
}

/// Evaluates Lua string expressions with proper operator precedence:
/// `or` (lowest) → `..` (concat) → atoms (strings, vars, calls)
fn eval_lua_string_expr(expr: &str, ctx: &HyprLuaContext) -> String {
    let expr = expr.trim();

    if expr.is_empty() {
        return String::new();
    }

    // Parenthesized expression — only strip if the outer parens wrap the whole thing
    if expr.starts_with('(') && expr.ends_with(')') && is_outer_parens(expr) {
        return eval_lua_string_expr(&expr[1..expr.len() - 1], ctx);
    }

    // `or` — lowest precedence, checked first
    if let Some(or_pos) = find_top_level_or(expr) {
        let primary = expr[..or_pos].trim();
        let fallback = expr[or_pos + 4..].trim();
        let primary_val = eval_lua_string_expr(primary, ctx);
        if primary_val.is_empty() {
            return eval_lua_string_expr(fallback, ctx);
        }
        return primary_val;
    }

    // `..` concatenation — respects nesting (strings, parens)
    let concat_parts = split_top_level_concat(expr);
    if concat_parts.len() > 1 {
        let mut result = String::new();
        for part in &concat_parts {
            result.push_str(&eval_lua_string_expr(part.trim(), ctx));
        }
        return result;
    }

    // --- atoms below ---

    // Quoted string
    if (expr.starts_with('"') && expr.ends_with('"'))
        || (expr.starts_with('\'') && expr.ends_with('\''))
    {
        return expr[1..expr.len() - 1].to_string();
    }

    // Long string [[...]]
    if expr.starts_with("[[") && expr.ends_with("]]") {
        return expr[2..expr.len() - 2].to_string();
    }

    // Variable reference
    if let Some(val) = ctx.variables.get(expr) {
        return val.clone();
    }

    // os.getenv("VAR")
    if expr.starts_with("os.getenv(") {
        if let Some(caps) = regex_os_getenv().captures(expr) {
            let var_name = &caps[1];
            if let Ok(val) = std::env::var(var_name) {
                return val;
            }
        }
        return String::new();
    }

    // Numeric literal
    if expr.parse::<i64>().is_ok() {
        return expr.to_string();
    }

    // Modulo operation: i % 10
    if let Some(caps) = regex_modulo().captures(expr) {
        let lhs = eval_lua_string_expr(caps[1].trim(), ctx);
        let rhs = eval_lua_string_expr(caps[2].trim(), ctx);
        if let (Ok(l), Ok(r)) = (lhs.parse::<i64>(), rhs.parse::<i64>()) {
            if r != 0 {
                return (l % r).to_string();
            }
        }
    }

    // Return as-is (unknown reference)
    expr.to_string()
}

/// Extracts description from options table: {description = "Open terminal", ...}
fn extract_description(opts: &str) -> Option<String> {
    let re = Regex::new(r#"description\s*=\s*"([^"]*)""#).ok()?;
    re.captures(opts).map(|c| c[1].to_string())
}

/// Simplifies dispatcher representation for display, resolving variables
fn simplify_dispatcher(raw: &str, ctx: &HyprLuaContext) -> String {
    let raw = raw.trim();

    // hl.dsp.exec_cmd(...) — extract inner expression and evaluate it
    if raw.starts_with("hl.dsp.exec_cmd(") {
        let inner_start = "hl.dsp.exec_cmd(".len();
        if let Some(inner) = extract_balanced_parens(&raw[inner_start..]) {
            let inner = inner.trim();
            // Multi-line string [[...]] — collapse whitespace for display
            if inner.starts_with("[[") && inner.ends_with("]]") {
                let content = &inner[2..inner.len() - 2];
                return content.split_whitespace().collect::<Vec<_>>().join(" ");
            }
            // Evaluate expression: resolves variables, string concat, etc.
            return eval_lua_string_expr(inner, ctx);
        }
    }

    // hl.dsp.exit() -> exit, hl.dsp.window.close() -> window.close(), etc.
    if raw.starts_with("hl.dsp.") {
        let without_prefix = &raw[7..];
        let cleaned = without_prefix
            .replace("({ ", "(")
            .replace("({" , "(")
            .replace(" })", ")")
            .replace("})", ")");
        return cleaned;
    }

    raw.to_string()
}

/// Extracts content inside balanced parentheses.
/// Called on the substring AFTER the opening delimiter, e.g. after "hl.bind(".
/// Returns everything up to the matching closing ')'.
fn extract_balanced_parens(input: &str) -> Option<String> {
    let mut depth = 1i32; // we're already inside the opening paren
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_long_string = false;
    let mut prev_char = '\0';

    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;

        if in_long_string {
            if ch == ']' && i + 1 < bytes.len() && bytes[i + 1] == b']' {
                in_long_string = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            i += 1;
            continue;
        }

        match ch {
            '[' if i + 1 < bytes.len() && bytes[i + 1] == b'[' => {
                in_long_string = true;
                i += 2;
                continue;
            }
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(input[..i].to_string());
                }
            }
            _ => {}
        }
        prev_char = ch;
        i += 1;
    }
    None
}

/// Splits arguments at top-level commas (respecting nesting and strings)
fn split_top_level_args(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth_paren = 0i32;
    let mut depth_brace = 0i32;
    let mut depth_bracket = 0i32;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_long_string = false;
    let mut prev_char = '\0';
    let mut last_split = 0;

    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;

        if in_long_string {
            if ch == ']' && i + 1 < bytes.len() && bytes[i + 1] == b']' {
                in_long_string = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            i += 1;
            continue;
        }

        match ch {
            '[' if i + 1 < bytes.len() && bytes[i + 1] == b'[' => {
                in_long_string = true;
                i += 2;
                continue;
            }
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth_paren += 1,
            ')' => depth_paren -= 1,
            '{' => depth_brace += 1,
            '}' => depth_brace -= 1,
            '[' => depth_bracket += 1,
            ']' => depth_bracket -= 1,
            ',' if depth_paren == 0 && depth_brace == 0 && depth_bracket == 0 => {
                result.push(&s[last_split..i]);
                last_split = i + 1;
            }
            _ => {}
        }
        prev_char = ch;
        i += 1;
    }
    if last_split < s.len() {
        result.push(&s[last_split..]);
    }
    result
}

/// Checks if parentheses in the string are balanced
fn parens_balanced(s: &str) -> bool {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_long_string = false;
    let mut prev_char = '\0';

    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;

        if in_long_string {
            if ch == ']' && i + 1 < bytes.len() && bytes[i + 1] == b']' {
                in_long_string = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            i += 1;
            continue;
        }

        match ch {
            '[' if i + 1 < bytes.len() && bytes[i + 1] == b'[' => {
                in_long_string = true;
                i += 2;
                continue;
            }
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        prev_char = ch;
        i += 1;
    }
    depth == 0
}

/// Resolves a require("path") to an actual file path
fn resolve_require_path(req_path: &str, file_dir: &Path, base_dir: &Path) -> PathBuf {
    // Convert Lua module path: "default/keybindings" -> "default/keybindings.lua"
    let rel_path = req_path.replace('.', "/");

    // Try relative to base_dir first (Lua convention)
    let candidate = base_dir.join(&rel_path).with_extension("lua");
    if candidate.exists() {
        return candidate;
    }

    // Try without replacing dots (already has slash separators)
    let candidate = base_dir.join(req_path).with_extension("lua");
    if candidate.exists() {
        return candidate;
    }

    // Try relative to current file's directory
    let candidate = file_dir.join(&rel_path).with_extension("lua");
    if candidate.exists() {
        return candidate;
    }

    let candidate = file_dir.join(req_path).with_extension("lua");
    if candidate.exists() {
        return candidate;
    }

    // Return best guess
    base_dir.join(req_path).with_extension("lua")
}

/// Strips an inline `-- comment` from a line, respecting string literals
fn strip_line_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut string_char = '"';
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_string {
            if ch == string_char {
                in_string = false;
            }
        } else {
            match ch {
                '"' | '\'' => {
                    in_string = true;
                    string_char = ch;
                }
                '-' if i + 1 < bytes.len() && bytes[i + 1] == b'-' => {
                    return line[..i].trim_end();
                }
                _ => {}
            }
        }
        i += 1;
    }
    line
}

/// Strips block comments --[[ ... ]] from Lua source
fn strip_block_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 3 < bytes.len()
            && bytes[i] == b'-'
            && bytes[i + 1] == b'-'
            && bytes[i + 2] == b'['
            && bytes[i + 3] == b'['
        {
            // Find closing ]]
            i += 4;
            while i + 1 < bytes.len() {
                if bytes[i] == b']' && bytes[i + 1] == b']' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Checks if a line opens a new block (for/while/if/function/do)
fn is_block_opener(line: &str) -> bool {
    let line = line.trim();
    // Match lines that open blocks but aren't just the word
    (line.starts_with("for ") && line.ends_with(" do"))
        || (line.starts_with("while ") && line.ends_with(" do"))
        || (line.starts_with("if ") && line.contains(" then"))
        || line.starts_with("function ")
        || line.starts_with("function(")
        || line == "do"
        || (line.contains("function(") && !line.contains("end"))
}

/// Expands loop variable in all contexts within the loop body
fn expand_loop_var(body: &str, var_name: &str, val: i64) -> String {
    let val_str = val.to_string();
    let mut result = body.to_string();

    // Replace patterns where the variable appears as a standalone identifier
    // Use word boundary matching
    let re = Regex::new(&format!(r"\b{}\b", regex::escape(var_name))).unwrap();
    result = re.replace_all(&result, val_str.as_str()).to_string();

    result
}

/// Checks if the opening '(' at position 0 wraps the entire expression
fn is_outer_parens(expr: &str) -> bool {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut string_char = '"';
    let bytes = expr.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        let ch = b as char;
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                // If depth hits 0 before the last char, the outer parens don't wrap everything
                if depth == 0 && i < bytes.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// Splits expression at top-level `..` concatenation operators (respects strings/parens)
fn split_top_level_concat(expr: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_long_string = false;
    let mut prev_char = '\0';
    let mut last_split = 0;

    let bytes = expr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;

        if in_long_string {
            if ch == ']' && i + 1 < bytes.len() && bytes[i + 1] == b']' {
                in_long_string = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = ch;
            i += 1;
            continue;
        }

        match ch {
            '[' if i + 1 < bytes.len() && bytes[i + 1] == b'[' => {
                in_long_string = true;
                i += 2;
                continue;
            }
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => depth -= 1,
            // Match `..` but not `...` (varargs)
            '.' if depth == 0
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'.'
                && (i + 2 >= bytes.len() || bytes[i + 2] != b'.') =>
            {
                parts.push(&expr[last_split..i]);
                last_split = i + 2;
                i += 2;
                prev_char = '.';
                continue;
            }
            _ => {}
        }
        prev_char = ch;
        i += 1;
    }
    if last_split < expr.len() {
        parts.push(&expr[last_split..]);
    }
    parts
}

/// Finds the position of a top-level "or" keyword (not inside strings/parens)
fn find_top_level_or(expr: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut string_char = '"';
    let bytes = expr.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '(' => depth += 1,
            ')' => depth -= 1,
            ' ' if depth == 0
                && i + 4 < bytes.len()
                && &bytes[i..i + 4] == b" or " =>
            {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// Cached regex helpers
fn regex_local_var() -> Regex {
    Regex::new(r#"^local\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)$"#).unwrap()
}

fn regex_require() -> Regex {
    Regex::new(r#"require\s*\(\s*"([^"]+)"\s*\)"#).unwrap()
}

fn regex_for_loop() -> Regex {
    Regex::new(r"^for\s+([a-zA-Z_]\w*)\s*=\s*(-?\d+)\s*,\s*(-?\d+)\s+do\s*$").unwrap()
}

fn regex_os_getenv() -> Regex {
    Regex::new(r#"os\.getenv\(\s*"([^"]+)"\s*\)"#).unwrap()
}

fn regex_modulo() -> Regex {
    Regex::new(r"^(.+)\s*%\s*(.+)$").unwrap()
}
