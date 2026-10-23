//! MotifAI Master Composition & Performance Palette Archive selector.
//!
//! Selects one flexible architecture seed per composition/evolution request, then
//! chooses a small relevance-ranked set of instrument, production, and visual palettes.
//! The caller should call `select_for_request_with_override` once before the retry loop and
//! reuse the returned `prompt_block` in each fresh attempt prompt. Nothing here generates Strudel.

use serde::Deserialize;
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

const RELATIVE_PATH: &str = "knowledge/composition_blueprints.json";
const PATH_ENV: &str = "MOTIFAI_COMPOSITION_BLUEPRINTS";
const MODE_ENV: &str = "MOTIFAI_COMPOSITION_BLUEPRINT";
const MAX_LIBRARY_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PROMPT_CHARS: usize = 14_000;
const MAX_DEBUG_LOG_BYTES: u64 = 4 * 1024 * 1024;

static CACHED_BLUEPRINTS: OnceLock<Mutex<Option<CachedBlueprintLibrary>>> = OnceLock::new();

#[derive(Debug, Clone)]
struct CachedBlueprintLibrary {
    path: PathBuf,
    modified: Option<SystemTime>,
    len: u64,
    library: BlueprintLibrary,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct BlueprintLibrary {
    schema_version: u64,
    library_name: String,
    library_version: String,
    blueprints: Vec<CompositionBlueprint>,
    instrument_palettes: Vec<InstrumentPalette>,
    visual_palettes: Vec<VisualPalette>,
    production_palettes: Vec<ProductionPalette>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct CompositionBlueprint {
    id: String,
    name: String,
    scale: String,
    relevance_tags: Vec<String>,
    selection_keywords: Vec<String>,
    premise: String,
    arc: Vec<BlueprintStage>,
    identity_anchors: Vec<String>,
    variation_axes: Vec<String>,
    escape_conditions: Vec<String>,
    duration_guidance: String,
    instrument_palette_ids: Vec<String>,
    visual_palette_ids: Vec<String>,
    production_palette_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct BlueprintStage {
    role: String,
    intent: String,
    possible_moves: Vec<String>,
    transition_intent: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct InstrumentPalette {
    id: String,
    name: String,
    relevance_tags: Vec<String>,
    selection_keywords: Vec<String>,
    role_guidance: String,
    source_guidance: String,
    arrangement_moves: Vec<String>,
    visual_palette_ids: Vec<String>,
    compatibility: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct VisualPalette {
    id: String,
    name: String,
    relevance_tags: Vec<String>,
    selection_keywords: Vec<String>,
    visual_call: String,
    target_roles: Vec<String>,
    when_to_use: String,
    placement_guidance: String,
    verification_status: String,
    requires_extension: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct ProductionPalette {
    id: String,
    name: String,
    relevance_tags: Vec<String>,
    selection_keywords: Vec<String>,
    principle: String,
    moves: Vec<String>,
    caveat: String,
}

#[derive(Debug, Clone)]
pub struct SelectedBlueprint {
    pub id: String,
    pub name: String,
    pub prompt_block: String,
}

/// Variant used by the TUI's blueprint picker. `selection_override` may be `auto`, `off`,
/// or a concrete blueprint ID; `None` reads MOTIFAI_COMPOSITION_BLUEPRINT from the environment.
pub fn select_for_request_with_override(
    user_vision: &str,
    style_ref: &str,
    selection_override: Option<&str>,
) -> Result<Option<SelectedBlueprint>, String> {
    let mode_owned = selection_override
        .map(str::to_owned)
        .or_else(|| env::var(MODE_ENV).ok())
        .unwrap_or_else(|| "auto".to_owned());
    let mode = mode_owned.trim();
    if mode.eq_ignore_ascii_case("off") || mode.eq_ignore_ascii_case("none") {
        return Ok(None);
    }

    let request = format!("{}\n{}", user_vision, style_ref);
    let lower = request.to_lowercase();
    let prompt_id = explicit_blueprint_id(&lower);
    let forced_id = if mode.is_empty() || mode.eq_ignore_ascii_case("auto") {
        prompt_id
    } else {
        Some(mode.to_owned())
    };

    let freeform_opt_out = [
        "no blueprint",
        "without a blueprint",
        "free-form only",
        "freeform only",
        "no template",
        "no arrangement seed",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase));

    if forced_id.is_none() && freeform_opt_out {
        log_selection("opt-out", "Prompt explicitly requested free-form composition.");
        return Ok(None);
    }

    // When Smart Auto is active and no explicit blueprint ID was requested, step
    // aside during narrow targeted stem edits or technical syntax repair requests
    // so a 14,000-character architecture block does not override a local edit.
    if forced_id.is_none() && is_targeted_edit_or_repair_request(&user_vision.to_lowercase()) {
        log_selection(
            "targeted-or-repair-skip",
            "Skipped auto-blueprint injection for narrow targeted edit or syntax repair request.",
        );
        return Ok(None);
    }

    let library = load_library()?;
    if library.schema_version > 2 {
        return Err(format!(
            "Composition blueprint schema {} is newer than reviewed schema 2; refusing to guess field semantics.",
            library.schema_version
        ));
    }
    if library.blueprints.is_empty() {
        return Err("Composition blueprint library contains no blueprints.".to_owned());
    }

    let selected = if let Some(id) = forced_id {
        let id_norm = normalize_id(&id);
        library
            .blueprints
            .iter()
            .find(|bp| normalize_id(&bp.id) == id_norm || normalize_id(&bp.name) == id_norm)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Requested composition blueprint '{}' was not found in {}.",
                    id, library.library_name
                )
            })?
    } else {
        choose_auto_blueprint(&library, &lower)
            .cloned()
            .ok_or_else(|| "No composition blueprint could be selected from the archive.".to_owned())?
    };

    let selected_instruments = choose_instrument_palettes(&library, &selected, &lower);
    let selected_visuals =
        choose_visual_palettes(&library, &selected, &selected_instruments, &lower);
    let selected_production = choose_production_palettes(&library, &selected, &lower);
    let prompt_block = render_archive_selection(
        &library,
        &selected,
        &selected_instruments,
        &selected_production,
        &selected_visuals,
        &lower,
    );

    Ok(Some(SelectedBlueprint {
        id: selected.id,
        name: selected.name,
        prompt_block,
    }))
}

fn is_targeted_edit_or_repair_request(prompt_lower: &str) -> bool {
    let has_broad_override = [
        "evolve",
        "evolution",
        "reimagine",
        "restructure",
        "redesign",
        "rebuild",
        "concert",
        "full set",
        "new song",
    ]
    .iter()
    .any(|phrase| prompt_lower.contains(phrase));

    if has_broad_override {
        return false;
    }

    let repair_or_narrow_phrases = [
        "syntax error",
        "parse error",
        "compile error",
        "runtime error",
        "fix the code",
        "repair the code",
        "fix the syntax",
        "repair the syntax",
        "fix the parse",
        "change only",
        "only change",
        "keep everything else",
        "keep the rest",
        "without changing",
    ];

    repair_or_narrow_phrases
        .iter()
        .any(|phrase| prompt_lower.contains(phrase))
}

/// Lightweight entries for MotifAI's Ctrl+P master archive picker.
/// The archive can also be selected with the environment override or `blueprint:<id>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintSummary {
    pub id: String,
    pub name: String,
    pub scale: String,
    pub tags: Vec<String>,
}

pub fn list_blueprints() -> Result<Vec<BlueprintSummary>, String> {
    let library = load_library()?;
    let mut items: Vec<_> = library
        .blueprints
        .into_iter()
        .map(|bp| BlueprintSummary {
            id: bp.id,
            name: bp.name,
            scale: bp.scale,
            tags: bp.relevance_tags,
        })
        .collect();
    items.sort_by(|a, b| a.scale.cmp(&b.scale).then_with(|| a.name.cmp(&b.name)));
    Ok(items)
}

/// Returns archive-family counts for the Ctrl+P browser without exposing mutable library state.
pub fn archive_summary() -> Result<String, String> {
    let library = load_library()?;
    Ok(format!(
        "{} blueprints | {} instrument palettes | {} production palettes | {} visual families",
        library.blueprints.len(),
        library.instrument_palettes.len(),
        library.production_palettes.len(),
        library.visual_palettes.len(),
    ))
}

fn explicit_blueprint_id(lower: &str) -> Option<String> {
    let start = lower.find("blueprint:")? + "blueprint:".len();
    let tail = lower[start..].trim_start();
    let id: String = tail
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    (!id.is_empty()).then_some(id)
}

fn choose_auto_blueprint<'a>(
    library: &'a BlueprintLibrary,
    request_lower: &str,
) -> Option<&'a CompositionBlueprint> {
    let requested_scale = requested_scale(request_lower);
    let mut best: Option<(&CompositionBlueprint, i32, u64)> = None;

    for bp in &library.blueprints {
        let mut score = 0i32;
        score += match_terms(&bp.selection_keywords, request_lower, 4, 8);
        score += match_terms(&bp.relevance_tags, request_lower, 2, 4);

        match requested_scale.as_str() {
            "concert" => {
                if bp.scale == "concert" {
                    score += 12;
                } else {
                    score -= 5;
                }
            }
            "extended" => {
                if bp.scale == "extended" {
                    score += 8;
                } else if bp.scale == "concert" {
                    score -= 2;
                }
            }
            "small" => {
                if bp.scale == "small" {
                    score += 7;
                } else if bp.scale == "concert" {
                    score -= 8;
                } else if bp.scale == "extended" {
                    score -= 3;
                }
            }
            _ => {
                if bp.scale == "medium" {
                    score += 3;
                }
                if bp.scale == "concert" {
                    score -= 3;
                }
            }
        }

        if contains_any(
            request_lower,
            &[
                "evolve",
                "evolution",
                "take it further",
                "reimagine",
                "transform",
                "develop the arrangement",
            ],
        ) && contains_any(
            &bp.id.to_lowercase(),
            &[
                "motif",
                "transform",
                "evolv",
                "development",
                "metamorphosis",
                "generative",
            ],
        ) {
            score += 5;
        }
        if contains_any(request_lower, &["solo", "lead break", "featured lead"])
            && contains_any(
                &bp.id.to_lowercase(),
                &["solo", "spotlight", "motif", "concert"],
            )
        {
            score += 3;
        }
        if contains_any(
            request_lower,
            &[
                "all visuals",
                "every visual",
                "visual palette",
                "visualizers",
                "visualizer",
            ],
        ) && bp.visual_palette_ids.len() >= 2
        {
            score += 1;
        }

        // Stable per-request tie breaking prevents archive order from permanently winning generic prompts.
        let tie_break = stable_hash(&format!("{}::{}", request_lower, bp.id));
        if best
            .as_ref()
            .map(|(_, current, current_tie)| {
                score > *current || (score == *current && tie_break > *current_tie)
            })
            .unwrap_or(true)
        {
            best = Some((bp, score, tie_break));
        }
    }

    best.map(|(bp, _, _)| bp)
}

