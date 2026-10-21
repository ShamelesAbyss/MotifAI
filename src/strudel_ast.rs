use crate::orchestrator::{
    auto_mix_and_carve_frequencies, detect_scale_context, harmonize_and_voice_note_string,
    ScaleMode,
};
use std::collections::HashSet;

/// Full pipeline: extracts code, preserves multi-movement `const` stems, normalizes quotes,
/// heals unquoted `[0 0.01]*8` modifiers, heals slider math, re-voices cluster chords into
/// tertian harmony while protecting melodies/solos, fixes `arrange()` vs `stack()`, and auto-mixes.
pub fn compile_and_heal_strudel(raw: &str) -> Result<String, String> {
    let stripped = strip_thinking_and_extract_code(raw);
    if stripped.trim().is_empty() {
        return Err("Model returned no recognizable Strudel code.".to_string());
    }

    // Detect the dominant tonic pitch class (0..11) and scale mode from the piece
    let (tonic_pc, scale_mode) = detect_scale_context(&stripped, "");

    let mut processed_lines = Vec::new();
    let mut seen_setcps = false;

    for raw_line in stripped.lines() {
        let mut line = raw_line.to_string();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            processed_lines.push(String::new());
            continue;
        }

        if trimmed.starts_with("//") {
            processed_lines.push(line);
            continue;
        }

        // 1. Heal BPM-to-CPS hallucinations (e.g., setcps(128) -> setcps(0.53))
        if trimmed.starts_with("setcps(") || trimmed.starts_with("setcpm(") {
            if seen_setcps {
                continue;
            }
            seen_setcps = true;
            line = normalize_tempo_line(trimmed);
            processed_lines.push(line);
            continue;
        }

        // 2. Normalize single-quoted strings ('...') into double-quoted strings ("...")
        line = normalize_single_quoted_strings(&line);

        // 3. Heal foreign dialects, unquoted mini-modifiers (.late([0 0.01]*8)), slider math, and JS arrays
        line = heal_foreign_dialects(&line);
        line = heal_unquoted_mini_modifiers(&line);
        line = heal_slider_math_expressions(&line);
        line = heal_bracketed_numeric_arrays(&line);
        line = heal_synth_dsp_traps(&line);

        // 4. Scrub chained methods against the official Strudel API whitelist
        line = scrub_method_chains(&line);

        // 5. Repair mini-notation strings, heal cluster chords, and harmonize to scale mode
        line = repair_and_harmonize_mini_strings(&line, tonic_pc, scale_mode);

        processed_lines.push(line);
    }

    // 6. Ensure setcps() exists at top
    if !seen_setcps {
        processed_lines.insert(0, "setcps(0.55)".to_string());
    }

    // 7. Order of operations is critical:
    //    First repair missing commas, then balance global parentheses/brackets,
    //    then merge multiple top-level roots, and finally heal arrange() vs stack() pairs!
    let joined = processed_lines.join("\n");
    let comma_healed = repair_stack_commas(&joined);
    let balanced = balance_global_parentheses(&comma_healed);
    let root_merged = merge_multiple_top_level_roots(&balanced);
    let arrange_healed = heal_broken_arrange_calls(&root_merged);
    let final_balanced = balance_global_parentheses(&arrange_healed);

    // 8. Auto-mix: isolate reverb orbits (.orbit(1..6)), carve muddy low-mids (.hpf(180)), and balance headroom
    let mixed = auto_mix_and_carve_frequencies(&final_balanced);

    validate_strudel_ast(&mixed)?;
    Ok(mixed)
}

/// Converts single-quoted strings `'...'` into double-quoted `"..."` so the mini-notation
/// parser never crashes with `Expected ... but "'" found`.
fn normalize_single_quoted_strings(line: &str) -> String {
    let code_end = line.find("//").unwrap_or(line.len());
    let (code_part, comment_part) = line.split_at(code_end);

    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = code_part.chars().collect();
    let mut i = 0;
    let mut in_double = false;
    let mut in_single = false;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' && !in_single {
            in_double = !in_double;
            out.push('"');
        } else if ch == '\'' && !in_double {
            in_single = !in_single;
            out.push('"');
        } else if ch == '\'' && in_double {
            // Strip stray apostrophes/single-quotes inside double-quoted mini-notation strings
        } else {
            out.push(ch);
        }
        i += 1;
    }

    if in_single || in_double {
        out.push('"');
    }

    out.push_str(comment_part);
    out
}

