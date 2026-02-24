use log::{debug, info, warn};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const LOCAL_LLM_FILENAME: &str = "Qwen2.5-1.5B-Instruct-Q4_K_M.gguf";

pub struct LocalLlmEngine {
    server_process: Option<Child>,
    port: u16,
    model_path: PathBuf,
}

/// Tauri managed state wrapper.
pub struct LocalLlmState(pub Mutex<LocalLlmEngine>);

impl LocalLlmEngine {
    pub fn new() -> Result<Self, String> {
        // Verify llama-server is available early so we can warn at startup.
        match find_llama_server() {
            Ok(path) => info!("llama-server found at: {:?}", path),
            Err(e) => warn!("llama-server not available, local post-processing will fail: {e}"),
        }

        Ok(Self {
            server_process: None,
            port: 0,
            model_path: PathBuf::new(),
        })
    }

    pub fn is_loaded(&self) -> bool {
        self.server_process.is_some()
    }

    /// Start `llama-server` as a subprocess with the given GGUF model.
    pub fn load(&mut self, model_path: &PathBuf) -> Result<(), String> {
        info!("Loading local LLM via llama-server: {:?}", model_path);

        if !model_path.exists() {
            return Err(format!("Model file does not exist: {:?}", model_path));
        }
        let metadata = std::fs::metadata(model_path)
            .map_err(|e| format!("Cannot read model file metadata: {e}"))?;
        info!("Model file size: {} bytes", metadata.len());

        // Kill any existing server
        self.unload();

        let server_bin = find_llama_server()?;
        info!("Using llama-server at: {:?}", server_bin);

        let port = pick_available_port()?;
        info!("Starting llama-server on port {}", port);

        let start = Instant::now();

        let child = Command::new(&server_bin)
            .args([
                "-m",
                model_path.to_str().ok_or("Model path is not valid UTF-8")?,
                "--port",
                &port.to_string(),
                "--host",
                "127.0.0.1",
                "-ngl",
                "99",
                "--log-disable",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start llama-server: {e}"))?;

        self.server_process = Some(child);
        self.port = port;
        self.model_path = model_path.clone();

        // Wait for the server to finish loading the model and be ready.
        self.wait_for_ready()?;

        info!("llama-server ready on port {} in {:?}", port, start.elapsed());
        Ok(())
    }

    fn wait_for_ready(&mut self) -> Result<(), String> {
        let start = Instant::now();
        let timeout = Duration::from_secs(120);

        while start.elapsed() < timeout {
            // Check if the process is still alive
            if let Some(ref mut child) = self.server_process {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        self.server_process = None;
                        return Err(format!("llama-server exited during startup with {status}"));
                    }
                    Err(e) => {
                        return Err(format!("Failed to check llama-server status: {e}"));
                    }
                    Ok(None) => {} // still running
                }
            }

            if let Ok(body) = http_request(self.port, "GET", "/health", None) {
                if body.contains("ok") {
                    return Ok(());
                }
                // Server is up but still loading ("loading model" status)
                debug!("llama-server health: {}", body.trim());
            }

            std::thread::sleep(Duration::from_millis(250));
        }

        // Timed out — kill the server
        self.unload();
        Err("llama-server failed to become ready within 120 seconds".into())
    }

    /// Send a chat completion request to the running llama-server.
    pub fn process(&self, transcription: &str, system_prompt: &str) -> Result<String, String> {
        if self.server_process.is_none() {
            return Err("Local LLM not loaded".into());
        }

        let start = Instant::now();

        let body = serde_json::json!({
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": transcription}
            ],
            "temperature": 0,
            "stream": false,
            "n_predict": 1024
        });

        let response =
            http_request(self.port, "POST", "/v1/chat/completions", Some(&body.to_string()))?;

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse llama-server response: {e}\nRaw: {response}"))?;

        // Extract the assistant's message content
        let content = parsed["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                format!(
                    "No content in llama-server response: {}",
                    serde_json::to_string_pretty(&parsed).unwrap_or_default()
                )
            })?
            .trim()
            .to_string();

        debug!("Local LLM post-processing took {:?}", start.elapsed());
        Ok(content)
    }

    pub fn unload(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            info!("Stopping llama-server (pid {})", child.id());
            let _ = child.kill();
            let _ = child.wait();
            info!("llama-server stopped");
        }
    }
}

impl Drop for LocalLlmEngine {
    fn drop(&mut self) {
        self.unload();
    }
}

