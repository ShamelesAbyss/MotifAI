//! Conservative Strudel source extraction and lexical preflight checks for MotifAI.
//!
//! This module is deliberately not a co-composer. It does not rewrite pitches,
//! impose a scale, replace DSP methods, clamp parameters, merge roots, or auto-mix.
//! It extracts candidate code, inserts MotifAI's default tempo only when no
//! top-level tempo call is present, applies one narrow repair for a malformed
//! standalone `@number` filter-cutoff string, and rejects a small set of clearly
//! identified source defects. It does NOT parse JavaScript into a full AST or execute Strudel.
//!
//! `validate_strudel_ast` is retained as a compatibility name. Its result means
//! lexical/structural preflight passed, not runtime evaluation or audible playback.

use crate::sonic_intelligence::ExternalAstAnalysis;
use std::collections::HashSet;

const DEFAULT_CPS: &str = "setcps(0.55)";

/// Extracts a likely Strudel program, applies safe host-level normalization, and
/// performs conservative source preflight. Musical expressions are not broadly
/// rewritten. A standalone `@number` filter-cutoff string such as `.lpf("@200")`
/// is normalized to the numeric cutoff `.lpf(200)`. Unknown APIs are preserved.
pub fn compile_and_heal_strudel(raw: &str) -> Result<String, String> {
    let extracted = strip_thinking_and_extract_code(raw);
    if extracted.trim().is_empty() {
        return Err("Model returned no recognizable Strudel code.".to_string());
    }

    // Keep generated source intact apart from one narrowly understood failure:
    // a standalone `@number` inside a filter-cutoff string (for example
    // `.lpf("@200")`) is not valid mini-notation. Since the parameter name
    // identifies the numeric domain and the literal contains exactly one number,
    // the conservative fallback is the equivalent fixed cutoff `.lpf(200)`.
    // We do not rewrite weighted events such as `note("c4@2 e4")` or genuine
    // cutoff patterns such as `.lpf("<200 400 800>")`.
    let mut code = heal_standalone_weighted_filter_values(extracted.trim());

    let tempo_count = count_top_level_tempo_calls(&code)?;
    if tempo_count == 0 {
        code = insert_default_tempo_after_leading_comments(&code);
    }

    validate_strudel_ast(&code)?;
    Ok(code)
}

/// Produces lightweight, text-derived analysis for Sonic Intelligence.
///
/// `success` means this module's static preflight succeeded. It does not mean a
/// JavaScript AST parser, browser evaluation, or playback engine ran. The output
/// schema's `ast_compiled_cleanly` field is legacy-named and is retained here for
/// API compatibility; callers must not interpret it as runtime verification.
pub fn analyze_ast_health(
    healed_code: &str,
    mut warnings: Vec<String>,
    success: bool,
) -> ExternalAstAnalysis {
    let masked = mask_comments_and_strings(healed_code).unwrap_or_else(|_| {
        // Analysis should remain available for diagnostics even for malformed
        // source. The caller's preflight result still determines `success`.
        healed_code.to_string()
    });

    let mut parsed_sections = Vec::new();
    let slider_names = extract_slider_names(&masked);
    let visualizer_names = extract_visualizer_names(&masked);
    let pattern_signatures = extract_trigger_strings(healed_code);
    let cycle_count = count_arranged_cycles(&masked);

    for line in masked.lines() {
        let trimmed = line.trim();
        if is_variable_declaration(trimmed) {
            if let Some((left, right)) = trimmed.split_once('=') {
                let name = declared_name(left);
                if !name.is_empty()
                    && contains_any_call(
                        right,
                        &[
                            "stack", "arrange", "cat", "slowcat", "seq", "fastcat", "stepcat",
                            "polymeter",
                        ],
                    )
                {
                    parsed_sections.push(name.to_string());
                }
            }
        }
    }

    // Slider type/signature details are intentionally runtime-dependent. Emit
    // conservative diagnostics instead of assuming the helper returns a number
    // or a Pattern and rewriting the generated expression.
    for name in &slider_names {
        let occurrences = identifier_occurrences(&masked, name);
        if occurrences <= 1 {
            warnings.push(format!(
                "Slider '{name}' appears to be declared but not referenced elsewhere; verify that it is wired to an audible parameter."
            ));
        }
        if identifier_has_arithmetic_or_comparison_use(&masked, name) {
            warnings.push(format!(
                "Slider '{name}' participates in JavaScript arithmetic/comparison. The Strudel REPL transpiles slider(...) into a Pattern-valued numeric control; pass it directly to a parameter or use Strudel pattern methods such as .mul(...), .add(...), or .range(...) as appropriate."
            ));
        }
    }

    let tempo_count = count_top_level_tempo_calls(&masked).unwrap_or(0);
    if tempo_count > 1 {
        warnings.push(format!(
            "Found {tempo_count} top-level tempo declarations; confirm that multiple tempo changes are intentional."
        ));
    }

    warnings = unique_preserving_order(&warnings);

    ExternalAstAnalysis {
        // Compatibility field: this reflects lexical preflight success only.
        // It does not assert a real JavaScript AST parse or browser compilation.
        ast_compiled_cleanly: success,
        warnings,
        parsed_sections: unique_preserving_order(&parsed_sections),
        slider_names,
        cycle_count,
        visualizer_names,
        pattern_signatures,
    }
}

/// Legacy function name retained for existing callers. This is a static
/// preflight, not a JavaScript AST parse and not an execution of Strudel.
pub fn validate_strudel_ast(code: &str) -> Result<(), String> {
    if code.trim().is_empty() {
        return Err("Strudel preflight failed: source is empty.".to_string());
    }

    let masked = mask_comments_and_strings(code)?;
    validate_delimiters(&masked)?;
    validate_mini_notation_delimiters(code)?;
    reject_standalone_weighted_filter_values(code)?;
    reject_unquoted_numeric_notation_candidates(&masked)?;
    reject_javascript_slider_arithmetic(&masked)?;
    reject_invented_root_wrapper(&masked)?;

    if !contains_trigger_call(&masked) {
        return Err(
            "Strudel preflight failed: no s(...), sound(...), note(...), or n(...) call was found outside comments and strings."
                .to_string(),
        );
    }

    if !has_final_root_expression(&masked) {
        return Err(
            "Strudel preflight failed: no final evaluated pattern expression was found. End the program with a root pattern expression or a variable that resolves to the intended Pattern; declarations alone are not a playback root."
                .to_string(),
        );
    }

    reject_unused_sliders_and_pattern_stems(&masked)?;

    Ok(())
}

fn is_variable_declaration(line: &str) -> bool {
    ["const ", "let ", "var ", "const\t", "let\t", "var\t"]
        .iter()
        .any(|prefix| line.starts_with(prefix))
}

fn declared_name(lhs: &str) -> &str {
    lhs.split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '$')
}

fn contains_any_call(source: &str, names: &[&str]) -> bool {
    names.iter().any(|name| contains_identifier_call(source, name))
}