/// Fixes `.late([0 0.01]*8)` or `.late([0, 0.01]*8)` which causes JS `Unexpected token` or `NaN`
/// by wrapping the bracketed pattern in double quotes: `.late("[0 0.01]*8")`.
fn heal_unquoted_mini_modifiers(line: &str) -> String {
    let mut out = line.to_string();
    for method in [".late(", ".early(", ".fast(", ".slow(", ".mask(", ".struct("] {
        let mut search_from = 0;
        while let Some(rel_pos) = out[search_from..].find(method) {
            let start_args = search_from + rel_pos + method.len();
            if let Some(close_rel) = find_matching_close_paren(&out[start_args..]) {
                let inner = out[start_args..start_args + close_rel].trim().to_string();
                if !inner.is_empty()
                    && !inner.starts_with('"')
                    && (inner.starts_with('[') || inner.starts_with('<'))
                {
                    let clean_mini = inner.replace(',', " ");
                    let rep = format!("\"{}\"", clean_mini);
                    out.replace_range(start_args..start_args + close_rel, &rep);
                    search_from = start_args + rep.len();
                    continue;
                }
            }
            search_from = start_args;
        }
    }
    out
}

/// Automatically converts space-separated JS numeric arrays like `[0 0.01]` into valid `"[0 0.01]"`
/// or `[0, 0.01]` depending on context.
fn heal_bracketed_numeric_arrays(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_quote = false;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            in_quote = !in_quote;
            out.push(ch);
            i += 1;
            continue;
        }

        if !in_quote && ch == '[' {
            let mut depth = 1;
            let mut j = i + 1;
            while j < chars.len() {
                if chars[j] == '[' {
                    depth += 1;
                } else if chars[j] == ']' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }

            if j < chars.len() && depth == 0 {
                let inner: String = chars[i + 1..j].iter().collect();
                let trimmed_inner = inner.trim();
                let is_numeric_list = !trimmed_inner.is_empty()
                    && !trimmed_inner.contains(',')
                    && trimmed_inner.chars().all(|c| {
                        c.is_ascii_digit() || c == '.' || c == '-' || c.is_whitespace()
                    });

                if is_numeric_list {
                    let parts: Vec<&str> = trimmed_inner.split_whitespace().collect();
                    if parts.len() > 1 {
                        // Check if followed by `*` or `/` (mini-notation multiplier outside quotes)
                        let mut k = j + 1;
                        while k < chars.len() && chars[k].is_whitespace() {
                            k += 1;
                        }
                        if k < chars.len() && (chars[k] == '*' || chars[k] == '/') {
                            let mut m = k + 1;
                            while m < chars.len()
                                && (chars[m].is_ascii_digit() || chars[m] == '.')
                            {
                                m += 1;
                            }
                            let mult: String = chars[k..m].iter().collect();
                            out.push('"');
                            out.push('[');
                            out.push_str(&parts.join(" "));
                            out.push(']');
                            out.push_str(&mult);
                            out.push('"');
                            i = m;
                            continue;
                        } else {
                            out.push('[');
                            out.push_str(&parts.join(", "));
                            out.push(']');
                            i = j + 1;
                            continue;
                        }
                    }
                }
            }
        }

        out.push(ch);
        i += 1;
    }
    out
}

/// If the model outputs two or more top-level `arrange(...)` or `stack(...)` blocks separated by commas
/// (which causes `Unexpected token` in Strudel), wraps all top-level root expressions inside a single `stack(...)`.
fn merge_multiple_top_level_roots(code: &str) -> String {
    let mut header_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_const_block = false;
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;

    for line in code.lines() {
        let trimmed = line.trim();
        let code_only = trimmed.split("//").next().unwrap_or("").trim();

        if paren_depth == 0 && bracket_depth == 0 {
            if code_only.starts_with("setcps(")
                || code_only.starts_with("const ")
                || code_only.starts_with("let ")
                || (trimmed.starts_with("//") && body_lines.is_empty())
                || (trimmed.is_empty() && body_lines.is_empty())
            {
                in_const_block =
                    code_only.starts_with("const ") || code_only.starts_with("let ");
                header_lines.push(line.to_string());
                update_depths(code_only, &mut paren_depth, &mut bracket_depth);
                if paren_depth == 0 && bracket_depth == 0 {
                    in_const_block = false;
                }
                continue;
            }
        }

        if in_const_block {
            header_lines.push(line.to_string());
            update_depths(code_only, &mut paren_depth, &mut bracket_depth);
            if paren_depth == 0 && bracket_depth == 0 {
                in_const_block = false;
            }
        } else {
            body_lines.push(line.to_string());
            update_depths(code_only, &mut paren_depth, &mut bracket_depth);
        }
    }

    let body = body_lines.join("\n");
    let trimmed_body = body.trim();
    if trimmed_body.is_empty() {
        return code.to_string();
    }

    // Count how many top-level root expressions exist in `trimmed_body`
    let mut top_expr_count = 0usize;
    let mut p_depth = 0i32;
    let mut b_depth = 0i32;
    let mut in_q = false;

    for ch in trimmed_body.chars() {
        if ch == '"' {
            in_q = !in_q;
        } else if !in_q {
            match ch {
                '(' => {
                    if p_depth == 0 && b_depth == 0 {
                        top_expr_count += 1;
                    }
                    p_depth += 1;
                }
                ')' => p_depth = (p_depth - 1).max(0),
                '[' => b_depth += 1,
                ']' => b_depth = (b_depth - 1).max(0),
                _ => {}
            }
        }
    }

    if top_expr_count > 1 {
        let clean_body = trimmed_body.trim_end_matches(',').trim();
        let mut combined = header_lines.join("\n");
        if !combined.is_empty() {
            combined.push_str("\n\n");
        }
        combined.push_str("stack(\n");
        combined.push_str(clean_body);
        combined.push_str("\n)");
        return combined;
    }

    code.to_string()
}

