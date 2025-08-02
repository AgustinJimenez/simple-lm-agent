use std::sync::Arc;
use std::env;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use candle_transformers::models::llama::{Llama, Config as LlamaConfig, Cache};
use candle_nn::VarBuilder;
use candle_core::quantized::gguf_file;
use tokenizers::Tokenizer;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

struct LLMAgent {
    model_path: Option<String>,
    conversation: Vec<ChatMessage>,
    system_prompt: String,
    is_initialized: bool,
    model_name: String,
    model: Option<Llama>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    cache: Option<Cache>,
    config: Option<LlamaConfig>,
}

impl LLMAgent {
    fn new() -> Self {
        Self {
            model_path: None,
            conversation: Vec::new(),
            system_prompt: "You are a helpful assistant.".to_string(),
            is_initialized: false,
            model_name: "local-model".to_string(),
            model: None,
            tokenizer: None,
            device: Device::Cpu,
            cache: None,
            config: None,
        }
    }

    async fn initialize(&mut self, model_path: &str) -> Result<String> {
        // Check if model file exists
        if !Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found: {}", model_path));
        }

        self.model_path = Some(model_path.to_string());
        
        // Extract model name from path
        let model_name = Path::new(model_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown");
        self.model_name = model_name.to_string();

        // Load GGUF model
        println!("Loading GGUF model from: {}", model_path);
        let mut file = std::fs::File::open(model_path)?;
        let content = gguf_file::Content::read(&mut file)?;
        
        // Extract model weights
        let mut tensors: HashMap<String, Tensor> = HashMap::new();
        for (name, tensor) in content.tensor_infos.iter() {
            let tensor_data = content.tensor_data(name)?;
            let shape: Vec<usize> = tensor.shape.iter().map(|&x| x as usize).collect();
            let tensor = match tensor.ggml_dtype {
                candle_core::quantized::GgmlDType::F32 => {
                    let data: &[f32] = bytemuck::cast_slice(&tensor_data);
                    Tensor::from_slice(data, &shape, &self.device)?
                }
                candle_core::quantized::GgmlDType::F16 => {
                    let data: &[half::f16] = bytemuck::cast_slice(&tensor_data);
                    let data: Vec<f32> = data.iter().map(|x| x.to_f32()).collect();
                    Tensor::from_slice(&data, &shape, &self.device)?
                }
                _ => {
                    return Err(anyhow!("Unsupported tensor dtype: {:?}", tensor.ggml_dtype));
                }
            };
            tensors.insert(name.clone(), tensor);
        }

        // Create VarBuilder from tensors
        let vb = VarBuilder::from_tensors(tensors, DType::F32, &self.device);

        // Create config from GGUF metadata
        let metadata = &content.metadata;
        let config = LlamaConfig {
            hidden_size: metadata.get("llama.embedding_length").and_then(|v| v.to_u32()).unwrap_or(4096) as usize,
            intermediate_size: metadata.get("llama.feed_forward_length").and_then(|v| v.to_u32()).unwrap_or(11008) as usize,
            vocab_size: metadata.get("llama.vocab_size").and_then(|v| v.to_u32()).unwrap_or(32000) as usize,
            num_hidden_layers: metadata.get("llama.block_count").and_then(|v| v.to_u32()).unwrap_or(32) as usize,
            num_attention_heads: metadata.get("llama.attention.head_count").and_then(|v| v.to_u32()).unwrap_or(32) as usize,
            num_key_value_heads: metadata.get("llama.attention.head_count_kv").and_then(|v| v.to_u32()).unwrap_or(32) as usize,
            rms_norm_eps: metadata.get("llama.attention.layer_norm_rms_epsilon").and_then(|v| v.to_f32()).unwrap_or(1e-6),
            rope_theta: metadata.get("llama.rope.freq_base").and_then(|v| v.to_f32()).unwrap_or(10000.0),
            max_position_embeddings: metadata.get("llama.context_length").and_then(|v| v.to_u32()).unwrap_or(2048) as usize,
            use_flash_attn: false,
        };

        // Load the model
        println!("Creating Llama model with config: {:?}", config);
        let model = Llama::load(&vb, &config)?;
        self.model = Some(model);
        self.config = Some(config.clone());

        // Initialize cache
        self.cache = Some(Cache::new(true, DType::F32, &config, &self.device)?);

        // Try to load tokenizer
        self.tokenizer = self.load_tokenizer().await.ok();
        
        if self.tokenizer.is_none() {
            return Err(anyhow!("Could not load tokenizer. Please ensure tokenizer.json is in the same directory as your model file."));
        }

        // Initialize conversation with system prompt
        self.conversation.clear();
        self.conversation.push(ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });

