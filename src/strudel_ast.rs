//! Conservative Strudel source extraction and preflight checks for MotifAI.
//!
//! This module intentionally avoids acting as a co-composer. It does not rewrite
//! pitches, force a scale, replace DSP methods, clamp ordinary parameter values,
//! merge roots, convert `stack()`/`arrange()`, or auto-mix the score. Those actions
//! can change valid musical intent. Only narrow syntax repairs with clear evidence
//! are performed; other structural problems are returned to the generation retry.
//!
//! Important: `validate_strudel_ast` is a legacy API name. This module performs
//! lexical/structural preflight, not a full JavaScript AST parse and not runtime
//! playback verification.

use crate::sonic_intelligence::ExternalAstAnalysis;
use std::collections::HashSet;

/// Extracts a Strudel candidate, applies only narrow source repairs, and checks
/// lexical structure. Valid code is otherwise preserved byte-for-byte as much as
/// possible. No harmony rewrite, method whitelist, or automatic mixing is applied.
pub fn compile_and_heal_strudel(raw: &str) -> Result<String, String> {
    let extracted = strip_thinking_and_extract_code(raw);
    if extracted.trim().is_empty() {
        return Err("Model returned no recognizable Strudel code.".to_string());
    }

    let mut processed_lines = Vec::new();
    for raw_line in extracted.lines() {
        let mut line = raw_line.to_string();
        let trimmed = line.trim();

        // Keep comments and blank lines untouched. They may explain intent and
        // should not be fed through code transformations.
        if trimmed.is_empty() || trimmed.starts_with("//") {
            processed_lines.push(line);
            continue;
        }

        // Narrow repairs for common malformed numeric slider arguments and the
        // specific unquoted numeric mini-notation form `[0 0.01]*8`.
        line = heal_slider_declarations(&line);
        line = heal_unquoted_numeric_mini_arrays(&line);
        processed_lines.push(line);
    }

    let mut code = processed_lines.join("\n");

    // Preserve valid setcps()/setcpm() declarations exactly, including the
    // user's intended unit and value. Add a default only when no declaration is
    // present as a top-level source line.
    if !has_tempo_declaration(&code) {
        code = format!("setcps(0.55)\n{code}");
    }

    validate_strudel_ast(&code)?;
    Ok(code)
}

fn has_tempo_declaration(code: &str) -> bool {
    code.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("//")
            && (trimmed.starts_with("setcps(") || trimmed.starts_with("setcpm("))
    })
}

/// Produces lightweight descriptive analysis for Sonic Intelligence.
/// These values are text-derived observations, not proof that the browser played
/// the composition. `success` is the successful preflight result supplied by the
/// caller; this function itself does not compile JavaScript or run Strudel.
pub fn analyze_ast_health(
    healed_code: &str,
    warnings: Vec<String>,
    success: bool,
) -> ExternalAstAnalysis {
    let code_without_comments = mask_line_comments(healed_code);
    let mut parsed_sections = Vec::new();
    let mut slider_names = Vec::new();
    let mut visualizer_names = Vec::new();
    let pattern_signatures = extract_trigger_strings(healed_code);
    let mut cycle_count = 0usize;

    for line in code_without_comments.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("const ") || trimmed.starts_with("let ") {
            if let Some((left, right)) = trimmed.split_once('=') {
                if right.contains("stack(") || right.contains("arrange(") {
                    let name = left.split_whitespace().nth(1).unwrap_or_default().trim();
                    if !name.is_empty() {
                        parsed_sections.push(name.to_string());
                    }
                }
                if right.contains("slider(") {
                    let name = left.split_whitespace().nth(1).unwrap_or_default().trim();
                    if !name.is_empty() {
                        slider_names.push(name.to_string());
                    }
                }
            }
        }

        for viz in [
            "_scope",
            "_pianoroll",
            "_punchcard",
            "_spiral",
            "_spectrum",
            "_pitchwheel",
            "scope",
            "pianoroll",
            "punchcard",
            "spiral",
            "spectrum",
            "pitchwheel",
        ] {
            if trimmed.contains(viz) {
                let clean_viz = viz.trim_start_matches('_').to_string();
                if !visualizer_names.contains(&clean_viz) {
                    visualizer_names.push(clean_viz);
                }
            }
        }
    }

    if let Some(arr_idx) = code_without_comments.find("arrange(") {
        let tail = &code_without_comments[arr_idx..];
        for segment in tail.split('[') {
            if let Some(comma_idx) = segment.find(',') {
                let num_str = segment[..comma_idx].trim();
                if let Ok(cycles) = num_str.parse::<usize>() {
                    cycle_count = cycle_count.saturating_add(cycles);
                }
            }
        }
    }

    ExternalAstAnalysis {
        ast_compiled_cleanly: success,
        warnings,
        parsed_sections: unique_preserving_order(&parsed_sections),
        slider_names: unique_preserving_order(&slider_names),
        cycle_count,
        visualizer_names: unique_preserving_order(&visualizer_names),
        pattern_signatures,
    }
}

