use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use log::{debug, info};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

pub const LOCAL_LLM_FILENAME: &str = "Qwen3-1.7B-Q4_K_M.gguf";

pub struct LocalLlmEngine {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    model_path: PathBuf,
}

/// Tauri managed state wrapper.
pub struct LocalLlmState(pub Mutex<LocalLlmEngine>);

impl LocalLlmEngine {
    pub fn new() -> Result<Self, String> {
        let backend = LlamaBackend::init().map_err(|e| format!("Failed to init llama backend: {e}"))?;
        Ok(Self {
            backend,
            model: None,
            model_path: PathBuf::new(),
        })
    }

    pub fn is_loaded(&self) -> bool {
        self.model.is_some()
    }

    /// Load a GGUF model from disk with full GPU offload.
    pub fn load(&mut self, model_path: &PathBuf) -> Result<(), String> {
        info!("Loading local LLM from: {:?}", model_path);
        let start = Instant::now();

        let params = LlamaModelParams::default().with_n_gpu_layers(1000);
        let model = LlamaModel::load_from_file(&self.backend, model_path, &params)
            .map_err(|e| format!("Failed to load model: {e}"))?;

        info!("Local LLM loaded in {:?}", start.elapsed());
        self.model = Some(model);
        self.model_path = model_path.clone();
        Ok(())
    }

    /// Run post-processing on transcribed text.
    pub fn process(&self, transcription: &str, system_prompt: &str) -> Result<String, String> {
        let model = self.model.as_ref().ok_or("Local LLM not loaded")?;
        let start = Instant::now();

        // Use the model built-in chat template (reads from GGUF metadata).
        let template = model.chat_template(None)
            .map_err(|e| format!("Failed to get chat template: {e}"))?;

        let messages = vec![
            LlamaChatMessage::new("system".into(), system_prompt.into())
                .map_err(|e| e.to_string())?,
            LlamaChatMessage::new("user".into(), transcription.into())
                .map_err(|e| e.to_string())?,
        ];

        let prompt = model.apply_chat_template(&template, &messages, true)
            .map_err(|e| format!("Failed to apply chat template: {e}"))?;

        let tokens = model.str_to_token(&prompt, AddBos::Always)
            .map_err(|e| format!("Tokenization failed: {e}"))?;

        // Context: input tokens + room for output (~2x input for cleanup tasks)
        let n_ctx = (tokens.len() * 3).max(512) as u32;
        let ctx_params = LlamaContextParams::default().with_n_ctx(std::num::NonZeroU32::new(n_ctx));
        let mut ctx = model.new_context(&self.backend, ctx_params)
            .map_err(|e| format!("Failed to create context: {e}"))?;

        // Feed prompt
        let mut batch = LlamaBatch::new(n_ctx as usize, 1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch.add(*token, i as i32, &[0], is_last)
                .map_err(|e| format!("Batch add failed: {e}"))?;
        }
        ctx.decode(&mut batch).map_err(|e| format!("Decode failed: {e}"))?;

        // Generate — greedy sampling (deterministic for text cleanup)
        let mut sampler = LlamaSampler::greedy();
        let max_tokens = tokens.len() * 2;
        let mut output = String::new();
        // Track the KV cache position for each new token (must be monotonically increasing)
        let mut n_cur = tokens.len() as i32;

        for _ in 0..max_tokens {
            let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(new_token);

            if model.is_eog_token(new_token) {
                break;
            }

            let piece = model.token_to_piece(new_token, Special::Tokenize)
                .map_err(|e| format!("Token decode failed: {e}"))?;
            output.push_str(&piece);

            batch.clear();
            batch.add(new_token, n_cur, &[0], true)
                .map_err(|e| format!("Batch add failed: {e}"))?;
            n_cur += 1;
            ctx.decode(&mut batch).map_err(|e| format!("Decode failed: {e}"))?;
        }

        debug!("Local LLM post-processing took {:?}", start.elapsed());
        Ok(output.trim().to_string())
    }

    pub fn unload(&mut self) {
        self.model = None;
        info!("Local LLM unloaded");
    }
}