fn contains_identifier_call(source: &str, name: &str) -> bool {
    let chars: Vec<char> = source.chars().collect();
    let wanted: Vec<char> = name.chars().collect();
    if wanted.is_empty() || chars.len() < wanted.len() {
        return false;
    }

    let mut i = 0usize;
    while i + wanted.len() <= chars.len() {
        if chars[i..i + wanted.len()] == wanted[..]
            && (i == 0 || !is_identifier_char(chars[i - 1]))
            && (i + wanted.len() == chars.len() || !is_identifier_char(chars[i + wanted.len()]))
        {
            let mut next = i + wanted.len();
            while next < chars.len() && chars[next].is_whitespace() {
                next += 1;
            }
            if next < chars.len() && chars[next] == '(' {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Repairs only a standalone `@number` passed as a quoted value to a filter
/// cutoff method, e.g. `.lpf("@200")` -> `.lpf(200)`. In mini-notation, `@` is
/// a temporal weight operator attached to an event; it is not a prefix for a
/// numeric parameter. This targeted fallback keeps the score playable without
/// inventing a dynamic curve. For movement, the generator should emit a real
/// value pattern such as `.lpf("<200 400 800>")` instead.
fn heal_standalone_weighted_filter_values(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut i = 0usize;
    let mut state = SourceScanState::Code;

    while i < source.len() {
        let rest = &source[i..];

        match state {
            SourceScanState::Code => {
                if rest.starts_with("//") {
                    output.push_str("//");
                    i += 2;
                    state = SourceScanState::LineComment;
                    continue;
                }
                if rest.starts_with("/*") {
                    output.push_str("/*");
                    i += 2;
                    state = SourceScanState::BlockComment;
                    continue;
                }

                if let Some((quote_start, quote_end, numeric)) =
                    standalone_weighted_filter_argument_at(source, i)
                {
                    output.push_str(&source[i..quote_start]);
                    output.push_str(&numeric);
                    i = quote_end + 1;
                    continue;
                }

                let ch = rest.chars().next().expect("i is inside source");
                output.push(ch);
                i += ch.len_utf8();
                state = match ch {
                    '\'' => SourceScanState::SingleQuoted,
                    '"' => SourceScanState::DoubleQuoted,
                    '`' => SourceScanState::Template,
                    _ => SourceScanState::Code,
                };
            }
            SourceScanState::SingleQuoted
            | SourceScanState::DoubleQuoted
            | SourceScanState::Template => {
                let ch = rest.chars().next().expect("i is inside source");
                output.push(ch);
                i += ch.len_utf8();
                if ch == '\\' {
                    if i < source.len() {
                        let escaped = source[i..].chars().next().expect("i is inside source");
                        output.push(escaped);
                        i += escaped.len_utf8();
                    }
                } else if (state == SourceScanState::SingleQuoted && ch == '\'')
                    || (state == SourceScanState::DoubleQuoted && ch == '"')
                    || (state == SourceScanState::Template && ch == '`')
                {
                    state = SourceScanState::Code;
                }
            }
            SourceScanState::LineComment => {
                let ch = rest.chars().next().expect("i is inside source");
                output.push(ch);
                i += ch.len_utf8();
                if ch == '\n' {
                    state = SourceScanState::Code;
                }
            }
            SourceScanState::BlockComment => {
                if rest.starts_with("*/") {
                    output.push_str("*/");
                    i += 2;
                    state = SourceScanState::Code;
                } else {
                    let ch = rest.chars().next().expect("i is inside source");
                    output.push(ch);
                    i += ch.len_utf8();
                }
            }
        }
    }

    output
}

/// If `start` is the beginning of a supported filter method call with the sole
/// argument string `"@<finite-nonnegative-number>"`, return the quote span and
/// numeric value. This deliberately does not attempt to parse general JavaScript.
fn standalone_weighted_filter_argument_at(
    source: &str,
    start: usize,
) -> Option<(usize, usize, String)> {
    const FILTER_CUTOFF_METHODS: &[&str] = &[
        "lpf", "hpf", "bpf", "cutoff", "ctf", "lp", "hp", "hcutoff", "bp", "bandf",
    ];

    if !source.get(start..)?.starts_with('.') {
        return None;
    }

    let mut method_end = start + 1;
    while method_end < source.len() {
        let ch = source[method_end..].chars().next()?;
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            method_end += ch.len_utf8();
        } else {
            break;
        }
    }
    if method_end == start + 1 {
        return None;
    }

    let method = &source[start + 1..method_end];
    if !FILTER_CUTOFF_METHODS.contains(&method) {
        return None;
    }

    let mut cursor = method_end;
    skip_ascii_whitespace(source, &mut cursor);
    if source.as_bytes().get(cursor) != Some(&b'(') {
        return None;
    }
    cursor += 1;
    skip_ascii_whitespace(source, &mut cursor);

    let quote = *source.as_bytes().get(cursor)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let quote_start = cursor;
    let content_start = cursor + 1;
    let relative_end = source[content_start..].find(quote as char)?;
    let quote_end = content_start + relative_end;
    let inner = source[content_start..quote_end].trim();
    let numeric = inner.strip_prefix('@')?.trim();
    if numeric.is_empty() {
        return None;
    }
    let value = numeric.parse::<f64>().ok()?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }

    cursor = quote_end + 1;
    skip_ascii_whitespace(source, &mut cursor);
    if source.as_bytes().get(cursor) != Some(&b')') {
        return None;
    }

    Some((quote_start, quote_end, value.to_string()))
}

fn skip_ascii_whitespace(source: &str, cursor: &mut usize) {
    while *cursor < source.len() {
        let Some(ch) = source[*cursor..].chars().next() else {
            break;
        };
        if !ch.is_ascii_whitespace() {
            break;
        }
        *cursor += ch.len_utf8();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceScanState {
    Code,
    SingleQuoted,
    DoubleQuoted,
    Template,
    LineComment,
    BlockComment,
}

/// Detect the same malformed expression without changing the source. Direct
/// callers of validation receive an actionable diagnostic; the main compile
/// path gets the narrow repair before validation.
fn reject_standalone_weighted_filter_values(source: &str) -> Result<(), String> {
    if heal_standalone_weighted_filter_values(source) != source {
        return Err(
            "Strudel preflight failed: a filter cutoff contains a standalone '@number' string (for example `.lpf(\"@200\")`). The '@' symbol is a mini-notation weight, not a cutoff prefix. Use `.lpf(200)` for a fixed cutoff or `.lpf(\"<200 400 800>\")` for event-stepped cutoff changes."
                .to_string(),
        );
    }
    Ok(())
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

/// Counts top-level tempo calls outside strings/comments and brace-delimited
/// function/block bodies. Any existing call prevents duplicate default insertion.
fn count_top_level_tempo_calls(code: &str) -> Result<usize, String> {
    let masked = mask_comments_and_strings(code)?;
    let chars: Vec<char> = masked.chars().collect();
    let mut count = 0usize;
    let mut brace_depth = 0i32;
    let mut i = 0usize;

    while i < chars.len() {
        if brace_depth == 0 {
            let matched = ["setcps", "setcpm"].iter().find_map(|name| {
                let target: Vec<char> = name.chars().collect();
                if i + target.len() > chars.len() || chars[i..i + target.len()] != target[..] {
                    return None;
                }
                if i > 0 && is_identifier_char(chars[i - 1]) {
                    return None;
                }
                let after_name = i + target.len();
                if after_name < chars.len() && is_identifier_char(chars[after_name]) {
                    return None;
                }
                let mut open = after_name;
                while open < chars.len() && chars[open].is_whitespace() {
                    open += 1;
                }
                (open < chars.len() && chars[open] == '(').then_some(open)
            });
            if matched.is_some() {
                count = count.saturating_add(1);
            }
        }

        match chars[i] {
            '{' => brace_depth += 1,
            '}' => brace_depth = (brace_depth - 1).max(0),
            _ => {}
        }
        i += 1;
    }

    Ok(count)
}

fn starts_with_call(source: &str, name: &str) -> bool {
    let Some(rest) = source.strip_prefix(name) else {
        return false;
    };
    if rest.chars().next().map(is_identifier_char).unwrap_or(false) {
        return false;
    }
    rest.trim_start().starts_with('(')
}

/// Preserve leading metadata/comments, including Strudel's `// @title` format,
/// and insert the host default tempo after that header rather than before it.
fn insert_default_tempo_after_leading_comments(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut prefix_end = 0usize;
    let mut in_block_comment = false;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if in_block_comment {
            prefix_end = index + 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            prefix_end = index + 1;
            continue;
        }
        if trimmed.starts_with("/*") {
            if let Some(close) = trimmed.find("*/") {
                // If executable code follows the closing marker on this same
                // line, leave the whole line in the remainder rather than lose it.
                if !trimmed[close + 2..].trim().is_empty() {
                    break;
                }
                prefix_end = index + 1;
            } else {
                prefix_end = index + 1;
                in_block_comment = true;
            }
            continue;
        }
        break;
    }

    let prefix = lines[..prefix_end].join("\n");
    let remainder = lines[prefix_end..].join("\n");
    match (prefix.is_empty(), remainder.is_empty()) {
        (true, true) => DEFAULT_CPS.to_string(),
        (true, false) => format!("{DEFAULT_CPS}\n{remainder}"),
        (false, true) => format!("{prefix}\n{DEFAULT_CPS}"),
        (false, false) => format!("{prefix}\n{DEFAULT_CPS}\n{remainder}"),
    }
}

/// Reject a well-known malformed JavaScript shape rather than silently converting
/// it into mini-notation. A space-separated numeric list belongs inside a quoted
/// Strudel pattern string; a JavaScript array requires commas.
fn reject_unquoted_numeric_notation_candidates(masked: &str) -> Result<(), String> {
    let chars: Vec<char> = masked.chars().collect();
    let mut line = 1usize;
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if chars[i] != '[' {
            i += 1;
            continue;
        }

        let start = i + 1;
        let mut close = start;
        while close < chars.len() && chars[close] != ']' && chars[close] != '[' {
            close += 1;
        }
        if close >= chars.len() || chars[close] != ']' {
            i += 1;
            continue;
        }

        let inside: String = chars[start..close].iter().collect();
        let trimmed = inside.trim();
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let looks_like_space_array = !trimmed.is_empty()
            && !trimmed.contains(',')
            && tokens.len() >= 2
            && tokens.iter().all(|token| token.parse::<f64>().is_ok());

        let mut after = close + 1;
        while after < chars.len() && chars[after].is_whitespace() && chars[after] != '\n' {
            after += 1;
        }
        if looks_like_space_array && after < chars.len() && matches!(chars[after], '*' | '/') {
            return Err(format!(
                "Strudel preflight failed on line {line}: found an unquoted space-separated numeric array followed by '{}' (for example `[0 0.01]*8`). If this is intended as mini-notation, quote the pattern; do not rely on automatic rewriting.",
                chars[after]
            ));
        }
        i = close + 1;
    }

    Ok(())
}

fn validate_delimiters(masked_code: &str) -> Result<(), String> {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut line = 1usize;

    for ch in masked_code.chars() {
        match ch {
            '(' | '[' | '{' => stack.push((ch, line)),
            ')' | ']' | '}' => {
                let expected_open = match ch {
                    ')' => '(',
                    ']' => '[',
                    '}' => '{',
                    _ => unreachable!(),
                };
                match stack.pop() {
                    Some((open, _)) if open == expected_open => {}
                    Some((open, open_line)) => {
                        return Err(format!(
                            "Strudel preflight failed: mismatched '{ch}' on line {line}; '{open}' was opened on line {open_line}."
                        ));
                    }
                    None => {
                        return Err(format!(
                            "Strudel preflight failed: unexpected closing '{ch}' on line {line}."
                        ));
                    }
                }
            }
            '\n' => line += 1,
            _ => {}
        }
    }

    if let Some((open, open_line)) = stack.last() {
        return Err(format!(
            "Strudel preflight failed: unclosed '{open}' opened on line {open_line}."
        ));
    }
    Ok(())
}

/// Checks bracket-like mini-notation delimiters inside string literals. This is
/// intentionally lexical: arbitrary operators/API semantics remain the runtime's job.
fn validate_mini_notation_delimiters(source: &str) -> Result<(), String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Code,
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }

    let chars: Vec<char> = source.chars().collect();
    let mut state = State::Code;
    let mut escaped = false;
    let mut line = 1usize;
    let mut literal_start_line = 1usize;
    let mut literal = String::new();
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        match state {
            State::Code => {
                if ch == '/' && next == Some('/') {
                    state = State::LineComment;
                    i += 2;
                    continue;
                }
                if ch == '/' && next == Some('*') {
                    state = State::BlockComment;
                    i += 2;
                    continue;
                }
                match ch {
                    '\'' => {
                        state = State::Single;
                        literal_start_line = line;
                        literal.clear();
                    }
                    '"' => {
                        state = State::Double;
                        literal_start_line = line;
                        literal.clear();
                    }
                    '`' => {
                        state = State::Template;
                        literal_start_line = line;
                        literal.clear();
                    }
                    '\n' => line += 1,
                    _ => {}
                }
            }
            State::Single | State::Double | State::Template => {
                let quote = match state {
                    State::Single => '\'',
                    State::Double => '"',
                    State::Template => '`',
                    _ => unreachable!(),
                };
                if ch == '\n' {
                    line += 1;
                }
                if escaped {
                    literal.push(ch);
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == quote {
                    validate_literal_delimiters(&literal, literal_start_line)?;
                    state = State::Code;
                } else {
                    literal.push(ch);
                }
            }
            State::LineComment => {
                if ch == '\n' {
                    line += 1;
                    state = State::Code;
                }
            }
            State::BlockComment => {
                if ch == '*' && next == Some('/') {
                    state = State::Code;
                    i += 2;
                    continue;
                }
                if ch == '\n' {
                    line += 1;
                }
            }
        }
        i += 1;
    }
    Ok(())
}

fn validate_literal_delimiters(literal: &str, start_line: usize) -> Result<(), String> {
    // Avoid interpreting ordinary strings as mini-notation unless they contain
    // one of its grouping/alternation delimiters.
    if !literal
        .chars()
        .any(|ch| matches!(ch, '[' | ']' | '{' | '}' | '<' | '>'))
    {
        return Ok(());
    }

    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut line = start_line;
    for ch in literal.chars() {
        match ch {
            '[' | '{' | '<' => stack.push((ch, line)),
            ']' | '}' | '>' => {
                let expected_open = match ch {
                    ']' => '[',
                    '}' => '{',
                    '>' => '<',
                    _ => unreachable!(),
                };
                match stack.pop() {
                    Some((open, _)) if open == expected_open => {}
                    Some((open, open_line)) => {
                        return Err(format!(
                            "Strudel preflight failed: mismatched mini-notation delimiter '{ch}' near line {line}; '{open}' opened near line {open_line}."
                        ));
                    }
                    None => {
                        return Err(format!(
                            "Strudel preflight failed: unexpected mini-notation delimiter '{ch}' near line {line}."
                        ));
                    }
                }
            }
            '\n' => line += 1,
            _ => {}
        }
    }
    if let Some((open, open_line)) = stack.last() {
        return Err(format!(
            "Strudel preflight failed: unclosed mini-notation delimiter '{open}' near line {open_line}."
        ));
    }
    Ok(())
}

/// Confirms the final top-level statement is an expression, not just a series of
/// declarations. This is a conservative line/statement heuristic, not a parser.
fn has_final_root_expression(masked_code: &str) -> bool {
    use impl_statement_kind::Kind;

    let chars: Vec<char> = masked_code.chars().collect();
    let mut last_kind: Option<Kind> = None;
    let mut segment = String::new();
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        if ch == ';' && paren == 0 && bracket == 0 && brace == 0 {
            update_statement_kind(&segment, &mut last_kind);
            segment.clear();
            i += 1;
            continue;
        }
        if ch == '\n' && paren == 0 && bracket == 0 && brace == 0 {
            update_statement_kind(&segment, &mut last_kind);
            segment.clear();
            i += 1;
            continue;
        }

        match ch {
            '(' => paren += 1,
            ')' => paren = (paren - 1).max(0),
            '[' => bracket += 1,
            ']' => bracket = (bracket - 1).max(0),
            '{' => brace += 1,
            '}' => brace = (brace - 1).max(0),
            _ => {}
        }
        segment.push(ch);
        i += 1;
    }
    update_statement_kind(&segment, &mut last_kind);
    last_kind == Some(Kind::Root)
}