fn update_depths(code_only: &str, paren_depth: &mut i32, bracket_depth: &mut i32) {
    let mut in_quote = false;
    for ch in code_only.chars() {
        if ch == '"' {
            in_quote = !in_quote;
        } else if !in_quote {
            match ch {
                '(' => *paren_depth += 1,
                ')' => *paren_depth = (*paren_depth - 1).max(0),
                '[' => *bracket_depth += 1,
                ']' => *bracket_depth = (*bracket_depth - 1).max(0),
                _ => {}
            }
        }
    }
}

/// Ensures `arrange(...)` only receives `[cycles, pattern]` pairs AND upgrades `stack([8, intro], [12, verse])`
/// into `arrange([8, intro], [12, verse])` if the model accidentally wrote `stack` for `[cycles, pattern]` pairs!
fn heal_broken_arrange_calls(code: &str) -> String {
    let mut out = code.to_string();

    // 1. If the model wrote `arrange(...)` where arguments are NOT `[cycles, pattern]` pairs, convert to `stack(...)`
    let mut search_pos = 0;
    while let Some(rel) = out[search_pos..].find("arrange(") {
        let pos = search_pos + rel;
        let after_open = &out[pos + 8..];

        if let Some(close_rel) = find_matching_close_paren(after_open) {
            let inner = &after_open[..close_rel];
            if !all_top_level_args_are_pairs(inner) {
                out.replace_range(pos..pos + 8, "stack(");
                search_pos = pos + 6;
            } else {
                search_pos = pos + 8;
            }
        } else {
            out.replace_range(pos..pos + 8, "stack(");
            search_pos = pos + 6;
        }
    }

    // 2. Conversely, if the model wrote `stack([8, ...], [12, ...])` with `[cycles, pattern]` pairs,
    //    upgrade `stack(` into `arrange(` so Strudel doesn't crash on `[number, Pattern]` inside `stack()`!
    let mut stack_search = 0;
    while let Some(rel) = out[stack_search..].find("stack(") {
        let pos = stack_search + rel;
        let after_open = &out[pos + 6..];
        if let Some(close_rel) = find_matching_close_paren(after_open) {
            let inner = &after_open[..close_rel];
            if all_top_level_args_are_pairs(inner) {
                out.replace_range(pos..pos + 6, "arrange(");
                stack_search = pos + 8;
                continue;
            }
        }
        stack_search = pos + 6;
    }

    out
}

fn all_top_level_args_are_pairs(inner: &str) -> bool {
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut in_quote = false;
    let mut current_arg = String::new();
    let mut args = Vec::new();

    for line in inner.lines() {
        let clean_line = line.split("//").next().unwrap_or("");
        for ch in clean_line.chars() {
            if ch == '"' {
                in_quote = !in_quote;
                current_arg.push(ch);
            } else if !in_quote {
                match ch {
                    '(' => {
                        paren_depth += 1;
                        current_arg.push(ch);
                    }
                    ')' => {
                        paren_depth = (paren_depth - 1).max(0);
                        current_arg.push(ch);
                    }
                    '[' => {
                        bracket_depth += 1;
                        current_arg.push(ch);
                    }
                    ']' => {
                        bracket_depth = (bracket_depth - 1).max(0);
                        current_arg.push(ch);
                    }
                    ',' if paren_depth == 0 && bracket_depth == 0 => {
                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }
                        current_arg.clear();
                    }
                    _ => current_arg.push(ch),
                }
            } else {
                current_arg.push(ch);
            }
        }
    }

    if !current_arg.trim().is_empty() {
        args.push(current_arg.trim().to_string());
    }

    if args.is_empty() {
        return false;
    }

    args.iter().all(|arg| {
        if !(arg.starts_with('[') && arg.ends_with(']')) {
            return false;
        }
        let inside = &arg[1..arg.len() - 1];
        if let Some((cycles_str, _pat_str)) = inside.split_once(',') {
            cycles_str.trim().parse::<f32>().is_ok()
        } else {
            false
        }
    })
}