/// Repairs a numeric slider argument list only when its entire argument text is
/// made of numeric tokens, commas, and whitespace. This is limited to slider()
/// so other JavaScript function arguments are never rewritten by this helper.
fn heal_slider_declarations(line: &str) -> String {
    let Some(pos) = line.find("slider(") else {
        return line.to_string();
    };

    let start_args = pos + "slider(".len();
    let Some(close_rel) = find_matching_close_paren(&line[start_args..]) else {
        return line.to_string();
    };

    let inner = &line[start_args..start_args + close_rel];
    if inner.contains('"') || inner.contains('\'') || inner.contains('`') {
        return line.to_string();
    }

    let tokens: Vec<&str> = inner
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .filter(|token| !token.is_empty())
        .collect();

    if tokens.len() < 2 || !tokens.iter().all(|token| token.parse::<f32>().is_ok()) {
        return line.to_string();
    }

    let repaired = tokens.join(", ");
    let mut output = line.to_string();
    output.replace_range(start_args..start_args + close_rel, &repaired);
    output
}

/// Repairs only the unmistakably malformed, unquoted numeric notation form
/// `[0 0.01]*8` (or `[0 0.01]/2`) when it appears outside a string/comment.
/// Valid comma-separated JavaScript arrays and all quoted strings are preserved.
fn heal_unquoted_numeric_mini_arrays(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut output = String::with_capacity(line.len());
    let mut i = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut line_comment = false;

    while i < chars.len() {
        let ch = chars[i];

        if line_comment {
            output.push(ch);
            i += 1;
            continue;
        }

        if let Some(active_quote) = quote {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            i += 1;
            continue;
        }

        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            line_comment = true;
            output.push('/');
            output.push('/');
            i += 2;
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
            output.push(ch);
            i += 1;
            continue;
        }

        if ch == '[' {
            let mut close = i + 1;
            while close < chars.len() && chars[close] != ']' {
                // Nested arrays are outside the scope of this narrow repair.
                if chars[close] == '[' {
                    break;
                }
                close += 1;
            }

            if close < chars.len() && chars[close] == ']' {
                let inner: String = chars[i + 1..close].iter().collect();
                let trimmed = inner.trim();
                let numeric_space_list = !trimmed.is_empty()
                    && !trimmed.contains(',')
                    && trimmed.split_whitespace().count() >= 2
                    && trimmed.chars().all(|value| {
                        value.is_ascii_digit()
                            || matches!(value, '.' | '-' | '+' | ' ' | '\t')
                    });

                let mut operator = close + 1;
                while operator < chars.len() && chars[operator].is_whitespace() {
                    operator += 1;
                }

                if numeric_space_list
                    && operator < chars.len()
                    && matches!(chars[operator], '*' | '/')
                {
                    let mut multiplier_end = operator + 1;
                    while multiplier_end < chars.len()
                        && chars[multiplier_end].is_whitespace()
                    {
                        multiplier_end += 1;
                    }
                    let number_start = multiplier_end;
                    while multiplier_end < chars.len()
                        && (chars[multiplier_end].is_ascii_digit() || chars[multiplier_end] == '.')
                    {
                        multiplier_end += 1;
                    }

                    if multiplier_end > number_start {
                        let notation: String = chars[i..=close].iter().collect();
                        let suffix: String = chars[operator..multiplier_end].iter().collect();
                        output.push('"');
                        output.push_str(&notation);
                        output.push_str(&suffix);
                        output.push('"');
                        i = multiplier_end;
                        continue;
                    }
                }
            }
        }

        output.push(ch);
        i += 1;
    }

    output
}

