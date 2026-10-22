#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    NaturalMinor,
    HarmonicMinor,
    Dorian,
    PhrygianDominant,
    Lydian,
    Major,
    MelodicMinor,
}

impl ScaleMode {
    pub fn intervals(self) -> [u8; 7] {
        match self {
            ScaleMode::NaturalMinor => [0, 2, 3, 5, 7, 8, 10],
            ScaleMode::HarmonicMinor => [0, 2, 3, 5, 7, 8, 11],
            ScaleMode::Dorian => [0, 2, 3, 5, 7, 9, 10],
            ScaleMode::PhrygianDominant => [0, 1, 4, 5, 7, 8, 10],
            ScaleMode::Lydian => [0, 2, 4, 6, 7, 9, 11],
            ScaleMode::Major => [0, 2, 4, 5, 7, 9, 11],
            ScaleMode::MelodicMinor => [0, 2, 3, 5, 7, 9, 11],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ScaleMode::NaturalMinor => "Natural Minor (Aeolian)",
            ScaleMode::HarmonicMinor => "Harmonic Minor (Classical / Symphonic)",
            ScaleMode::Dorian => "Dorian (Cinematic / French House / Deep Groove)",
            ScaleMode::PhrygianDominant => "Phrygian Dominant (Exotic / Dark Bass Tension)",
            ScaleMode::Lydian => "Lydian (Astral / Dreamscape Film Score)",
            ScaleMode::Major => "Major (Ionian / Euphoric Anthemic)",
            ScaleMode::MelodicMinor => "Melodic Minor (Modern Orchestral / Jazz Fusion)",
        }
    }
}

/// Detects the dominant tonic pitch class (0..11) and scale mode from existing code and prompt hints.
/// Explicit prompt/code scale declarations take priority, followed by a bass-weighted pitch histogram.
pub fn detect_scale_context(code: &str, prompt_hint: &str) -> (u8, ScaleMode) {
    let (prompt_root, prompt_mode) = infer_prompt_key_and_mode(prompt_hint);

    // 1. Check if the code explicitly declares `.scale("root:mode")` or `.scale('root:mode')`
    if let Some((code_root, code_mode)) = extract_explicit_scale_call(code) {
        return (
            prompt_root.unwrap_or(code_root),
            prompt_mode.unwrap_or(code_mode),
        );
    }

    // 2. Determine mode from prompt first, then code comments/keywords, defaulting to NaturalMinor
    let mode = prompt_mode
        .or_else(|| infer_mode_from_text(code))
        .unwrap_or(ScaleMode::NaturalMinor);

    if let Some(explicit_root) = prompt_root {
        return (explicit_root, mode);
    }

    // 3. Weighted histogram of pitch classes in `note("...")` / `note('...')`, favoring bass octaves (0..=2)
    let mut weights = [0u32; 12];
    let mut first_bass_pc: Option<u8> = None;

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.contains("s(\"cb\")") || trimmed.contains("s('cb')") {
            continue;
        }
        for note_str in extract_note_call_strings(trimmed) {
            for token in note_str.split(|c: char| !c.is_ascii_alphanumeric() && c != '#') {
                if let Some((pc, oct, _)) = parse_pitch_token(token) {
                    if oct <= 2 {
                        if first_bass_pc.is_none() {
                            first_bass_pc = Some(pc);
                            // Give the opening bass root a tie-breaker bonus without discarding the histogram
                            weights[pc as usize] += 4;
                        }
                        weights[pc as usize] += 6;
                    } else {
                        weights[pc as usize] += 1;
                    }
                }
            }
        }
    }

    let best_pc = weights
        .iter()
        .enumerate()
        .max_by_key(|(_, &w)| w)
        .filter(|(_, &w)| w > 0)
        .map(|(pc, _)| pc as u8)
        .or(first_bass_pc)
        .unwrap_or(2);

    (best_pc, mode)
}

fn extract_explicit_scale_call(code: &str) -> Option<(u8, ScaleMode)> {
    let lower_code = code.to_lowercase();
    let pos = lower_code.find(".scale(")?;
    let after = &lower_code[pos + ".scale(".len()..];
    let quote_pos = after.find(|c| c == '"' || c == '\'')?;
    let quote_char = after[quote_pos..].chars().next()?;
    let rest = &after[quote_pos + quote_char.len_utf8()..];
    let end_q = rest.find(quote_char)?;
    let scale_str = &rest[..end_q];

    let (root_s, mode_s) = scale_str.split_once(':')?;
    let pc = parse_pitch_class_only(root_s.trim())?;
    let mode = parse_mode_keyword(mode_s).unwrap_or(ScaleMode::NaturalMinor);
    Some((pc, mode))
}

