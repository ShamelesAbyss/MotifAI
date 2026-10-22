# MOTIFAI // STRUDEL PRODUCTION, SYNTAX & RUNTIME CONTRACT

**Purpose:** This file is the technical contract for generating Strudel programs in MotifAI. It defines safe syntax, composition primitives, sound-design practices, compatibility boundaries, and a final preflight checklist. It is not a fixed songwriting template and must not be used to force every composition into the same arrangement.

**Source baseline:** Official Strudel documentation linked at the end of this file, checked on 2026-10-21. MotifAI's actual embedded REPL and any MotifAI-specific helpers remain authoritative for runtime compatibility. Strudel evolves; when documentation and the embedded runtime disagree, record the discrepancy and validate against the pinned runtime rather than guessing.

---

## 1. OUTPUT CONTRACT: RETURN A COMPLETE PROGRAM

- Output only raw, executable Strudel JavaScript. Do not wrap it in Markdown fences and do not add conversational prose outside `//` comments.
- Begin with one intentional top-level tempo declaration, normally `setcps(...)`.
- Declare constants after the tempo line when they make the program easier to understand. Finish with one clear root pattern expression, such as `stack(...)`, `cat(...)`, `arrange(...)` when supported, or a named pattern variable.
- Every expression and method chain must have a real receiver. Never begin a chain with a detached expression such as `.lpf(800)` or `.gain(0.5)`.
- Close every `()`, `[]`, and `{}`. Pass the correct argument types and counts to functions. Do not leave placeholders, ellipses, TODOs, empty song sections, or half-written chains in the output.
- The final root expression must evaluate to a pattern or another return type explicitly supported by the target REPL. Do not nest a complete arrangement inside an unrelated `stack(...)` merely to make the code look organized.
- Comments may explain intent, but comments do not make unsupported syntax valid. Code takes precedence over the comment's claim.
- Produce the simplest complete program that meets the musical brief. Additional layers, slider controls, visualizers, effects, and section boundaries are optional unless the user explicitly asks for them or they clearly improve the composition.

## 2. RUNTIME COMPATIBILITY: CORE STRUDEL VS. MOTIFAI EXTENSIONS

- Strudel patterns are written in JavaScript-like syntax with Strudel mini-notation and pattern functions. The REPL transpiles that syntax, evaluates it, and schedules the resulting pattern. A JavaScript-looking expression is not automatically a valid Strudel pattern.
- Treat the target environment as the Strudel REPL embedded by MotifAI, not an abstract or imagined Strudel version. Available globals, loaded samples, synths, visualizers, and helper functions can differ by runtime version or configuration.
- Use documented APIs when possible. If an unfamiliar method or helper is not documented for the target runtime, do not invent it. Either use a verified equivalent or keep the design simpler.
- The library of methods in this file is a curated reference, **not an exhaustive whitelist**. New methods may be used when verified against the official documentation or the actual embedded runtime. Do not treat a missing entry as proof that a method does not exist.
- `slider(...)` is runtime-dependent for MotifAI. Do not assume that a particular Strudel installation provides it, what it returns, or how it is rendered. Use it only when the embedded runtime or MotifAI adapter actually defines it. Verify its argument order, return type, UI wiring, and supported control operations before relying on it.
- The same rule applies to helper APIs such as `arrange(...)`, custom macros, automatic visualizer hooks, and any project-specific control bridge: use them only when the active runtime supports their exact signatures.
- Do not add imports, fetch calls, DOM access, external packages, Node.js APIs, or arbitrary JavaScript runtime assumptions to generated user code unless MotifAI explicitly supports them.
- When a stable runtime version is available, pin it in the application rather than silently depending on `@latest`. Documentation and generated output should target the same version.

## 3. TEMPO, CYCLES, AND REAL-WORLD DURATION