// A private namespace lets the statement classifier keep a tiny internal enum
// without exposing it in this module's public interface.
mod impl_statement_kind {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Declaration,
        Tempo,
        Root,
    }
}

fn update_statement_kind(segment: &str, last_kind: &mut Option<impl_statement_kind::Kind>) {
    use impl_statement_kind::Kind;
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return;
    }

    // A leading dot on a new line is a continuation of the previous chain. It
    // does not turn a preceding declaration into an evaluated root expression.
    if trimmed.starts_with('.') {
        return;
    }
    if is_variable_declaration(trimmed)
        || trimmed.starts_with("function ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("import ")
        || trimmed.starts_with("export const ")
        || trimmed.starts_with("export let ")
        || trimmed.starts_with("export var ")
    {
        *last_kind = Some(Kind::Declaration);
        return;
    }
    if starts_with_call(trimmed, "setcps") || starts_with_call(trimmed, "setcpm") {
        *last_kind = Some(Kind::Tempo);
        return;
    }

    let has_call = trimmed.contains('(');
    let has_member_or_operator = trimmed.contains('.') || trimmed.contains('=');
    let is_bare_identifier = trimmed
        .trim_end_matches(';')
        .split_whitespace()
        .next()
        .map(|first| {
            trimmed.trim_end_matches(';') == first
                && first.chars().next().map(is_identifier_start).unwrap_or(false)
                && first.chars().all(is_identifier_char)
        })
        .unwrap_or(false);

    if has_call || has_member_or_operator || is_bare_identifier {
        *last_kind = Some(Kind::Root);
    } else {
        // Leave a previous kind intact only for ordinary blank/comment pieces
        // (which have already been filtered). An unrecognized statement cannot
        // establish a valid final root.
        *last_kind = None;
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn contains_trigger_call(masked_code: &str) -> bool {
    ["s", "sound", "note", "n", "chord"]
        .iter()
        .any(|name| contains_identifier_call(masked_code, name))
}

/// Masks comments and string contents while preserving newlines and enough source
/// structure to validate delimiters. Unterminated strings/block comments fail.
fn mask_comments_and_strings(source: &str) -> Result<String, String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Code,
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }

    let chars: Vec<char> = source.chars().collect();
    let mut output = String::with_capacity(source.len());
    let mut state = State::Code;
    let mut escaped = false;
    let mut i = 0usize;
    let mut line = 1usize;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();
        match state {
            State::Code => {
                if ch == '/' && next == Some('/') {
                    output.push(' ');
                    output.push(' ');
                    state = State::LineComment;
                    i += 2;
                    continue;
                }
                if ch == '/' && next == Some('*') {
                    output.push(' ');
                    output.push(' ');
                    state = State::BlockComment;
                    i += 2;
                    continue;
                }
                match ch {
                    '\'' => {
                        output.push(' ');
                        state = State::Single;
                    }
                    '"' => {
                        output.push(' ');
                        state = State::Double;
                    }
                    '`' => {
                        output.push(' ');
                        state = State::Template;
                    }
                    _ => output.push(ch),
                }
            }
            State::Single | State::Double | State::Template => {
                let quote = match state {
                    State::Single => '\'',
                    State::Double => '"',
                    State::Template => '`',
                    _ => unreachable!(),
                };
                if ch == '\n' {
                    line += 1;
                    output.push('\n');
                    if state != State::Template && !escaped {
                        return Err(format!(
                            "Strudel preflight failed: unterminated string before line {line}."
                        ));
                    }
                } else {
                    output.push(' ');
                }
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == quote {
                    state = State::Code;
                }
            }
            State::LineComment => {
                if ch == '\n' {
                    output.push('\n');
                    line += 1;
                    state = State::Code;
                } else {
                    output.push(' ');
                }
            }
            State::BlockComment => {
                if ch == '*' && next == Some('/') {
                    output.push(' ');
                    output.push(' ');
                    state = State::Code;
                    i += 2;
                    continue;
                }
                if ch == '\n' {
                    output.push('\n');
                    line += 1;
                } else {
                    output.push(' ');
                }
            }
        }
        i += 1;
    }

    match state {
        State::Single | State::Double | State::Template => {
            return Err(format!(
                "Strudel preflight failed: unterminated string at end of input near line {line}."
            ));
        }
        State::BlockComment => {
            return Err("Strudel preflight failed: unterminated block comment.".to_string());
        }
        State::Code | State::LineComment => {}
    }
    Ok(output)
}