pub fn check_clone_or_stagnation(new_code: &str, previous_code: Option<&str>) -> Option<String> {
    let lower = new_code.to_lowercase();
    let has_cliche_bass = lower.contains("<[c1 c2]*4 [ab0 ab1]*4 [f0 f1]*4 [g0 g1]*4>");
    let has_cliche_lead = lower.contains("c5 [eb5 g5] bb5 g5 eb5 [d5 c5] b4 <g4 d5>");

    if has_cliche_bass && has_cliche_lead {
        return Some(
            "CLONE DETECTED: You output banned hardcoded template notes. Compose 100% original notes and sections!"
                .to_string(),
        );
    }

    if let Some(prev) = previous_code {
        let new_triggers = extract_trigger_strings(new_code);
        let prev_triggers = extract_trigger_strings(prev);

        if !new_triggers.is_empty() && !prev_triggers.is_empty() {
            let matching = new_triggers
                .iter()
                .filter(|t| prev_triggers.contains(t))
                .count();
            let ratio = matching as f32 / new_triggers.len().max(1) as f32;

            let did_not_expand = new_triggers.len() <= prev_triggers.len();
            if did_not_expand && ratio >= 0.88 {
                return Some(format!(
                    "STAGNATION DETECTED: {:.0}% pattern match with no expansion. Add new song sections (intro, verse, chorus, bridge, solo) or mutate the patterns!",
                    ratio * 100.0
                ));
            }
        }
    }

    None
}

fn extract_trigger_strings(code: &str) -> Vec<String> {
    let mut triggers = Vec::new();
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        for prefix in ["note(\"", "s(\"", "n(\""] {
            let mut search_idx = 0;
            while let Some(pos) = trimmed[search_idx..].find(prefix) {
                let start = search_idx + pos + prefix.len();
                if let Some(end) = trimmed[start..].find('"') {
                    let content = trimmed[start..start + end].trim().to_lowercase();
                    if !content.is_empty()
                        && content != "sawtooth"
                        && content != "supersaw"
                        && content != "square"
                        && content != "triangle"
                        && content != "sine"
                        && content != "pulse"
                    {
                        triggers.push(content);
                    }
                    search_idx = start + end + 1;
                } else {
                    break;
                }
            }
        }
    }
    triggers
}

/// Strips <think>...</think> blocks, markdown fences, leading filler, and trailing prose,
/// while preserving multi-movement `const intro = ...`, `const verse = ...`, and `arrange(...)` blocks.
fn strip_thinking_and_extract_code(raw: &str) -> String {
    let mut text = raw.to_string();

    while let Some(start) = text.find("<think>") {
        if let Some(end) = text[start..].find("</think>") {
            text.replace_range(start..start + end + 8, "");
        } else {
            text.replace_range(start.., "");
            break;
        }
    }

    if let Some(fence_start) = text.find("```") {
        let after_first = &text[fence_start + 3..];
        let body_start = after_first.find('\n').map(|i| i + 1).unwrap_or(0);
        let after_lang = &after_first[body_start..];
        if let Some(fence_end) = after_lang.find("```") {
            text = after_lang[..fence_end].to_string();
        }
    }

    let mut lines = Vec::new();
    let mut code_started = false;
    let mut paren_depth: i32 = 0;
    let mut seen_top_level_root = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            continue;
        }

        if !code_started {
            if is_valid_code_start(trimmed) {
                code_started = true;
            } else {
                continue;
            }
        }

        if is_trailing_prose(trimmed, paren_depth, seen_top_level_root) {
            break;
        }

        // Only mark `seen_top_level_root` when a standalone `arrange(` or `stack(` (NOT `const intro = stack(`) starts at depth 0
        if paren_depth == 0
            && (trimmed.starts_with("stack(")
                || trimmed.starts_with("arrange(")
                || trimmed.starts_with("cat(")
                || trimmed.starts_with("slowcat("))
        {
            seen_top_level_root = true;
        }

        let code_part = trimmed.split("//").next().unwrap_or("");
        let mut in_quote = false;
        for ch in code_part.chars() {
            if ch == '"' {
                in_quote = !in_quote;
            } else if !in_quote {
                match ch {
                    '(' | '[' => paren_depth += 1,
                    ')' | ']' => paren_depth = (paren_depth - 1).max(0),
                    _ => {}
                }
            }
        }

        lines.push(line.to_string());
    }

    lines.join("\n").trim().to_string()
}

