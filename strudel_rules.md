# MOTIFAI // STRUDEL COMPOSITION, SYNTAX & RUNTIME CONTRACT

**Version:** 2.2.1  
**Reviewed:** 2026-10-22  
**Purpose:** Give MotifAI a deep, extensible working reference for writing, evolving, and debugging executable Strudel compositions. This document describes the structures and tools available to the composer without prescribing one arrangement, genre formula, instrument count, macro set, or section count.

**v2.2.1 additions:** Explicitly distinguishes mini-notation weight syntax from numeric DSP parameters, documents the invalid `lpf("@200")` pattern and its fixed-cutoff/dynamic-pattern alternatives, and aligns preflight repair behavior with this distinction. Retains the v2.2.0 recipe, metadata, and sample-system additions.

> **Governing principle:** Structure is a language of possibilities, not a song template. Learn the time model, syntax, pattern algebra, sound sources, and control systems well enough to choose the structure that serves the brief. Do not turn this document into a checklist that every song must visibly demonstrate.

## 0. PRIORITY, EVIDENCE, AND EVOLUTION

When instructions or assumptions conflict, use this order:

1. The user's current musical intent and explicit constraints.
2. The exact Strudel runtime embedded by MotifAI, including its loaded packages, helpers, sample map, and UI bridge.
3. Official Strudel documentation and examples for a clearly identified version.
4. This reference's documented conventions and examples.
5. General music-production intuition.

Do not silently resolve a conflict by guessing. Identify the conflicting assumption, pick the safest documented path, or simplify the code until the target runtime is verified.

### Evidence labels

Use these terms precisely in internal reasoning, experiments, memory, logs, and repair prompts:

- **Documentation-supported:** The feature or behavior appears in official Strudel documentation. This does not prove it exists in MotifAI's embedded version.
- **Syntax-reviewed:** The exact generated expression was reviewed for language structure, expected receiver, arguments, and data types.
- **Preflight-passed:** MotifAI's static/structural checks accepted the code. This is not proof that Strudel evaluated it successfully.
- **Runtime-verified:** The exact code was evaluated by the target embedded Strudel runtime without a reported evaluation error.
- **Playback-verified:** Playback was actually initiated and sound production was observed. Evaluation success alone does not establish this.
- **User-approved:** The user explicitly expressed approval. Saving, opening, auditioning, evolving, or silence after generation is not approval by itself.

Never promote an event or technique to a stronger evidence label without the corresponding observation. Preserve failure details and version context so knowledge can keep evolving.

### This is both a language reference and a MotifAI export contract

The standalone Strudel REPL supports workflows that may use multiple `$:` roots. MotifAI's generated artifact contract is intentionally different: produce one complete program with named constants and one final root expression unless the application is explicitly changed to accept multiple roots. Do not mistake this project convention for a universal Strudel language limitation.

---

## 1. OUTPUT CONTRACT: A COMPLETE, EVALUATABLE PROGRAM

- Return only raw Strudel JavaScript code. Do not use Markdown code fences or prose outside `//` comments in a generated score.
- Put one intentional tempo declaration near the beginning, normally `setcps(...)`, unless the host contract explicitly supplies tempo externally.
- Use `const` declarations for meaningful reusable patterns, shared motifs, control values, or DSP chains. Do not create constants solely to inflate apparent complexity.
- End with one clear root pattern expression that the target REPL can evaluate, commonly `stack(...)`, `cat(...)`, `seq(...)`, `stepcat(...)`, `polymeter(...)`, or a named pattern variable. Use `arrange(...)` only when the target runtime actually exposes that helper and its exact semantics are known.
- A final variable is acceptable only if it resolves to the intended Strudel pattern. A set of declarations with no evaluated output is not a complete score.
- Every chained method must have a real receiver, a supported method name, the right argument count, and compatible argument types. Never emit a detached leading method chain.
- Balance every parenthesis, square bracket, curly brace, angle bracket inside mini-notation, quote, and JavaScript template literal. Do not confuse mini-notation punctuation inside a string with JavaScript punctuation outside it.
- Avoid unfinished expressions, `TODO`, ellipses, placeholder variables, speculative imports, and API calls invented to satisfy a musical description.
- Prefer a small, coherent program over a long brittle one. Add complexity when it changes musical behavior in a purposeful way.
- Comments should describe actual code behavior. A comment that claims a filter sweeps, a slider works, a chord is voiced, or a section lasts 16 bars is not evidence that it does.
- Do not introduce `fetch`, DOM access, Node.js APIs, package imports, arbitrary globals, or external code loading unless MotifAI explicitly supports them and the task authorizes their use.

## 2. RUNTIME CAPABILITIES: CORE STRUDEL VS. MOTIFAI FEATURES

Strudel is a pattern language evaluated by a specific REPL/runtime. A JavaScript-looking expression is not automatically a valid Strudel expression, and an API in current online docs is not a guarantee that the embedded build provides it.

### Core-versus-extension rules

- Prefer the documented Strudel API and examples linked at the end of this file.
- Confirm the active runtime before relying on extensions, custom sample maps, visualizer hooks, UI controls, or non-core helpers.
- Treat `arrange(...)`, `slider(...)`, `register(...)`, custom macro bridges, application-specific auto-wiring, and project-only helper functions as **runtime-dependent unless the application source or a real runtime test confirms them**. Do not describe them as guaranteed core Strudel APIs.
- Do not assume `setcpm(...)`, custom tuning helpers, sample aliases, a synth preset, a drum bank, a visualizer, or a function with a familiar name is available merely because a related concept exists in another audio environment.
- The `strudel_rules.md` file is a curated reference, not an exhaustive whitelist. An omitted method is not necessarily invalid; a listed method is not automatically valid in every embedded version. Verify uncertain APIs rather than freezing the vocabulary.
- Do not emit multiple `$:` roots under MotifAI's current single-root export contract. If the app's parser or studio changes, update this contract deliberately and test it instead of silently assuming a new format.
- Pin the embedded Strudel version when possible. Avoid a production runtime that silently tracks `@latest` if reproducible debugging matters.

### Minimal safe fallback

When a requested advanced effect or helper is uncertain, retain the musical idea and express it using verified primitives. A simple playable riff with correct timing is better than a sophisticated-looking score that invokes a missing API.

---

## 3. CYCLES, TEMPO, PULSE, AND DURATION

Strudel's clock is expressed in **cycles per second (CPS)**. Its default is `0.5` CPS, so one cycle lasts two seconds. The number of events within a cycle changes their spacing, not the total duration of that cycle.

- `setcps(value)` means cycles per second, not beats per minute. `setcps(120)` is almost never what a person requesting 120 BPM intends.
- If one cycle is being treated as one conventional four-beat bar in 4/4, a useful conversion is `CPS ≈ BPM / 240`, and `BPM ≈ CPS * 240`. This is a reference convention, not a universal definition of perceived tempo. Different subdivisions, meters, swing, and sound choices can alter the perceived pulse.
- Examples under that four-beat-cycle assumption: `setcps(0.375)` is approximately 90 BPM, `setcps(0.5)` approximately 120 BPM, and `setcps(0.625)` approximately 150 BPM.
- Choose tempo and subdivision together. A slow cycle with rapid events can feel fast; a fast cycle with sparse events can feel spacious. Do not force every genre into a fixed CPS range.
- Distinguish cycles, beats, bars, events, and seconds. Do not claim a section is eight bars merely because it lasts eight cycles unless the musical meter definition supports that interpretation.
- If the target runtime's arrangement helper accepts section lengths in cycles, approximate total duration as `totalCycles / CPS`. For a target duration in seconds, estimate `targetCycles = targetSeconds * CPS`, then verify how the active helper interprets its first argument.
- Include note releases, sample tails, delay, and reverb when estimating audible ending time. The final triggered event and the end of its audible tail are not necessarily the same moment.
- `setcpm(...)` or pattern-level tempo overrides should be used only when the target runtime's exact behavior is known. Prefer the simplest consistent tempo mechanism in one score.
- Meter, polymeter, tuplets, and metric ambiguity are creative options. Use them when they create the desired pulse relationships, not simply because the syntax is available.

## 4. JAVASCRIPT, STRUDEL CHAINS, AND MINI-NOTATION STRINGS

### Strings and program syntax

