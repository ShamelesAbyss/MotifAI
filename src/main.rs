#[allow(non_snake_case, dead_code)]
mod orchestrator;
#[allow(dead_code)]
mod strudel_ast;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs, io,
    process::Command,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, RwLock},
};

const LOOPBACK_IP: &str = "127.0.0.1";
const LOCAL_PORT: u16 = 7777;
const OLLAMA_PORT: u16 = 11434;

// Split protocol literals so copy-paste / Markdown formatters never corrupt URLs
const STRUDEL_WEB_HASH: &str = concat!("https", "://", "strudel.cc/#");
const STRUDEL_REPL_CDN: &str = concat!("https", "://", "unpkg.com/@strudel/repl@latest");
const STRUDEL_HOME_URL: &str = concat!("https", "://", "strudel.cc");

const STANDBY_BUFFER: &str = r##"// [AGENTIC STANDBY] Zero Presets. Zero Templates. 100% Autonomous AI Composition.
// Enter your Vision in Pane [1] and Style/URL DNA in Pane [2], then press [Ctrl+G] or [F5].
setcps(0.55)

stack(
  s("bd ~ ~ ~").bank("RolandTR909").gain(0.75).color("#00f0ff")._scope()
)"##;

fn ollama_endpoint(path: &str) -> String {
    format!("http{}{}:{}{}", "://", LOOPBACK_IP, OLLAMA_PORT, path)
}

fn local_studio_endpoint(path: &str) -> String {
    format!("http{}{}:{}{}", "://", LOOPBACK_IP, LOCAL_PORT, path)
}

fn fresh_entropy_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_nanos() as u64) ^ 0x5DEECE66Du64)
        .unwrap_or(42)
}

fn log_diag(stage: &str, detail: &str) {
    use std::io::Write;
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("forge_debug.log")
    {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{}] [{}] {}", ts, stage, detail);
    }
}

fn get_strudel_system_prompt() -> String {
    if let Ok(rules) = fs::read_to_string("strudel_rules.md") {
        format!(
            "You are an autonomous algorithmic composer and master of Strudel ({}). \
             You operate with 100% creative agency—never use stock templates or repetitive loops. \
             Follow ONLY the syntax and DSP rules below and the user's prompt to invent a bespoke musical work. \
             Your sole output must be valid, executable Strudel JavaScript code—no markdown fences, no prose outside '//' comments.\n\n\
             {}\n\n\
             Return ONLY raw executable Strudel code starting with setcps():",
            STRUDEL_HOME_URL, rules
        )
    } else {
        r##"You are an autonomous algorithmic composer and master of Strudel. Output ONLY valid executable Strudel code starting with setcps():"##.to_string()
    }
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    repeat_penalty: f32,
    num_ctx: u32,
    seed: u64,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
    think: bool,
    options: OllamaOptions,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
}

enum BotEvent {
    ModelsDiscovered(Vec<String>),
    StatusUpdate(String),
    Generated(String),
    Error(String),
}

struct App {
    prompt_input: String,
    style_reference: String,
    active_pane: usize,
    strudel_code: String,
    shared_code: Arc<RwLock<String>>,
    status_message: String,
    is_generating: bool,
    available_models: Vec<String>,
    model_index: usize,
    scroll_offset: u16,
    evolution_count: u32,
}

impl App {
    fn new(shared_code: Arc<RwLock<String>>) -> Self {
        let default_model =
            std::env::var("STRUDEL_MODEL").unwrap_or_else(|_| "qwen2.5-coder:14b".to_string());
        log_diag(
            "INIT",
            &format!(
                "Starting Strudel Forge v6.0 [PURE AGENTIC]; default_model={}",
                default_model
            ),
        );
        Self {
            prompt_input: String::new(),
            style_reference: String::new(),
            active_pane: 0,
            strudel_code: String::from(STANDBY_BUFFER),
            shared_code,
            status_message: String::from(
                "STRUDEL FORGE v6.0 [PURE AGENTIC] | Zero Templates | [Ctrl+G / F5] Autonomous Compose | [Ctrl+E] Evolve | [Ctrl+M] Model",
            ),
            is_generating: false,
            available_models: vec![default_model],
            model_index: 0,
            scroll_offset: 0,
            evolution_count: 0,
        }
    }