        self.is_initialized = true;
        Ok(format!("Successfully loaded GGUF model: {} with real inference capability!", model_name))
    }

    async fn load_tokenizer(&self) -> Result<Tokenizer> {
        if let Some(model_path) = &self.model_path {
            let model_dir = Path::new(model_path).parent().unwrap_or(Path::new("."));
            
            let tokenizer_paths = [
                model_dir.join("tokenizer.json"),
                model_dir.join("tokenizer_config.json"),
                Path::new("tokenizer.json").to_path_buf(),
            ];
            
            for path in &tokenizer_paths {
                if path.exists() {
                    println!("Loading tokenizer from: {:?}", path);
                    return Tokenizer::from_file(path).map_err(|e| anyhow!("Tokenizer error: {}", e));
                }
            }
        }
        
        Err(anyhow!("Tokenizer file not found"))
    }

    async fn send_message(&mut self, message: &str) -> Result<String> {
        if !self.is_initialized {
            return Err(anyhow!("Model not initialized"));
        }

        // Add user message to conversation
        self.conversation.push(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });

        // Generate response using the actual model
        let response = self.generate_response_with_model(message).await?;
        
        // Add assistant response to conversation
        self.conversation.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });
        
        Ok(response)
    }

    async fn generate_response_with_model(&mut self, _message: &str) -> Result<String> {
        let model = self.model.as_ref().ok_or_else(|| anyhow!("Model not loaded"))?;
        let tokenizer = self.tokenizer.as_ref().ok_or_else(|| anyhow!("Tokenizer not loaded"))?;
        let cache = self.cache.as_mut().ok_or_else(|| anyhow!("Cache not initialized"))?;

        // Build conversation context
        let mut prompt = String::new();
        for msg in &self.conversation {
            match msg.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n", msg.content)),
                "user" => prompt.push_str(&format!("User: {}\n", msg.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n", msg.content)),
                _ => {}
            }
        }
        prompt.push_str("Assistant: ");

        println!("Generating response for prompt: {}", prompt);

        // Tokenize the prompt
        let encoding = tokenizer.encode(prompt, false).map_err(|e| anyhow!("Tokenization failed: {}", e))?;
        let tokens = encoding.get_ids();
        println!("Tokenized to {} tokens", tokens.len());
        
        let input_tokens = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;

        // Generate response tokens
        let mut generated_tokens = Vec::new();
        let mut current_tokens = input_tokens;
        let max_new_tokens = 256;

        for i in 0..max_new_tokens {
            println!("Generation step {}", i);
            
            // Forward pass through model
            let logits = model.forward(&current_tokens, 0, cache)?;
            
            // Get logits for the last token
            let last_token_logits = logits.i((0, logits.dim(1)? - 1))?;
            
            // Simple greedy sampling - pick the token with highest probability
            let next_token_id = last_token_logits.argmax(0)?.to_scalar::<u32>()?;
            
            // Check for end of sequence (common EOS tokens)
            if next_token_id == 2 || next_token_id == 0 {
                println!("Hit EOS token: {}", next_token_id);
                break;
            }
            
            generated_tokens.push(next_token_id);
            
            // Prepare next iteration - append the new token
            let new_token = Tensor::new(&[next_token_id], &self.device)?.unsqueeze(0)?;
            current_tokens = Tensor::cat(&[&current_tokens, &new_token], 1)?;
            
            // Stop if we've generated a reasonable amount
            if generated_tokens.len() > 50 && generated_tokens.len() % 10 == 0 {
                // Try to decode periodically to see if we have a complete thought
                if let Ok(partial) = tokenizer.decode(&generated_tokens, true) {
                    if partial.trim().ends_with('.') || partial.trim().ends_with('!') || partial.trim().ends_with('?') {
                        break;
                    }
                }
            }
        }

        println!("Generated {} tokens", generated_tokens.len());

        // Decode the generated tokens
        let response = tokenizer.decode(&generated_tokens, true).map_err(|e| anyhow!("Decoding failed: {}", e))?;
        
        println!("Generated response: {}", response);
        Ok(response.trim().to_string())
    }

    fn reset_conversation(&mut self) -> Result<()> {
        self.conversation.clear();
        self.conversation.push(ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });
        
        // Reset cache
        if let Some(config) = &self.config {
            self.cache = Some(Cache::new(true, DType::F32, config, &self.device)?);
        }
        
        Ok(())
    }
}

struct AppState {
    llm_agent: Arc<Mutex<LLMAgent>>,
}

#[tauri::command]
async fn initialize_model(state: State<'_, AppState>) -> Result<String, String> {
    // Load .env file if it exists
    if let Err(_) = dotenvy::dotenv() {
        // .env file not found, that's okay
    }
    
    // Get model path from environment variable or use default
    let model_path = env::var("MODEL_PATH").unwrap_or_else(|_| {
        r"E:\.lmstudio\models\lmstudio-community\Qwen3-30B-A3B-Instruct-2507-GGUF\Qwen3-30B-A3B-Instruct-2507-Q4_K_M.gguf".to_string()
    });
    
    // Clone the Arc to avoid holding the lock across await
    let agent_arc = state.llm_agent.clone();
    
    // Use tokio mutex for async compatibility
    let mut agent = agent_arc.lock().await;
    let result = agent.initialize(&model_path).await;
    
    match result {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("Failed to initialize model: {}", e)),
    }
}

#[tauri::command]
async fn send_message(message: String, state: State<'_, AppState>) -> Result<String, String> {
    // Clone the Arc to avoid holding the lock across await
    let agent_arc = state.llm_agent.clone();
    
    let result = {
        let mut agent = agent_arc.lock().await;
        agent.send_message(&message).await
    };
    
    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("Failed to generate response: {}", e)),
    }
}

#[tauri::command]
async fn reset_conversation(state: State<'_, AppState>) -> Result<String, String> {
    let mut agent = state.llm_agent.lock().await;
    
    match agent.reset_conversation() {
        Ok(_) => Ok("Conversation reset".to_string()),
        Err(e) => Err(format!("Failed to reset conversation: {}", e)),
    }
}

#[tauri::command]
async fn update_system_prompt(_prompt: String, _state: State<'_, AppState>) -> Result<String, String> {
    // This would require implementing system prompt update
    Ok("System prompt updated".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        llm_agent: Arc::new(Mutex::new(LLMAgent::new())),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            initialize_model,
            send_message,
            reset_conversation,
            update_system_prompt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}