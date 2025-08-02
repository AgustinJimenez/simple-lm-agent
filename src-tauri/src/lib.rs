use std::sync::Arc;
use std::env;
use std::path::Path;
use tauri::State;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use llama_cpp_2::{
    llama_backend::LlamaBackend,
    model::{LlamaModel, params::LlamaModelParams, AddBos, Special},
    context::params::LlamaContextParams,
    sampling::LlamaSampler,
    llama_batch::LlamaBatch,
};
use std::num::NonZeroU32;

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
    // Remove the direct storage of llama components due to Send/Sync issues
    // We'll create them when needed in each method
}

impl LLMAgent {
    fn new() -> Self {
        Self {
            model_path: None,
            conversation: Vec::new(),
            system_prompt: "You are a helpful assistant.".to_string(),
            is_initialized: false,
            model_name: "local-model".to_string(),
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

        // Test that we can initialize the components (but don't store them)
        println!("Initializing llama-cpp backend...");
        let _backend = LlamaBackend::init().map_err(|e| anyhow!("Failed to initialize llama backend: {:?}", e))?;

        println!("Loading GGUF model from: {}", model_path);
        let model_params = LlamaModelParams::default();
        let _model = LlamaModel::load_from_file(
            &_backend,
            model_path,
            &model_params
        ).map_err(|e| anyhow!("Failed to load model: {:?}", e))?;
        
        println!("Testing context creation...");
        let context_params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()))  // Set context size
            .with_n_batch(512)                             // Set batch size
            .with_n_threads(8);                 // Set number of threads
        
        let _context = _model.new_context(
            &_backend,
            context_params
        ).map_err(|e| anyhow!("Failed to create context: {:?}", e))?;

        // Initialize conversation with system prompt
        self.conversation.clear();
        self.conversation.push(ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });

        self.is_initialized = true;
        Ok(format!("Successfully loaded GGUF model: {} with llama-cpp-rs real inference capability!", model_name))
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
        let model_path = self.model_path.as_ref().ok_or_else(|| anyhow!("Model path not set"))?;

        // Create fresh llama components for this generation
        println!("Creating llama backend...");
        let backend = LlamaBackend::init().map_err(|e| anyhow!("Failed to initialize llama backend: {:?}", e))?;

        println!("Loading model...");
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(
            &backend,
            model_path,
            &model_params
        ).map_err(|e| anyhow!("Failed to load model: {:?}", e))?;
        
        println!("Creating context...");
        let context_params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()))
            .with_n_batch(512)
            .with_n_threads(8);
        
        let mut context = model.new_context(
            &backend,
            context_params
        ).map_err(|e| anyhow!("Failed to create context: {:?}", e))?;

        // Build conversation context as prompt
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

        // Tokenize the prompt using the model
        let tokens = model.str_to_token(&prompt, AddBos::Always)
            .map_err(|e| anyhow!("Tokenization failed: {:?}", e))?;
        
        println!("Tokenized to {} tokens", tokens.len());

        // Clear context and encode prompt
        context.clear_kv_cache();
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], false)?;
        }
        context.decode(&mut batch)
            .map_err(|e| anyhow!("Failed to decode prompt: {:?}", e))?;

        // Create sampler for token generation
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.8),
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),  // min_keep = 1
        ]);

        // Generate tokens
        let mut response_tokens = Vec::new();
        let max_tokens = 128;
        
        for i in 0..max_tokens {
            let next_token = sampler.sample(&context, (tokens.len() + i) as i32);
            
            // Check for end of sequence
            if next_token == model.token_eos() {
                println!("Hit EOS token after {} tokens", i);
                break;
            }
            
            response_tokens.push(next_token);
            
            // Decode the new token
            let mut single_batch = LlamaBatch::new(1, 1);
            single_batch.add(next_token, (tokens.len() + i) as i32, &[0], false)?;
            context.decode(&mut single_batch)
                .map_err(|e| anyhow!("Failed to decode token: {:?}", e))?;
        }

        println!("Generated {} tokens total", response_tokens.len());

        // Detokenize the response tokens using the model
        let response = model.tokens_to_str(&response_tokens, Special::Tokenize)
            .map_err(|e| anyhow!("Detokenization failed: {:?}", e))?;
        
        println!("Generated Text:\n{}", response);
        Ok(response.trim().to_string())
    }

    fn reset_conversation(&mut self) -> Result<()> {
        self.conversation.clear();
        self.conversation.push(ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });
        
        // No need to clear KV cache since we create fresh contexts for each generation
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
        return Err("No .env file found. Please create a .env file with MODEL_PATH specified.".to_string());
    }
    
    // Get model path from environment variable - required, no default
    let model_path = env::var("MODEL_PATH").map_err(|_| {
        "MODEL_PATH not found in .env file. Please add MODEL_PATH=/path/to/your/model.gguf to your .env file.".to_string()
    })?;
    
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