- `setcps(value)` sets **cycles per second**, not beats per minute.
- For the conventional four-beat cycle, the approximate conversion is `CPS = BPM / 240`, and `BPM = CPS * 240`. Values around `0.25` to `0.85` CPS can be a useful starting search range for many grooves, not a hard limit; slower ambient or faster experimental material may sit outside it. Examples: `setcps(0.375)` is about 90 BPM, `setcps(0.5)` is about 120 BPM, `setcps(0.58)` is about 139 BPM, and `setcps(0.75)` is about 180 BPM.
- Never pass a raw BPM integer such as `setcps(120)` when the intent is 120 BPM.
- Select tempo for the musical brief. A moderate or fast tempo is not automatically better; half-time, double-time, broken-beat, ambient, and freely phrased material may need different choices.
- If the target supports `arrange([cycles, pattern], ...)`, its duration can be estimated as `total_cycles / CPS` seconds. For a target duration in seconds, estimate `target_cycles = target_seconds * CPS`, then confirm how the target arrangement helper interprets cycle counts.
- Do not confuse cycles, beats, bars, and seconds. Do not assume a number passed to `arrange` means bars or seconds without checking that API's documented semantics.
- If an arrangement uses sections with explicit cycle counts, calculate their sum and estimate its duration. Keep a reasonable margin for tails and reverb, which may continue after the final triggered note.
- A continuous generative pattern may intentionally have no fixed duration. Follow the user's requested form rather than manufacturing section counts just to satisfy a rule.

## 4. MINI-NOTATION: GRAMMAR THAT MUST STAY CONSISTENT

Mini-notation is a small pattern language inside strings. The following structures serve different purposes; do not mix them up:

- **Sequential events:** whitespace separates events in sequence. Example: `note("c4 e4 g4 b4")`.
- **Grouped subdivision:** square brackets group a nested sequence into a time slot. Example: `note("[c4 e4 g4 b4] [d4 f4 a4 c5]")`.
- **Simultaneous events / chords:** commas combine events at the same time. Example: `note("[c4,e4,g4]")`.
- **A sequence of chords:** combine comma-separated notes inside each chord, then place chords in a sequence or alternate them. Example: `note("<[c4,e4,g4] [d4,f4,a4]>")`.
- **Alternation:** angle brackets choose among events or grouped patterns across the pattern's cycle structure. Example: `note("<c3 eb3 g3 bb3>")`.
- **Rest:** use `~` or `-` to leave silence. Example: `s("bd ~ sd ~")`.
- **Repetition / speed:** `*N` repeats a token or group more densely; `/N` stretches/slows a token or group across more time; `!N` repeats a value without increasing its speed. Use parentheses or brackets to make the intended group unambiguous.
- **Euclidean rhythms:** append a pulse/step expression to an event in mini-notation, for example `s("bd(3,8)")`. The third optional value can rotate the placement, for example `s("bd(3,8,1)")` where supported by the parser.
- **Probabilistic events:** `?` can make events occur probabilistically, with an optional probability value documented by Strudel. Use probability to add controlled variation, not to make every foundational kick, bass note, or harmonic anchor disappear unpredictably.
- **Weighted events:** `@` changes the relative duration/weight of a grouped item when used according to the mini-notation grammar.
- Prefer simple notation while composing. Add nesting only when it creates a clear rhythmic or melodic result. Deeply nested notation is harder for small language models to write correctly and for humans to debug.
- Never prepend or append an empty pattern such as `"[]" + "<c3 e3 g3>"` when the actual intended pattern can be written directly as `"<c3 e3 g3>"`.
- A string is parsed as mini-notation in the Strudel REPL context. Do not treat mini-notation punctuation as ordinary JavaScript syntax outside the string. Use double quotes for mini-notation strings consistently in generated code, as a project convention; do not gratuitously mix quote styles.

## 5. PITCH SPELLING, REGISTER, AND HARMONY