/// Legacy function name retained for callers. This is a source preflight, not a
/// full AST validator. It checks balanced lexical structure and that the source
/// contains an actual trigger call outside comments and strings.
pub fn validate_strudel_ast(code: &str) -> Result<(), String> {
    if code.trim().is_empty() {
        return Err("Strudel preflight failed: source is empty.".to_string());
    }

    let masked = mask_comments_and_strings(code)?;
    validate_delimiters(&masked)?;

    if !contains_trigger_call(&masked) {
        return Err(
            "Strudel preflight failed: no s(...), sound(...), note(...), or n(...) call was found outside comments and strings."
                .to_string(),
        );
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

fn contains_trigger_call(masked_code: &str) -> bool {
    let chars: Vec<char> = masked_code.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' || chars[i] == '$' {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '$')
            {
                i += 1;
            }

            let identifier: String = chars[start..i].iter().collect();
            if matches!(identifier.as_str(), "s" | "sound" | "note" | "n") {
                let mut next = i;
                while next < chars.len() && chars[next].is_whitespace() {
                    next += 1;
                }
                if next < chars.len() && chars[next] == '(' {
                    return true;
                }
            }
        } else {
            i += 1;
        }
    }

    false
}

/// Masks comments and string contents while preserving newlines and source
/// positions, enabling checks that don't mistake punctuation in a mini-notation
/// string or comment for JavaScript syntax. Unterminated strings/comments fail.
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
                } else if (state == State::Single && ch == '\'')
                    || (state == State::Double && ch == '"')
                    || (state == State::Template && ch == '`')
                {
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

/// Convenience mask for descriptive analysis. Preserves string contents but
/// removes line comments; it is deliberately not used to rewrite source code.
fn mask_line_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            } else {
                output.push(' ');
            }
            i += 1;
            continue;
        }

        if let Some(active_quote) = quote {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            i += 1;
            continue;
        }

        if ch == '/' && chars.get(i + 1) == Some(&'/') {
            line_comment = true;
            output.push(' ');
            output.push(' ');
            i += 2;
            continue;
        }

        if ch == '\'' || ch == '"' || ch == '`' {
            quote = Some(ch);
        }
        output.push(ch);
        i += 1;
    }

    output
}