fn requested_scale(text: &str) -> String {
    if contains_any(
        text,
        &[
            "concert",
            "live set",
            "dj set",
            "festival set",
            "full set",
            "full concert",
            "concept album",
            "full album",
            "electronic opera",
            "multi-movement",
            "multi movement",
        ],
    ) {
        "concert".to_owned()
    } else if contains_any(
        text,
        &[
            "extended",
            "long-form",
            "long form",
            "long composition",
            "long track",
            "extended track",
            "full-length",
            "full length",
            "suite",
            "album side",
            "multi-section",
            "multi section",
        ],
    ) {
        "extended".to_owned()
    } else if contains_any(
        text,
        &[
            "short",
            "brief",
            "small",
            "minimal",
            "lofi",
            "lo-fi",
            "study beat",
            "loop",
            "sketch",
            "simple beat",
            "quick idea",
            "compact",
        ],
    ) {
        "small".to_owned()
    } else {
        "medium".to_owned()
    }
}

fn choose_instrument_palettes<'a>(
    library: &'a BlueprintLibrary,
    blueprint: &CompositionBlueprint,
    request_lower: &str,
) -> Vec<&'a InstrumentPalette> {
    let max_items = if contains_any(
        request_lower,
        &[
            "full orchestration",
            "all instruments",
            "full instrument palette",
            "every instrument",
        ],
    ) {
        8
    } else if blueprint.scale == "concert" || requested_scale(request_lower) == "concert" {
        5
    } else {
        3
    };
    let referenced: HashSet<&str> = blueprint
        .instrument_palette_ids
        .iter()
        .map(String::as_str)
        .collect();
    let mut scored: Vec<(&InstrumentPalette, i32, u64)> = library
        .instrument_palettes
        .iter()
        .map(|palette| {
            let mut score = if referenced.contains(palette.id.as_str()) {
                12
            } else {
                0
            };
            score += match_terms(&palette.selection_keywords, request_lower, 5, 8);
            score += match_terms(&palette.relevance_tags, request_lower, 2, 3);
            let tie = stable_hash(&format!("{}::{}", request_lower, palette.id));
            (palette, score, tie)
        })
        .filter(|(_, score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));

    let mut result: Vec<&InstrumentPalette> = scored
        .into_iter()
        .take(max_items)
        .map(|(palette, _, _)| palette)
        .collect();
    // Even a minimally described blueprint should contribute a basic musical role palette.
    if result.is_empty() {
        for id in &blueprint.instrument_palette_ids {
            if let Some(palette) = library.instrument_palettes.iter().find(|p| p.id == *id) {
                if !result.iter().any(|existing| existing.id == palette.id) {
                    result.push(palette);
                }
            }
            if result.len() >= max_items {
                break;
            }
        }
    }
    result
}

