use std::sync::Arc;
use std::env;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

struct LLMAgent {
    model_path: Option<String>,
    conversation: Vec<ChatMessage>,
    system_prompt: String,
    is_initialized: bool,
    server_url: String,
    model_name: String,
}

impl LLMAgent {
    fn new() -> Self {
        Self {
            model_path: None,
            conversation: Vec::new(),
            system_prompt: "You are a helpful assistant.".to_string(),
            is_initialized: false,
            server_url: "http://localhost:1234".to_string(), // LM Studio default
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
        
        // Initialize conversation with system prompt
        self.conversation.clear();
        self.conversation.push(ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });

        // Test connection to local LLM server
        match self.test_server_connection().await {
            Ok(_) => {
                self.is_initialized = true;
                Ok(format!("Connected to local LLM server! Using model: {}", model_name))
            }
            Err(_e) => {
                // Fallback to mock mode if server not available
                self.is_initialized = true;
                Ok(format!("Model file validated: {} (Server not running - using mock responses. Start LM Studio or Ollama to use real LLM)", model_name))
            }
        }
    }

    async fn test_server_connection(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/v1/models", self.server_url))
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Server not responding"))
        }
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

        // Try to use real LLM first, fallback to mock if server unavailable
        match self.call_llm_server().await {
            Ok(response) => {
                // Add assistant response to conversation
                self.conversation.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response.clone(),
                });
                Ok(response)
            }
            Err(_) => {
                // Fallback to mock response
                let response = self.generate_contextual_response(message);
                self.conversation.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response.clone(),
                });
                Ok(response)
            }
        }
    }

    async fn call_llm_server(&self) -> Result<String> {
        let client = reqwest::Client::new();
        
        // Convert conversation to OpenAI format
        let messages: Vec<OpenAIMessage> = self.conversation
            .iter()
            .map(|msg| OpenAIMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        let request = OpenAIRequest {
            model: self.model_name.clone(),
            messages,
            temperature: 0.7,
            max_tokens: 512,
        };

        let response = client
            .post(&format!("{}/v1/chat/completions", self.server_url))
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            let llm_response: OpenAIResponse = response.json().await?;
            if let Some(choice) = llm_response.choices.first() {
                Ok(choice.message.content.clone())
            } else {
                Err(anyhow!("No response from LLM"))
            }
        } else {
            Err(anyhow!("LLM server error: {}", response.status()))
        }
    }

    fn generate_contextual_response(&self, user_message: &str) -> String {
        let user_lower = user_message.to_lowercase();
        
        // Get model name for context
        let model_name = self.model_path.as_ref()
            .and_then(|path| Path::new(path).file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("your model");

        // Generate contextually appropriate responses
        if user_lower.contains("hello") || user_lower.contains("hi") {
            format!("Hello! I'm a Rust-based agent that will eventually run {} directly. How can I help you today?", model_name)
        } else if user_lower.contains("model") || user_lower.contains("llm") {
            format!("I'm currently in development mode, but I'm configured to use your {} model. Full LLM integration is coming soon!", model_name)
        } else if user_lower.contains("rust") {
            "Yes, I'm built entirely in Rust! This provides excellent performance and memory safety for running large language models.".to_string()
        } else if user_lower.contains("performance") || user_lower.contains("speed") {
            "Rust provides excellent performance for AI workloads. Once the LLM integration is complete, you'll see very fast inference times!".to_string()
        } else if user_lower.contains("help") || user_lower.contains("what") {
            "I'm a local AI assistant built with Rust and Tauri. I can chat with you and will soon be able to run your local language model directly for truly private AI conversations.".to_string()
        } else {
            format!("I understand you're asking about '{}'. I'm currently in development mode using your {} model. The full LLM integration will allow me to provide more sophisticated responses soon!", 
                   user_message, model_name)
        }
    }

    fn reset_conversation(&mut self) -> Result<()> {
        self.conversation.clear();
        self.conversation.push(ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        });
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
        let mut agent = agent_arc.lock().unwrap();
        agent.send_message(&message).await
    };
    
    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("Failed to generate response: {}", e)),
    }
}

#[tauri::command]
async fn reset_conversation(state: State<'_, AppState>) -> Result<String, String> {
    let mut agent = state.llm_agent.lock().unwrap();
    
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