fn is_valid_code_start(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with("setcps")
        || trimmed.starts_with("setcpm")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("stack(")
        || trimmed.starts_with("arrange(")
        || trimmed.starts_with("cat(")
        || trimmed.starts_with("slowcat(")
        || trimmed.starts_with("s(")
        || trimmed.starts_with("note(")
        || trimmed.starts_with("n(")
}

fn is_trailing_prose(trimmed: &str, paren_depth: i32, seen_top_level_root: bool) -> bool {
    if trimmed.is_empty() || trimmed.starts_with("//") {
        return false;
    }

    if paren_depth == 0
        && (trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("###")
            || trimmed.starts_with("The ")
            || trimmed.starts_with("This ")
            || trimmed.starts_with("Here ")
            || trimmed.starts_with("In this ")
            || trimmed.starts_with("Note:")
            || trimmed.starts_with("Explanation:"))
    {
        return true;
    }

    if seen_top_level_root && paren_depth == 0 {
        let valid_continuation = trimmed.starts_with('.')
            || trimmed.starts_with(')')
            || trimmed.starts_with(']')
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("stack(")
            || trimmed.starts_with("arrange(")
            || trimmed.starts_with("cat(")
            || trimmed.starts_with("slowcat(")
            || trimmed.starts_with("s(")
            || trimmed.starts_with("note(")
            || trimmed.starts_with("n(");
        if !valid_continuation {
            return true;
        }
    }

    false
}

fn normalize_tempo_line(trimmed: &str) -> String {
    let is_cpm = trimmed.starts_with("setcpm(");
    let open = trimmed.find('(').unwrap_or(0);
    let close = trimmed.rfind(')').unwrap_or(trimmed.len());
    if open + 1 < close {
        let inner = trimmed[open + 1..close].trim();
        let val = if let Some((num, den)) = inner.split_once('/') {
            let n = num.trim().parse::<f32>().unwrap_or(120.0);
            let d = den.trim().parse::<f32>().unwrap_or(4.0).max(1.0);
            n / d
        } else {
            inner.parse::<f32>().unwrap_or(0.55)
        };

        let cps = if is_cpm {
            if val > 50.0 {
                (val / 240.0).clamp(0.25, 0.85)
            } else {
                (val / 60.0).clamp(0.25, 0.85)
            }
        } else if val > 2.5 {
            (val / 240.0).clamp(0.28, 0.82)
        } else {
            val.clamp(0.25, 0.95)
        };

        return format!("setcps({:.2})", cps);
    }
    "setcps(0.55)".to_string()
}

/// Heals broken JS math on Pattern sliders like `.gain(VOLUME * 0.88)` -> `.gain(VOLUME.mul(0.88))`
/// and heals `sine.range(BASS_CUT * 0.8, BASS_CUT * 2)` -> safe numeric range `[450, 2200]`.
fn heal_slider_math_expressions(line: &str) -> String {
    let mut out = line.to_string();

    for method in [
        ".gain(",
        ".lpf(",
        ".hpf(",
        ".lpq(",
        ".shape(",
        ".room(",
        ".delay(",
        ".size(",
    ] {
        let mut search_from = 0;
        while let Some(rel_pos) = out[search_from..].find(method) {
            let start_args = search_from + rel_pos + method.len();
            if let Some(close_rel) = find_matching_close_paren(&out[start_args..]) {
                let inner = out[start_args..start_args + close_rel].trim().to_string();
                if !inner.contains('"') && !inner.starts_with("sine.") {
                    if let Some((left, right)) = inner.split_once('*') {
                        let l = left.trim();
                        let r = right.trim();
                        if is_ident(l) && r.parse::<f32>().is_ok() {
                            let rep = format!("{}.mul({})", l, r);
                            out.replace_range(start_args..start_args + close_rel, &rep);
                        } else if is_ident(r) && l.parse::<f32>().is_ok() {
                            let rep = format!("{}.mul({})", r, l);
                            out.replace_range(start_args..start_args + close_rel, &rep);
                        }
                    }
                }
            }
            search_from = start_args;
        }
    }
    out
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        && s.chars().next().unwrap().is_ascii_alphabetic()
}

