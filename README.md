🌀 MOTIFAI 🌀

An autonomous, local-AI symphonic song architect built on Rust and Strudel.

MotifAI bridges local Large Language Models (like Qwen, Mistral, and DeepSeek) with the Strudel.cc live-coding music environment. It operates in a "Pure Agentic" mode—meaning zero hardcoded templates and zero preset loops. You feed it a musical vision and style DNA, and the agent autonomously structures, codes, and mixes a complete multi-movement electronic track in real-time.

Interface preview: [https://github.com/user-attachments/assets/2628abad-2a76-4080-bed7-4b044ed9a2d2](https://github.com/user-attachments/assets/2628abad-2a76-4080-bed7-4b044ed9a2d2)

✨ CORE FEATURES ✨

• 🧠 Pure Agentic Composition: Builds full songs (Intro, Verse, Chorus, Bridge, Solo) that genuinely evolve over time using Strudel's arrange() architecture.
• 🎛️ Live Macro Console: Automatically wires 5 to 8 interactive macro sliders (e.g., DRIVE, FILTER_CUTOFF, SUB_BOOST) for real-time browser tweaking.
• 🎚️ Intelligent AST Auto-Mixer: The Rust backend intercepts the AI's code to automatically carve muddy low-frequencies (.hpf(180)), isolate spatial orbits, and re-voice dissonant cluster chords.
• 🔒 100% Local & Private: Runs entirely on your own hardware via Ollama. No cloud APIs, no subscriptions.

🛠️ PREREQUISITES & INSTALLATION 🛠️

To run MotifAI, you need Rust and Ollama installed on your machine.

STEP 1: INSTALL OLLAMA (THE AI ENGINE)
Ollama runs the local AI models that power the MotifAI composer.

For Windows:

1. Download and install Ollama from [https://ollama.com/download/windows](https://ollama.com/download/windows)
2. Crucial Optimization for Large Models: To prevent the AI from timing out during massive concert builds and to unlock the full 64k token memory, you must set two System Environment Variables:
* Press Win + R, type sysdm.cpl, hit Enter, and click Environment Variables.
* Add a New variable: OLLAMA_NUM_CTX with value 65536
* Add a New variable: OLLAMA_KEEP_ALIVE with value 20m
* Restart your computer or terminal to apply.



For Linux:
Run the official install script in your terminal:
curl -fsSL [https://ollama.com/install.sh](https://ollama.com/install.sh) | sh

STEP 2: PULL THE RECOMMENDED AI MODEL
Open your terminal and download a capable coding model. qwen2.5-coder:14b is highly recommended for Strudel syntax:
Command: ollama pull qwen2.5-coder:14b

STEP 3: INSTALL RUST & MOTIFAI

1. Install Rust via [https://rustup.rs](https://rustup.rs) if you haven't already.
2. Clone this repository and launch the Forge:
* Command 1: git clone [https://github.com/ShamelesAbyss/MotifAI.git](https://www.google.com/search?q=https://github.com/ShamelesAbyss/MotifAI.git)
* Command 2: cd MotifAI
* Command 3: cargo run --release



🖥️ RUNNING THE OLLAMA SERVER MANUALLY (FOR MASSIVE BUILDS) 🖥️

By default, Ollama runs in the background. However, if you are generating massive 96-cycle stadium sets or using heavy 32B models, you might want to run the server manually in a dedicated terminal to monitor the logs and force the extended memory and timeout variables on the fly.

If you did not set the system-wide environment variables mentioned in Step 1, you can launch the server with them temporarily using the commands below. Ensure the background Ollama app is closed first (right-click the tray icon and exit).

For Windows (PowerShell):
Type: $env:OLLAMA_NUM_CTX="65536"
Type: $env:OLLAMA_KEEP_ALIVE="20m"
Type: ollama serve

For Linux / macOS (Bash):
Type: OLLAMA_NUM_CTX=65536 OLLAMA_KEEP_ALIVE=20m ollama serve

🎛️ HOW TO USE THE FORGE 🎛️

When you run MotifAI, you will be presented with a Terminal User Interface (TUI) split into input panes. Use Tab to navigate between panes.

1. Pane [1] Agentic Musical Vision: Describe the song structure, movements, energy levels, and specific melodies or drops you want.
2. Pane [2] Style DNA: Detail the genre, artist influences, specific instrumentation, or paste a YouTube/Web URL for the AI to scrape and reference.

⌨️ KEYBINDINGS ⌨️

• Ctrl+G or F5 : GENERATE
Instructs the AI to build a brand new song from scratch based on your prompts.

• Ctrl+E : EVOLVE
Feeds the current score back to the AI to expand, add new sections, or mutate the arrangement.

• Ctrl+L : LAUNCH STUDIO
Opens the generated code in a local browser studio ([http://127.0.0.1:7777](https://www.google.com/search?q=http://127.0.0.1:7777)) with live macro sliders and visualizers. Press Ctrl+Enter in the browser to play.

• Ctrl+O : OPEN STRUDEL.CC
Pushes the raw code directly to the public Strudel REPL at [https://strudel.cc](https://strudel.cc)

• Ctrl+M : CYCLE MODELS
Switches between installed Ollama models (e.g., Qwen, Mistral, DeepSeek).

• Ctrl+S : SAVE
Saves the current generated code to masterpiece.strudel.js in your project folder.

• Left / Right : NUDGE TEMPO
Adjusts the setcps() tempo up or down directly from the TUI.

• Esc : EXIT
Closes MotifAI.

Disclaimer: MotifAI is an experimental algorithmic composition tool. Generated Strudel code is heavily dependent on the chosen local LLM's understanding of JavaScript and audio synthesis.