- Use documented note names with an octave number. The standard sharp form uses `#` and the standard flat form uses `b`; examples: `c#4`, `eb4`, `f#3`, `bb2`. Do not assume `s` is a universally supported spelling for sharp.
- Use `note(...)` for note names. Use `n(...)` for note/sample indices where that is the intended API, and use frequency notation only when it serves a clear sound-design purpose.
- Separate chords from melodies. A chord uses simultaneous notes, normally comma-separated; a melody uses sequential notes. Do not write several sequential notes and describe them as a simultaneous chord.
- Keep octave/register choices intentional. Sub-bass often occupies a lower register than a mid-bass riff; pads and leads should not all compete in the same frequency range by default.
- Choose a tonal center, mode, harmonic vocabulary, or deliberate non-tonal strategy that serves the brief. These are options, not universal restrictions. Chromatic notes and modulation are appropriate when musically intentional. Microtonal or deliberately inharmonic material is also allowed when the requested sound calls for it, but only use tuning or detuning APIs confirmed in the active runtime.
- For tonal material, maintain voice-leading and a meaningful relationship between the bass and upper voices. Chord changes can be sparse; melodic motion and rhythm can create development without changing harmony every bar.
- Avoid accidental harmonic incoherence caused by mixing unrelated note sets simply to create variation. If contrasting sections change key or mode, make the transition intentional enough to sound authored.
- Do not assume that a scale name, chord helper, voicing function, or tuning feature is loaded unless documented for the active environment.

## 6. PATTERN COMPOSITION: SEQUENCE, LAYER, TRANSFORM

- `stack(patternA, patternB, ...)` layers patterns in parallel. Use it for simultaneous roles such as drums, bass, chord bed, lead, and texture.
- `cat(patternA, patternB, ...)` concatenates patterns in sequence. Use only when its semantics fit the desired temporal structure.
- `arrange([cycles, patternA], [cycles, patternB], ...)` is useful in Strudel environments that expose the `arrange` helper. Verify availability and the cycle-count semantics in MotifAI's actual REPL. If unavailable, use supported sequential pattern composition instead.
- One final root expression should define the intended output. Constants are definitions, not separate audible songs. Do not leave multiple unrelated root expressions when only the final evaluated expression will be played.
- Prefer `stack(...)` when combining different instrument types. A comma inside mini-notation can layer events within a single pattern string, but it does not replace `stack(...)` for general-purpose composition of separately processed patterns.
- Pattern methods are functions with defined signatures. Respect argument count and argument types. In particular, methods that transform another pattern commonly expect a callback, not an arbitrary standalone pattern.
- `.superimpose(transform)` transforms a version of the pattern it is called on; do not pass an unrelated `note(...)` pattern unless the actual API documents that signature. When adding a separately composed counter-melody, use `stack(existing_pattern, new_pattern)` for straightforward parallel layering.
- `.every(n, transform)` and conditional modifiers such as `.sometimes(transform)` take transformations where documented. Do not pass mini-notation strings to callback parameters or invent overloads. If the exact signature is unknown, consult the official documentation or choose a simpler construction.
- `.mask(...)` and `.struct(...)` have their own documented pattern-structure inputs. Use mini-notation strings where supported, and confirm that a mask's rhythm aligns with the pattern it is controlling.
- Keep related patterns in named constants when it improves readability and enables deliberate evolution. Do not create many constants that all duplicate the same pattern without a musical reason.

## 7. RHYTHM AND DRUM PROGRAMMING