- In the REPL, double-quoted strings such as `"c3 e3 g3"` are parsed as mini-notation. Backtick template literals can be used for multi-line mini-notation. Single-quoted strings are ordinary strings rather than mini-notation in the documented REPL behavior; use them where a plain string is intended and the API expects it.
- As a MotifAI house style, use double quotes consistently for mini-notation. Use backticks only where multi-line formatting improves readability. Avoid unnecessary quote-style mixing.
- JavaScript arrays, callback functions, arithmetic, and `const` declarations may be useful, but the resulting expression must still be valid in the target REPL's evaluation context.
- Parameter functions give values meaning: `note("c3 e3 g3")` identifies pitches, `s("bd hh")` identifies sound/sample names, and `n("0 2 4")` commonly supplies numeric degrees or indices to another parameter operation. Plain text values with no parameter context may not produce sound.
- A common pattern chain is `note("c3 e3 g3").s("sawtooth")`. The receiver and all subsequent calls must accept the supplied value types.
- A method chain is evaluated left to right. Consider what type and event structure each operation receives after the previous operation.

### Choosing between functions and mini-notation

Strudel is built from functions and patterns; mini-notation is a concise textual language for expressing many common event and timing relationships. These are complementary interfaces, not competing ways of composing.

- Prefer **mini-notation** for compact rhythms, note sequences, rests, subdivisions, simple alternation, Euclidean patterns, and concise event-level variation. It is often easier to inspect a drum groove or short motif in this form.
- Prefer **pattern functions and factories** when combining different pattern types, expressing multi-layer structure, applying callbacks, reusing an existing pattern, or transforming patterns in ways mini-notation cannot express concisely.
- Mix the two naturally. Mini-notation can describe the internal rhythm of a drum pattern while JavaScript functions such as `stack(...)` combine that rhythm with a pitched synth pattern, chord progression, or another independently processed role.
- Strudel documents many mini-notation structures as expressible through functions. Do not assume every mini-notation operator has a one-to-one JavaScript method with the same spelling. Learn the target function's actual signature and semantics.
- The mini-notation has a compact set of operators; function patterns expose a broader transformation vocabulary. When nested notation becomes difficult to audit, extract meaningful subpatterns into named constants or use the documented factory/function form.
- Keep the musical meaning the same when changing representations. Translating a pattern is not a license to change its event density, event timing, simultaneity, or random behavior.
- Function composition still obeys receiver types and pattern alignment. Replacing a string with a function does not remove the need to reason about which pattern supplies event structure.

A representative hybrid approach is to define a pitched line with `note(...)` and its sound/effects, define percussion with `s("bd(5,8), hh*8")`, and combine the two with `stack(bass, drums)`. The rhythm remains concise while the outer program expresses the mix of different musical roles clearly.

### Mini-notation grammar

Inside mini-notation strings:

- **Spaces:** sequence events in the same cycle. Adding more events usually divides the cycle into more, shorter event slots rather than lengthening the cycle.
- **Square brackets `[...]`:** group a nested sequence into the time slot of one outer event. Nest groups to create subdivisions, not as decorative parentheses.
- **Commas `,`:** play events simultaneously. In a pitch pattern, this is the basic chord/polyphony form, for example `note("[c4,e4,g4]")`. To sequence multiple chords, group each chord and place the groups in an outer sequence, such as `note("<[c4,e4,g4] [d4,f4,a4]>")`.
- **Angle brackets `<...>`:** spread/alternate items over the cycle structure. They do **not** mean that only one note is played during an entire cycle. For example, `note("<e5 b4 d5 c5>")` fits the sequence into its cycle; adding or removing events changes the subdivision.
- **Rests `~` and `-`:** equivalent rest markers within mini-notation.
- **Multiplication `*N`:** repeat/speed up a token or grouped pattern. Use brackets to make the intended group explicit, for example `s("[bd hh]*2")`.
- **Division `/N`:** stretch a token or bracketed pattern across more cycles, for example `note("[c4 e4 g4]/2")`.
- **Replication `!N`:** repeat a value without speeding it up. It changes how the item occupies the sequence relative to its neighbors; do not treat it as interchangeable with `*N`.
- **Weight `@N`:** assign relative temporal weight to an item; an item weighted `@2` lasts twice as long as a weight-1 item within that weighted sequence.
- **Weight syntax is event-local:** `@N` follows an event/token inside mini-notation, such as `s("bd@2 hh")`. It is not a prefix for a parameter value. Never write `.lpf("@200")`, `.hpf("@1200")`, or similar standalone weighted numeric strings. For a fixed cutoff use a numeric parameter such as `.lpf(200)`; for an event-stepped cutoff pattern, use valid value mini-notation such as `.lpf("<200 400 800>")`. Do not use `@` to mean “set cutoff to” or “start the cutoff at.”
- **Random removal `?`:** an event has a 50% chance of being removed. `?0.3` means a 30% removal chance, not a 30% chance of being retained. Use stochastic removal intentionally.
- **Random choice `|`:** choose between alternative events or groups. Group alternatives carefully so the intended choices are unambiguous.
- **Euclidean rhythm `(pulses, steps, offset)`:** distribute pulses across a number of slots, optionally rotated. A common form is `s("bd(3,8)")`; the third argument is the offset. This mini-notation form is distinct from a chained `.euclid(...)` method.
- **Curly braces / polymeter:** Strudel supports polymetric forms such as `{a b c, x y}` and a corresponding `polymeter(...)` factory. Use them only when their asynchronous step relationships are intentional. Do not invent extra suffix syntax such as `%8` unless it is verified for the target parser.
- The mini-notation is a grammar, not a collection of arbitrary punctuation. If nesting becomes hard to reason about, simplify the expression or define the parts separately.

### Frequent notation traps

- A chord and a sequence are not the same thing: `note("c4 e4 g4")` is sequential; `note("[c4,e4,g4]")` is simultaneous.
- Angle brackets, square brackets, and commas have different roles. Do not substitute one for another simply because a phrase contains several notes.
- A value such as `"800:15"` may mean multiple parameter values only for methods that explicitly document colon-separated mini-notation. Do not generalize this syntax to all methods.
- Use a simple sample pattern such as `s("bd sd,hh*8")` as a starting point only when the names exist in the active sample map.

## 5. PATTERN FACTORIES AND STRUCTURAL ALGEBRA

Factories create new patterns from other values or patterns. Choose the factory based on the desired time relationship.

- `stack(a, b, ...)` plays patterns in parallel. Aliases include `polyrhythm` and `pr`. Use `stack` to combine independently processed roles such as drums, bass, harmony, melody, or ambience.
- `cat(a, b, ...)` concatenates items so each item occupies one cycle. Alias: `slowcat`.
- `seq(a, b, ...)` fits items sequentially into one cycle. Alias: `fastcat`.
- `stepcat(...)` concatenates items proportionally by temporal weight; aliases include `timeCat` and `timecat`. Its signature can accept `[weight, pattern]` pairs, e.g. `stepcat([3, "e3"], [1, "g3"])`.
- `polymeter(...)` combines patterns with different step counts, corresponding to the polymetric mini-notation form. Confirm input shape against the docs/runtime rather than assuming it accepts the same arguments as `stack`.
- `silence` is a silent pattern in supported Strudel contexts and can be used where a whole section or branch needs to rest.
- `arrange(...)` is useful only if the active MotifAI runtime provides it. The current output contract requires its exact signature and cycle semantics to be confirmed before relying on it.
- Do not confuse a JavaScript list of patterns, a pattern containing events, and a mini-notation string. They are different kinds of inputs even when they describe similar music.
- Do not use `stack(...)` merely to make code look modular. A single cohesive pattern is valid when it satisfies the brief.
- Reusable constants help share a harmonic spine or motif across stems, but only reuse a variable when the relationship is intentional and its type remains compatible with each operation.

## 6. ALIGNMENT, RECEIVERS, AND EVENT STRUCTURE

Alignment is one of Strudel's most important concepts. By default, combining patterns lines up their cycles and takes events from the left/first pattern as the event structure, matching values from the other pattern onto that structure. This affects the number and timing of events, not only their pitch.

- Put the pattern whose event onsets/density should drive the result on the left when using parameter-chain composition.
- For example, `note("c4 a3 f3 e3").s("piano")` preserves the four-note structure. `s("piano").note("c4 a3 f3 e3")` can retain the left side's coarser structure and therefore not emit all four notes as expected.
- Before reversing a chain, ask: **which pattern supplies event timing, and which pattern supplies values?** Do not infer this from musical labels alone.
- Where appropriate, explicit alignment variants such as `.in`, `.out`, `.mix`, or squeeze-based alignment may be available on value operators. Use only the variants documented for that operator and target version. They are not a universal method family guaranteed on every transformation.
- `.struct(...)` imposes trigger structure; `.mask(...)` silences events according to a mask. These operations do not mean the same thing as placing a pattern on the left, and neither should be used as a vague repair for every alignment problem.
- If a pattern unexpectedly loses events, first inspect receiver order, grouping, and event density. Do not immediately rewrite the melody or add arbitrary repetitions.
- If two patterns need independent timing and sound processing, define them separately and combine them with `stack(...)` instead of forcing them into one long chain.

