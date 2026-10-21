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

/// Detects the dominant tonic pitch class (0..11) and scale mode from the code and user prompt.
/// Weights low-octave bass notes (octaves 0..=2) 6x higher than upper voices so a passing melody
/// note does not override the piece's tonal center.
pub fn detect_scale_context(code: &str, prompt_hint: &str) -> (u8, ScaleMode) {
    let lower_code = code.to_lowercase();
    let lower_hint = prompt_hint.to_lowercase();

    // 1. Check if code explicitly declares `.scale("root:mode")`
    if let Some(pos) = lower_code.find(".scale(\"") {
        let after = &lower_code[pos + 8..];
        if let Some(end_q) = after.find('"') {
            let scale_str = &after[..end_q];
            if let Some((root_s, mode_s)) = scale_str.split_once(':') {
                let pc = parse_pitch_class_only(root_s.trim()).unwrap_or(2);
                let mode = if mode_s.contains("harmonic") {
                    ScaleMode::HarmonicMinor
                } else if mode_s.contains("dorian") {
                    ScaleMode::Dorian
                } else if mode_s.contains("phrygian") {
                    ScaleMode::PhrygianDominant
                } else if mode_s.contains("lydian") {
                    ScaleMode::Lydian
                } else if mode_s.contains("melodic") {
                    ScaleMode::MelodicMinor
                } else if mode_s.contains("major") || mode_s.contains("ionian") {
                    ScaleMode::Major
                } else {
                    ScaleMode::NaturalMinor
                };
                return (pc, mode);
            }
        }
    }

    // 2. Infer mode from prompt or comments inside code
    let combined_text = format!("{} {}", lower_hint, lower_code);
    let mode = if combined_text.contains("harmonic minor")
        || combined_text.contains("symphon")
        || combined_text.contains("orchestra")
        || combined_text.contains("classical")
        || combined_text.contains("baroque")
        || combined_text.contains("requiem")
        || combined_text.contains("choir")
    {
        ScaleMode::HarmonicMinor
    } else if combined_text.contains("phrygian")
        || combined_text.contains("dubstep")
        || combined_text.contains("growl")
        || combined_text.contains("industrial")
        || combined_text.contains("boss")
    {
        ScaleMode::PhrygianDominant
    } else if combined_text.contains("lydian")
        || combined_text.contains("astral")
        || combined_text.contains("celestial")
        || combined_text.contains("cosmic")
        || combined_text.contains("dream")
    {
        ScaleMode::Lydian
    } else if combined_text.contains("dorian")
        || combined_text.contains("daft")
        || combined_text.contains("french house")
        || combined_text.contains("funk")
        || combined_text.contains("deep house")
        || combined_text.contains("groove")
    {
        ScaleMode::Dorian
    } else if combined_text.contains("melodic minor") || combined_text.contains("jazz") {
        ScaleMode::MelodicMinor
    } else if combined_text.contains("major")
        || combined_text.contains("uplifting")
        || combined_text.contains("triumphant")
        || combined_text.contains("euphoric")
    {
        ScaleMode::Major
    } else {
        ScaleMode::NaturalMinor
    };

    // 3. Weighted histogram of pitch classes in `note("...")`, favoring bass octaves (0, 1, 2)
    let mut weights = [0u32; 12];
    let mut first_bass_pc: Option<u8> = None;

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.contains("s(\"cb\")") {
            continue;
        }
        let mut search_idx = 0;
        while let Some(pos) = trimmed[search_idx..].find("note(\"") {
            let start = search_idx + pos + 6;
            if let Some(end_q) = trimmed[start..].find('"') {
                let note_str = &trimmed[start..start + end_q];
                for token in note_str.split(|c: char| !c.is_ascii_alphanumeric() && c != '#') {
                    if let Some((pc, oct, _)) = parse_pitch_token(token) {
                        if oct <= 2 {
                            if first_bass_pc.is_none() {
                                first_bass_pc = Some(pc);
                            }
                            weights[pc as usize] += 6;
                        } else {
                            weights[pc as usize] += 1;
                        }
                    }
                }
                search_idx = start + end_q + 1;
            } else {
                break;
            }
        }
    }

    if let Some(bass_pc) = first_bass_pc {
        return (bass_pc, mode);
    }

    let best_pc = weights
        .iter()
        .enumerate()
        .max_by_key(|(_, &w)| w)
        .filter(|(_, &w)| w > 0)
        .map(|(pc, _)| pc as u8)
        .unwrap_or(2);

    (best_pc, mode)
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

