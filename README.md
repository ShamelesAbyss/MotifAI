# 🌀 MOTIFAI 🌀

**An autonomous, local-AI symphonic song architect built on Rust and Strudel.**

MotifAI bridges local Large Language Models (like Qwen, Mistral, and DeepSeek) with the Strudel.cc live-coding music environment. As of v6.4.0, it operates in a strict **"Pure Agentic Default"** mode—meaning zero hardcoded templates, zero preset loops, and zero hidden background steering.

You feed it a musical vision and style DNA, and the agent autonomously structures, codes, and arranges a complete multi-movement electronic track in real-time. When you need inspiration, you can explicitly opt-in to advanced context tools like the **Golden Scorebook** and **Empirical Memory Advisory**.

Interface preview: https://github.com/user-attachments/assets/2628abad-2a76-4080-bed7-4b044ed9a2d2

## ✨ CORE FEATURES ✨

* 🧠 **Pure Agentic Composition (Default):** Builds full songs that genuinely evolve over time using Strudel's architecture. No forced structure—just raw AI creativity guided by your prompt.

* 📀 **Opt-In Golden Scorebook:** Press Ctrl+B to open an interactive overlay managing your knowledge/golden_tracks folder. Pin specific tracks as reference material or let the SplitMix64 stochastic engine rotate through them. *Golden Tracks are completely disabled until you explicitly turn them on (Ctrl+T).*

* 📚 **Strudel Production Intelligence:** Context-aware technique retrieval dynamically pulls syntax and synthesis cards from your production library based on your current brief.

* 🗄️ **Conditional Empirical Memory:** Learns your preferences via explicit Like (Ctrl+Y) and Revise (Ctrl+N) signals, but only injects past findings into the AI's prompt when you enable the Memory Advisory (Ctrl+A).

* 🛡️ **Conservative Lexical Preflight:** A non-destructive AST checker ensures balanced syntax and active audio triggers without silently rewriting the AI's mixing or harmonic choices.

* 🔒 **100% Local & Private (64k Context):** Runs entirely on your own hardware via Ollama, pushing a massive 64k token context window for deep, complex tracking. No cloud APIs, no subscriptions.

## 🛠️ PREREQUISITES & INSTALLATION 🛠️

To run MotifAI, you need Rust and Ollama installed on your machine.

### STEP 1: INSTALL OLLAMA (THE AI ENGINE)

Ollama runs the local AI models that power the MotifAI composer.

**For Windows:**

1. Download and install Ollama from https://ollama.com/download/windows

2. **Crucial Optimization for 64k Context:** To prevent the AI from timing out and to unlock the full 64k token memory for massive concert builds, you must set two System Environment Variables:

   * Press Win + R, type sysdm.cpl, hit Enter, and click Environment Variables.

   * Add a New variable: OLLAMA_NUM_CTX with value 65536

   * Add a New variable: OLLAMA_KEEP_ALIVE with value 20m

   * Restart your computer or terminal to apply.

**For Linux:**
Run the official install script in your terminal: curl -fsSL https://ollama.com/install.sh | sh

### STEP 2: PULL THE RECOMMENDED AI MODEL

Open your terminal and download a capable coding model. qwen2.5-coder:14b or qwen2.5-coder:32b are highly recommended for Strudel syntax:

* Command: ollama pull qwen2.5-coder:14b

### STEP 3: INSTALL RUST & MOTIFAI

1. Install Rust via https://rustup.rs if you haven't already.

2. Clone this repository and launch the Forge:

   * Command 1: git clone https://github.com/ShamelesAbyss/MotifAI.git

   * Command 2: cd MotifAI

   * Command 3: cargo run --release

## 🖥️ RUNNING THE OLLAMA SERVER MANUALLY 🖥️

By default, Ollama runs in the background. However, if you are generating massive 96-cycle stadium sets or using heavy 32B models, you might want to run the server manually in a dedicated terminal to monitor the logs and force the extended memory variables on the fly.

Ensure the background Ollama app is closed first (right-click the tray icon and exit).

**For Windows (PowerShell):**
Type: \$env:OLLAMA_NUM_CTX="65536"
Type: \$env:OLLAMA_KEEP_ALIVE="20m"
Type: ollama serve

**For Linux / macOS (Bash):**
Type: OLLAMA_NUM_CTX=65536 OLLAMA_KEEP_ALIVE=20m ollama serve

## 🎛️ HOW TO USE THE FORGE 🎛️

When you run MotifAI, you will be presented with a Terminal User Interface (TUI) split into input panes. Use Tab to navigate between panes.

1. **Pane \[1\] Agentic Musical Vision:** Describe the song structure, movements, energy levels, and specific melodies or drops you want.

2. **Pane \[2\] Style DNA:** Detail the genre, artist influences, specific instrumentation, or paste a YouTube/Web URL for the AI to scrape and reference.

3. **Pane \[3\] Autonomous Score:** The raw, generated Strudel JavaScript code.

## ⌨️ KEYBINDINGS ⌨️

### Generation & Evolution

* `Ctrl+G` or `F5` : **GENERATE** - Instructs the AI to build a brand new song from scratch based on your prompts.

* `Ctrl+E` : **EVOLVE** - Feeds the current score back to the AI to expand, add new sections, or mutate the arrangement.

### Architecture Controls (Opt-In Tools)

* `Ctrl+T` : **TOGGLE GOLDEN TRACKS** - Turns Golden Scorebook reference injection ON/OFF.

* `Ctrl+B` : **SCOREBOOK OVERLAY** - Opens the interactive Golden Scorebook. Use Up/Down to navigate, and Space/Enter to pin up to 2 specific reference tracks for the next generation.

* `Ctrl+A` : **TOGGLE MEMORY ADVISORY** - Turns the Empirical Memory Advisory ON/OFF for the system prompt.

* `Ctrl+R` : **RESCAN** - Rescans the knowledge/golden_tracks folder for new files.

### Feedback & Memory

* `Ctrl+Y` : **LIKE (ENDORSE)** - Explicitly marks the active composition as a success in the memory database.

* `Ctrl+N` : **REVISE (REJECT)** - Explicitly marks the active composition for revision in the memory database.

* `Ctrl+S` : **SAVE** - Saves the current generated code to masterpiece.strudel.js in your project folder.

### Playback & Studio

* `Ctrl+L` : **LAUNCH STUDIO** - Opens the generated code in a local browser studio (http://127.0.0.1:7777) with interactive visualizers. Press Ctrl+Enter in the browser to play.

* `Ctrl+O` : **OPEN STRUDEL.CC** - Pushes the raw code directly to the public Strudel REPL at https://strudel.cc/#.

* `Left` / `Right` : **NUDGE TEMPO** - Adjusts the setcps() tempo up or down directly from the TUI.

### Application

* `Ctrl+M` : **CYCLE MODELS** - Switches between installed Ollama models (e.g., Qwen, Mistral, DeepSeek).

* `Esc` : **EXIT** - Closes overlays or exits MotifAI.

**Disclaimer:** MotifAI is an experimental algorithmic composition tool. Generated Strudel code is heavily dependent on the chosen local LLM's understanding of JavaScript and audio synthesis.