## 7. TIME AND RHYTHMIC MODIFIERS

Use temporal operators to alter event timing and density with a clear rhythmic reason.

- `.fast(factor)` / `.density(factor)`: speed up a pattern. A pattern-valued factor can create changing density when the API supports it.
- `.slow(factor)` / `.sparsity(factor)`: stretch a pattern over more cycles.
- `.early(cycles)` and `.late(cycles)`: shift a pattern in cycle-time units. These are not milliseconds. Small values are useful for microtiming and call-and-response.
- `.clip(factor)` / `.legato(factor)`: scale event duration; clipping can cut samples off at the end of the new duration. Use it to shape articulation, not as a universal mix fix.
- `.euclid(pulses, steps[, rotation])`: apply a Euclidean trigger structure. Negative pulse counts invert the structure into the gaps where supported.
- `.euclidLegato(pulses, steps[, rotation])`: related Euclidean behavior that holds pulses toward the next pulse rather than leaving the normal gaps.
- `.rev()`: reverse each cycle's pattern. `.palindrome()` alternates forward and reversed cycles.
- `.iter(n)` / `.iterBack(n)`: rotate the starting subdivision over successive cycles, forward or backward.
- `.ply(n)`: repeat each event within its own event slot. Use sparingly for rolls, stutters, and fills.
- `.segment(n)` / `.seg(n)`: sample a continuous pattern into `n` events per cycle. This is useful when turning a signal into discrete values.
- `.compress(start, end)`: compress cycle content into a time window and leave silence outside it. `.zoom(start, end)` extracts a time slice and expands that slice over a full cycle.
- `.inside(n, fn)` and `.outside(n, fn)` may be useful for transformations at different time scales when supported by the embedded version. Verify less familiar modifiers before use.
- Additional variants such as `.fastGap(...)`, `.linger(...)`, `.ribbon(...)`, `.swing(...)`, or `.swingBy(...)` should be used only when their behavior and signature are documented for the active version. Do not treat this list as proof that a host exposes them.
- A timing modifier is meaningful only relative to the underlying cycle. Inspect the resulting groove and duration rather than stacking modifiers because their names sound appropriate.

## 8. RANDOMNESS, CONDITIONALS, AND CONTROLLED VARIATION

Randomness should create living detail while retaining recognizable identity. Protect any motif or pulse that the brief needs the listener to recognize.

### Random functions and degradation

- `choose(...)` selects among values or patterns; `wchoose(...)` provides weighted choices where the corresponding function signature is supported.
- `.degrade()` randomly removes about half the events. `.degradeBy(p)` controls the removal amount from 0 to 1. In mini-notation, `?p` uses `p` as the chance of removal.
- `.undegradeBy(p)` is the complementary selection relative to `.degradeBy(p)` when used with the documented semantics. It can split a shared event stream into complementary variants, including separate timbres or pan positions.
- Use randomness for ornament, fills, sparse textures, note choices, and timbral variation. Avoid stochastic removal of every downbeat, every bass anchor, or the only exposed motif unless instability is the point.
- If repeatability matters, use deterministic structure, a controlled cycle pattern, or a documented locking approach. Do not assume each random helper has deterministic seeding support.

### Conditional and structural transformations

- `.sometimes(fn)`, `.often(fn)`, `.rarely(fn)`, `.almostAlways(fn)`, `.almostNever(fn)`, and probability-based variants apply transformations with different rates where documented.
- `.every(n, fn)`, `.firstOf(n, fn)`, and `.lastOf(n, fn)` are useful for periodic variations or turnarounds. Respect their exact cycle-based definitions.
- `.when(binaryPattern, fn)` conditionally applies a transformation based on a pattern's true state.
- `.chunk(n, fn)`, `.chunkBack(n, fn)`, and `.fastChunk(n, fn)` rotate transformations through portions of a pattern. Do not assume their cycle behavior is identical.
- `.struct(binaryPattern)` imposes a trigger skeleton; `.mask(binaryPattern)` silences masked events without treating the mask as a melody.
- `.reset(trigger)` and `.restart(trigger)` may reset pattern timing when supported. Check precise behavior before depending on a phase reset for a complex structure.
- `.arp(indexPattern)` selects indices from stacked notes. Do not invent symbolic labels like `"up"` or `"down"` unless that exact notation is documented.
- Pattern lookup/transformation helpers such as `.pick(...)`, `.pickmod(...)`, `.inhabit(...)`, or `.pickF(...)` can be powerful but should be treated as advanced, version-sensitive tools until the target signature has been verified.
- Conditional logic should create phrasing, call-and-response, controlled variation, or transitions. Do not make every stem switch independently at random; the combined piece can become incoherent even if every stem is individually valid.

## 9. ACCUMULATION, LAYERING, AND PATTERN EFFECTS

These methods transform or duplicate an existing pattern. They are not interchangeable.

- `.superimpose(fn)` preserves the original pattern and plays the transformed version alongside it.
- `.layer(fn1, fn2, ...)` plays transformed branches without the original dry pattern.
- `.off(time, fn)` adds a transformed copy delayed by the given cycle-time offset. It is a pattern-time offset, not a millisecond delay effect.
- `.echo(times, time, feedback)` repeats events at cycle-time offsets and scales later repeats by the feedback/velocity multiplier. Avoid combining many echoes with long reverb and high feedback without listening.
- `.jux(fn)` and `.juxBy(width, fn)` create a transformed stereo counterpart in supported Strudel contexts. Treat stereo motion as a production choice, not an automatic improvement.
- Use a shared motif as the source for variation when possible, so transformed voices sound related. A counter-melody with independent notes may be clearer as a separate pattern inside `stack(...)`.
- Check the result's event density and loudness when using accumulation. Duplicated patterns can increase masking, polyphony, and perceived level.
- Advanced variants such as `.echoWith(...)` or `.stutWith(...)` should not be assumed to exist merely because they are conceptually useful. Verify their names and signatures in the target version.

## 10. VALUES, PARAMETERS, AND CONTINUOUS SIGNALS

### Value operations

- `.add(x)`, `.sub(x)`, `.mul(x)`, `.div(x)`, `.round()`, `.floor()`, and `.ceil()` transform numeric pattern values where supported.
- These operations can alter scale degrees, pitch values, frequency values, timing factors, or effect parameters depending on what the receiving parameter expects. Do not apply arithmetic without checking the value's semantic domain.
- `.range(min, max)` maps a supported signal's unipolar values into a chosen range. `.range2(min, max)` maps a bipolar signal when that type of signal is used. `.rangex(...)` and other alternative mappings should be used only when documented for the active build.
- `ratio(...)`, `.as(...)`, or batch parameter encodings should not be used from memory alone. Check exact syntax, accepted data, and semantics in current docs or the target runtime before relying on them.
- Plain values must be wrapped in the parameter function that gives them meaning. A string of numbers alone is not always a playable pattern.

### Pattern-level signals

Documented signal patterns include `sine`, `cosine`, `saw`, `tri`, `square`, `rand`, `perlin`, `irand(n)`, and bipolar counterparts such as `sine2` and `rand2` in supported versions.

- Signals generate continuous values that can be sampled at event times. If a sound triggers only once per cycle, a pattern-level signal controlling its filter may only be sampled once per cycle. It may therefore sound stepped rather than smoothly swept.
- For a discrete modulation curve, sample the signal with `.segment(n)` / `.seg(n)`, then map values to the desired parameter range.
- For genuinely continuous modulation during a sustained sound, use an audio-level LFO or a suitable filter envelope if the target runtime supports it.
- Pattern-level signals and UI slider values are different concepts. Never assume they share a type or can be chained through the same methods.
- Random signals are not the same as stochastic event removal. A random signal varies a value; degradation removes events.

### Audio-level `.lfo(...)`

Strudel documents `.lfo()` as a continuous audio-level modulator. By default it modulates the parameter immediately preceding it; a `control`/`c` option can target another parameter.