    fn current_model(&self) -> String {
        self.available_models
            .get(self.model_index)
            .cloned()
            .unwrap_or_else(|| "qwen2.5-coder:14b".to_string())
    }

    fn cycle_model(&mut self) {
        if !self.available_models.is_empty() {
            self.model_index = (self.model_index + 1) % self.available_models.len();
            log_diag("MODEL_SWITCH", &format!("Switched to {}", self.current_model()));
            self.status_message = format!(
                "Active Agent Model: '{}' | [Ctrl+G] Autonomous Compose | [Ctrl+E] Evolve Current Score",
                self.current_model()
            );
        }
    }

    fn nudge_tempo(&mut self, delta: f32) {
        let mut updated = Vec::new();
        let mut changed = false;
        for line in self.strudel_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("setcps(") && trimmed.ends_with(')') {
                let inner = &trimmed[7..trimmed.len() - 1];
                if let Ok(val) = inner.trim().parse::<f32>() {
                    let next = (val + delta).clamp(0.25, 0.95);
                    updated.push(format!("setcps({:.2})", next));
                    changed = true;
                    self.status_message = format!(
                        "Tempo updated to setcps({:.2}) (~{:.0} BPM). Press [Ctrl+L] or [Ctrl+O] to hear it!",
                        next,
                        next * 240.0
                    );
                    continue;
                }
            }
            updated.push(line.to_string());
        }
        if changed {
            self.strudel_code = updated.join("\n");
            if let Ok(mut guard) = self.shared_code.try_write() {
                *guard = self.strudel_code.clone();
            }
        }
    }
}

fn open_browser_url(url: &str) {
    log_diag("OPEN_BROWSER", url);
    if open::that(url).is_ok() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("xdg-open")
            .arg(url)
            .spawn()
            .or_else(|_| Command::new("open").arg(url).spawn());
    }
}

async fn start_local_studio_server(shared_code: Arc<RwLock<String>>) {
    let addr = format!("{}:{}", LOOPBACK_IP, LOCAL_PORT);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            log_diag("STUDIO_BIND_OK", &format!("Listening on {}", addr));
            l
        }
        Err(e) => {
            log_diag("STUDIO_BIND_ERR", &format!("Failed binding {}: {}", addr, e));
            return;
        }
    };

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let code_ref = shared_code.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    return;
                }
                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or("");
                let code = code_ref.read().await.clone();
                let b64 = BASE64.encode(code.as_bytes());

                let response = if first_line.contains("GET /api/code") {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{}",
                        code.len(),
                        code
                    )
                } else if first_line.contains("GET /studio") {
                    let body = build_embedded_studio_html(&code, &b64);
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                } else {
                    let target_url = format!("{}{}", STRUDEL_WEB_HASH, url_encode_b64(&b64));
                    let body = build_redirect_html(&b64);
                    format!(
                        "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{}",
                        target_url,
                        body.len(),
                        body
                    )
                };

                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
                let _ = socket.shutdown().await;
            });
        }
    }
}

fn url_encode_b64(b64: &str) -> String {
    b64.replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
}