fn choose_visual_palettes<'a>(
    library: &'a BlueprintLibrary,
    blueprint: &CompositionBlueprint,
    instruments: &[&InstrumentPalette],
    request_lower: &str,
) -> Vec<&'a VisualPalette> {
    let asks_for_every_visual = contains_any(
        request_lower,
        &[
            "all visuals",
            "every visual",
            "all visualizers",
            "every visualizer",
            "full visual palette",
            "all the visualizers",
            "exotic visuals",
            "every kind of visual",
            "every type of visual",
            "every kind of visualizer",
            "full visual suite",
            "all visual kinds",
            "all visual types",
        ],
    );
    let asks_for_visuals = asks_for_every_visual
        || contains_any(
            request_lower,
            &[
                "visual",
                "visualizer",
                "spectrum",
                "pianoroll",
                "punchcard",
                "scope",
                "spiral",
                "pitchwheel",
                "hydra",
                "shader",
            ],
        );
    let includes_hydra = asks_for_every_visual
        || contains_any(
            request_lower,
            &[
                "hydra",
                "shader visual",
                "audio reactive visual",
                "audio-reactive visual",
                "exotic visuals",
                "all exotic",
                "every kind of visual",
                "every type of visual",
            ],
        );
    let max_items = if asks_for_every_visual {
        11
    } else if blueprint.scale == "concert" && asks_for_visuals {
        4
    } else if asks_for_visuals {
        3
    } else {
        2
    };

    let mut referenced: HashSet<&str> = blueprint
        .visual_palette_ids
        .iter()
        .map(String::as_str)
        .collect();
    for instrument in instruments {
        for id in &instrument.visual_palette_ids {
            referenced.insert(id.as_str());
        }
    }

    let mut scored: Vec<(&VisualPalette, i32, u64)> = library
        .visual_palettes
        .iter()
        .filter(|visual| !visual.requires_extension || includes_hydra)
        .map(|visual| {
            let mut score = if referenced.contains(visual.id.as_str()) {
                10
            } else {
                0
            };
            score += match_terms(&visual.selection_keywords, request_lower, 5, 8);
            score += match_terms(&visual.relevance_tags, request_lower, 2, 3);
            if asks_for_every_visual && !visual.requires_extension {
                score += 10;
            }
            if !asks_for_visuals
                && contains_any(
                    &visual.id.to_lowercase(),
                    &["punchcard", "pianoroll", "color-highlighting"],
                )
            {
                score += 2;
            }
            if visual.requires_extension && includes_hydra {
                score += 18;
            }
            let tie = stable_hash(&format!("{}::{}", request_lower, visual.id));
            (visual, score, tie)
        })
        .filter(|(_, score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));
    scored
        .into_iter()
        .take(max_items)
        .map(|(visual, _, _)| visual)
        .collect()
}