- `control` / `c`: parameter to modulate, such as `'lpf'` or `'gain'`.
- `rate` / `r`: oscillation rate in Hertz.
- `sync` / `s`: rate expressed as repetitions per cycle, useful for synchronizing modulation to the pattern clock.
- `depth` / `dr` / `dep`: relative modulation depth.
- `depthabs` / `da`: absolute modulation depth in the parameter's own units, such as a cutoff-frequency range.
- `dcoffset` / `dc`: shift the modulation's centre so the movement can be biased upward or downward.
- `shape` / `sh`: documented shapes include `triangle`, `sine`, `ramp`, `saw`, and `square`; the numeric mapping is version/documentation-defined.
- `skew` / `sk`: changes the symmetry of supported shapes.
- LFO parameters can themselves be patterns in documented examples. Use conservative depth and range; modulation can push a filter, gain, or frequency into an inaudible or undesirable region.
- Do not assume speculative LFO keys such as `curve` exist unless the active documentation/runtime confirms them.
- Put `.lfo()` where it targets the intended parameter. A misplaced LFO may modulate a default or different control than intended.

## 11. TONAL MATERIAL, SCALES, CHORDS, AND VOICE LEADING

Tonal tools should support the musical idea, not force every track into a scale. Tonality, modal mixture, chromatic writing, drones, and deliberate non-tonal or microtonal material are all valid when intentional.

### Pitches, degrees, and scales

- Use `note(...)` for named pitches and `n(...)` for numeric scale-degree or index material when appropriate.
- `.scale("C:minor")` maps zero-indexed values to a scale or quantizes named notes according to the documented behavior. Scale syntax is root note followed by a colon and scale type; use known documented spelling rather than inventing a scale name.
- Negative scale degrees wrap below the nominal starting degree, and chromatic alterations may be represented with sharp/flat degree modifiers where documented.
- `.transpose(...)` is chromatic/semitone-oriented; `.scaleTranspose(...)` moves by scale steps when the active scale is defined. Do not confuse the two.
- `.arp(indexPattern)` selects voices from a stacked-note pattern; it is not a universal direction keyword parser.
- When deriving bass, pads, and arpeggios from a common harmonic source, verify whether the chosen helper passes pitch, scale degree, chord symbol, or a list of voiced notes. Don't assume methods like `.set(...)` or `.rootNotes(...)` work on every pattern type.

### Chord construction and voicings

- Simultaneous notes form a chord. A sequence of chord groups creates a progression. Use explicit grouping so a parser or a future reader can distinguish them.
- Strudel's tonal helpers support chord-symbol voicing through `chord(...)` and `.voicing()` in documented contexts. Control parameters include `dict`, `anchor`, and `mode`; documented modes include `below`, `duck`, `above`, and `root`.
- `anchor` controls where the voicing is placed; changing it can change register and voice-leading behavior. It is not a substitute for selecting musically sensible chord symbols.
- Prefer common-tone retention, smooth voice leading, a deliberate register, and clear harmonic rhythm where suitable. Dense extensions are not inherently richer if they mask the melody or destabilize the bass.
- Chord-symbol dictionaries may differ. If an unusual extension or alteration fails, reduce to a simpler documented symbol and isolate the dictionary/runtime issue. Do not guess a chord spelling and silently treat it as correct.
- A shared harmonic spine can keep chords, bass, and melody related, but each derived layer should have its own purposeful rhythm and register.

### Xenharmony and microtonality

Strudel documents experimental xenharmonic functions. Treat them as a specialized creative tool, not a default embellishment.

- `.xen(scaleOrRatios)` maps a numeric index pattern to a nonstandard scale, equal division of the octave such as `"31edo"`, or a frequency-ratio array in supported contexts. The documented base reference is 220 Hz and the mapping assumes octave repetition.
- An example form documented by Strudel is `i("0 8 18").xen("31edo")`.
- Xenharmonic functions are explicitly marked experimental in the documentation. Verify the target version and listening result before relying on them for a critical melody or harmony.
- Avoid claiming that a microtonal scale is a conventional twelve-tone scale, or that all arbitrary ratios preserve familiar harmonic expectations.
- Do not call unverified tuning helpers such as `getFreq(...)`, `.tune(...)`, or `.freq()` combinations based only on their apparent names. Confirm their exact documented availability and semantics first.

## 12. SYNTHESIS AND TIMBRE

### Basic oscillators and noise

Documented basic synth sources include `sine`, `sawtooth`/`saw`, `square`, and `triangle`/`tri`. Strudel documentation notes that a note without an explicit sound source defaults to a triangle wave in the documented synth context.

- `note("c3 eb3 g3").sound("sawtooth")` or `.s("sawtooth")` is a common pitched-synth form where both aliases are supported.
- `white`, `pink`, and `brown` noise can act as sound sources in supported contexts. `.noise(...)` can blend noise into an oscillator.
- A waveform name selects a source; it does not by itself create a musical role. Shape rhythm, register, envelope, spectral content, and articulation for the part's function.
- `supersaw`, `pulse`, or other oscillator names should be used when documented and available in the target synth setup; avoid treating synth-source names as guaranteed presets.

### Additive / partial synthesis

- `.partials([...])` can shape the harmonic magnitudes of supported synth sources. The documented `"user"` source can use partials to construct custom waveforms.
- Arrays created with normal JavaScript may be valid when the embedded evaluator supports those expressions. Keep them bounded and readable; do not introduce complicated array generation when a simple oscillator meets the brief.
- The documented examples show compatibility with helper patterns such as `randL(...)` or `binaryL(...)`; still verify availability when using such helpers in MotifAI.
- `.phases(...)` or less familiar synthesis parameters should be confirmed against the current synth documentation/target runtime before using them in a generation that must play first try.

### FM synthesis

- `.fm(...)` / `.fmi(...)` controls the FM modulation index/brightness.
- `.fmh(...)` controls the FM harmonicity ratio and timbral character.
- The synth documentation describes indexed operator variants such as `.fm2(...)`, `.fmh2(...)`, operator envelope controls, and cross-modulation matrix names such as `.fm13(...)` in supported contexts.
- `.fmattack(...)` / `.fmatt(...)`, `.fmdecay(...)` / `.fmdec(...)`, `.fmsustain(...)` / `.fmsus(...)`, and `.fmenv(...)` / `.fme(...)` shape FM modulation envelopes.
- FM parameters can create bell-like, metallic, glassy, percussive, or complex harmonic spectra. Use the modulation index and ratios as timbral decisions rather than assuming a ratio always maps to one fixed instrument category.
- The fact that a family of indexed controls exists in the docs does not mean every source or embedded version supports every combination. Verify complex operator routings when they are central to the piece.

### Wavetables and less-common engines

- Strudel's synth documentation describes wavetable sources and a default collection of wavetable data in supported contexts. Names, loading, looping, and scanning parameters must follow the active sound map and docs.
- Do not assume `wt_*`, `z_*`, or arbitrary sound names are universally loaded or behave consistently in custom embeddings. Check the sound map and exact documentation before generating a score that depends on them.
- Sound-design details are optional. A coherent simple oscillator with purposeful filtering can outperform an unverified exotic synthesis chain.

## 13. SAMPLES, DRUM BANKS, AND AUDIO MATERIAL

### Sample naming and indices

- `s("...")` selects sample/sound names using the active sample map. Strudel's documented default map includes common drum abbreviations such as `bd`, `sd`, `rim`, `cp`, `hh`, `oh`, `cr`, `rd`, `ht`, `mt`, `lt`, `sh`, `cb`, `tb`, `perc`, `misc`, and `fx`.
- The REPL's `sounds`/sample browser is the authority for exact names. Do not assume every sound bank has every abbreviation.
- Sample variations may be selected with `:index` notation, for example `hh:2`, or through a documented index parameter such as `.n(...)` in a compatible sample pattern.
- Default samples can lazy-load on first use. A delayed first sound may indicate loading latency rather than invalid composition syntax.

### Sound map and custom sound registration

Strudel's sound system resolves the name passed to `s(...)` through a runtime sound map. A name is playable only when that sound has been registered or otherwise provided by the active REPL. This is why a plausible-looking identifier such as `pad1` is not evidence that the sound exists.