- Build an identifiable rhythmic foundation before adding decorative percussion. Kick, snare/clap, closed hat, open hat, and fills serve different functions; choose only the elements the style needs.
- Simple, documented sample pattern: `s("bd sd, hh*8")`. A banked drum pattern can look like `s("bd sd, hh*8").bank("RolandTR808")` when that bank and those sample names are available in the configured sample map.
- Use `.bank("RolandTR808")`, `.bank("RolandTR909")`, or `.bank("RolandTR707")` on sample-trigger patterns when the corresponding names exist in the active sound map. A bank prefixes sample names; it is not a synthesizer oscillator.
- Do not use a drum-machine bank as if it were a pitched keyboard or synth preset. A `note("c3 eb3 ...").bank("RolandTR909")` chain should not be the default for musical bass or a lead. Use a supported synth source for pitched synthesis, or deliberately trigger pitched samples with verified sample semantics.
- Not every bank contains every sample name, and a particular `hh`, `cp`, or `rim` sample may be missing. Do not assume all banks are complete. First-time playback may also be delayed while samples load.
- Add syncopation, ghost notes, swing-like displacement, sparse fills, and density changes only when they support the groove. Avoid filling every subdivision simply because faster patterns are possible.
- Euclidean rhythms can generate useful pulse distributions, but they are one tool among many. Do not make all parts Euclidean or force the same kick pattern onto every genre.
- Check the mix: kick and sub-bass can mask each other. Consider rhythm, note duration, filtering, or selective ducking where actually supported, rather than simply increasing every gain.

## 8. SYNTHESIS AND SOUND SOURCES

- Use a documented synthesis source for pitched sound. A common form is `note("c3 eb3 g3").s("sawtooth")`; source availability still depends on the target synth setup.
- Documented oscillator-style sources used in Strudel contexts include names such as `sawtooth`, `supersaw`, `square`, `triangle`, `sine`, and `pulse`. Treat this as a practical starting list, not a guarantee that every source is exposed in every custom installation.
- Use `s("..." )` for sample/sound names and `note("...")` to express pitches. The `.s(...)` method can select a source for a note pattern in standard Strudel usage; verify the exact source name if playback is silent.
- Make roles distinct through register, envelope, timing, timbre, stereo position, and spectral range. A bass, pad, and lead should not all be the same chord pattern with different colors attached.
- For a bass, consider a suitable low register and enough upper harmonics to remain audible on smaller speakers. A cutoff of 40–80 Hz can remove most audible harmonics; do not derive filter cutoffs from a slider without checking the resulting range.
- For pads, slower note changes, longer envelopes, and restrained spatial effects can support atmosphere. For leads, a recognizable motif plus variation often works better than an unchanging scale run.
- `.attack(...)`, `.decay(...)`, `.sustain(...)`, `.release(...)`, and `.adsr(...)` are envelope-related controls available in Strudel contexts. Use documented values and verify the chosen source responds to them.
- `.shape(...)`, `.crush(...)`, `.coarse(...)`, `.fm(...)`, `.detune(...)`, `.vib(...)`, `.phaser(...)`, and `.tremolo(...)` are examples of timbre or modulation tools. Do not pile them on by default; choose effects for a defined sound-design purpose.
- Use filters in sensible frequency ranges and apply them to intended roles. `lpf(...)` attenuates high frequencies; `hpf(...)` attenuates low frequencies; `bpf(...)` focuses a frequency band. Resonance controls such as `lpq(...)` and `hpq(...)` can make the cutoff more pronounced. Verify actual units and range for each parameter.
- Do not assume every method in a list applies to every sound source, sample, or installed version.

## 9. EFFECTS, ORBITS, AND MIX CLARITY