fn choose_production_palettes<'a>(
    library: &'a BlueprintLibrary,
    blueprint: &CompositionBlueprint,
    request_lower: &str,
) -> Vec<&'a ProductionPalette> {
    let max_items = if blueprint.scale == "concert" || requested_scale(request_lower) == "concert" {
        3
    } else {
        2
    };
    let referenced: HashSet<&str> = blueprint
        .production_palette_ids
        .iter()
        .map(String::as_str)
        .collect();
    let mut scored: Vec<(&ProductionPalette, i32, u64)> = library
        .production_palettes
        .iter()
        .map(|palette| {
            let mut score = if referenced.contains(palette.id.as_str()) {
                10
            } else {
                0
            };
            score += match_terms(&palette.selection_keywords, request_lower, 5, 8);
            score += match_terms(&palette.relevance_tags, request_lower, 2, 3);
            let tie = stable_hash(&format!("{}::{}", request_lower, palette.id));
            (palette, score, tie)
        })
        .filter(|(_, score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.2.cmp(&a.2)));
    let mut result: Vec<&ProductionPalette> = scored
        .into_iter()
        .take(max_items)
        .map(|(palette, _, _)| palette)
        .collect();
    if result.is_empty() {
        for id in &blueprint.production_palette_ids {
            if let Some(palette) = library.production_palettes.iter().find(|p| p.id == *id) {
                result.push(palette);
            }
            if result.len() >= max_items {
                break;
            }
        }
    }
    result
}