- Treat the active REPL sound browser / sound map as the source of truth for available sound names. Confirm custom sample maps and banks separately from standard synth oscillator names.
- The technical manual documents `registerSound(name, onTrigger, data)` as a sound-engine API. The `onTrigger(time, value, onended)` callback is expected to create/trigger a sound and return an object containing the output `node` plus a `stop(time)` function. The node is connected to Strudel's standard effects chain, and the stop lifecycle matters for sustained notes and MIDI-like note-off behavior.
- This registration API is **runtime-development machinery**, not a routine technique for composing a song. Do not generate `registerSound(...)`, `getAudioContext()`, `OscillatorNode`, or other custom Web Audio setup merely to obtain a synth timbre when a documented `s(...)` source already meets the brief.
- Use custom sound registration only when the task explicitly concerns implementing or extending the sound engine and the target REPL exposes the necessary globals and Web Audio support. Such code should be tested as runtime code, not treated as an ordinary pattern-only composition.
- Do not confuse three different layers: a sound source name selected with `s(...)`, a sample bank that prefixes sample names, and a custom sound registration that adds a new source to the runtime. They solve different problems.

### Banks

- `.bank("RolandTR808")` and similar bank names shorten prefixed sample names in sample patterns. Strudel's docs demonstrate 808/909 switches with names that exist in the active sample map.
- Banks work by prefixing the bank name to sample names; they are not general-purpose synth presets or chord voicing dictionaries.
- Not all banks contain every sample. Keep the pattern's sample names and selected bank consistent and verify any unfamiliar bank name.
- Avoid attaching a drum bank to pitched oscillator leads as though it were a keyboard timbre. Pitched sample playback can be intentional, but it must use the actual sample and pitch semantics supported by the engine.

### Sample playback, slicing, and external material

- Use documented sample window, speed, looping, fit, chop, striate, slice, splice, or scrub operations only when the exact API is available and the target sample supports the intended behavior.
- Sample duration, loop points, pitch transposition, and event duration can interact. A pattern can be syntactically correct and still sound clipped, too short, too slow, or repetitive.
- Custom sample loaders or remote sample repositories can require network access and configuration. They are not automatically available in a local, offline, or sandboxed MotifAI studio.
- Prefer the supplied sound map unless the task explicitly calls for external/custom samples and the application supports their loading.

## 14. AUDIO EFFECTS, ENVELOPES, AND SIGNAL FLOW

### Effect slots and chain order

Strudel documents a fixed signal-flow order for supported effects, and each effect slot is single-use within a pattern. Repeating the same effect call typically replaces the earlier value for that slot; it does not create a serial stack of two independently configured instances.

A simplified documented order is: source-level pitch/detune behavior → time-stretching → gain and amplitude envelope → low-pass filter → high-pass filter → band-pass filter → vowel filter → coarse/sample-rate reduction → bit crush → waveshaping → distortion → tremolo → compressor → pan → phaser → postgain → dry/send routing to orbit effects → orbit ducking where supported.

- `s("bd").lpf(100).distort(2).lpf(800)` does not create two independent low-pass filters around distortion; the second `lpf` overwrites the first filter setting in that slot.
- Multiple effects with different roles can be chained in documented order, but don't confuse chain syntax with physically serial instances of the same effect.
- If a sound needs more than one distinct stage of the same kind of processing, use a documented alternative architecture or a separate audio route only if the active system supports it. Do not invent nested chain syntax.

### Amplitude, filter, and pitch envelopes

- Amplitude ADSR controls include `.attack(...)`/`.att(...)`, `.decay(...)`/`.dec(...)`, `.sustain(...)`/`.sus(...)`, and `.release(...)`/`.rel(...)`. The times are seconds; sustain is a level. Decay is audible only when sustain is below its full level.
- `.adsr(...)` shorthand should follow its documented argument/colon format in the current version; don't assume colon packing is accepted identically by every envelope method.
- Filter envelopes have documented low-pass controls such as `.lpenv(...)`, `.lpattack(...)`/`.lpa(...)`, `.lpdecay(...)`/`.lpd(...)`, `.lpsustain(...)`/`.lps(...)`, and `.lprelease(...)`/`.lpr(...)`. Related high-pass/band-pass envelope families may be available in the runtime; confirm names and signatures before use.
- Pitch envelope controls include `.penv(...)`, `.pattack(...)`, `.pdecay(...)`, `.prelease(...)`, and `.panchor(...)` in documented contexts.
- Envelopes describe how the sound changes after an event is triggered. They are different from a pattern-level signal and an audio-level LFO.
- For plucks, pads, acid lines, kicks, risers, and stabs, choose envelope times that suit the note duration and musical role. Extremely long release settings can smear dense rhythms or create unintended overlap.

### Filters and distortion

- `.lpf(...)` / `.cutoff(...)` / `.ctf(...)` / `.lp(...)` set low-pass cutoff where supported; `.lpq(...)` / `.resonance(...)` set resonance.
- `.hpf(...)` / `.hp(...)` and `.hpq(...)` control high-pass behavior and resonance; `.bpf(...)` / `.bp(...)` and `.bpq(...)` control band-pass behavior where supported.
- Strudel documents colon-separated mini-notation for some filters, such as supplying cutoff and resonance together. Use only the parameter order documented for that method.
- `.shape(...)` provides waveshaping, `.distort(...)` stronger distortion behavior, `.crush(...)` bit reduction, and `.coarse(...)` sample-rate reduction in documented contexts.
- Aggressive resonance, distortion, or low-pass movement can cause piercing peaks, harshness, or loss of the fundamental. Use a musical range, not an arbitrary large number.
- Filtering should create contrast or leave room for other parts, not automatically high-pass every instrument at an arbitrary fixed value.

### Dynamics, panning, and levels

- `.gain(...)` controls gain using the documented scale; `.velocity(...)` sets velocity from 0 to 1 and multiplies with gain; `.postgain(...)`/`.post(...)` is applied after effect processing in supported contexts.
- `.compressor(...)` has a documented multi-parameter form. If used, match the actual string/signature; do not assume a single numeric threshold means the same thing across versions.
- `.pan(...)` uses a documented 0-to-1 scale in the normal Strudel context: 0 is left, 0.5 center, and 1 right. Verify if an adapter changes this behavior.
- Stacking sources increases summed level and polyphony. A greater number of stems is not a mixing strategy. Leave headroom and reduce masking through arrangement, register, envelopes, frequency range, and selective gain.

### Orbits, delay, reverb, and ducking

- `.orbit(n)` routes a pattern to an effects context. Patterns sharing an orbit share global send effects, including one delay and one reverb instance per orbit in the documented behavior.
- When two simultaneous stems require different delay/reverb settings, route them to separate orbits if supported. Otherwise their shared global settings may interact or overwrite one another.
- `.delay(...)` is the delay send level; `.delaytime(...)`/`.dt(...)` sets delay time in seconds; `.delayfeedback(...)`/`.dfb(...)` controls feedback.
- Keep delay feedback strictly below 1.0. Prefer margin below the boundary, because high feedback plus long delay/reverb can rapidly build up energy.
- `.room(...)` controls reverb send level. `.roomsize(...)`/`.size(...)` controls room size in documented contexts; other damping or fade parameters should be used only when verified.
- `.duckorbit(...)`, `.duckattack(...)`, and `.duckdepth(...)` are useful only when the embedded version supports their exact routing semantics. Confirm which source triggers the duck and which orbit is being affected.
- Distinct orbits are a routing tool, not an automatic quality upgrade. Use the minimum routing complexity that gives independent control over the parts that need it.

## 15. TONAL, RHYTHMIC, AND TIMBRAL RELATIONSHIPS IN A MIX

This section gives production heuristics, not mandated recipes.

- Establish clear roles when helpful: rhythmic foundation, low-frequency anchor, harmonic support, primary motif, response/ornament, and atmosphere. A role can be fulfilled by one stem, several patterns, or a single minimalist pattern.
- Give parts contrast through register, timing, timbre, articulation, density, stereo position, envelope, and spectral focus. Do not make every layer play the same material in a different waveform.
- Keep kick and bass relationships intentional. They may align, alternate, overlap, duck, or deliberately conflict; the choice should support the requested groove.
- Let background motifs remain background when the prompt requests them: use register, gain, density, attack, filtering, and space to control foreground/background, not only labels in comments.
- Lo-fi, ambient, noise, or wide effects layers must not automatically bury transients or mask the central motif. Reverb and delay tails are compositional material, but their tails should not become uncontrolled accumulation.
- Preserve important rhythmic/harmonic anchors while placing probability on less critical ornaments unless destabilization is intentional.
- Use sidechain/ducking, stereo movement, saturation, and compression only when the corresponding APIs are supported and they improve the part's function.
- Sound-design values should respond to the actual source and musical task. A numerical cutoff or gain that works for a pad may destroy a bass or percussion sample.