fn heal_synth_dsp_traps(line: &str) -> String {
    let mut out = line.to_string();

    if (out.contains("note(\"")
        || out.contains(".s(\"supersaw\"")
        || out.contains(".s(\"sawtooth\""))
        && out.contains(".chop(")
    {
        if let Some(pos) = out.find(".chop(") {
            let start_args = pos + 6;
            if let Some(close_rel) = find_matching_close_paren(&out[start_args..]) {
                out.replace_range(
                    pos..start_args + close_rel + 1,
                    ".struct(\"x ~ x x ~ x [x x] ~\")",
                );
            }
        }
    }

    let mut search_idx = 0;
    while let Some(rel_pos) = out[search_idx..].find("sine.range(") {
        let pos = search_idx + rel_pos;
        let start_args = pos + 11;
        if let Some(close_rel) = find_matching_close_paren(&out[start_args..]) {
            let close_idx = start_args + close_rel;
            let inner = out[start_args..close_idx].to_string();
            let after_call = &out[close_idx + 1..];
            let has_slow_or_fast =
                after_call.starts_with(".slow(") || after_call.starts_with(".fast(");

            let mut fixed_inner = inner.clone();
            if inner.contains('*') || inner.contains('/') {
                // Heal `sine.range(BASS_CUT * 0.8, BASS_CUT * 2)` where JS `*` on Pattern slider causes NaN
                fixed_inner = "500, 2600".to_string();
            } else if let Some((low_s, high_s)) = inner.split_once(',') {
                if let (Ok(low), Ok(high)) =
                    (low_s.trim().parse::<f32>(), high_s.trim().parse::<f32>())
                {
                    if high > 10.0 && low < 180.0 {
                        let safe_low = low.max(240.0);
                        let safe_high = high.max(1400.0);
                        fixed_inner = format!("{:.0}, {:.0}", safe_low, safe_high);
                    }
                }
            }

            let replacement = if has_slow_or_fast {
                format!("sine.range({})", fixed_inner)
            } else {
                format!("sine.range({}).slow(4)", fixed_inner)
            };

            out.replace_range(pos..close_idx + 1, &replacement);
            search_idx = pos + replacement.len();
        } else {
            break;
        }
    }

    out
}

fn heal_foreign_dialects(line: &str) -> String {
    let mut s = line.to_string();

    if s.trim_end().ends_with(';') {
        let idx = s.rfind(';').unwrap();
        s.remove(idx);
    }

    if s.trim_start().starts_with(".stack(") {
        s = s.replacen(".stack(", "stack(", 1);
    }

    s = s
        .replace("GAIN(", "gain(")
        .replace("NOTE(", "note(")
        .replace("SOUND(", "s(")
        .replace("STACK(", "stack(")
        .replace("ARRANGE(", "arrange(")
        .replace("SLIDER(", "slider(")
        .replace("LPF(", "lpf(")
        .replace("HPF(", "hpf(")
        .replace("[object Object]", "0.5")
        .replace("sound(", "s(")
        .replace("\"rolandtr808\"", "\"RolandTR808\"")
        .replace("\"rolandtr909\"", "\"RolandTR909\"")
        .replace("\"rolandtr707\"", "\"RolandTR707\"")
        .replace("\"tr808\"", "\"RolandTR808\"")
        .replace("\"tr909\"", "\"RolandTR909\"")
        .replace("\"tr707\"", "\"RolandTR707\"");

    s
}