fn extract_trigger_strings(code: &str) -> Vec<String> {
    let mut triggers = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        for function in ["note(", "s(", "sound(", "n("] {
            let mut search_idx = 0usize;
            while search_idx < trimmed.len() {
                let Some(relative_pos) = trimmed[search_idx..].find(function) else {
                    break;
                };
                let call_start = search_idx + relative_pos + function.len();
                let tail = &trimmed[call_start..];
                let quote_pos = tail.find(|ch| ch == '\'' || ch == '"');
                if let Some(quote_pos) = quote_pos {
                    let quote = tail[quote_pos..].chars().next().unwrap_or('"');
                    let content_start = call_start + quote_pos + quote.len_utf8();
                    let content_tail = &trimmed[content_start..];
                    if let Some(end) = content_tail.find(quote) {
                        let content = content_tail[..end].trim().to_lowercase();
                        if !content.is_empty()
                            && !matches!(
                                content.as_str(),
                                "sawtooth" | "supersaw" | "square" | "triangle" | "sine" | "pulse"
                            )
                        {
                            triggers.push(content);
                        }
                        search_idx = content_start + end + quote.len_utf8();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    unique_preserving_order(&triggers)
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

/// Rejects only an exact unchanged score during evolution. Shared motifs and
/// repeated patterns remain valid musical techniques.
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

/// Removes model reasoning blocks and, when supplied, extracts the first fenced
/// code block. For unfenced output, drops leading filler and common trailing prose.
/// It does not normalize quotes or rewrite music syntax.
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

    if let Some(fence_start) = text.find("```") {
        let after_first = &text[fence_start + 3..];
        let body_start = after_first.find('\n').map(|idx| idx + 1).unwrap_or(0);
        let after_language = &after_first[body_start..];
        if let Some(fence_end) = after_language.find("```") {
            text = after_language[..fence_end].to_string();
        }
    }

    let lines: Vec<&str> = text.lines().collect();
    let first_code_line = lines
        .iter()
        .position(|line| is_valid_code_start(line.trim()));
    let Some(start_index) = first_code_line else {
        return String::new();
    };

    let mut output = Vec::new();
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut seen_root = false;

    for line in lines.into_iter().skip(start_index) {
        let trimmed = line.trim();
        if is_trailing_prose(trimmed, paren_depth, bracket_depth, brace_depth, seen_root) {
            break;
        }

        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && ["stack(", "arrange(", "cat(", "slowcat(", "s(", "sound(", "note(", "n("]
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
        {
            seen_root = true;
        }

        update_delimiter_depths(
            line,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
        output.push(line.to_string());
    }

    output.join("\n").trim().to_string()
}

fn is_valid_code_start(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with("setcps")
        || trimmed.starts_with("setcpm")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var ")
        || trimmed.starts_with("stack(")
        || trimmed.starts_with("arrange(")
        || trimmed.starts_with("cat(")
        || trimmed.starts_with("slowcat(")
        || trimmed.starts_with("s(")
        || trimmed.starts_with("sound(")
        || trimmed.starts_with("note(")
        || trimmed.starts_with("n(")
}

fn is_trailing_prose(
    trimmed: &str,
    paren_depth: i32,
    bracket_depth: i32,
    brace_depth: i32,
    seen_root: bool,
) -> bool {
    if trimmed.is_empty() || trimmed.starts_with("//") {
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

    if seen_root && at_top_level {
        let valid_continuation = trimmed.starts_with('.')
            || trimmed.starts_with(')')
            || trimmed.starts_with(']')
            || trimmed.starts_with('}')
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("var ")
            || trimmed.starts_with("stack(")
            || trimmed.starts_with("arrange(")
            || trimmed.starts_with("cat(")
            || trimmed.starts_with("slowcat(")
            || trimmed.starts_with("s(")
            || trimmed.starts_with("sound(")
            || trimmed.starts_with("note(")
            || trimmed.starts_with("n(")
            || trimmed.starts_with("setcps(")
            || trimmed.starts_with("setcpm(")
            || trimmed.starts_with("export ");
        if !valid_continuation {
            return true;
        }
    }

    false
}

fn update_delimiter_depths(
    line: &str,
    paren_depth: &mut i32,
    bracket_depth: &mut i32,
    brace_depth: &mut i32,
) {
    let chars: Vec<char> = line.chars().collect();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            i += 1;
            continue;
        }

        if ch == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        if ch == '\'' || ch == '"' || ch == '`' {
            quote = Some(ch);
            i += 1;
            continue;
        }

        match ch {
            '(' => *paren_depth += 1,
            ')' => *paren_depth = (*paren_depth - 1).max(0),
            '[' => *bracket_depth += 1,
            ']' => *bracket_depth = (*bracket_depth - 1).max(0),
            '{' => *brace_depth += 1,
            '}' => *brace_depth = (*brace_depth - 1).max(0),
            _ => {}
        }
        i += 1;
    }
}

fn find_matching_close_paren(source: &str) -> Option<usize> {
    let chars: Vec<char> = source.chars().collect();
    let mut depth = 1i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            i += 1;
            continue;
        }

        if ch == '\'' || ch == '"' || ch == '`' {
            quote = Some(ch);
        } else if ch == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                // Return a UTF-8 byte offset, matching str::find/range usage.
                return Some(chars[..i].iter().map(|value| value.len_utf8()).sum());
            }
        }
        i += 1;
    }

    None
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
}