pub fn format_pitch_class(pc: u8, tonic_pc: u8, oct: i32) -> String {
    let use_sharps = matches!(tonic_pc % 12, 4 | 6 | 9 | 11 | 1 | 2);
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

/// Normalizes `#` accidentals into Strudel-safe `s` spelling (e.g. `f#3!2` -> `fs3!2`)
/// WITHOUT snapping or altering the composer's chosen pitch class!
pub fn normalize_pitch_spelling(token: &str, tonic_pc: u8) -> String {
    for sep in ['!', '*', '@', '/'] {
        if let Some(idx) = token.find(sep) {
            let (note_part, suffix) = token.split_at(idx);
            return format!("{}{}", normalize_pitch_spelling(note_part, tonic_pc), suffix);
        }
    }

    let Some((pc, oct, _)) = parse_pitch_token(token) else {
        return token.to_string();
    };

    format_pitch_class(pc, tonic_pc, oct)
}

/// Quantizes a single note token to `(tonic_pc, mode)` when explicit diatonic snapping is requested.
pub fn quantize_pitch_token(token: &str, tonic_pc: u8, mode: ScaleMode) -> String {
    for sep in ['!', '*', '@', '/'] {
        if let Some(idx) = token.find(sep) {
            let (note_part, suffix) = token.split_at(idx);
            return format!(
                "{}{}",
                quantize_pitch_token(note_part, tonic_pc, mode),
                suffix
            );
        }
    }

    let Some((pc, oct, _)) = parse_pitch_token(token) else {
        return token.to_string();
    };

    let intervals = mode.intervals();
    let rel = (pc + 12 - (tonic_pc % 12)) % 12;

    let snapped_rel = if intervals.contains(&rel) {
        rel
    } else {
        intervals
            .iter()
            .copied()
            .min_by_key(|&deg| {
                let diff = (deg as i32 - rel as i32).abs();
                diff.min(12 - diff)
            })
            .unwrap_or(0)
    };

    let final_pc = (tonic_pc + snapped_rel) % 12;
    format_pitch_class(final_pc, tonic_pc, oct)
}

/// Given a root note `(pc, oct)`, builds a tertian chord (`[root, 3rd, 5th, 7th]`)
/// inside `(tonic_pc, mode)` to rescue accidental semitone cluster collisions like `[a3,b3,c4,d4]`.
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
        voices.push(format_pitch_class(v_pc, tonic_pc, v_oct));
    }

    format!("[{}]", voices.join(","))
}