fn render_archive_selection(
    library: &BlueprintLibrary,
    blueprint: &CompositionBlueprint,
    instruments: &[&InstrumentPalette],
    production: &[&ProductionPalette],
    visuals: &[&VisualPalette],
    request_lower: &str,
) -> String {
    let mut out = String::new();
    out.push_str("\n<OPTIONAL_COMPOSITION_BLUEPRINT>\n");
    out.push_str(&format!(
        "ARCHIVE: {} v{} | SCHEMA: {}\nSELECTED ARCHITECTURE: {} ({}) | SCOPE: {}\n",
        clean(&library.library_name, 120),
        clean(&library.library_version, 24),
        library.schema_version,
        clean(&blueprint.name, 160),
        clean(&blueprint.id, 100),
        clean(&blueprint.scale, 40)
    ));
    out.push_str("\nINJECTION CONTRACT: This single block is the only selected architecture/palette block for this request. It is an optional creative seed, never a source-code template or command to copy a fixed form. The same selection remains stable across technical retries. On evolution, develop the existing score using this guidance without replacing unrelated working material. The user's current brief wins.\n");
    out.push_str(&format!("\nCORE PREMISE: {}\n", clean(&blueprint.premise, 650)));
    out.push_str("\nSUGGESTED MUSICAL ARC (adapt rather than obey mechanically):\n");
    for (index, stage) in blueprint.arc.iter().take(8).enumerate() {
        out.push_str(&format!(
            "{}. {}: {}\n",
            index + 1,
            clean(&stage.role, 80),
            clean(&stage.intent, 330)
        ));
        if !stage.possible_moves.is_empty() {
            out.push_str(&format!(
                "   Possible moves: {}\n",
                format_list(&stage.possible_moves, 4, 85)
            ));
        }
        if !stage.transition_intent.trim().is_empty() {
            out.push_str(&format!(
                "   Transition intention: {}\n",
                clean(&stage.transition_intent, 180)
            ));
        }
    }
    push_list(
        &mut out,
        "Identity anchors",
        &blueprint.identity_anchors,
        6,
        140,
    );
    push_list(&mut out, "Variation axes", &blueprint.variation_axes, 8, 100);
    push_list(
        &mut out,
        "Escape conditions",
        &blueprint.escape_conditions,
        5,
        190,
    );
    if !blueprint.duration_guidance.trim().is_empty() {
        out.push_str(&format!(
            "Duration guidance: {}\n",
            clean(&blueprint.duration_guidance, 280)
        ));
    }

    if !instruments.is_empty() {
        out.push_str(
            "\nSELECTED INSTRUMENT / ROLE PALETTE (choose only roles serving this piece):\n",
        );
        for palette in instruments {
            out.push_str(&format!(
                "- {} [{}]: {}\n",
                clean(&palette.name, 90),
                clean(&palette.id, 80),
                clean(&palette.role_guidance, 260)
            ));
            out.push_str(&format!(
                "  Source safety: {}\n",
                clean(&palette.source_guidance, 260)
            ));
            if !palette.arrangement_moves.is_empty() {
                out.push_str(&format!(
                    "  Useful moves: {}\n",
                    format_list(&palette.arrangement_moves, 3, 85)
                ));
            }
            if !palette.compatibility.trim().is_empty() {
                out.push_str(&format!(
                    "  Compatibility: {}\n",
                    clean(&palette.compatibility, 180)
                ));
            }
        }
    }

    if !production.is_empty() {
        out.push_str("\nSELECTED PRODUCTION PALETTE:\n");
        for palette in production {
            out.push_str(&format!(
                "- {}: {}\n",
                clean(&palette.name, 90),
                clean(&palette.principle, 260)
            ));
            if !palette.moves.is_empty() {
                out.push_str(&format!("  Moves: {}\n", format_list(&palette.moves, 3, 90)));
            }
            if !palette.caveat.trim().is_empty() {
                out.push_str(&format!("  Guardrail: {}\n", clean(&palette.caveat, 180)));
            }
        }
    }

    if !visuals.is_empty() {
        out.push_str("\nSELECTED VISUAL PALETTE (all visual families remain archived; these are the relevant candidates):\n");
        for visual in visuals {
            out.push_str(&format!(
                "- {} [{}], suggested call: {}\n",
                clean(&visual.name, 90),
                clean(&visual.id, 80),
                clean(&visual.visual_call, 200)
            ));
            out.push_str(&format!(
                "  Useful for: {}\n",
                clean(&visual.when_to_use, 230)
            ));
            out.push_str(&format!(
                "  Placement: {}\n",
                clean(&visual.placement_guidance, 250)
            ));
            out.push_str(&format!(
                "  Evidence: {}\n",
                clean(&visual.verification_status, 160)
            ));
        }
    }

    let explicit_visual_request = contains_any(
        request_lower,
        &[
            "visual",
            "visualizer",
            "pianoroll",
            "punchcard",
            "spectrum",
            "scope",
            "spiral",
            "pitchwheel",
            "hydra",
            "shader",
        ],
    );
    out.push_str("\nPALETTE EXECUTION POLICY:\n");
    out.push_str("- Treat every source, method, sample name, bank, visualizer, slider, arrange helper, and effect as subject to MotifAI's pinned embedded-runtime contract and strudel_rules.md. A documented API is not proof of embedded availability or audible playback.\n");
    out.push_str("- Keep the instrument roles distinct and audible. Prefer proven sources and source-map names; use any exotic source only when confirmed. If a source is missing, preserve the musical role with a verified fallback instead of inventing an API.\n");
    out.push_str("- Use visualizers as diagnostics/display, not audio effects. Map the display to its data: punchcard for event timing, pianoroll for pitched note placement, scope/spectrum for waveform/timbre, spiral for cyclic motion, pitchwheel for pitch-class relationships. `.color(...)` affects visual highlighting, not sound.\n");
    if explicit_visual_request {
        out.push_str("- The brief explicitly emphasizes visuals: use a restrained but visible spread of relevant inline visual feedback across representative instrument roles, provided each selected hook is supported. Do not attach every visualizer to every stem, and do not let visuals compromise audio execution. Hydra remains extension-only unless the compatible host is enabled.\n");
    } else {
        out.push_str("- Visual feedback is an optional part of the palette. Add a small number only when they genuinely help this piece; do not force a visualizer onto every instrument or destabilize working audio to add decoration.\n");
    }
    out.push_str("- Every instrument card in the archive maps to suitable visual palette IDs; use that mapping for the right representative layer instead of guessing. Not every instrument needs its own visualization simultaneously.\n");
    out.push_str("- Expand, compress, reorder, merge, transform, or abandon any stage that conflicts with the current request. Do not print this block or a plan in the output. Return only the complete executable Strudel program.\n");
    out.push_str("</OPTIONAL_COMPOSITION_BLUEPRINT>\n");

    if out.chars().count() > MAX_PROMPT_CHARS {
        let closing = "</OPTIONAL_COMPOSITION_BLUEPRINT>\n";
        let keep = MAX_PROMPT_CHARS.saturating_sub(closing.chars().count() + 64);
        out = out.chars().take(keep).collect();
        out.push_str("\n[Selected archive guidance compacted to the prompt budget.]\n");
        out.push_str(closing);
    }
    out
}