/// Scans documented pattern constructors and captures the first quoted argument.
/// Results are descriptive signatures, not proof the sound or pattern is available.
fn extract_trigger_strings(code: &str) -> Vec<String> {
    let chars: Vec<char> = code.chars().collect();
    let mut output = Vec::new();
    let mut i = 0usize;
    let mut state = LexState::Code;
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();
        match state {
            LexState::Code => {
                if ch == '/' && next == Some('/') {
                    state = LexState::LineComment;
                    i += 2;
                    continue;
                }
                if ch == '/' && next == Some('*') {
                    state = LexState::BlockComment;
                    i += 2;
                    continue;
                }
                if ch == '\'' || ch == '"' || ch == '`' {
                    state = match ch {
                        '\'' => LexState::Single,
                        '"' => LexState::Double,
                        _ => LexState::Template,
                    };
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                }
                if is_identifier_start(ch) {
                    let start = i;
                    i += 1;
                    while i < chars.len() && is_identifier_char(chars[i]) {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    if matches!(name.as_str(), "s" | "sound" | "note" | "n") {
                        let mut open = i;
                        while open < chars.len() && chars[open].is_whitespace() {
                            open += 1;
                        }
                        if open < chars.len() && chars[open] == '(' {
                            let mut arg = open + 1;
                            while arg < chars.len() && chars[arg].is_whitespace() {
                                arg += 1;
                            }
                            if arg < chars.len() && matches!(chars[arg], '\'' | '"' | '`') {
                                let quote = chars[arg];
                                let mut end = arg + 1;
                                let mut local_escaped = false;
                                let mut value = String::new();
                                while end < chars.len() {
                                    let current = chars[end];
                                    if local_escaped {
                                        value.push(current);
                                        local_escaped = false;
                                    } else if current == '\\' {
                                        local_escaped = true;
                                    } else if current == quote {
                                        break;
                                    } else {
                                        value.push(current);
                                    }
                                    end += 1;
                                }
                                let value = value.trim().to_lowercase();
                                if !value.is_empty()
                                    && !matches!(
                                        value.as_str(),
                                        "sawtooth"
                                            | "supersaw"
                                            | "square"
                                            | "triangle"
                                            | "sine"
                                            | "pulse"
                                    )
                                {
                                    output.push(value);
                                }
                            }
                        }
                    }
                    continue;
                }
                i += 1;
            }
            LexState::Single | LexState::Double | LexState::Template => {
                let quote = match state {
                    LexState::Single => '\'',
                    LexState::Double => '"',
                    LexState::Template => '`',
                    _ => unreachable!(),
                };
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == quote {
                    state = LexState::Code;
                }
                i += 1;
            }
            LexState::LineComment => {
                if ch == '\n' {
                    state = LexState::Code;
                }
                i += 1;
            }
            LexState::BlockComment => {
                if ch == '*' && next == Some('/') {
                    state = LexState::Code;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }
    }
    unique_preserving_order(&output)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LexState {
    Code,
    Single,
    Double,
    Template,
    LineComment,
    BlockComment,
}

/// Rejects a high-confidence silent-output hazard: treating a standard Strudel
/// `slider(...)` control as a plain JavaScript number. In the REPL, slider calls
/// are transpiled into pattern-valued numeric controls, so JavaScript operators
/// such as `cutoff * 0.8` or `intensity > 0.7 ? ...` can escape Strudel's pattern
/// algebra and produce invalid parameter values. Returning an error causes the
/// existing generation loop to retry with specific feedback instead of accepting
/// a structurally balanced but potentially silent score.
fn reject_javascript_slider_arithmetic(masked_code: &str) -> Result<(), String> {
    for name in extract_slider_names(masked_code) {
        if identifier_has_arithmetic_or_comparison_use(masked_code, &name) {
            return Err(format!(
                "Strudel preflight failed: slider '{name}' is used with ordinary JavaScript arithmetic or comparison. In the Strudel REPL, slider(...) is transpiled to a Pattern-valued numeric control; expressions such as '{name} * 0.8' or '{name} > 0.7 ? ...' can break the parameter signal and may silence the composition. Pass the slider directly to a parameter, or use Strudel pattern methods such as .mul(...), .add(...), or .range(...) where their input range fits the intended units. Replace threshold-based JavaScript ternaries with a supported pattern-aware design, or simplify to a directly controlled parameter."
            ));
        }
    }
    Ok(())
}

/// Rejects invented `root(...)` wrappers so the model evaluates its final
/// Pattern expression (`stack(...)`, `arrange(...)`, or a pattern variable) directly.
fn reject_invented_root_wrapper(masked_code: &str) -> Result<(), String> {
    for line in masked_code.lines() {
        let trimmed = line.trim();
        if starts_with_call(trimmed, "root") {
            return Err(
                "Strudel preflight failed: found an invented top-level `root(...)` wrapper. Do not wrap the final Pattern in `root(...)`; end the score directly with `stack(...)`, `arrange(...)`, `cat(...)`, or the final Pattern variable name."
                    .to_string(),
            );
        }
    }
    Ok(())
}

/// Rejects declared `slider(...)` controls and named Strudel pattern stems that
/// are never referenced anywhere else in the program. This prevents orphaned
/// solos, silent sub-bass layers, and dead UI sliders from passing preflight.
fn reject_unused_sliders_and_pattern_stems(masked_code: &str) -> Result<(), String> {
    for name in extract_slider_names(masked_code) {
        if identifier_occurrences(masked_code, &name) <= 1 {
            return Err(format!(
                "Strudel preflight failed: slider '{name}' is declared but never wired to an audible parameter. Reference '{name}' in a parameter such as `.lpf({name})` or `.gain({name})`, or remove the unused slider declaration."
            ));
        }
    }

    const PATTERN_CONSTRUCTORS: &[&str] = &[
        "s", "sound", "note", "n", "chord", "stack", "arrange", "cat", "slowcat", "fastcat",
        "seq", "stepcat", "polymeter",
    ];

    for line in masked_code.lines() {
        let trimmed = line.trim();
        if !is_variable_declaration(trimmed) {
            continue;
        }
        let Some((left, right)) = trimmed.split_once('=') else {
            continue;
        };
        let name = declared_name(left);
        if name.is_empty() || name.starts_with('_') {
            continue;
        }
        // Ignore arrow/function definitions and plain non-pattern constants.
        if right.contains("=>") || right.trim_start().starts_with("function") {
            continue;
        }
        if contains_any_call(right, PATTERN_CONSTRUCTORS)
            && identifier_occurrences(masked_code, name) <= 1
        {
            return Err(format!(
                "Strudel preflight failed: pattern stem '{name}' is declared but never referenced in the final Pattern or section stacks. Include '{name}' in your arrangement (`stack(...)` / `arrange(...)`) or remove the unused variable."
            ));
        }
    }

    Ok(())
}

fn extract_slider_names(masked_code: &str) -> Vec<String> {
    let mut output = Vec::new();
    for line in masked_code.lines() {
        let trimmed = line.trim();
        if !is_variable_declaration(trimmed) {
            continue;
        }
        let Some((left, right)) = trimmed.split_once('=') else {
            continue;
        };
        if contains_identifier_call(right, "slider") {
            let name = declared_name(left);
            if !name.is_empty() {
                output.push(name.to_string());
            }
        }
    }
    unique_preserving_order(&output)
}

fn extract_visualizer_names(masked_code: &str) -> Vec<String> {
    let names = [
        "punchcard",
        "pianoroll",
        "scope",
        "tscope",
        "spectrum",
        "spectrogram",
        "spiral",
        "pitchwheel",
    ];
    let mut output = Vec::new();
    for name in names {
        let inline_name = format!("_{name}");
        if contains_identifier_call(masked_code, name)
            || contains_identifier_call(masked_code, &inline_name)
        {
            output.push(name.to_string());
        }
    }
    output
}

fn identifier_occurrences(source: &str, identifier: &str) -> usize {
    if identifier.is_empty() {
        return 0;
    }
    let chars: Vec<char> = source.chars().collect();
    let target: Vec<char> = identifier.chars().collect();
    let mut count = 0usize;
    let mut i = 0usize;
    while i + target.len() <= chars.len() {
        if chars[i..i + target.len()] == target[..]
            && (i == 0 || !is_identifier_char(chars[i - 1]))
            && (i + target.len() == chars.len() || !is_identifier_char(chars[i + target.len()]))
        {
            count += 1;
            i += target.len();
        } else {
            i += 1;
        }
    }
    count
}

fn identifier_has_arithmetic_or_comparison_use(source: &str, identifier: &str) -> bool {
    let chars: Vec<char> = source.chars().collect();
    let target: Vec<char> = identifier.chars().collect();
    let mut i = 0usize;
    while i + target.len() <= chars.len() {
        if chars[i..i + target.len()] == target[..]
            && (i == 0 || !is_identifier_char(chars[i - 1]))
            && (i + target.len() == chars.len() || !is_identifier_char(chars[i + target.len()]))
        {
            let mut before = i;
            while before > 0 && chars[before - 1].is_whitespace() {
                before -= 1;
            }
            let mut after = i + target.len();
            while after < chars.len() && chars[after].is_whitespace() {
                after += 1;
            }
            let operators = ['+', '-', '*', '/', '%', '>', '<'];
            if (before > 0 && operators.contains(&chars[before - 1]))
                || (after < chars.len() && operators.contains(&chars[after]))
            {
                return true;
            }
            i += target.len();
        } else {
            i += 1;
        }
    }
    false
}

fn count_arranged_cycles(masked_code: &str) -> usize {
    let chars: Vec<char> = masked_code.chars().collect();
    let Some(start) = find_identifier_call_start(&chars, "arrange") else {
        return 0;
    };
    let mut i = start + 1; // opening parenthesis
    let mut paren_depth = 1i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut total = 0usize;

    while i < chars.len() && paren_depth > 0 {
        let ch = chars[i];
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' if paren_depth == 1 && bracket_depth == 0 && brace_depth == 0 => {
                let mut number_start = i + 1;
                while number_start < chars.len() && chars[number_start].is_whitespace() {
                    number_start += 1;
                }
                let mut number_end = number_start;
                while number_end < chars.len() && chars[number_end].is_ascii_digit() {
                    number_end += 1;
                }
                if number_end > number_start {
                    let mut comma = number_end;
                    while comma < chars.len() && chars[comma].is_whitespace() {
                        comma += 1;
                    }
                    if comma < chars.len() && chars[comma] == ',' {
                        let number: String = chars[number_start..number_end].iter().collect();
                        if let Ok(cycles) = number.parse::<usize>() {
                            total = total.saturating_add(cycles);
                        }
                    }
                }
                bracket_depth += 1;
            }
            '[' => bracket_depth += 1,
            ']' => bracket_depth = (bracket_depth - 1).max(0),
            '{' => brace_depth += 1,
            '}' => brace_depth = (brace_depth - 1).max(0),
            _ => {}
        }
        i += 1;
    }
    total
}

fn find_identifier_call_start(chars: &[char], name: &str) -> Option<usize> {
    let target: Vec<char> = name.chars().collect();
    let mut i = 0usize;
    while i + target.len() <= chars.len() {
        if chars[i..i + target.len()] == target[..]
            && (i == 0 || !is_identifier_char(chars[i - 1]))
            && (i + target.len() == chars.len() || !is_identifier_char(chars[i + target.len()]))
        {
            let mut open = i + target.len();
            while open < chars.len() && chars[open].is_whitespace() {
                open += 1;
            }
            if open < chars.len() && chars[open] == '(' {
                return Some(open);
            }
        }
        i += 1;
    }
    None
}

fn unique_preserving_order(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
            output.push(trimmed.to_string());
        }
    }
    output
}