- Use actual documented parameter names. For example, `.room(...)` sets reverb level; `.roomsize(...)` or its documented alias controls reverb size. Do not invent `.reverb(...)` just because the creative intent is reverb.
- `.delay(...)` controls delay send level. `.delaytime(...)` sets delay time in seconds. `.delayfeedback(...)` controls feedback. Setting only delay time and feedback does not necessarily send audible signal into the delay; set a deliberate `.delay(...)` level as well.
- Keep delay feedback below `1` unless a special experiment has been intentionally verified. Feedback at or above `1` can build up and grow dangerously loud.
- Strudel global effects are shared by patterns routed to the same orbit. If two patterns on one orbit specify different global reverb or delay settings, later pattern settings may affect the shared context. Use distinct `.orbit(n)` values for parts that genuinely need different global effects, and avoid unnecessary orbit proliferation.
- `.orbit(...)` routes a pattern to an effects context; it is not a substitute for setting an effect's send level. Route deliberately and keep gain under control.
- `.gain(...)` should receive a meaningful value or supported control pattern. Never write `.gain()` with no argument unless the exact runtime documents that use.
- Use panning, reverb, delay, filtering, distortion, and bit reduction in service of the sound. Keep low-end information relatively focused and prevent long tails from smearing rapid percussion or masking a later section.
- Effects can make code longer without improving the music. Start with a dry or minimally processed sound, then add a small number of intentional effects.
- Watch summed levels when stacking layers or duplicating a sound. More layers can increase loudness and masking; do not solve every mix issue by turning parts up.

## 10. MOTIFAI SLIDERS AND INTERACTIVE CONTROLS

Interactive controls are a capability, not a required ornament. Do not add sliders to every song automatically.

- Only generate a slider if the user requested controls, the orchestration layer asks for them, or a live parameter adds meaningful performance value.
- Before using `slider(default, min, max, step)`, verify that the active MotifAI runtime exposes this exact helper and argument order. This is a runtime extension unless the active Strudel runtime explicitly documents it.
- Verify the slider's return type. If it returns a Strudel `Pattern`, pattern methods such as `.mul(...)` and `.div(...)` may be appropriate when supported. If it returns a JavaScript number, numeric arithmetic may be appropriate instead. Never assume one behavior without checking the implementation.
- Give each slider a useful name, sensible min/max bounds, a valid default within those bounds, and a step size appropriate to the parameter. Avoid scales whose values are technically legal but musically useless.
- Connect every declared slider to at least one intended audible parameter. A declared-but-unused slider is a defect. A slider named `Atmosphere` should control an actual atmospheric parameter, not bass gain by accident.
- Avoid duplicating controls that act on the same parameter without a reason. Prefer a few high-value controls over a long menu of cosmetic knobs.
- Check unit mapping. Example: if a slider value between `0.2` and `0.8` is multiplied by `100`, the target becomes `20` to `80`, not `200` to `800`. If mapping to filter frequency, gain, room size, or delay feedback, ensure the resulting range matches the parameter's real units.
- Never call a pattern method on a variable until the variable's type and supported methods are known. Never assume `.mul(...)` or `.div(...)` exists on an ordinary number.
- A control is not considered functional because its name appears in the code. It must be wired to an audible parameter and produce a change when manipulated in MotifAI's UI.

## 11. VISUALIZERS ARE OPTIONAL AND MUST NOT BREAK AUDIO

- Inline visualizers such as `._scope()`, `._pianoroll()`, `._punchcard()`, `._spiral()`, `._scopex()`, `._pitchwheel()`, and `._spectrogram()` can provide useful feedback where the embedded REPL supports them.
- Visualizers are not required for every stem, every track, or every section. Do not attach one to each pattern merely to satisfy a prompt instruction.
- If visual feedback is useful, attach a compatible visualizer to one or a few relevant stems and use `.color("#RRGGBB")` only where color metadata is useful. Colors are not audio processing.
- If a visualizer causes evaluation or scheduling trouble, remove the visualizer first while preserving the underlying sound pattern. Never sacrifice playability for decoration.
- Treat `_`-prefixed visualizer methods as REPL/UI-specific rather than universal audio-engine methods. Confirm support in the embedded version.

## 12. ARRANGEMENT AND MUSICAL DEVELOPMENT WITHOUT A FIXED TEMPLATE

There is no mandated verse/chorus/bridge/drop/solo sequence. Let the user's brief and the composition's own musical idea determine its form.