fn find_matching_close_paren(s: &str) -> Option<usize> {
    let mut depth = 1;
    let mut in_quote = false;
    for (i, ch) in s.char_indices() {
        if ch == '"' {
            in_quote = !in_quote;
        } else if !in_quote {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn scrub_method_chains(line: &str) -> String {
    let whitelist: HashSet<&'static str> = [
        "s", "sound", "note", "n", "bank", "scale", "transpose", "add", "sub", "mul", "div",
        "lpf", "lpq", "hpf", "hpq", "bpf", "bpq", "vowel", "lpenv", "hpenv", "bpenv", "lpa",
        "lpd", "lps", "lpr", "gain", "velocity", "attack", "decay", "sustain", "release", "adsr",
        "clip", "legato", "shape", "crush", "coarse", "room", "size", "sz", "delay", "delaytime",
        "dt", "delayfeedback", "dfb", "echo", "stut", "pan", "jux", "juxBy", "phaser",
        "phaserrate", "phaserdepth", "vib", "vibmod", "fm", "fmh", "fmenv", "detune", "speed",
        "begin", "end", "chop", "striate", "loopAt", "slice", "splice", "cut", "orbit",
        "duckorbit", "duckattack", "duckdepth", "leslie", "tremolo", "fast", "slow", "early",
        "late", "ply", "every", "sometimes", "sometimesBy", "often", "rarely", "almostAlways",
        "almostNever", "degrade", "degradeBy", "mask", "struct", "euclid", "euclidRot", "rev",
        "palindrome", "iter", "off", "arp", "superimpose", "layer", "linger", "zoom", "within",
        "compress", "hurry", "scramble", "segment", "seg", "chunk", "range", "rangex", "color",
        "pianoroll", "_pianoroll", "scope", "_scope", "punchcard", "_punchcard", "spiral",
        "_spiral", "pitchwheel", "_pitchwheel",
    ]
    .into_iter()
    .collect();

    let s = line
        .replace(".vol(", ".gain(")
        .replace(".volume(", ".gain(")
        .replace(".amp(", ".gain(")
        .replace(".cutoff(", ".lpf(")
        .replace(".filter(", ".lpf(")
        .replace(".lp(", ".lpf(")
        .replace(".hp(", ".hpf(")
        .replace(".bp(", ".bpf(")
        .replace(".resonance(", ".lpq(")
        .replace(".res(", ".lpq(")
        .replace(".reverb(", ".room(")
        .replace(".distort(", ".shape(")
        .replace(".distortion(", ".shape(")
        .replace(".dist(", ".shape(")
        .replace(".drive(", ".shape(")
        .replace(".delayfb(", ".delayfeedback(")
        .replace(".delayt(", ".delaytime(")
        .replace(".vibrato(", ".vib(");

    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut in_quote = false;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            in_quote = !in_quote;
            out.push(ch);
            i += 1;
            continue;
        }

        if !in_quote
            && ch == '.'
            && i + 1 < chars.len()
            && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '_')
        {
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }

            if j < chars.len() && chars[j] == '(' {
                let ident: String = chars[i + 1..j].iter().collect();
                let rest: String = chars[j + 1..].iter().collect();

                if let Some(close_offset) = find_matching_close_paren(&rest) {
                    let close_idx = j + 1 + close_offset;
                    if !whitelist.contains(ident.as_str()) {
                        i = close_idx + 1;
                        continue;
                    } else {
                        let args_str: String = chars[j + 1..close_idx].iter().collect();
                        let safe_args = clamp_dsp_args(&ident, &args_str);
                        out.push('.');
                        out.push_str(&ident);
                        out.push('(');
                        out.push_str(&safe_args);
                        out.push(')');
                        i = close_idx + 1;
                        continue;
                    }
                }
            }
        }

        out.push(ch);
        i += 1;
    }

    out
}

fn clamp_dsp_args(method: &str, args: &str) -> String {
    let trimmed = args.trim();
    if let Ok(val) = trimmed.parse::<f32>() {
        match method {
            "lpq" | "hpq" | "bpq" => return format!("{:.1}", val.clamp(0.5, 9.0)),
            "gain" => return format!("{:.2}", val.clamp(0.05, 0.98)),
            "room" => return format!("{:.2}", val.clamp(0.0, 0.82)),
            "size" | "sz" => return format!("{:.1}", val.clamp(1.0, 4.8)),
            "shape" => return format!("{:.2}", val.clamp(0.0, 0.65)),
            "delayfeedback" | "dfb" => return format!("{:.2}", val.clamp(0.0, 0.65)),
            "attack" => return format!("{:.2}", val.clamp(0.005, 0.35)),
            _ => {}
        }
    }
    args.to_string()
}