/// Rejects only an exact unchanged score during evolution. Recurring motifs and
/// intentional repetition remain legitimate compositional techniques.
pub fn check_clone_or_stagnation(new_code: &str, previous_code: Option<&str>) -> Option<String> {
    if let Some(previous) = previous_code {
        if new_code.trim() == previous.trim() {
            return Some(
                "EXACT EVOLUTION CLONE: The generated score is unchanged from the previous version. Make a purposeful change, or keep the original if no change is needed."
                    .to_string(),
            );
        }
    }
    None
}

/// Removes model reasoning blocks and extracts a fenced candidate when available.
/// For unfenced output, leading prose is discarded and common trailing prose is
/// removed. Leading metadata comments are retained. Source syntax is not rewritten.
fn strip_thinking_and_extract_code(raw: &str) -> String {
    let mut text = raw.to_string();
    while let Some(start) = text.find("<think>") {
        if let Some(relative_end) = text[start..].find("</think>") {
            let end = start + relative_end + "</think>".len();
            text.replace_range(start..end, "");
        } else {
            text.replace_range(start.., "");
            break;
        }
    }

    if text.contains("```") {
        if let Some(block) = first_code_fence(&text) {
            return block.trim().to_string();
        }
    }

    let lines: Vec<&str> = text.lines().collect();
    let first_code_line = lines
        .iter()
        .position(|line| is_valid_code_start(line.trim()));
    let Some(start_index) = first_code_line else {
        return String::new();
    };

    let candidate = lines[start_index..].join("\n");
    let masked = mask_comments_and_strings(&candidate).unwrap_or_else(|_| candidate.clone());
    let source_lines: Vec<&str> = candidate.lines().collect();
    let masked_lines: Vec<&str> = masked.lines().collect();

    let mut output = Vec::new();
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut seen_root = false;

    for (index, line) in source_lines.iter().enumerate() {
        let trimmed = line.trim();
        let masked_line = masked_lines.get(index).copied().unwrap_or("");
        if is_trailing_prose(trimmed, paren_depth, bracket_depth, brace_depth, seen_root) {
            break;
        }

        let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
        if at_top_level && starts_root_expression(trimmed) {
            seen_root = true;
        }
        update_delimiter_depths(
            masked_line,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
        output.push((*line).to_string());
    }
    output.join("\n").trim().to_string()
}

fn first_code_fence(text: &str) -> Option<String> {
    let mut cursor = 0usize;
    while let Some(relative_start) = text[cursor..].find("```") {
        let fence_start = cursor + relative_start;
        let after_fence = fence_start + 3;
        let line_end = text[after_fence..].find('\n').map(|n| after_fence + n);
        let Some(line_end) = line_end else {
            return None;
        };
        let body_start = line_end + 1;
        let Some(relative_end) = text[body_start..].find("```") else {
            return None;
        };
        let body_end = body_start + relative_end;
        let body = &text[body_start..body_end];
        if looks_like_strudel_code(body) {
            return Some(body.to_string());
        }
        cursor = body_end + 3;
        if cursor >= text.len() {
            break;
        }
    }
    None
}

fn looks_like_strudel_code(candidate: &str) -> bool {
    [
        "setcps",
        "setcpm",
        "stack(",
        "arrange(",
        "note(",
        "sound(",
        "s(",
        "n(",
        "chord(",
        "const ",
        "let ",
        "var ",
    ]
    .iter()
    .any(|marker| candidate.contains(marker))
}

fn is_valid_code_start(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with("setcps")
        || trimmed.starts_with("setcpm")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var ")
        || trimmed.starts_with("stack(")
        || trimmed.starts_with("arrange(")
        || trimmed.starts_with("cat(")
        || trimmed.starts_with("slowcat(")
        || trimmed.starts_with("seq(")
        || trimmed.starts_with("stepcat(")
        || trimmed.starts_with("polymeter(")
        || trimmed.starts_with("s(")
        || trimmed.starts_with("sound(")
        || trimmed.starts_with("note(")
        || trimmed.starts_with("n(")
        || trimmed.starts_with("chord(")
}

fn is_trailing_prose(
    trimmed: &str,
    paren_depth: i32,
    bracket_depth: i32,
    brace_depth: i32,
    seen_root: bool,
) -> bool {
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
        return false;
    }
    let at_top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
    if at_top_level
        && (trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("###")
            || trimmed.starts_with("The ")
            || trimmed.starts_with("This ")
            || trimmed.starts_with("Here ")
            || trimmed.starts_with("In this ")
            || trimmed.starts_with("Note:")
            || trimmed.starts_with("Explanation:")
            || trimmed.starts_with("Generated track:")
            || trimmed.starts_with("This track"))
    {
        return true;
    }

    if seen_root && at_top_level && !looks_like_code_continuation(trimmed) {
        return true;
    }
    false
}