- Possible structures include an intro and outro, a gradual ambient evolution, a groove with variations, a short focused loop, a through-composed piece, a sequence of movements, a long-form generative texture, or something else entirely.
- If the brief asks for distinct sections, give them distinguishable musical functions. Change selected elements over time: density, motif, harmony, register, rhythm, texture, filter movement, articulation, or space. Do not change everything at once without purpose.
- Develop motifs. A lead solo should generally have a phrase, a response, a variation, and/or a climax; merely naming a repeated chord pattern `melodicSolo` does not create a solo.
- Repetition is musically valid. Avoid accidental repetition, not repetition itself. Reprise an earlier idea with a meaningful change when that serves the piece.
- If a named section has no musical contrast, remove the label or create the contrast. Names and comments are not arrangement.
- Choose the amount of layering appropriate to the style. A minimalist piece may need only two patterns; a dense track may need several carefully separated roles. Do not force a fixed number of layers, parts, slider controls, visualizers, or sections.
- Make transitions audible when the form requires them. Use changes in density, rests, fills, filtering, note length, harmony, register, or effects. Use silence intentionally when it adds impact.
- Endings can resolve, fade, thin out, collapse to a single motif, or remain open-ended according to the brief. Do not always fade out just because it is easy.
- Preserve creative autonomy. Production knowledge and prior memories are optional sources of inspiration, not commands to reproduce a previous composition or force a specific genre's conventions onto unrelated music.
- When the user supplies separate vision and style prompts, treat both as important. The vision defines the musical outcome and emotional journey; the style direction shapes the palette and production character. Reconcile tensions creatively instead of simply concatenating every request into one oversized arrangement.

## 13. ORIGINALITY, REFERENCES, AND PRODUCTION-KNOWLEDGE USE

- Use the production knowledge library to learn **techniques, principles, and capabilities**, not to copy a complete song.
- The user's community repositories (including Awesome Strudel and Strudel Songs Collection) are discovery and research pointers. Do not paste their songs, covers, riffs, lead melodies, distinctive drum patterns, or arrangements into prompts, the knowledge library, or newly generated music.
- Prefer independently written examples that illustrate a narrow technique, with source attribution, provenance/reuse notes, and a clear verification status.
- Keep these evidence labels distinct: documentation-supported means a reference appears in documentation; syntax-reviewed means its code was inspected; runtime-verified means it was actually executed in the target environment. Never promote an entry to runtime-verified without a real successful run.
- Retrieved examples are inspiration only. Recombine general concepts into new notes, rhythms, timbres, and structures. Do not mechanically reproduce example code when a better original arrangement fits the user's brief.
- Do not confuse a prior generation's success, the presence of an API name, or a model's confidence with proof of runtime validity or musical quality.

## 14. PRE-OUTPUT TECHNICAL PREFLIGHT

Before returning code, check every applicable item:

1. **JavaScript structure:** Are all parentheses, brackets, braces, strings, declarations, commas, and semicolons valid? Is there any stray leading method chain, empty function argument, unfinished block, or mismatched closing delimiter?
2. **Strudel expression:** Does the code's final expression evaluate to the intended pattern? Are `stack`, `cat`, `arrange`, and other helpers used with their actual signatures and correct types?
3. **Mini-notation:** Are commas, spaces, rests, brackets, angle brackets, and modifiers being used according to their distinct grammar? Are empty `[]` groups accidental?
4. **Method compatibility:** Is each method real in the target runtime, with the correct capitalization, receiver type, arguments, and callback signatures? Is a method from another music framework being mistakenly used here?
5. **Source / bank compatibility:** Does each sample name or bank exist in the target sample map? Is a drum sample bank being incorrectly used as a synth preset? Does a synth source have the intended pitch behavior?
6. **Slider integrity:** Are controls supported, correctly typed, sensibly scaled, and actually connected to the named parameters? Are any variables declared and never used?
7. **Audible result:** Are there active sound-producing patterns? Could the filters make all useful frequencies inaudible? Are the samples likely to exist? Do layers overlap in a way that will mask one another or overload the mix?
8. **Musical result:** Is there a coherent musical identity? If the prompt requires development, are motifs and sections genuinely varied? Are the bass and lead serving distinguishable roles?
9. **Duration:** If a duration is requested and section cycles are explicit, estimate playback time from CPS and cycle counts using the target helper's documented semantics.
10. **Optional features:** Are sliders, visualizers, effects, and extra layers helping the piece? Remove decorative or risky features that do not earn their place.