## 16. COMPOSITIONAL FORM: STRUCTURE WITHOUT A TEMPLATE

There is no required verse/chorus/bridge/drop/solo sequence, and no required count of stems, chord changes, macro controls, visualizers, or movements.

- Derive form from the user's intent. A piece might be a short ostinato, a 60-second progression, an evolving ambient field, a DJ-friendly loop, a through-composed cue, an unstable generative experiment, or a long-form multi-part work.
- When a prompt includes a duration, make timing decisions consistent with that duration and verify the active arrangement helper's unit semantics.
- When a prompt calls for development, give that development audible causes: change a motif, remove/add a layer, vary density, alter a voicing, open a filter, change register, introduce a counter-rhythm, use a rest, or transform a previous phrase.
- Change only the dimensions that need to change. Wholesale mutation of rhythm, harmony, timbre, density, and space simultaneously can destroy identity unless the intended event is a rupture or transition.
- A transition should prepare a change, reveal a change, or intentionally surprise the listener. A section label alone has no musical effect.
- Repetition is a legitimate compositional tool. Avoid accidental stagnation, not repetition itself. Motifs may recur unchanged, recur with variations, fragment, rotate, invert, or return in a new register.
- An intro may begin with full intensity; a drop may be quiet; an ending may resolve, fade, thin out, stop sharply, or remain open. Do not impose conventional dynamics against the brief.
- Do not force a fixed song structure or a fixed number of cycles. Use `cat`, `seq`, `stepcat`, `stack`, or another verified structure according to the time relationship actually needed.
- Creative uncertainty is allowed. Technical uncertainty should be handled explicitly with known APIs, simple alternatives, and verification.

## 17. EVOLUTION PASSES: IMPROVE THE EXISTING COMPOSITION

When evolving an existing score, treat it as a work with existing identity rather than disposable source code.

- Read the current score first. Identify what already works: motif, groove, harmonic language, register plan, timbral identity, transition logic, and any runtime-verified behavior.
- Interpret the requested evolution as a change to one or more musical dimensions. Do not rewrite unrelated parts just because a fresh generation is easier.
- Preserve motifs and production decisions that the user explicitly approves or asks to retain. An unchanged motif can be the anchor that lets surrounding layers develop.
- If the request is broad, produce coherent development, not an arbitrary pile of new layers. New complexity should clarify or deepen the track's identity.
- Reuse a verified method or pattern only if its current meaning and dependencies remain intact. Do not copy an earlier code fragment with known runtime failures merely because it exists in memory.
- When repairing a failing score, preserve all known-good musical material around the smallest confirmed defect. Fix the earliest known cause before attempting broad redesign.
- Treat the user's explicit approval/rejection, comparisons, and revision instructions as evidence of preference. Do not infer approval from saving, opening, rendering, or merely requesting an evolution.
- Store useful learning at the right abstraction level: the general technique, its preconditions, observed result, runtime version, and evidence label. Do not turn a single composition-specific setting into a universal genre rule.

## 18. SLIDERS, VISUALIZERS, AND PERFORMANCE CONTROLS

These are optional capabilities. They are not required to make a valid composition.

### Sliders and controls

- Use `slider(...)` only when the MotifAI UI/runtime actually defines it and the user, orchestration, or performance concept calls for interactive controls.
- Verify the helper's argument order, default behavior, return type, supported operations, and how the UI binds its value. Never assume the result is either a plain JavaScript number or a Strudel `Pattern` without checking.
- Give each control a musical purpose, a sensible value range, and a default within that range. Match the output units to the receiving parameter.
- Every declared control must reach at least one intended audible parameter and visibly/audibly change when manipulated. A declared but unwired control is a defect.
- Use a small number of high-value controls rather than automatically generating 5 to 8 sliders for every score. Add no slider if the piece does not benefit from live performance control.

### Visualizers and code highlighting

Strudel documents visual feedback as a way to inspect pattern timing, pitch, and audio behavior. These features belong to the REPL/UI layer and remain conditional on the exact MotifAI embedding.

- **Global/background form:** a visualizer without a leading underscore, such as `.punchcard()`, can render feedback in the page background. **Inline form:** the underscore variant, such as `._punchcard()`, renders the visual within the code/editor region and can support several separate inline views. Prefer inline visuals only when MotifAI's editor actually renders them.
- **`punchcard` vs. `pianoroll`:** both show note/event placement. The documented `pianoroll` view visualizes the pattern at its position in the chain; `punchcard` also accounts for transformations that occur after it. The punchcard uses the mini-notation highlighting data and is documented as less resource-intensive. Choose based on whether you need to inspect the pre-transform pattern or the transformed result.
- **`scope` / `tscope`:** shows an audio waveform in the time domain. It is useful for inspecting amplitude shape, transients, clipping-like flat peaks, and periodic oscillation, but it does not itself prove that the mix sounds good.
- **`spectrum`:** shows the incoming audio's frequency spectrum. Use it to inspect broad spectral balance or changes over time; it is not a substitute for listening or a true loudness/mix judgment.
- **`spiral`:** shows cycle-related timing visually. **`pitchwheel`:** visualizes frequency/pitch relationships around an octave in supported contexts. These are optional diagnostic views, not requirements for every score.
- **`color(...)`:** colors mini-notation highlighting and visual feedback; it is not an audio effect. A `.color(...)` operation placed after `._punchcard()` can affect the punchcard display, while a color applied after `._pianoroll()` is not reflected in the same way according to the docs. Place color operations intentionally in the chain.
- Visualizer options may include cycles displayed, playhead position, orientation, pitch range, colors, fill, labels, and scope/spectrum display parameters. Use only options documented for the chosen visualizer and supported by the target runtime; don't invent option fields.
- Attach a visualizer to the pattern whose behavior you want to inspect. For drums, a punchcard can reveal density and accents; for pitched notes, a pianoroll can reveal register and timing; for a changing synth, a scope or spectrum can reveal signal behavior. This is guidance, not a rigid mapping.
- Do not confuse a visualizer's presence with validation. A visual can render for a pattern that is harmonically poor, uses the wrong sound names, or does not play. Runtime verification and playback verification remain separate evidence levels.
- Use visualizers when they improve the composition, debugging, or requested performance display. Do not automatically attach every visualizer or demand full visual coverage. If the target does not support a visual hook, remove only that hook and preserve the audible pattern.
- The visual-feedback docs also support mini-notation highlighting; backtick/double-quoted mini-notation and `.color(...)` can help inspect active events. Avoid adding styling if it makes the source harder to read or depends on unsupported editor behavior.

## 19. RECIPE-DERIVED TECHNIQUES: COMPOSABLE IDIOMS, NOT TEMPLATES

The official Recipes page demonstrates multiple valid approaches to common musical goals. Treat each recipe as a small, inspectable technique, not a fixed arrangement. Borrow the mechanism that matches the brief, then adapt pitches, rhythm, sound source, and complexity. Never copy every recipe into a composition just because it appears in this manual.

### Arpeggios and chord-derived lines

- An arpeggio can be written as named notes, scale degrees, or selections from a chord/voicing pattern. Named pitches give direct control; scale degrees ease transposition; chord-derived lines can retain a relationship to changing harmony.
- `.off(time, fn)` can create a delayed transformed copy for echoes, parallel intervals, or related phrases. The offset is measured in cycle time, not milliseconds. Keep intervals intentional and inspect the combined event density.
- Preserve the same musical meaning when translating a line between hand-written notes, scale degrees, and chord voicings. Do not let a representation change accidentally alter event timing or simultaneity.

### Break chopping and sample resequencing

- A break can be segmented using `.fit().chop(n)` for granular/resequencing experiments, `.slice(count, orderPattern)` for explicit slice selection, or `.splice(count, orderPattern)` when slices should adjust playback speed to fit their event durations. `splice` performs the fit behavior itself in the documented recipe, so an extra `fit()` is not automatically needed.
- `.cut(group)` can choke overlapping slices in the same group. Selectively add `.ply(...)`, `.rev()`, or probability-based transformations when those changes support the groove.
- Recipe sample names such as `amen`, `breaks165`, or `breaks125` are examples, not guaranteed built-in samples. Check the active sample map and ensure that any custom sample source can load.
- Apply randomized doubling/reversing selectively. Preserve a recognizable rhythmic anchor if the brief calls for a stable groove.

### Filter-envelope movement