fn starts_root_expression(trimmed: &str) -> bool {
    if trimmed.starts_with('.') || trimmed.starts_with("//") || trimmed.starts_with("/*") {
        return false;
    }
    if is_variable_declaration(trimmed)
        || starts_with_call(trimmed, "setcps")
        || starts_with_call(trimmed, "setcpm")
    {
        return false;
    }
    looks_like_code_continuation(trimmed)
}

fn looks_like_code_continuation(trimmed: &str) -> bool {
    if trimmed.starts_with('.')
        || trimmed.starts_with(')')
        || trimmed.starts_with(']')
        || trimmed.starts_with('}')
        || is_variable_declaration(trimmed)
        || starts_with_call(trimmed, "setcps")
        || starts_with_call(trimmed, "setcpm")
        || trimmed.starts_with("export ")
    {
        return true;
    }
    let first = trimmed.split_whitespace().next().unwrap_or_default();
    let identifier = first
        .trim_start_matches('(')
        .trim_end_matches(|ch: char| matches!(ch, '(' | ')' | ';' | ','));
    if identifier.is_empty()
        || !identifier
            .chars()
            .next()
            .map(is_identifier_start)
            .unwrap_or(false)
    {
        return false;
    }
    trimmed.contains('(') || trimmed.contains('.') || trimmed.trim_end_matches(';') == identifier
}