A language-model self-check is only a preflight, not a substitute for execution. The application should validate the program against the actual embedded Strudel REPL whenever possible, capture evaluation/runtime errors, and feed the exact errors into targeted repair. A successful JavaScript parse alone does not prove that Strudel evaluates the code, samples load, audio starts, or the composition sounds good.

## 15. FAILURE TRIAGE: DO NOT REPAIR THE WRONG THING

When a program fails, classify the observed failure before rewriting it:

- **JavaScript/transpilation failure:** unmatched delimiters, malformed call, invalid expression, or mini-notation parsing problem. Fix the first reported syntax error before changing the musical idea.
- **Unknown method or wrong signature:** confirm whether the method exists in the pinned runtime, then correct its receiver and arguments. Do not replace several unrelated methods at once.
- **Helper or control missing:** check whether `slider`, `arrange`, or another helper is installed by MotifAI; do not claim it is native or invent a fallback API.
- **Samples or sound source missing:** check the configured sample map and synth source. A missing sample is not a harmony or arrangement failure.
- **Runs but is silent:** inspect sample availability/loading, gain, filtering, routing, and whether the root expression returns the intended pattern.
- **Plays but sounds wrong:** preserve code validity and make targeted musical changes to rhythm, register, envelopes, note content, balance, and arrangement.
- **Control is inert:** trace the UI control through its actual runtime value to the pattern parameter. A variable name or prompt description is not evidence that a live slider works.
- **Wrong length:** check CPS, arrangement units, total cycles, and any section helper semantics.
- **Prompt bloat or overbuilding:** reduce unnecessary requirements in the generation prompt. A syntax/rules document should not require bespoke sliders, visualizers, multiple movements, or elaborate production in every piece.

Fix the earliest confirmed cause first. Do not perform a broad rewrite in response to one isolated error, and do not assert a fix is verified until it has been tested in the actual target environment.

## 16. CURATED API REMINDERS

These are representative documented Strudel features. Confirm signatures and availability in the active runtime before relying on advanced variants.