fn extract_note_call_strings(line: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let mut search_idx = 0usize;

    while let Some(rel) = line[search_idx..].find("note(") {
        let start_args = search_idx + rel + "note(".len();
        let tail = &line[start_args..];
        let Some(q_rel) = tail.find(|c| c == '"' || c == '\'') else {
            break;
        };
        let quote = tail[q_rel..].chars().next().unwrap_or('"');
        let content_start = start_args + q_rel + quote.len_utf8();
        let content_tail = &line[content_start..];
        if let Some(end_rel) = content_tail.find(quote) {
            results.push(&content_tail[..end_rel]);
            search_idx = content_start + end_rel + quote.len_utf8();
        } else {
            break;
        }
    }

    results
}

/// Parses explicit key and mode requests from user prompt text (e.g. "in F# minor", "Eb dorian", "Key of C major").
pub fn infer_prompt_key_and_mode(text: &str) -> (Option<u8>, Option<ScaleMode>) {
    let lower = text.to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '#')
        .filter(|s| !s.is_empty())
        .collect();

    let mode_words = [
        "minor", "major", "dorian", "phrygian", "lydian", "aeolian", "ionian", "harmonic",
        "melodic",
    ];

    let mut detected_root: Option<u8> = None;

    for i in 0..tokens.len() {
        let candidate = tokens[i];

        // Case 1: Pitch token immediately followed by a mode keyword (e.g., "f# minor", "eb dorian", "c# harmonic minor")
        if let Some(pc) = parse_pitch_class_token_strict(candidate) {
            if let Some(&next_word) = tokens.get(i + 1) {
                if mode_words.contains(&next_word) {
                    detected_root = Some(pc);
                    break;
                }
            }
            // Case 2: Preceded by "key of <pitch>" or "in <pitch>" when unambiguously a musical key
            if i >= 2 && tokens[i - 2] == "key" && tokens[i - 1] == "of" {
                detected_root = Some(pc);
                break;
            }
            if i >= 1 && tokens[i - 1] == "key" {
                detected_root = Some(pc);
                break;
            }
        }
    }

    let detected_mode = infer_mode_from_text(&lower);
    (detected_root, detected_mode)
}

fn parse_mode_keyword(text: &str) -> Option<ScaleMode> {
    let lower = text.to_lowercase();
    if lower.contains("harmonic") {
        Some(ScaleMode::HarmonicMinor)
    } else if lower.contains("melodic") {
        Some(ScaleMode::MelodicMinor)
    } else if lower.contains("phrygian") {
        Some(ScaleMode::PhrygianDominant)
    } else if lower.contains("lydian") {
        Some(ScaleMode::Lydian)
    } else if lower.contains("dorian") {
        Some(ScaleMode::Dorian)
    } else if lower.contains("major") || lower.contains("ionian") {
        Some(ScaleMode::Major)
    } else if lower.contains("minor") || lower.contains("aeolian") {
        Some(ScaleMode::NaturalMinor)
    } else {
        None
    }
}

fn infer_mode_from_text(text: &str) -> Option<ScaleMode> {
    let lower = text.to_lowercase();
    if lower.contains("harmonic minor")
        || lower.contains("symphon")
        || lower.contains("orchestra")
        || lower.contains("classical")
        || lower.contains("baroque")
        || lower.contains("requiem")
        || lower.contains("choir")
    {
        Some(ScaleMode::HarmonicMinor)
    } else if lower.contains("phrygian")
        || lower.contains("dubstep")
        || lower.contains("growl")
        || lower.contains("industrial")
        || lower.contains("boss")
    {
        Some(ScaleMode::PhrygianDominant)
    } else if lower.contains("lydian")
        || lower.contains("astral")
        || lower.contains("celestial")
        || lower.contains("cosmic")
        || lower.contains("dream")
    {
        Some(ScaleMode::Lydian)
    } else if lower.contains("dorian")
        || lower.contains("daft")
        || lower.contains("french house")
        || lower.contains("funk")
        || lower.contains("deep house")
        || lower.contains("groove")
    {
        Some(ScaleMode::Dorian)
    } else if lower.contains("melodic minor") || lower.contains("jazz") {
        Some(ScaleMode::MelodicMinor)
    } else if lower.contains("major")
        || lower.contains("ionian")
        || lower.contains("uplifting")
        || lower.contains("triumphant")
        || lower.contains("euphoric")
    {
        Some(ScaleMode::Major)
    } else if lower.contains("natural minor")
        || lower.contains("aeolian")
        || lower.contains("minor")
    {
        Some(ScaleMode::NaturalMinor)
    } else {
        None
    }
}