/// Non-destructive note & chord healer:
/// 1. Repairs accidental 3+ note semitone clusters (`[a3,b3,c4,d4]`) into playable tertian chords.
/// 2. Converts space-separated block chords (`[c3 eb3 g3 bb3]`) on slow pad lines into comma-separated
///    chords (`[c3,eb3,g3,bb3]`) without altering their pitch classes.
/// 3. Preserves all intentional chromaticism, borrowed chords, blues notes, and modal inflections,
///    normalizing only `#` syntax into Strudel-safe `s` notation.
pub fn harmonize_and_voice_note_string(
    mini: &str,
    tonic_pc: u8,
    mode: ScaleMode,
    is_chord_pad_line: bool,
) -> String {
    let mut bracket_healed = String::with_capacity(mini.len());
    let chars: Vec<char> = mini.chars().collect();
    let mut idx = 0;

    while idx < chars.len() {
        if chars[idx] == '[' {
            let mut close = idx + 1;
            while close < chars.len() && chars[close] != ']' && chars[close] != '[' {
                close += 1;
            }
            if close < chars.len() && chars[close] == ']' {
                let inside: String = chars[idx + 1..close].iter().collect();

                // Case A: Comma-separated chord `[n1,n2,n3,n4]`
                if inside.contains(',') {
                    let tokens: Vec<&str> = inside.split(',').map(|s| s.trim()).collect();
                    let parsed: Vec<(u8, i32, bool)> =
                        tokens.iter().filter_map(|t| parse_pitch_token(t)).collect();

                    if parsed.len() >= 3 && parsed.len() == tokens.len() {
                        if is_stepwise_cluster(&parsed) {
                            let (r_pc, r_oct, _) = parsed[0];
                            let healed_chord = build_tertian_chord_from_root(
                                r_pc,
                                r_oct,
                                parsed.len(),
                                tonic_pc,
                                mode,
                            );
                            bracket_healed.push_str(&healed_chord);
                            idx = close + 1;
                            continue;
                        } else {
                            // Preserve exact chord pitches (including chromatic/borrowed voicings), normalizing only `#` -> `s`
                            let spelled: Vec<String> = tokens
                                .iter()
                                .map(|t| normalize_pitch_spelling(t, tonic_pc))
                                .collect();
                            bracket_healed.push('[');
                            bracket_healed.push_str(&spelled.join(","));
                            bracket_healed.push(']');
                            idx = close + 1;
                            continue;
                        }
                    }
                } else {
                    // Case B: Space-separated group `[n1 n2 n3 n4]` on a slow chord/pad line
                    let tokens: Vec<&str> = inside.split_whitespace().collect();
                    let parsed: Vec<(u8, i32, bool)> =
                        tokens.iter().filter_map(|t| parse_pitch_token(t)).collect();

                    if parsed.len() >= 3
                        && parsed.len() <= 5
                        && parsed.len() == tokens.len()
                        && is_chord_pad_line
                        && is_tertian_chord_spacing(&parsed)
                    {
                        let spelled: Vec<String> = tokens
                            .iter()
                            .map(|t| normalize_pitch_spelling(t, tonic_pc))
                            .collect();
                        bracket_healed.push('[');
                        bracket_healed.push_str(&spelled.join(","));
                        bracket_healed.push(']');
                        idx = close + 1;
                        continue;
                    }
                }
            }
        }
        bracket_healed.push(chars[idx]);
        idx += 1;
    }

    // Normalize `#` pitch spelling on individual tokens while preserving all chromatic/modal note choices
    let mut final_str = String::with_capacity(bracket_healed.len());
    let mut token_buf = String::new();

    for ch in bracket_healed.chars() {
        if ch.is_ascii_alphanumeric()
            || ch == '#'
            || ch == '!'
            || ch == '*'
            || ch == '@'
            || ch == '.'
        {
            token_buf.push(ch);
        } else {
            if !token_buf.is_empty() {
                final_str.push_str(&normalize_pitch_spelling(&token_buf, tonic_pc));
                token_buf.clear();
            }
            final_str.push(ch);
        }
    }
    if !token_buf.is_empty() {
        final_str.push_str(&normalize_pitch_spelling(&token_buf, tonic_pc));
    }

    final_str
}

/// Returns true if 3+ notes in a simultaneous chord are crammed within 1–2 semitones (e.g. `[a3,b3,c4,d4]`).
fn is_stepwise_cluster(notes: &[(u8, i32, bool)]) -> bool {
    let mut semis: Vec<i32> = notes.iter().map(|&(pc, oct, _)| oct * 12 + pc as i32).collect();
    semis.sort_unstable();
    let mut narrow_steps = 0;
    for w in semis.windows(2) {
        let diff = (w[1] - w[0]).abs();
        if diff <= 2 {
            narrow_steps += 1;
        }
    }
    narrow_steps >= 2
}