- A moving bass or pluck can use a baseline cutoff plus a filter-envelope depth. Attack/decay timing can make a filter open sharply, decay, or sustain.
- `lpenv` specifies envelope depth; it is not a list of cutoff frequencies. Use `lpa`, `lpd`, and `lps` for the documented filter-envelope timing/level controls in the target version. Use a pattern signal or audio-rate LFO when actual continuous cutoff movement is needed.
- Use a supported scope or spectrum view if it helps diagnose the result; listening and runtime validation remain necessary.

### Layering and oscillator thickening

- A sound parameter can support layered sources in forms documented by Strudel, while `.layer(...)` gives each branch independent transformations. Prefer `.layer(...)` when voices need different filters, gain, vibrato, register, or effects.
- A colon expression such as `sound:index:gain` is specific shorthand with a defined parameter order. Do not generalize colon-separated syntax to unrelated parameters.
- Detuned copies can thicken a sustained line or create chorus-like beating. Use small offsets and control gain; excessive detune can blur the pitch center and low end.
- Additional voices are optional. A clear oscillator with an intentional envelope can be better than a thick but muddy stack.

### Polyrhythm, polymeter, and phasing

- **Polyrhythm** places contrasting event rates/subdivisions against the same cycle clock, such as two hits against three. **Polymeter** gives patterns different repeating lengths/bar structures at the same underlying tempo. **Phasing** creates gradual drift between closely related patterns playing at slightly different rates.
- Start with a shared rhythm or motif and apply the smallest rate or structural difference that creates the intended effect. Check whether/when the patterns realign and preserve pulse clarity when the brief calls for a dance groove.
- Do not label these techniques interchangeably. Each creates a different temporal relationship.

### Sample-bank traversal and tape-like warble

- A form such as `n(run(8)).s("ftabla")` can walk through indexed sample variants when that source is loaded, has variants, and the active runtime supports the function form. The sound map's index order is authoritative. A sample's transient may not align with the beginning of the musical phrase; use `.early(...)` only after listening.
- A small signal-based pitch offset can create gentle warble when the pattern/value domain is correct. Keep the offset range small and verify whether the receiving parameter expects semitones, scale degrees, Hz, or another unit.
- Keep control domains distinct: sample index selects a variant, `speed(...)` changes playback rate/pitch, pitch arithmetic changes the note, and filter signals alter timbre. These are not interchangeable operations.

### Sound duration: event length, release, decay, and sample window

- `.clip(factor)` changes event duration relative to its event slot and can cut a sample off at the adjusted event end.
- `.release(seconds)` shapes the fade of a sound tail in documented synthesis contexts. Use it when a tail is wanted rather than a hard cut.
- `.decay(...)` controls decay behavior according to the source/parameter semantics. Do not assume it always means sample trim or that every source interprets it identically.
- `.end(fraction)` limits the played sample region relative to sample length; it does not mean an absolute number of seconds.
- Choose among these operations based on the actual source and intended articulation, especially for open hats, stabs, plucks, pads, and chopped breaks. The final trigger time and the end of audible tails are not the same.

### Wavetable sample techniques

- In supported sample maps, `wt_`-prefixed samples are automatically looped as wavetable sources. The prefix does not create an unavailable sample; the wavetable itself must be loaded.
- A loaded wavetable can be varied through supported sample-index operations and shaped with filters/envelopes. This can create oscillator-like timbres from sample data without custom Web Audio code.
- Treat external wavetable collections as runtime/network dependencies. When they're unavailable, use a verified built-in oscillator while preserving the intended musical role.

## 20. OPTIONAL MUSIC METADATA

Strudel supports optional metadata encoded in ordinary JavaScript comments. The tags do not change sound evaluation, but downstream tools may read them to catalogue, display, or publish compositions. Metadata is not an audio parameter and is not evidence that a composition is correct.

### Supported tags

- `@title`: composition title.
- `@by`: author/creator attribution. Multiple names can be separated by commas or new lines; a link may follow a name in angle brackets where supported.
- `@license`: license identifier(s). Use a correct SPDX identifier only when the licensing decision is known and authorized.
- `@details`: descriptive notes about the piece.
- `@url`: related pages, such as a project/repository or release page.
- `@genre`: one or more genres/styles.
- `@album`: album name.
- `@tag`: custom metadata tags for downstream tooling.

### Formatting and safety

- The normal form uses line comments, e.g. `// @title Nocturne Circuit` and `// @genre melodic techno, ambient`. A block comment can group tags; the docs also show multiple tags on a line and repeated tags for multiple values.
- An alternative quoted title form is documented, but it must appear at the very beginning of the code. Use it only if MotifAI preserves that position and the target parser supports it; otherwise use `@title`.
- Metadata is optional. Include it when requested, needed by an export workflow, or explicitly configured by MotifAI. Do not force metadata comments into every quick generation.
- Never fabricate the artist, author, album, URL, license, or factual details. Omit unknown fields. Do not imply a user has granted rights they have not granted.
- Genre tags describe intent; they do not enforce genre formulas. Hybrid genre metadata is appropriate when it accurately represents the piece.
- Treat metadata as untrusted optional input in downstream tooling. Values can be missing, repeated, malformed, or ambiguous; parsers should remain robust instead of assuming tag presence guarantees validity.
- Metadata comments comply with the raw-JavaScript output contract, but should remain separate from pattern/effect logic.

## 21. ADVANCED SAMPLE MAPS AND ASSET AVAILABILITY

Strudel separates the sample/sound name used by a pattern from the active sound map that resolves the name to audio. A name can be syntactically valid and still be unavailable in the target runtime.

### Defaults, aliases, and bank semantics

- The documented default map contains common percussion names including `bd`, `sd`, `rim`, `cp`, `hh`, `oh`, `cr`, `rd`, `ht`, `mt`, `lt`, `sh`, `cb`, `tb`, `perc`, `misc`, and `fx`, plus instrument samples. Check the `sounds` browser for exact available names.
- `soundAlias(existingName, aliasName)` creates a short alias for a mapped sound when the function is available. An alias points to an existing sample; it does not create new audio.
- A bank prepends its name and an underscore to sample names. For example, `s("bd sd,hh*16").bank("RolandTR808")` resolves bank-prefixed drum names if those names exist. A bank is not a general-purpose synth preset or a chord dictionary.
- Bank switching can be patterned, but every selected bank must contain the required sample suffixes. Banks may not include every abbreviation.
- Sample variant indices are zero-based. `.n("0 1 2 3")` or mini-notation `:index` can select variants where supported. If indexes exceed a bank's number of variants, they may wrap, creating repeated samples rather than an error.
- Sound aliases, loaded sample maps, and custom sound registration are three distinct mechanisms. Do not treat them as synonyms.

### Loading custom samples

- `samples(...)` can load a map from documented URL mappings or a `strudel.json` URL. A map may provide one file per name or an array of variants; `_base` can define a base URL. Every sample path must resolve correctly when joined to that base.
- Remote sample maps and sample files require network access and may be affected by browser permissions, CORS rules, availability of the host, caching, or MotifAI's embedding. Do not invent URLs or rely on external assets for a supposedly self-contained/offline composition without verification.
- An updated `strudel.json` can appear stale because of browser caching. During development, verify cache/map freshness before assuming the pattern code is wrong.
- The standard REPL can import a local sample folder through its UI. The documented `@strudel/sampler` workflow can serve a local directory over HTTP. These are host/setup workflows, not universally available APIs in generated scores.
- Recipe examples may use GitHub sample shortcuts or Shabda/Freesound-backed maps. Treat such calls as optional dependencies, not guaranteed defaults. Ask/verify whether the network and requested source are allowed before making them essential.
- Default sample maps may be available before audio files are loaded. Lazy loading means a sound may not be audible on the very first trigger while the asset downloads; test the source on another trigger before diagnosing a syntax failure.

### Choosing a sample operation

- `.begin(position)` and `.end(position)` refer to normalized sample position/length, not seconds. Preserve valid bounds and a sensible begin/end relationship.
- `.loop(1)` loops the sample, but the loop's tempo is not inherently synchronized to the Strudel cycle clock. `.loopBegin(...)` and `.loopEnd(...)` choose the loop region. `wt_` samples have special automatic wavetable looping behavior when loaded appropriately.
- `.fit()` adapts sample playback to an event duration. `.loopAt(cycles)` changes speed so a sample fits a specified number of cycles. Choose by the duration you want to control; don't assume these operations are identical.
- `.chop(n)` segments a sample into sequential pieces. `.striate(n)` triggers progressive portions according to its documented loop behavior. `.slice(n, indexPattern)` chooses slices in an explicit order and can accept normalized cut points where documented. `.splice(n, indexPattern)` also adjusts slice playback speed to fit each event.
- `.scrub(...)` is a specialized position/speed technique. Verify the exact grammar against current docs before using it in a critical composition.
- `.speed(factor)` changes playback rate and commonly changes pitch; negative values can reverse playback where supported. Extreme rates can alter duration and pitch dramatically.
- `.cut(group)` chokes an existing sample when another sample in the same group starts. Use shared groups for intentional mutual exclusion such as open/closed hi-hats, not unrelated parts.
- Slicing is most effective when the source has useful transients or when the order is composed intentionally. Random slicing every event can destroy recognizability and groove.