- **Pattern construction:** `s("...")`, `note("...")`, `n("...")`, `stack(...)`, `cat(...)`.
- **Time / structure:** `.fast(...)`, `.slow(...)`, `.early(...)`, `.late(...)`, `.ply(...)`, `.iter(...)`, `.rev()`, `.palindrome()`, `.off(...)`, `.every(...)`, `.sometimes(...)`, `.often(...)`, `.rarely(...)`, `.degradeBy(...)`, `.mask(...)`, `.struct(...)`.
- **Samples:** `.bank("RolandTR808")`, `.bank("RolandTR909")`, `.bank("RolandTR707")` when the matching map and sample names are loaded; `.n(...)` or sample indices only where the corresponding pattern semantics apply.
- **Synthesis / filters:** `.s("sawtooth")`, `.s("supersaw")`, `.s("square")`, `.lpf(...)`, `.lpq(...)`, `.hpf(...)`, `.hpq(...)`, `.bpf(...)`, `.bpq(...)` where supported.
- **Envelope / timbre:** `.attack(...)`, `.decay(...)`, `.sustain(...)`, `.release(...)`, `.adsr(...)`, `.shape(...)`, `.crush(...)`, `.coarse(...)`, `.fm(...)`, `.detune(...)`, `.vib(...)`, `.phaser(...)`, `.tremolo(...)` where supported by the selected source/runtime.
- **Space / mix:** `.gain(...)`, `.pan(...)`, `.jux(rev)`, `.juxBy(0.5, rev)`, `.room(...)`, `.roomsize(...)` / `.size(...)`, `.delay(...)`, `.delaytime(...)`, `.delayfeedback(...)`, `.orbit(...)`, `.duckorbit(...)`, `.duckattack(...)`, `.duckdepth(...)` when supported by the target release.
- **Tonal / melodic tools:** use documented pitch, scale, chord, and arpeggiation functions when their exact API is known. `.arp(...)` selects indices from stacked notes using its documented pattern syntax; do not assume an arbitrary label such as `"up"` is supported. Do not invent names or signatures for a desired musical effect.
- **Signals / modulation:** signal patterns such as `sine`, `tri`, `saw`, `square`, `perlin`, `.range(min, max)`, and `.slow(...)` can automate parameters where available. For example, `s("supersaw").lpf(sine.range(300, 2400).slow(4))`. Confirm a signal source is exposed and the parameter accepts a Pattern. Do not assume UI slider variables and signal patterns have interchangeable types.
- **Visual feedback:** `.color("#RRGGBB")`, `._scope()`, `._scopex()`, `._pianoroll()`, `._punchcard()`, `._spiral()`, `._pitchwheel()`, `._spectrogram()` only where exposed by the embedded REPL. All visual feedback is optional.
- **Known traps:** do not substitute invented `.vol()`, `.volume()`, `.amp()`, `.cutoff()`, `.resonance()`, `.filter()`, `.distortion()`, `.reverb()`, `.sidechain()`, `.repeat()`, `.lfo()`, `.sync()`, or `.bpm()` methods for documented Strudel APIs. Some similarly named concepts may exist under a different documented name or package; use the documented spelling and signature rather than inventing aliases.
- **Important distinction:** `.distort(...)` is a documented distortion method in Strudel contexts; do not confuse it with the unverified name `.distortion(...)`.
- **Callback trap:** `.superimpose(...)`, `.every(...)`, `.sometimes(...)`, `.often(...)`, and similar transformation methods may require callback functions. Read the signature; do not pass an unrelated pattern object or mini-notation string where a function is expected.

## 17. OFFICIAL REFERENCE LINKS

Use these docs to verify uncertain APIs and learn capabilities. Prefer the official docs for syntax and runtime behavior; use community examples as pointers for research, not as material to copy into compositions.

- Getting started: https://strudel.cc/learn/getting-started/
- Mini-notation: https://strudel.cc/learn/mini-notation/
- Notes and pitch spellings: https://strudel.cc/learn/notes/
- Samples and banks: https://strudel.cc/learn/samples/
- Synths: https://strudel.cc/learn/synths/
- Audio effects and orbit behavior: https://strudel.cc/learn/effects/
- Pattern functions overview: https://strudel.cc/functions/intro/
- REPL evaluation and scheduling: https://strudel.cc/technical-manual/repl/
- Pattern transformations / conditional modifiers: https://strudel.cc/learn/conditional-modifiers/
- Accumulation modifiers (including `superimpose`): https://strudel.cc/learn/accumulation/
- Time modifiers: https://strudel.cc/learn/time-modifiers/
- Tonal functions: https://strudel.cc/learn/tonal/
- Visual feedback: https://strudel.cc/learn/visual-feedback/

Community discovery links (discovery only; do not copy songs into production knowledge or prompts):
- Awesome Strudel: https://github.com/terryds/awesome-strudel
- Strudel Songs Collection: https://github.com/eefano/strudel-songs-collection

---

## FINAL OPERATING PRINCIPLE

**Be strict about executable syntax and honest about compatibility; be generous about musical imagination.** These rules exist to stop invalid code, broken API assumptions, dead controls, missing sound sources, and avoidable production mistakes. They do not prescribe the same genre, sound palette, section sequence, slider count, effect chain, visualizer, or arrangement for every song. Let the brief lead. Use production knowledge as optional inspiration. Validate the result in the real runtime whenever possible.