/// Returns true ONLY if space-separated notes are strictly ascending in 3rd/4th intervals (3..=5 semitones)
/// in octaves 2..=5, ensuring melodic phrases and arpeggios with leaps or contour changes stay melodies.
fn is_tertian_chord_spacing(notes: &[(u8, i32, bool)]) -> bool {
    let semis: Vec<i32> = notes.iter().map(|&(pc, oct, _)| oct * 12 + pc as i32).collect();
    if semis.iter().any(|&s| s < 24 || s > 72) {
        return false;
    }
    for w in semis.windows(2) {
        let diff = w[1] - w[0];
        if !(3..=5).contains(&diff) {
            return false;
        }
    }
    true
}

/// Checks if a line's `note("...")` pattern contains low bass octaves (0, 1, or 2).
fn line_contains_bass_octaves(line: &str) -> bool {
    if let Some(pos) = line.find("note(\"") {
        let after = &line[pos + 6..];
        if let Some(end_q) = after.find('"') {
            let note_str = &after[..end_q];
            for token in note_str.split(|c: char| !c.is_ascii_alphanumeric() && c != '#') {
                if let Some((_, oct, _)) = parse_pitch_token(token) {
                    if oct <= 2 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Studio Auto-Mixer & Orbit Separator:
/// - Assigns distinct `.orbit(1..6)` to reverb/delay layers so multi-layer scores never turn into mud.
/// - Injects `.hpf(180)` on upper chord/pad/lead/vocal layers that have no high-pass filter so low-mids stay clean.
/// - Scales `.gain(...)` intelligently, accounting for whether the score uses sequential `arrange(...)` movements.
pub fn auto_mix_and_carve_frequencies(code: &str) -> String {
    let uses_arrange = code.contains("arrange(");
    let total_triggers = code
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("//")
                && (t.contains("s(\"") || t.contains("note(\"") || t.contains("n(\""))
        })
        .count();

    let gain_scale = if uses_arrange {
        if total_triggers >= 18 {
            0.88
        } else {
            0.95
        }
    } else if total_triggers >= 9 {
        0.76
    } else if total_triggers >= 6 {
        0.85
    } else {
        1.0
    };

    let mut orbit_counter = 1usize;
    let mut out_lines = Vec::new();

    for raw_line in code.lines() {
        let mut line = raw_line.to_string();
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.is_empty() {
            out_lines.push(line);
            continue;
        }

        // 1. Isolate reverb/delay buses with `.orbit(1..6)` if not already set
        if (line.contains(".room(") || line.contains(".delay(")) && !line.contains(".orbit(") {
            let orb = (orbit_counter % 6) + 1;
            orbit_counter += 1;
            if let Some(pos) = line.find(".room(").or_else(|| line.find(".delay(")) {
                line.insert_str(pos, &format!(".orbit({})", orb));
            }
        }

        // 2. Carve muddy low frequencies from upper supersaw/pad/vocal/lead layers (excluding bass octaves 0..=2)
        let is_upper_harmonic = (line.contains("\"supersaw\"")
            || line.contains(".vowel(")
            || line.contains("\"triangle\"")
            || line.contains("\"pulse\"")
            || line.contains("\"square\""))
            && !line_contains_bass_octaves(&line)
            && !line.contains(".hpf(");

        if is_upper_harmonic {
            if let Some(pos) = line.find(".lpf(") {
                line.insert_str(pos, ".hpf(180)");
            } else if let Some(pos) = line.find(".gain(") {
                line.insert_str(pos, ".hpf(180)");
            }
        }

        // 3. Scale numeric `.gain(val)` when high simultaneous layer counts are active
        if gain_scale < 0.99 {
            if let Some(pos) = line.find(".gain(") {
                let start = pos + 6;
                if let Some(end_rel) = line[start..].find(')') {
                    let arg = line[start..start + end_rel].trim();
                    if let Ok(val) = arg.parse::<f32>() {
                        let scaled = (val * gain_scale).clamp(0.14, 0.92);
                        line.replace_range(start..start + end_rel, &format!("{:.2}", scaled));
                    }
                }
            }
        }

        out_lines.push(line);
    }

    out_lines.join("\n")
}

/// Optional Harmonic & Architectural Context:
/// Supplies a suggested tonal center and chord palette for inspiration or continuity without
/// forcing diatonic conformity, fixed section counts, or mandatory slider/visualizer quotas.
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
        let roots: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let root = roots[(seed as usize) % roots.len()];
        let (_, inferred_mode) = detect_scale_context("", &combined);
        let all_modes = [
            ScaleMode::NaturalMinor,
            ScaleMode::HarmonicMinor,
            ScaleMode::Dorian,
            ScaleMode::PhrygianDominant,
            ScaleMode::Lydian,
            ScaleMode::Major,
            ScaleMode::MelodicMinor,
        ];
        let final_mode = if inferred_mode == ScaleMode::NaturalMinor && (seed % 3 != 0) {
            all_modes[((seed / 13) as usize) % all_modes.len()]
        } else {
            inferred_mode
        };
        (root, final_mode)
    };

    let scale_notes = build_scale_note_list(tonic_pc, mode);
    let degree_table = build_diatonic_degree_table(tonic_pc, mode);
    let root_name = format_pitch_class(tonic_pc, tonic_pc, 1);
    let key_label = format!(
        "{} {}",
        root_name[..root_name.len() - 1].to_uppercase(),
        mode.name()
    );

    if let Some(current) = existing_code {
        let current_sections = count_sections_and_triggers(current);
        format!(
            "EVOLUTION CONTEXT & OPTIONAL HARMONIC PALETTE (Generation #{} | Active Triggers: {})\n\
             - DETECTED TONAL CENTER: **{}** (Reference pitch classes: `{}`)\n\
             - OPTIONAL DIATONIC CHORD PALETTE (Use commas inside `[...]` for simultaneous chords): `{}`\n\
             - HARMONIC & STRUCTURAL FREEDOM: Preserve the core musical identity of the score where appropriate, or introduce chromaticism, modal mixture, key modulations, polyrhythms, new movements, or structural transformations as the user's evolution directive demands.",
            evo_gen, current_sections, key_label, scale_notes, degree_table
        )
    } else {
        format!(
            "OPTIONAL HARMONIC & ARCHITECTURAL RESOURCES (Entropy Seed #{})\n\
             - SUGGESTED TONAL CENTER (Optional starting point unless overridden by prompt): **{}**\n\
             - REFERENCE SCALE PITCH CLASSES: `{}` (Chromatic passing tones, borrowed chords, modal shifts, and microtonal detuning are welcome whenever musically appropriate).\n\
             - OPTIONAL TERTIAN CHORD PALETTE (Use commas inside `[...]` when voicing simultaneous chords): `{}`\n\
             - FORM FOLLOWS MUSICAL INTENT:\n\
               * Choose the structure that genuinely fits the brief—whether that is a multi-movement `arrange([cycles, section], ...)` journey, a continuous evolving `stack(...)` groove using `.every(...)` / `.mask(...)` / `.struct(...)`, an ambient soundscape with no drums, or an asymmetric polyrhythmic study.\n\
               * Include top-level `const` `slider(...)` macros and inline visualizers (`._scope()`, `._pianoroll()`, `._punchcard()`, `._spiral()`) when they enhance the piece or when requested by the user, rather than forcing a fixed quota.",
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
            let s = format_pitch_class((tonic_pc + deg) % 12, tonic_pc, 3);
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

fn count_sections_and_triggers(code: &str) -> usize {
    code.lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("//")
                && (t.contains("s(\"") || t.contains("note(\"") || t.contains("n(\""))
        })
        .count()
}