fn build_redirect_html(b64: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Strudel Forge Bridge</title>
<style>
  body {{ background:#08090d; color:#00f0ff; font-family:monospace; display:flex; align-items:center; justify-content:center; height:100vh; margin:0; }}
</style>
</head>
<body>
<div>🌀 Injecting Agentic Score into Strudel.cc REPL...</div>
<script>
  const b64 = "{}";
  const target = "{}" + encodeURIComponent(b64);
  window.location.replace(target);
</script>
</body>
</html>"##,
        b64, STRUDEL_WEB_HASH
    )
}

fn build_embedded_studio_html(code: &str, b64: &str) -> String {
    let safe_code = code.replace("-->", "-- >");
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>🌀 STRUDEL FORGE v6.0 // Pure Agentic Studio</title>
<script src="{}"></script>
<style>
  body {{ background:#07080c; color:#e0f7ff; font-family:'Consolas',monospace; margin:0; padding:0; }}
  header {{ background:#0f121d; border-bottom:1px solid #00f0ff; padding:12px 20px; display:flex; justify-content:space-between; align-items:center; }}
  .title {{ color:#00f0ff; font-weight:bold; font-size:16px; letter-spacing:1px; }}
  .btn {{ background:#ff007f; color:#fff; text-decoration:none; padding:8px 16px; border-radius:4px; font-weight:bold; margin-left:10px; }}
  .hint {{ padding:8px 20px; background:#121624; color:#ffd700; font-size:13px; border-bottom:1px solid #222; }}
</style>
</head>
<body>
<header>
  <div class="title">🌀 STRUDEL FORGE v6.0 // PURE AGENTIC CREATION STUDIO (ZERO TEMPLATES)</div>
  <div>
    <a class="btn" id="popout" href="#">Open Full Strudel.cc REPL ↗</a>
  </div>
</header>
<div class="hint">⚡ <b>100% Agentic Score Compiled, Orbits Isolated & Scale Locked!</b> Click inside the editor and press <b>Ctrl+Enter</b> to play.</div>
<strudel-editor id="repl">
<!--
{}
-->
</strudel-editor>
<script>
  const b64 = "{}";
  const webHash = "{}";
  document.getElementById("popout").href = webHash + encodeURIComponent(b64);
</script>
</body>
</html>"##,
        STRUDEL_REPL_CDN, safe_code, b64, STRUDEL_WEB_HASH
    )
}

async fn discover_ollama_models(tx: mpsc::Sender<BotEvent>) {
    let tags_url = ollama_endpoint("/api/tags");
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    if let Ok(resp) = client.get(&tags_url).send().await {
        if let Ok(tags) = resp.json::<TagsResponse>().await {
            let names: Vec<String> = tags.models.into_iter().map(|m| m.name).collect();
            if !names.is_empty() {
                let _ = tx.send(BotEvent::ModelsDiscovered(names)).await;
            }
        }
    }
}

async fn scrape_reference_if_url(style_ref: &str, client: &reqwest::Client) -> String {
    let mut extra_context = String::new();
    let http_prefix = concat!("http", "://");
    let https_prefix = concat!("https", "://");

    for token in style_ref.split_whitespace() {
        if token.starts_with(http_prefix) || token.starts_with(https_prefix) {
            log_diag("SCRAPE_START", token);
            if let Ok(resp) = client.get(token).timeout(Duration::from_secs(8)).send().await {
                if let Ok(html) = resp.text().await {
                    let stripped = strip_html_tags(&html);
                    let snippet: String = stripped.chars().take(1800).collect();
                    extra_context.push_str(&format!(
                        "\n[Scraped Web/YouTube Reference Context from {}]:\n{}\n",
                        token, snippet
                    ));
                }
            }
        }
    }
    extra_context
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

async fn generate_strudel_track(
    model: String,
    user_prompt: String,
    style_ref: String,
    existing_code: Option<String>,
    evo_gen: u32,
    tx: mpsc::Sender<BotEvent>,
) {
    let endpoint = ollama_endpoint("/api/generate");
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap_or_default();

    let scraped_context = scrape_reference_if_url(&style_ref, &client).await;
    let mut last_ast_error: Option<String> = None;
    let mut best_fallback_code: Option<String> = None;

    for attempt in 1..=2 {
        let seed = fresh_entropy_seed().wrapping_add((attempt as u64) * 7919);
        let orchestration_dna = orchestrator::build_orchestration_directive(
            &user_prompt,
            &style_ref,
            existing_code.as_deref(),
            evo_gen,
            seed,
        );

        let error_feedback = match &last_ast_error {
            Some(err) => format!(
                "\n\nCRITICAL AST/VALIDATION FEEDBACK FROM PREVIOUS PASS: {}\n\
                 Resolve this cleanly while keeping your composition 100% original and faithful to the user's prompt.",
                err
            ),
            None => String::new(),
        };

        let effective_vision = if user_prompt.trim().is_empty() {
            "Autonomous Agentic Choice: Invent a complete, deeply expressive multi-movement composition with evolving sections, bespoke macro sliders, and visualizers."
        } else {
            user_prompt.as_str()
        };

        let effective_style = if style_ref.trim().is_empty() {
            "Autonomous Agentic Choice: Select the ideal instrumentation, synthesis techniques, and dynamic arrangement arc."
        } else {
            style_ref.as_str()
        };

        let full_prompt = match &existing_code {
            Some(current) => format!(
                "PURE AGENTIC EVOLUTION MODE (ZERO TEMPLATES / ZERO PRESETS).\n\
                 You have full autonomous creative agency to evolve, restructure, and elevate the current score based strictly on the user's direction and the official Strudel rules.\n\n\
                 {}\n\n\
                 User's Vision / Evolution Directive: {}\n\
                 Style / Reference Context: {}{}{}\n\n\
                 Current Score to Evolve:\n{}\n\n\
                 Output ONLY raw, executable Strudel JavaScript code starting with setcps():",
                orchestration_dna,
                effective_vision,
                effective_style,
                scraped_context,
                error_feedback,
                current
            ),
            None => format!(
                "PURE AGENTIC CREATION MODE (ZERO TEMPLATES / ZERO PRESETS).\n\
                 You have full autonomous creative agency. Do not rely on stock templates or generic loops. \
                 Design a bespoke musical architecture, custom top-level `const` slider macros, expressive melodies, harmony, bridges, and time-sequenced movements (`arrange(...)` or cycle-phrased `stack(...)`) tailored exclusively to the user's prompt and the official Strudel rules.\n\n\
                 {}\n\n\
                 User's Musical Vision: {}\n\
                 Style / Reference Context: {}{}{}\n\n\
                 Output ONLY raw, executable Strudel JavaScript code starting with setcps():",
                orchestration_dna,
                effective_vision,
                effective_style,
                scraped_context,
                error_feedback
            ),
        };

        let temp = if attempt == 1 {
            if existing_code.is_some() { 0.65 } else { 0.62 }
        } else {
            0.72
        };

        let req_body = OllamaRequest {
            model: model.clone(),
            prompt: full_prompt,
            system: get_strudel_system_prompt(),
            stream: false,
            think: false,
            options: OllamaOptions {
                temperature: temp,
                top_p: 0.92,
                repeat_penalty: 1.15,
                num_ctx: 8192,
                seed,
            },
        };

        if attempt > 1 {
            let _ = tx
                .send(BotEvent::StatusUpdate(format!(
                    "Agent refining score via '{}' (Attempt {}/2, temp={:.2})...",
                    model, attempt, temp
                )))
                .await;
        }

        match client.post(&endpoint).json(&req_body).send().await {
            Ok(resp) => {
                let status = resp.status();
                match resp.json::<Value>().await {
                    Ok(json) => {
                        if let Some(err_msg) = json.get("error").and_then(|v| v.as_str()) {
                            let _ = tx
                                .send(BotEvent::Error(format!(
                                    "Ollama HTTP {} Error ([Ctrl+M] to switch model): {}",
                                    status, err_msg
                                )))
                                .await;
                            return;
                        } else if let Some(code_text) =
                            json.get("response").and_then(|v| v.as_str())
                        {
                            match strudel_ast::compile_and_heal_strudel(code_text) {
                                Ok(healed_code) => {
                                    best_fallback_code = Some(healed_code.clone());
                                    if let Some(clone_warn) =
                                        strudel_ast::check_clone_or_stagnation(
                                            &healed_code,
                                            existing_code.as_deref(),
                                        )
                                    {
                                        log_diag("CLONE_REJECTED", &clone_warn);
                                        last_ast_error = Some(clone_warn);
                                        continue;
                                    }
                                    log_diag(
                                        "AST_HEAL_OK",
                                        &format!(
                                            "Agentic score compiled & mixed: {} bytes (seed={})",
                                            healed_code.len(),
                                            seed
                                        ),
                                    );
                                    let _ = tx.send(BotEvent::Generated(healed_code)).await;
                                    return;
                                }
                                Err(ast_err) => {
                                    log_diag("AST_HEAL_RETRY", &ast_err);
                                    last_ast_error = Some(ast_err);
                                }
                            }
                        } else {
                            let _ = tx
                                .send(BotEvent::Error(format!(
                                    "Unexpected Ollama JSON (HTTP {})",
                                    status
                                )))
                                .await;
                            return;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(BotEvent::Error(format!(
                                "Failed reading JSON payload (HTTP {}): {}",
                                status, e
                            )))
                            .await;
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = tx
                    .send(BotEvent::Error(format!("Connection error to {}: {}", endpoint, e)))
                    .await;
                return;
            }
        }
    }

    if let Some(valid_code) = best_fallback_code {
        let _ = tx.send(BotEvent::Generated(valid_code)).await;
    } else {
        let _ = tx
            .send(BotEvent::Error(
                last_ast_error.unwrap_or_else(|| "Generation failed AST validation.".to_string()),
            ))
            .await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let shared_code = Arc::new(RwLock::new(String::from(STANDBY_BUFFER)));
    let mut app = App::new(shared_code.clone());
    let (tx, mut rx) = mpsc::channel::<BotEvent>(10);

    let server_code = shared_code.clone();
    tokio::spawn(async move {
        start_local_studio_server(server_code).await;
    });

    let tx_discover = tx.clone();
    tokio::spawn(async move {
        discover_ollama_models(tx_discover).await;
    });

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Ok(bot_event) = rx.try_recv() {
            match bot_event {
                BotEvent::ModelsDiscovered(models) => {
                    let preferred_idx = models
                        .iter()
                        .position(|m| m.to_lowercase().contains("32b"))
                        .or_else(|| models.iter().position(|m| m.to_lowercase().contains("14b")))
                        .or_else(|| models.iter().position(|m| m.to_lowercase().contains("coder")))
                        .or_else(|| models.iter().position(|m| m.to_lowercase().contains("qwen")))
                        .unwrap_or(0);
                    app.available_models = models;
                    app.model_index = preferred_idx;
                    app.status_message = format!(
                        "Agent Ready ('{}') | [Ctrl+G / F5] Autonomous Compose | [Ctrl+E] Evolve | [Ctrl+M] Switch Model",
                        app.current_model()
                    );
                }
                BotEvent::StatusUpdate(msg) => {
                    app.status_message = msg;
                }
                BotEvent::Generated(code) => {
                    app.strudel_code = code.clone();
                    if let Ok(mut guard) = app.shared_code.try_write() {
                        *guard = code;
                    }
                    app.is_generating = false;
                    app.scroll_offset = 0;

                    let studio_url = local_studio_endpoint("/studio");
                    open_browser_url(&studio_url);

                    app.status_message = format!(
                        "Agentic Composition Ready (Evo #{})! Studio launched → Press Ctrl+Enter to play.",
                        app.evolution_count
                    );
                }
                BotEvent::Error(err) => {
                    app.is_generating = false;
                    app.status_message = err;
                }
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if key.code == KeyCode::Esc {
                    break;
                }

                if key.code == KeyCode::F(5)
                    || (key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('g') | KeyCode::Char('G')))
                {
                    if !app.is_generating {
                        app.is_generating = true;
                        app.evolution_count = 0;
                        let model = app.current_model();
                        app.status_message = format!(
                            "Agent '{}' composing bespoke score from rules & prompt...",
                            model
                        );
                        let tx_clone = tx.clone();
                        let prompt = app.prompt_input.clone();
                        let style = app.style_reference.clone();
                        tokio::spawn(async move {
                            generate_strudel_track(model, prompt, style, None, 0, tx_clone).await;
                        });
                    }
                    continue;
                }

                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            if !app.is_generating {
                                app.is_generating = true;
                                app.evolution_count += 1;
                                let evo_gen = app.evolution_count;
                                let model = app.current_model();
                                app.status_message = format!(
                                    "Agent '{}' evolving score into Gen #{}...",
                                    model, evo_gen
                                );
                                let tx_clone = tx.clone();
                                let prompt = app.prompt_input.clone();
                                let style = app.style_reference.clone();
                                let current = if app.strudel_code.contains("[AGENTIC STANDBY]") {
                                    None
                                } else {
                                    Some(app.strudel_code.clone())
                                };
                                tokio::spawn(async move {
                                    generate_strudel_track(
                                        model, prompt, style, current, evo_gen, tx_clone,
                                    )
                                    .await;
                                });
                            }
                            continue;
                        }
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            if let Ok(mut guard) = app.shared_code.try_write() {
                                *guard = app.strudel_code.clone();
                            }
                            let play_url = local_studio_endpoint("/play");
                            open_browser_url(&play_url);
                            app.status_message = String::from(
                                "Launched directly on Strudel.cc! Press Play (Ctrl+Enter) in the browser tab.",
                            );
                            continue;
                        }
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            if let Ok(mut guard) = app.shared_code.try_write() {
                                *guard = app.strudel_code.clone();
                            }
                            let studio_url = local_studio_endpoint("/studio");
                            open_browser_url(&studio_url);
                            app.status_message = String::from(
                                "Opened Local Studio with Sliders & Visualizers → Press Ctrl+Enter to play",
                            );
                            continue;
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            app.cycle_model();
                            continue;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            let _ = fs::write("masterpiece.strudel.js", &app.strudel_code);
                            app.status_message =
                                String::from("Saved current song to 'masterpiece.strudel.js'!");
                            continue;
                        }
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Tab => {
                        app.active_pane = (app.active_pane + 1) % 3;
                    }
                    KeyCode::Up => {
                        if app.active_pane == 2 && app.scroll_offset > 0 {
                            app.scroll_offset -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.active_pane == 2 {
                            app.scroll_offset += 1;
                        }
                    }
                    KeyCode::Left => {
                        if app.active_pane == 2 {
                            app.nudge_tempo(-0.02);
                        }
                    }
                    KeyCode::Right => {
                        if app.active_pane == 2 {
                            app.nudge_tempo(0.02);
                        }
                    }
                    KeyCode::Backspace => match app.active_pane {
                        0 => {
                            app.prompt_input.pop();
                        }
                        1 => {
                            app.style_reference.pop();
                        }
                        _ => {}
                    },
                    KeyCode::Enter => match app.active_pane {
                        0 => app.prompt_input.push('\n'),
                        1 => app.style_reference.push('\n'),
                        _ => {}
                    },
                    KeyCode::Char(c) => match app.active_pane {
                        0 => app.prompt_input.push(c),
                        1 => app.style_reference.push(c),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(30),
            Constraint::Percentage(61),
            Constraint::Length(3),
        ])
        .split(f.area());

    let model_name = app.current_model();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " 🌀 STRUDEL FORGE v6.0 [PURE AGENTIC] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("| Model: "),
        Span::styled(model_name, Style::default().fg(Color::Magenta)),
        Span::raw(format!(" | Evo #{} | ", app.evolution_count)),
        Span::styled("[Ctrl+G / F5]", Style::default().fg(Color::Yellow)),
        Span::raw(" Compose "),
        Span::styled("[Ctrl+E]", Style::default().fg(Color::Magenta)),
        Span::raw(" Evolve "),
        Span::styled("[Ctrl+L]", Style::default().fg(Color::Cyan)),
        Span::raw(" Studio "),
        Span::styled("[Ctrl+O]", Style::default().fg(Color::Green)),
        Span::raw(" Strudel.cc "),
        Span::styled("[Ctrl+M]", Style::default().fg(Color::Green)),
        Span::raw(" Model"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, main_chunks[0]);

    let input_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    let prompt_style = if app.active_pane == 0 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let style_ref_border = if app.active_pane == 1 {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let code_border = if app.active_pane == 2 {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let prompt_box = Paragraph::new(app.prompt_input.as_str())
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(prompt_style)
                .title(" [1] Agentic Musical Vision (Describe movements, melodies, bridges, solos, sliders) "),
        );
    f.render_widget(prompt_box, input_chunks[0]);

    let style_box = Paragraph::new(app.style_reference.as_str())
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style_ref_border)
                .title(" [2] Style DNA / Instrumentation / Paste Any Web or YouTube URL to Scrape Reference "),
        );
    f.render_widget(style_box, input_chunks[1]);

    let code_box = Paragraph::new(app.strudel_code.as_str())
        .scroll((app.scroll_offset, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(code_border)
                .title(" [3] Autonomous Agentic Strudel Score (Up/Down: Scroll | Left/Right: Tempo | Ctrl+S: Save) "),
        );
    f.render_widget(code_box, main_chunks[2]);

    let status_bar = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, main_chunks[3]);
}