/// Locate the `llama-server` binary on the system.
fn find_llama_server() -> Result<PathBuf, String> {
    // 1. Bundled sidecar — next to the app binary (production builds)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sidecar = dir.join("llama-server");
            if sidecar.exists() {
                return Ok(sidecar);
            }
        }
    }

    // 2. Explicit env var override
    if let Ok(path) = std::env::var("LLAMA_SERVER_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
        warn!("LLAMA_SERVER_PATH={path} does not exist, falling back to PATH");
    }

    // 3. Search PATH via `which`
    if let Ok(output) = Command::new("which").arg("llama-server").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    // 4. Common Homebrew locations
    for candidate in [
        "/opt/homebrew/bin/llama-server",
        "/usr/local/bin/llama-server",
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }

    Err("llama-server not found. Install via: brew install llama.cpp".into())
}

/// Bind to port 0 to let the OS assign a free ephemeral port.
fn pick_available_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find available port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {e}"))?
        .port();
    drop(listener);
    Ok(port)
}

/// Minimal HTTP client for communicating with the local llama-server.
/// Uses raw TCP with `Connection: close` to avoid needing a full HTTP library.
fn http_request(
    port: u16,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| format!("Invalid address: {e}"))?;

    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| format!("Connection to llama-server failed: {e}"))?;

    // Generous timeout for model inference (can take a while for long inputs)
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .ok();

    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n"
    );
    if let Some(body) = body {
        request.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    request.push_str("\r\n");
    if let Some(body) = body {
        request.push_str(body);
    }

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Write to llama-server failed: {e}"))?;

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("Read from llama-server failed: {e}"))?;

    let response = String::from_utf8_lossy(&raw);

    // Split headers from body
    let header_end = response
        .find("\r\n\r\n")
        .ok_or("Invalid HTTP response from llama-server (no header terminator)")?;

    let headers = &response[..header_end];
    let body_raw = &response[header_end + 4..];

    // Check HTTP status
    let status_line = headers.split("\r\n").next().unwrap_or("");
    if !status_line.contains(" 200") {
        return Err(format!(
            "llama-server returned error: {status_line}\n{body_raw}"
        ));
    }

    // Handle chunked transfer encoding if present
    if headers.to_lowercase().contains("transfer-encoding: chunked") {
        decode_chunked(body_raw)
    } else {
        Ok(body_raw.to_string())
    }
}

/// Decode an HTTP chunked transfer-encoded body.
fn decode_chunked(data: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut remaining = data;

    loop {
        // Each chunk: <hex-size>\r\n<data>\r\n
        let line_end = remaining
            .find("\r\n")
            .ok_or("Invalid chunked encoding: missing chunk size")?;
        let size_str = remaining[..line_end].trim();
        let size = usize::from_str_radix(size_str, 16)
            .map_err(|e| format!("Invalid chunk size '{size_str}': {e}"))?;

        if size == 0 {
            break; // Terminal chunk
        }

        remaining = &remaining[line_end + 2..];
        if remaining.len() < size {
            return Err("Truncated chunked response".into());
        }
        result.push_str(&remaining[..size]);
        remaining = &remaining[size..];
        if remaining.starts_with("\r\n") {
            remaining = &remaining[2..];
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_llama_server() {
        match find_llama_server() {
            Ok(path) => eprintln!("Found llama-server at: {:?}", path),
            Err(e) => eprintln!("llama-server not found (install via: brew install llama.cpp): {e}"),
        }
    }

    #[test]
    fn test_load_and_process() {
        let model_path = PathBuf::from(std::env::var("HOME").unwrap())
            .join("Library/Application Support/com.pais.handy/models/llm/Qwen3-1.7B-Q4_K_M.gguf");

        if !model_path.exists() {
            eprintln!("Model file not found, skipping test");
            return;
        }

        if find_llama_server().is_err() {
            eprintln!("llama-server not installed, skipping test");
            return;
        }

        let mut engine = LocalLlmEngine::new().expect("Failed to create engine");
        engine.load(&model_path).expect("Failed to load model");
        assert!(engine.is_loaded());

        let result = engine
            .process(
                "so i went to the store and bought like twenty five dollars worth of stuff",
                "Clean up this speech transcription. Fix grammar, punctuation, and formatting. Output only the cleaned text.",
            )
            .expect("Process failed");

        eprintln!("Result: {}", result);
        assert!(!result.is_empty());

        engine.unload();
        assert!(!engine.is_loaded());
    }
}