fn parse_pitch_class_token_strict(s: &str) -> Option<u8> {
    if s.is_empty() || s.len() > 2 {
        return None;
    }
    let bytes = s.as_bytes();
    let base: i32 = match (bytes[0] as char).to_ascii_lowercase() {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return None,
    };
    if bytes.len() == 1 {
        return Some(base as u8);
    }
    match (bytes[1] as char).to_ascii_lowercase() {
        's' | '#' => Some(((base + 1) % 12) as u8),
        'b' => Some(((base + 11) % 12) as u8),
        _ => None,
    }
}

fn parse_pitch_class_only(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let base: i32 = match (bytes[0] as char).to_ascii_lowercase() {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return None,
    };
    if bytes.len() >= 2 {
        match (bytes[1] as char).to_ascii_lowercase() {
            's' | '#' => return Some(((base + 1) % 12) as u8),
            'b' => return Some(((base + 11) % 12) as u8),
            _ => {}
        }
    }
    Some(base as u8)
}

pub fn parse_pitch_token(token: &str) -> Option<(u8, i32, bool)> {
    let core = token
        .split(|c| c == '!' || c == '*' || c == '@' || c == '/')
        .next()
        .unwrap_or(token);
    let bytes = core.as_bytes();
    if bytes.len() < 2 || bytes.len() > 3 {
        return None;
    }
    let letter = (bytes[0] as char).to_ascii_lowercase();
    let base_pc: i32 = match letter {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return None,
    };

    if bytes.len() == 2 {
        let oct_ch = bytes[1] as char;
        if oct_ch.is_ascii_digit() {
            let oct = (oct_ch as u8 - b'0') as i32;
            return Some((base_pc as u8, oct, false));
        }
        return None;
    }

    let acc = (bytes[1] as char).to_ascii_lowercase();
    let oct_ch = bytes[2] as char;
    if !oct_ch.is_ascii_digit() {
        return None;
    }
    let oct = (oct_ch as u8 - b'0') as i32;
    match acc {
        's' | '#' => Some((((base_pc + 1) % 12) as u8, oct, true)),
        'b' => Some((((base_pc + 11) % 12) as u8, oct, false)),
        _ => None,
    }
}

/// Determines whether a pitch class should be spelled with sharps (`cs`, `fs`) or flats (`db`, `eb`)
/// based on both the tonic pitch class and the active `ScaleMode`.
fn use_sharps_for_scale(pc: u8, tonic_pc: u8, mode: ScaleMode) -> bool {
    let t = tonic_pc % 12;
    let p = pc % 12;

    // Raised 7th leading tone in Harmonic Minor / Melodic Minor (e.g. F# in G minor, C# in D minor)
    if matches!(mode, ScaleMode::HarmonicMinor | ScaleMode::MelodicMinor)
        && p == (t + 11) % 12
        && matches!(p, 1 | 3 | 6 | 8 | 10)
    {
        return true;
    }

    match mode {
        ScaleMode::Major | ScaleMode::Lydian => {
            // Major/Lydian keys using sharps: C(0 -> Lydian F#), G(7), D(2), A(9), E(4), B(11), F#(6), C#(1)
            matches!(t, 0 | 7 | 2 | 9 | 4 | 11 | 6 | 1)
        }
        ScaleMode::Dorian => {
            // Dorian keys using sharps: A(9), E(4), B(11), F#(6), C#(1), G#(8)
            matches!(t, 9 | 4 | 11 | 6 | 1 | 8)
        }
        ScaleMode::NaturalMinor | ScaleMode::HarmonicMinor | ScaleMode::MelodicMinor => {
            // Minor keys using sharps: E(4), B(11), F#(6), C#(1), G#(8), D#(3)
            matches!(t, 4 | 11 | 6 | 1 | 8 | 3)
        }
        ScaleMode::PhrygianDominant => {
            // Phrygian Dominant has a flat-2 and major-3; sharp tonics use sharps
            matches!(t, 4 | 11 | 6 | 1 | 8 | 3 | 9)
        }
    }
}