fn push_list(
    out: &mut String,
    label: &str,
    values: &[String],
    max_items: usize,
    max_chars: usize,
) {
    if values.is_empty() {
        return;
    }
    out.push_str(&format!(
        "{}: {}\n",
        label,
        format_list(values, max_items, max_chars)
    ));
}

fn format_list(values: &[String], max_items: usize, max_chars: usize) -> String {
    values
        .iter()
        .take(max_items)
        .map(|value| clean(value, max_chars))
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn clean(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(max_chars)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn normalize_id(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

fn contains_any(text: &str, values: &[&str]) -> bool {
    values.iter().any(|value| text.contains(value))
}

/// Checks whether `needle` appears in `haystack` bounded by non-alphanumeric
/// characters, avoiding false substring hits (such as `"dub"` inside `"subdued"`
/// or `"set"` inside `"setting"`).
fn contains_bounded_term(haystack: &str, needle: &str) -> bool {
    let needle = needle.trim();
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    let hay_bytes = haystack.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = haystack[start..].find(needle) {
        let idx = start + rel;
        let end = idx + needle.len();
        let left_ok = idx == 0 || !hay_bytes[idx - 1].is_ascii_alphanumeric();
        let right_ok = end == hay_bytes.len() || !hay_bytes[end].is_ascii_alphanumeric();
        if left_ok && right_ok {
            return true;
        }
        start = idx + 1;
        if start >= haystack.len() {
            break;
        }
    }
    false
}

fn match_terms(
    terms: &[String],
    request_lower: &str,
    tag_points: i32,
    phrase_points: i32,
) -> i32 {
    terms
        .iter()
        .map(|term| {
            let term = term.trim().to_lowercase();
            if term.is_empty() {
                return 0;
            }
            let is_phrase = term.contains(' ') || term.contains('-');
            if contains_bounded_term(request_lower, &term) {
                if is_phrase {
                    phrase_points
                } else {
                    tag_points
                }
            } else if term.len() >= 5 && request_lower.contains(&term) {
                // Allow stem prefix matches on longer words (e.g. "orchestra" in "orchestral").
                (tag_points / 2).max(1)
            } else {
                0
            }
        })
        .sum()
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn rotate_debug_log_if_needed(log_path: &Path) {
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() > MAX_DEBUG_LOG_BYTES {
            let backup = log_path.with_extension("log.1");
            let _ = fs::rename(log_path, backup);
        }
    }
}

fn log_selection(stage: &str, detail: &str) {
    use std::io::Write;
    let log_path = Path::new("forge_debug.log");
    rotate_debug_log_if_needed(log_path);
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{ts}] [BLUEPRINT:{stage}] {detail}");
    }
}

fn load_library() -> Result<BlueprintLibrary, String> {
    let mut paths = Vec::<PathBuf>::new();
    if let Some(explicit) = env::var_os(PATH_ENV) {
        let path = PathBuf::from(explicit);
        if !path.is_file() {
            return Err(format!(
                "{} points to missing blueprint library '{}'.",
                PATH_ENV,
                path.display()
            ));
        }
        paths.push(path);
    } else {
        if let Ok(cwd) = env::current_dir() {
            add_candidates(&mut paths, &cwd);
        }
        if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
            add_candidates(&mut paths, Path::new(manifest));
        }
        if let Ok(exe) = env::current_exe() {
            if let Some(parent) = exe.parent() {
                add_candidates(&mut paths, parent);
            }
        }
    }

    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(normalize_path(path)));
    for path in paths.iter().filter(|path| path.is_file()) {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Could not inspect blueprint library '{}': {e}", path.display()))?;
        if metadata.len() > MAX_LIBRARY_BYTES {
            return Err(format!(
                "Blueprint library '{}' is {} bytes, above the {} byte safety limit.",
                path.display(),
                metadata.len(),
                MAX_LIBRARY_BYTES
            ));
        }

        let modified = metadata.modified().ok();
        let len = metadata.len();
        let cache_cell = CACHED_BLUEPRINTS.get_or_init(|| Mutex::new(None));
        if let Ok(guard) = cache_cell.lock() {
            if let Some(cached) = guard.as_ref() {
                if cached.path == *path && cached.modified == modified && cached.len == len {
                    return Ok(cached.library.clone());
                }
            }
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Could not read blueprint library '{}': {e}", path.display()))?;
        let library: BlueprintLibrary = serde_json::from_str(&content).map_err(|e| {
            format!("Malformed composition blueprint JSON at '{}': {e}", path.display())
        })?;
        if library.blueprints.is_empty() {
            return Err(format!(
                "Composition blueprint library '{}' has no blueprints.",
                path.display()
            ));
        }

        if let Ok(mut guard) = cache_cell.lock() {
            *guard = Some(CachedBlueprintLibrary {
                path: path.clone(),
                modified,
                len,
                library: library.clone(),
            });
        }
        return Ok(library);
    }
    Err(format!(
        "Composition blueprint library not found. Expected '{}' or set {} to an explicit path.",
        RELATIVE_PATH, PATH_ENV
    ))
}