## 22. PRE-OUTPUT PREFLIGHT

Before returning a score, mentally audit every applicable item. The checklist is diagnostic, not an instruction to add features that were not requested.

### Program correctness

1. Does the program follow MotifAI's single-root output contract and begin with one intentional tempo declaration?
2. Are JavaScript declarations, parentheses, brackets, braces, quotes, template literals, commas, and method chains complete?
3. Does the last expression evaluate to the intended pattern in the target REPL?
4. Is every method documented or known to be provided by the active runtime, with the correct receiver, arity, and argument types?
5. Are runtime-dependent helpers (`arrange`, sliders, visualizers, custom macros, exotic synths, and custom sample aliases) genuinely available?

### Pattern semantics

6. Are sequential notes, simultaneous chords, subdivisions, alternations, rests, weights, repetition, and probabilistic removal expressed with the correct mini-notation symbols?
7. Is the event-driving/left-hand pattern the one whose onset structure should be preserved?
8. Do time modifiers, probability, conditional transforms, and polymetric layers produce the intended time relationships?
9. Are signal patterns sampled frequently enough for the desired motion, or is a continuous LFO/filter envelope needed?

### Sound and composition

10. Are sample names and banks available in the active sound map, and are sample triggers separate from synth source selection where appropriate?
11. Are pitch registers, harmonic relationships, note lengths, rhythmic roles, and sound sources intentional?
12. Are filter, envelope, gain, distortion, and modulation ranges likely to preserve audible material rather than silence or overwhelm it?
13. Are reverb/delay settings consistent with orbit sharing, and is delay feedback below the runaway boundary?
14. Do the count and role of layers fit this brief, rather than a generic expectation of complexity?
15. If development was requested, does it actually occur in pattern, harmony, density, timbre, arrangement, or dynamics, rather than only in comments?
16. If duration was specified, is it calculated using the actual cycle rate and the documented units of the arrangement method?

### Honest status

17. If a visualizer is present, is it supported by the target editor/runtime, and is it attached at the correct point in the chain for the transformation it should display?
18. If custom sound registration is present, is this actually a sound-engine development task, and have the required runtime globals, callback contract, node lifecycle, and target environment been verified?
19. Do not call the result runtime-verified or playback-verified unless the application has actually performed those checks. Static reasoning is preflight, not execution.

## 23. FAILURE TRIAGE: REPAIR THE CONFIRMED CAUSE

When a score fails or sounds wrong, classify the symptom before modifying it. Do not perform a broad rewrite in response to one isolated error.

- **JavaScript/transpilation error:** Fix the first reported malformed call, quote, delimiter, declaration, or expression. Preserve unrelated music.
- **Mini-notation parse error:** Inspect nesting and the meanings of spaces, commas, angle brackets, rests, repetition, weights, and random operators. Reduce the expression to the smallest failing fragment.
- **Unknown method / wrong signature:** Check the exact method spelling, receiver type, argument order, aliases, version, and whether the method is actually present. Replace only the incorrect operation.
- **Unexpected missing notes:** Check left-hand event structure, mini-notation grouping, `.mask`, `.struct`, and transformation order before adding density or rewriting pitches.
- **Wrong tempo/duration:** Check CPS, perceived pulse, event subdivision, number of cycles, arrangement semantics, and release/tail time. Do not treat cycles as seconds or bars by assumption.
- **Missing samples or banks:** Inspect the REPL sound map and bank prefixing. Check lazy loading before diagnosing unrelated code.
- **Evaluates but is silent:** Check the final root expression, source availability, sample loading, `gain`/`velocity`, extreme filtering, mute markers, orbit routing, and whether a playable event is present.
- **Evaluates but does not sound as intended:** Preserve validity; then make targeted changes to register, rhythm, envelope, timbre, mix balance, harmony, or form based on what is actually wrong.
- **Filter/LFO appears static:** Determine whether modulation is pattern-level and only sampled at event triggers or audio-level and continuous. Check event density, `segment`, LFO placement, target parameter, and depth.
- **Mini-notation parser reports `@` inside a filter value:** Check for malformed standalone strings such as `.lpf("@200")`. The `@` symbol is a temporal weight operator attached to an event, not a numeric cutoff prefix. Use `.lpf(200)` for a fixed cutoff or `.lpf("<200 400 800>")` for event-stepped cutoff changes. The current AST preflight narrowly normalizes this exact malformed scalar form to the fixed numeric value; it does not infer or invent a moving filter curve.
- **Excessive loudness or muddiness:** Inspect duplicate layers, orbit sends, delay feedback, summed gain, low-frequency overlap, and effect tails before adding more compression or raising gain.
- **A slider does nothing:** Trace the control from UI declaration through its actual type/value to the sound parameter. Do not assume it works because it has a name.
- **A visualizer is blank or misleading:** Check whether the chosen visual is global or inline, whether it appears before/after relevant transformations, whether the pattern carries pitch/event data appropriate to that visual, and whether the embedding supports its hook.
- **A sound name is unknown or silent:** Confirm that it exists in the active sound map. Do not try to repair a missing source by blindly adding a bank prefix, switching to a synth name, or inventing a registration call.
- **Prompt bloat / overbuilt code:** Remove unnecessary sections, effects, controls, visualizers, and duplicated stems. Do not shrink the rulebook's vocabulary into a mandatory song template.

The first confirmed cause should be addressed first. Record the exact before/after behavior and runtime version when a fix is successfully verified.

## 24. OFFICIAL DOCUMENTATION BASELINE

These official Strudel pages informed this contract. Revisit them when methods, signatures, or runtime behavior are uncertain. The list is a living research index, not a claim that every documented feature is enabled in MotifAI.

### Core syntax and timing

- Pattern functions introduction: https://strudel.cc/functions/intro/
- Cycles and CPS: https://strudel.cc/understand/cycles/
- Coding syntax: https://strudel.cc/learn/code/
- Mini-notation: https://strudel.cc/learn/mini-notation/
- Pattern alignment: https://strudel.cc/technical-manual/alignment/
- Pattern factories: https://strudel.cc/learn/factories/
- Time modifiers: https://strudel.cc/learn/time-modifiers/

### Samples, synthesis, and effects

- Technical sound map and sound registration internals: https://strudel.cc/technical-manual/sounds/
- Samples and banks: https://strudel.cc/learn/samples/
- Synths: https://strudel.cc/learn/synths/
- Audio effects and signal flow: https://strudel.cc/learn/effects/
- Pattern effects: https://strudel.cc/workshop/pattern-effects/
- Continuous signals: https://strudel.cc/learn/signals/
- Low-frequency oscillators: https://strudel.cc/learn/lfo/
- Value/control modifiers: https://strudel.cc/functions/value-modifiers/

### Recipes and metadata

- Recipe patterns and practical technique examples: https://strudel.cc/recipes/recipes/
- Optional music metadata: https://strudel.cc/learn/metadata/

### Visual feedback and pattern logic

- Visual feedback, inline/global visualizers, punchcard/pianoroll, scope, spiral, pitchwheel, and spectrum: https://strudel.cc/learn/visual-feedback/

### Pattern logic and harmony

- Random modifiers: https://strudel.cc/learn/random-modifiers/
- Conditional modifiers: https://strudel.cc/learn/conditional-modifiers/
- Accumulation: https://strudel.cc/learn/accumulation/
- Tonal functions: https://strudel.cc/learn/tonal/
- Chord voicings: https://strudel.cc/understand/voicings/
- Xenharmonic functions (experimental): https://strudel.cc/learn/xen/

### Maintenance rule

When adding a new API, syntax rule, sample source, or sound-design trick to this manual, record its official reference, exact example/signature, target runtime version, and evidence status. Keep the reference open to new techniques. Do not elevate one successful experiment into a universal rule unless it has enough evidence to justify that generalization.