pub fn format_pitch_class_in_mode(pc: u8, tonic_pc: u8, mode: ScaleMode, oct: i32) -> String {
    let use_sharps = use_sharps_for_scale(pc, tonic_pc, mode);
    let name = match (pc % 12, use_sharps) {
        (0, _) => "c",
        (1, true) => "cs",
        (1, false) => "db",
        (2, _) => "d",
        (3, true) => "ds",
        (3, false) => "eb",
        (4, _) => "e",
        (5, _) => "f",
        (6, true) => "fs",
        (6, false) => "gb",
        (7, _) => "g",
        (8, true) => "gs",
        (8, false) => "ab",
        (9, _) => "a",
        (10, true) => "as",
        (10, false) => "bb",
        (11, _) => "b",
        _ => "c",
    };
    format!("{}{}", name, oct.clamp(0, 7))
}

fn build_tertian_chord_from_root(
    root_pc: u8,
    root_oct: i32,
    num_voices: usize,
    tonic_pc: u8,
    mode: ScaleMode,
) -> String {
    let intervals = mode.intervals();
    let rel = (root_pc + 12 - (tonic_pc % 12)) % 12;

    let root_deg_idx = intervals
        .iter()
        .enumerate()
        .min_by_key(|(_, &deg)| {
            let diff = (deg as i32 - rel as i32).abs();
            diff.min(12 - diff)
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let mut voices = Vec::with_capacity(num_voices);
    let base_abs_semitone = root_oct * 12 + (tonic_pc as i32) + (intervals[root_deg_idx] as i32);

    for v in 0..num_voices.clamp(3, 5) {
        let deg_offset = v * 2;
        let deg_idx = (root_deg_idx + deg_offset) % 7;
        let octave_carry = ((root_deg_idx + deg_offset) / 7) as i32;
        let abs_semi =
            root_oct * 12 + (tonic_pc as i32) + (intervals[deg_idx] as i32) + octave_carry * 12;
        let safe_semi = abs_semi.max(base_abs_semitone);
        let v_oct = (safe_semi / 12).clamp(2, 6);
        let v_pc = (safe_semi.rem_euclid(12)) as u8;
        voices.push(format_pitch_class_in_mode(v_pc, tonic_pc, mode, v_oct));
    }

    format!("[{}]", voices.join(","))
}

/// Optional Harmonic & Architectural Context:
/// Respects explicit user key/mode requests first, falls back to seed entropy when unstated,
/// and never forces fixed section counts, slider quotas, or visualizer checklists.
pub fn build_orchestration_directive(
    user_prompt: &str,
    style_ref: &str,
    existing_code: Option<&str>,
    evo_gen: u32,
    seed: u64,
) -> String {
    let combined = format!("{} {}", user_prompt, style_ref);
    let (tonic_pc, mode) = if let Some(code) = existing_code {
        detect_scale_context(code, &combined)
    } else {
        let (prompt_root, prompt_mode) = infer_prompt_key_and_mode(&combined);
        let root = prompt_root.unwrap_or_else(|| (seed % 12) as u8);
        let all_modes = [
            ScaleMode::NaturalMinor,
            ScaleMode::HarmonicMinor,
            ScaleMode::Dorian,
            ScaleMode::PhrygianDominant,
            ScaleMode::Lydian,
            ScaleMode::Major,
            ScaleMode::MelodicMinor,
        ];
        let final_mode =
            prompt_mode.unwrap_or_else(|| all_modes[((seed / 13) as usize) % all_modes.len()]);
        (root, final_mode)
    };

    let scale_notes = build_scale_note_list(tonic_pc, mode);
    let degree_table = build_diatonic_degree_table(tonic_pc, mode);
    let root_name = format_pitch_class_in_mode(tonic_pc, tonic_pc, mode, 1);
    let key_label = format!(
        "{} {}",
        root_name[..root_name.len() - 1].to_uppercase(),
        mode.name()
    );

    if let Some(current) = existing_code {
        let current_triggers = count_triggers(current);
        format!(
            "EVOLUTION CONTEXT & OPTIONAL HARMONIC PALETTE (Generation #{} | Active Triggers: {})\n\
             - DETECTED TONAL CENTER: **{}** (Reference pitch classes: `{}`)\n\
             - OPTIONAL DIATONIC CHORD PALETTE (Use commas inside `[...]` for simultaneous chords): `{}`\n\
             - HARMONIC & STRUCTURAL FREEDOM: Preserve the core musical identity of the score where appropriate, or introduce chromaticism, modal mixture, key modulations, polyrhythms, or structural transformations as the user's evolution directive demands.",
            evo_gen, current_triggers, key_label, scale_notes, degree_table
        )
    } else {
        format!(
            "OPTIONAL HARMONIC & ARCHITECTURAL RESOURCES (Entropy Seed #{})\n\
             - SUGGESTED TONAL CENTER (Optional starting point unless overridden by prompt): **{}**\n\
             - REFERENCE SCALE PITCH CLASSES: `{}` (Chromatic passing tones, borrowed chords, modal shifts, and microtonal detuning are welcome whenever musically appropriate).\n\
             - OPTIONAL TERTIAN CHORD PALETTE (Use commas inside `[...]` when voicing simultaneous chords): `{}`\n\
             - FORM FOLLOWS MUSICAL INTENT: Choose the structure that genuinely fits the brief—whether that is a multi-section `arrange([cycles, section], ...)` journey, a continuous evolving `stack(...)` groove, a sparse ambient soundscape, or an asymmetric polyrhythmic study.",
            seed % 10000,
            key_label,
            scale_notes,
            degree_table
        )
    }
}

fn build_scale_note_list(tonic_pc: u8, mode: ScaleMode) -> String {
    let intervals = mode.intervals();
    let names: Vec<String> = intervals
        .iter()
        .map(|&deg| {
            let s = format_pitch_class_in_mode((tonic_pc + deg) % 12, tonic_pc, mode, 3);
            s[..s.len() - 1].to_string()
        })
        .collect();
    names.join(", ")
}

fn build_diatonic_degree_table(tonic_pc: u8, mode: ScaleMode) -> String {
    let intervals = mode.intervals();
    let roman = ["I", "II", "III", "IV", "V", "VI", "VII"];
    let entries: Vec<String> = (0..7)
        .map(|d| {
            let root_pc = (tonic_pc + intervals[d]) % 12;
            let chord = build_tertian_chord_from_root(root_pc, 3, 4, tonic_pc, mode);
            format!("{}: {}", roman[d], chord)
        })
        .collect();
    entries.join(" | ")
}

fn count_triggers(code: &str) -> usize {
    code.lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("//")
                && (t.contains("s(")
                    || t.contains("sound(")
                    || t.contains("note(")
                    || t.contains("n("))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_prompt_key_and_mode_are_respected() {
        let (root, mode) = infer_prompt_key_and_mode("Compose a dark synthwave track in F# minor");
        assert_eq!(root, Some(6));
        assert_eq!(mode, Some(ScaleMode::NaturalMinor));

        let directive = build_orchestration_directive(
            "Compose a dark synthwave track in F# minor",
            "",
            None,
            0,
            999,
        );
        assert!(directive.contains("FS Natural Minor (Aeolian)"));
    }

    #[test]
    fn d_minor_uses_flat_spelling_while_g_major_uses_sharp_spelling() {
        let d_minor_notes = build_scale_note_list(2, ScaleMode::NaturalMinor);
        assert_eq!(d_minor_notes, "d, e, f, g, a, bb, c");

        let g_major_notes = build_scale_note_list(7, ScaleMode::Major);
        assert_eq!(g_major_notes, "g, a, b, c, d, e, fs");
    }

    #[test]
    fn histogram_outweighs_single_passing_bass_note() {
        let code = "setcps(0.5)\n\
                    note('g2 c2 c2 c2 c2').s('sawtooth')\n\
                    note(\"c4 eb4 g4 c5\").s(\"supersaw\")";
        let (pc, _) = detect_scale_context(code, "");
        assert_eq!(pc, 0); // C (0) outweighs a single opening G2 (7)
    }
}