fn add_candidates(paths: &mut Vec<PathBuf>, root: &Path) {
    for ancestor in root.ancestors() {
        paths.push(ancestor.join(RELATIVE_PATH));
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        use std::path::Component;
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_library() -> BlueprintLibrary {
        serde_json::from_str(include_str!("../knowledge/composition_blueprints.json"))
            .expect("master archive fixture must remain valid JSON")
    }

    #[test]
    fn master_archive_has_all_blueprints_and_palette_families() {
        let library = test_library();
        assert_eq!(library.schema_version, 2);
        assert!(library.blueprints.len() >= 60);
        assert!(library.instrument_palettes.len() >= 25);
        assert!(library.visual_palettes.len() >= 10);
        assert!(library.production_palettes.len() >= 10);
    }

    #[test]
    fn every_blueprint_palette_reference_resolves() {
        let library = test_library();
        let instruments: HashSet<String> = library
            .instrument_palettes
            .iter()
            .map(|x| x.id.clone())
            .collect();
        let visuals: HashSet<String> =
            library.visual_palettes.iter().map(|x| x.id.clone()).collect();
        let production: HashSet<String> = library
            .production_palettes
            .iter()
            .map(|x| x.id.clone())
            .collect();
        for blueprint in &library.blueprints {
            for id in &blueprint.instrument_palette_ids {
                assert!(
                    instruments.contains(id),
                    "missing instrument palette id {id} in {}",
                    blueprint.id
                );
            }
            for id in &blueprint.visual_palette_ids {
                assert!(
                    visuals.contains(id),
                    "missing visual palette id {id} in {}",
                    blueprint.id
                );
            }
            for id in &blueprint.production_palette_ids {
                assert!(
                    production.contains(id),
                    "missing production palette id {id} in {}",
                    blueprint.id
                );
            }
        }
        for instrument in &library.instrument_palettes {
            for id in &instrument.visual_palette_ids {
                assert!(
                    visuals.contains(id),
                    "missing visual id {id} in instrument {}",
                    instrument.id
                );
            }
        }
    }

    #[test]
    fn concert_request_selects_concert_scale() {
        let library = test_library();
        let selected = choose_auto_blueprint(&library, "write a full electronic concert").unwrap();
        assert_eq!(selected.scale, "concert");
    }

    #[test]
    fn lofi_request_selects_relevant_blueprint() {
        let library = test_library();
        let selected = choose_auto_blueprint(&library, "make a cozy lo-fi study beat").unwrap();
        assert!(
            selected.id.contains("lofi")
                || selected.id.contains("minimalist-piano")
                || selected.name.to_lowercase().contains("lo-fi"),
            "selected {}",
            selected.id
        );
    }

    #[test]
    fn generic_song_request_still_selects_one_flexible_seed() {
        let library = test_library();
        let selected = choose_auto_blueprint(&library, "create a new song").unwrap();
        assert!(!selected.id.is_empty());
        assert!(matches!(
            selected.scale.as_str(),
            "small" | "medium" | "extended" | "concert" | "experimental"
        ));
    }

    #[test]
    fn visual_selection_skips_hydra_unless_explicitly_requested() {
        let library = test_library();
        let bp = library
            .blueprints
            .iter()
            .find(|bp| bp.id == "full-electronic-concert-journey")
            .unwrap();
        let instruments =
            choose_instrument_palettes(&library, bp, "write a full electronic concert");
        let visuals =
            choose_visual_palettes(&library, bp, &instruments, "write a full electronic concert");
        assert!(visuals.iter().all(|v| !v.requires_extension));
        let hydra_visuals = choose_visual_palettes(
            &library,
            bp,
            &instruments,
            "write a full concert with Hydra audio reactive visuals",
        );
        assert!(hydra_visuals
            .iter()
            .any(|v| v.id == "visual-hydra-generative"));
    }

    #[test]
    fn rendered_block_is_delimited_once_and_bounded() {
        let library = test_library();
        let bp =
            choose_auto_blueprint(&library, "write a full electronic concert with all visualizers")
                .unwrap();
        let instruments = choose_instrument_palettes(
            &library,
            bp,
            "write a full electronic concert with all visualizers",
        );
        let visuals = choose_visual_palettes(
            &library,
            bp,
            &instruments,
            "write a full electronic concert with all visualizers",
        );
        let production = choose_production_palettes(
            &library,
            bp,
            "write a full electronic concert with all visualizers",
        );
        let rendered = render_archive_selection(
            &library,
            bp,
            &instruments,
            &production,
            &visuals,
            "write a full electronic concert with all visualizers",
        );
        assert_eq!(rendered.matches("<OPTIONAL_COMPOSITION_BLUEPRINT>").count(), 1);
        assert_eq!(
            rendered
                .matches("</OPTIONAL_COMPOSITION_BLUEPRINT>")
                .count(),
            1
        );
        assert!(rendered.chars().count() <= MAX_PROMPT_CHARS);
    }

    #[test]
    fn every_visual_palette_is_documented_or_flagged_as_extension_only() {
        let library = test_library();
        assert!(library
            .visual_palettes
            .iter()
            .all(|v| !v.verification_status.trim().is_empty()));
        assert!(library
            .visual_palettes
            .iter()
            .any(|v| v.requires_extension && v.id == "visual-hydra-generative"));
    }

    #[test]
    fn word_boundary_matching_prevents_accidental_substring_hits() {
        let terms = vec!["dub".to_string(), "acid".to_string()];
        assert_eq!(match_terms(&terms, "a subdued acoustic piano piece", 4, 8), 0);
        assert_eq!(match_terms(&terms, "a dark dub and acid groove", 4, 8), 8);
    }

    #[test]
    fn smart_auto_skips_blueprint_on_narrow_targeted_edit_or_syntax_repair() {
        let res = select_for_request_with_override(
            "Change only the bass and keep everything else",
            "",
            Some("auto"),
        )
        .unwrap();
        assert!(res.is_none());

        let repair_res = select_for_request_with_override(
            "Fix the parse error without changing the music",
            "",
            Some("auto"),
        )
        .unwrap();
        assert!(repair_res.is_none());
    }
}