fn update_delimiter_depths(
    masked_line: &str,
    paren_depth: &mut i32,
    bracket_depth: &mut i32,
    brace_depth: &mut i32,
) {
    for ch in masked_line.chars() {
        match ch {
            '(' => *paren_depth += 1,
            ')' => *paren_depth = (*paren_depth - 1).max(0),
            '[' => *bracket_depth += 1,
            ']' => *bracket_depth = (*bracket_depth - 1).max(0),
            '{' => *brace_depth += 1,
            '}' => *brace_depth = (*brace_depth - 1).max(0),
            _ => {}
        }
    }
}

#[cfg(test)]
mod autonomy_regression_tests {
    use super::*;

    #[test]
    fn recurring_motif_across_sections_is_preserved() {
        let code = "setcps(0.5)\nconst intro = stack(note(\"[c4 eb4 g4]\"))\nconst drop = stack(note(\"[c4 eb4 g4]\").gain(0.8))\narrange([4, intro], [4, drop])";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn valid_chop_call_is_not_rewritten() {
        let code = "setcps(0.5)\nstack(s(\"bd\").chop(8), note(\"c4 eb4 g4\"))";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn single_quoted_sound_alias_is_accepted_and_preserved() {
        let code = "setcps(0.5)\nsound('bd')";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn existing_tempo_is_preserved_without_guessing_units() {
        let code = "setcpm(30)\ns(\"bd\")";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn method_not_in_a_static_whitelist_is_preserved() {
        let code = "setcps(0.5)\ns(\"bd\").fit()";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn deliberately_chromatic_notes_are_preserved() {
        let code = "setcps(0.5)\nnote(\"c4 db4 d4 eb4 e4 g4\")";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn unbalanced_code_returns_an_error_instead_of_inventing_closers() {
        let code = "setcps(0.5)\nstack(s(\"bd\")";
        assert!(compile_and_heal_strudel(code).is_err());
    }

    #[test]
    fn a_specific_progression_is_not_globally_banned() {
        let code = "setcps(0.5)\nconst bass = note(\"<[c1 c2]*4 [ab0 ab1]*4 [f0 f1]*4 [g0 g1]*4>\")\nconst lead = note(\"c5 [eb5 g5] bb5 g5 eb5 [d5 c5] b4 <g4 d5>\")\nstack(bass, lead)";
        assert_eq!(check_clone_or_stagnation(code, None), None);
    }

    #[test]
    fn identical_evolution_is_still_rejected() {
        let code = "setcps(0.5)\nstack(note(\"[c4 eb4 g4]\"))";
        assert!(check_clone_or_stagnation(code, Some(code)).is_some());
    }

    #[test]
    fn preflight_does_not_count_trigger_text_inside_a_comment() {
        let result = validate_strudel_ast("setcps(0.5)\n// s(\"bd\")");
        assert!(result.is_err());
    }

    #[test]
    fn metadata_comments_survive_default_tempo_insertion() {
        let code = "// @title Night Circuit\n// @by MotifAI\nstack(s(\"bd\"), note(\"c3 e3\"))";
        let result = compile_and_heal_strudel(code).unwrap();
        assert!(result.starts_with("// @title Night Circuit\n// @by MotifAI\nsetcps(0.55)\n"));
    }

    #[test]
    fn block_metadata_survives_extraction() {
        let code =
            "/*\n@title Night Circuit\n@by MotifAI\n*/\nsetcps(0.5)\nstack(s(\"bd\"), note(\"c3 e3\"))";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn missing_root_expression_is_rejected() {
        let code = "setcps(0.5)\nconst bass = note(\"c2 e2\").s(\"sawtooth\")";
        let result = compile_and_heal_strudel(code);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("final evaluated pattern expression"));
    }

    #[test]
    fn quoted_mini_notation_delimiters_are_checked() {
        assert!(validate_strudel_ast("setcps(0.5)\nnote(\"<[c4 e4]\")").is_err());
        assert!(validate_strudel_ast("setcps(0.5)\nnote(\"<[c4 e4]>\")").is_ok());
    }

    #[test]
    fn unquoted_numeric_array_like_mini_notation_is_rejected_not_rewritten() {
        let code = "setcps(0.5)\ns([0 0.01]*8)";
        let result = compile_and_heal_strudel(code).unwrap_err();
        assert!(result.contains("unquoted space-separated numeric array"));
    }

    #[test]
    fn direct_slider_parameter_is_preserved_and_accepted() {
        let code =
            "setcps(0.5)\nconst level = slider(0.4, 0, 1, 0.05)\nstack(note(\"c3 e3\").gain(level))";
        let result = compile_and_heal_strudel(code).unwrap();
        assert!(result.contains("slider(0.4, 0, 1, 0.05)"));
        let analysis = analyze_ast_health(&result, Vec::new(), true);
        assert_eq!(analysis.slider_names, vec!["level"]);
    }

    #[test]
    fn slider_javascript_arithmetic_is_rejected_before_generation_is_accepted() {
        let code = "setcps(0.5)\nconst intensity = slider(0.5, 0, 1)\nconst leadCutoff = slider(800, 200, 4000)\nconst lead = note(\"c4 eb4 g4\").lpf(leadCutoff * intensity).gain(intensity > 0.7 ? 1 : 0.4)\nlead";
        let error = compile_and_heal_strudel(code).unwrap_err();
        assert!(error.contains("slider 'intensity'") || error.contains("slider 'leadCutoff'"));
        assert!(error.contains("Pattern-valued numeric control"));
        assert!(error.contains(".mul(...)"));
    }

    #[test]
    fn slider_pattern_methods_are_not_mistaken_for_javascript_arithmetic() {
        let code = "setcps(0.5)\nconst cutoff = slider(0.5, 0, 1)\nconst lead = note(\"c4 eb4 g4\").lpf(cutoff.range(200, 4000))\nlead";
        assert!(compile_and_heal_strudel(code).is_ok());
    }

    #[test]
    fn standalone_weight_prefix_in_filter_cutoff_is_repaired_narrowly() {
        let code = "setcps(0.5)\nconst lead = note(\"c4 e4 g4\").lpf(\"@200\")\nlead";
        let result = compile_and_heal_strudel(code).unwrap();
        assert!(result.contains(".lpf(200)"));
        assert!(!result.contains(".lpf(\"@200\")"));
    }

    #[test]
    fn reported_build_error_line_is_repaired_without_changing_other_sections() {
        let code = "setcps(0.5)\nconst leadMelody = note(\"<[e3 fs3 gs3 a3 b3 cs4 ds4]*2>\").s(\"supersaw\")\nconst buildUp = stack(leadMelody.fast(1.5).lpf(\"@200\"))\nconst fullTrack = buildUp\nfullTrack";
        let result = compile_and_heal_strudel(code).unwrap();
        assert!(result.contains("leadMelody.fast(1.5).lpf(200)"));
        assert!(result.contains("const fullTrack = buildUp"));
        assert!(result.ends_with("fullTrack"));
    }

    #[test]
    fn direct_preflight_reports_standalone_weight_prefix_in_filter_cutoff() {
        let code = "setcps(0.5)\nnote(\"c4 e4\").lpf(\"@200\")";
        let error = validate_strudel_ast(code).unwrap_err();
        assert!(error.contains("'@number'"));
        assert!(error.contains(".lpf(200)"));
    }

    #[test]
    fn valid_cutoff_patterns_and_mini_notation_weights_are_preserved() {
        let code = "setcps(0.5)\nconst lead = note(\"c4@2 e4 g4\").lpf(\"<200 400 800>\")\nlead";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn standalone_weight_prefix_in_comments_and_unrelated_strings_is_untouched() {
        let code = "// .lpf(\"@200\") is an example\nsetcps(0.5)\nconst label = \".lpf('@200')\"\nnote(\"c4 e4\")";
        assert_eq!(compile_and_heal_strudel(code).unwrap(), code);
    }

    #[test]
    fn visualizer_calls_are_detected_without_false_hits_from_strings() {
        let code = "setcps(0.5)\nconst notePattern = note(\"c4 e4\")\n// ._spectrum()\nnotePattern._pianoroll()";
        let result = analyze_ast_health(code, Vec::new(), true);
        assert_eq!(result.visualizer_names, vec!["pianoroll"]);
    }

    #[test]
    fn arrange_cycle_count_only_counts_direct_section_pairs() {
        let code = "setcps(0.5)\nconst a = stack(s(\"bd\"))\nconst b = stack(s(\"sd\"))\narrange([4, a], [8, b])\nconst unrelated = [99, 100]";
        let result = analyze_ast_health(code, Vec::new(), true);
        assert_eq!(result.cycle_count, 12);
    }

    #[test]
    fn comments_and_strings_do_not_fake_a_tempo_declaration() {
        let code = "// setcps(0.1)\n/* setcpm(1) */\nstack(s(\"bd\"))";
        let result = compile_and_heal_strudel(code).unwrap();
        assert!(result.contains("setcps(0.55)"));
    }

    #[test]
    fn tempo_inside_a_brace_block_does_not_suppress_default_insertion() {
        let code = "const ignored = () => {\nsetcps(0.2)\n}\nstack(s(\"bd\"))";
        let result = compile_and_heal_strudel(code).unwrap();
        assert!(result.starts_with("setcps(0.55)\n"));
    }

    #[test]
    fn unwired_slider_is_rejected_by_preflight() {
        let code = "setcps(0.5)\nconst deadCutoff = slider(800, 200, 4000)\nconst lead = note(\"c4 eb4 g4\").s(\"sawtooth\")\nlead";
        let err = compile_and_heal_strudel(code).unwrap_err();
        assert!(err.contains("slider 'deadCutoff' is declared but never wired"));
    }

    #[test]
    fn unused_pattern_stem_is_rejected_by_preflight() {
        let code = "setcps(0.5)\nconst drums = s(\"bd sd\")\nconst forgottenSolo = note(\"c5 eb5 g5\").s(\"supersaw\")\nstack(drums)";
        let err = compile_and_heal_strudel(code).unwrap_err();
        assert!(err.contains("pattern stem 'forgottenSolo' is declared but never referenced"));
    }

    #[test]
    fn invented_root_wrapper_is_rejected_by_preflight() {
        let code = "setcps(0.5)\nconst drums = s(\"bd sd\")\nroot(drums)";
        let err = compile_and_heal_strudel(code).unwrap_err();
        assert!(err.contains("invented top-level `root(...)` wrapper"));
    }
}