/// Repairs mini-notation strings AND harmonizes `note("...")` strings via `crate::orchestrator`.
/// Distinguishes true chord/pad progressions from fast melodies/arpeggios/solos so melodic phrases
/// are never flattened into block chords!
fn repair_and_harmonize_mini_strings(line: &str, tonic_pc: u8, mode: ScaleMode) -> String {
    if !line.contains('"') {
        return line.to_string();
    }

    let is_melodic_or_arp = line.contains("*2")
        || line.contains("*4")
        || line.contains("*8")
        || line.contains(".off(")
        || line.contains(".arp(")
        || line.contains(".vowel(")
        || line.contains("._spiral(");

    let is_chord_or_pad_line = !is_melodic_or_arp
        && (line.contains("/2")
            || line.contains("/4")
            || line.contains(".attack(")
            || line.contains(".phaser("));

    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '"' {
            let prefix: String = out
                .chars()
                .rev()
                .take(8)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            let is_note_arg = prefix.ends_with("note(");

            let mut j = i + 1;
            let mut content = String::new();
            while j < chars.len() && chars[j] != '"' {
                content.push(chars[j]);
                j += 1;
            }

            let mut cleaned = sanitize_mini_string(&content);
            if is_note_arg {
                cleaned = harmonize_and_voice_note_string(
                    &cleaned,
                    tonic_pc,
                    mode,
                    is_chord_or_pad_line,
                );
            }

            out.push('"');
            out.push_str(&cleaned);
            out.push('"');
            i = if j < chars.len() { j + 1 } else { chars.len() };
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}

fn sanitize_mini_string(s: &str) -> String {
    let cleaned = s
        .replace('\'', "")
        .replace(" * ", "*")
        .replace(" *", "*")
        .replace("* ", "*")
        .replace(" / ", "/")
        .replace(" /", "/")
        .replace("/ ", "/");

    let mut sq = 0i32;
    let mut ang = 0i32;
    let mut par = 0i32;
    let mut filtered = String::with_capacity(cleaned.len());

    for ch in cleaned.chars() {
        match ch {
            '[' => {
                sq += 1;
                filtered.push(ch);
            }
            ']' => {
                if sq > 0 {
                    sq -= 1;
                    filtered.push(ch);
                }
            }
            '<' => {
                ang += 1;
                filtered.push(ch);
            }
            '>' => {
                if ang > 0 {
                    ang -= 1;
                    filtered.push(ch);
                }
            }
            '(' => {
                par += 1;
                filtered.push(ch);
            }
            ')' => {
                if par > 0 {
                    par -= 1;
                    filtered.push(ch);
                }
            }
            _ => filtered.push(ch),
        }
    }

    while sq > 0 {
        filtered.push(']');
        sq -= 1;
    }
    while ang > 0 {
        filtered.push('>');
        ang -= 1;
    }
    while par > 0 {
        filtered.push(')');
        par -= 1;
    }

    filtered
}

fn repair_stack_commas(code: &str) -> String {
    let mut lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
    let len = lines.len();
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;

    for i in 0..len {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        let code_only = trimmed.split("//").next().unwrap_or("").trim_end();
        update_depths(code_only, &mut paren_depth, &mut bracket_depth);

        // Only insert missing commas when we are INSIDE an open `stack(...)` or `arrange(...)` (paren_depth >= 1)
        if paren_depth >= 1
            && (code_only.ends_with(')') || code_only.ends_with(']'))
            && !code_only.starts_with("setcps")
            && !code_only.starts_with("const ")
            && !code_only.starts_with("let ")
        {
            let mut next_idx = None;
            for j in (i + 1)..len {
                let nt = lines[j].trim();
                if !nt.is_empty() && !nt.starts_with("//") {
                    next_idx = Some(j);
                    break;
                }
            }

            if let Some(j) = next_idx {
                let next_trimmed = lines[j].trim();
                let starts_new_layer = next_trimmed.starts_with("s(")
                    || next_trimmed.starts_with("note(")
                    || next_trimmed.starts_with("n(")
                    || next_trimmed.starts_with("stack(")
                    || next_trimmed.starts_with("cat(")
                    || next_trimmed.starts_with("slowcat(")
                    || next_trimmed.starts_with("arrange(");

                if starts_new_layer {
                    if let Some(comment_pos) = lines[i].find("//") {
                        lines[i].insert(comment_pos, ',');
                    } else {
                        lines[i].push(',');
                    }
                }
            }
        }
    }

    lines.join("\n")
}

fn balance_global_parentheses(code: &str) -> String {
    let mut out = code.to_string();
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut in_quote = false;

    for line in out.lines() {
        let code_part = line.split("//").next().unwrap_or("");
        for ch in code_part.chars() {
            if ch == '"' {
                in_quote = !in_quote;
            } else if !in_quote {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => paren_depth = (paren_depth - 1).max(0),
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth = (bracket_depth - 1).max(0),
                    _ => {}
                }
            }
        }
    }

    while bracket_depth > 0 {
        out.push(']');
        bracket_depth -= 1;
    }
    while paren_depth > 0 {
        out.push_str("\n)");
        paren_depth -= 1;
    }

    out
}

pub fn validate_strudel_ast(code: &str) -> Result<(), String> {
    let has_trigger = code.contains("s(\"") || code.contains("note(\"") || code.contains("n(\"");
    if !has_trigger {
        return Err(
            "AST Validation Failed: No valid s(\"...\") or note(\"...\") triggers found."
                .to_string(),
        );
    }
    Ok(())
}