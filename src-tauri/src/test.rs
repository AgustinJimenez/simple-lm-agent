use anyhow::{anyhow, Result};
use candle_core::quantized::gguf_file;
use candle_core::safetensors::{self, SafeTensors};
use candle_core::{Device, Tensor};
use candle_transformers::models::llama::{Config, Llama};
use std::{fs::File, path::Path};
use tokenizers::Tokenizer;

fn main() -> Result<()> {
    let model_path = "/Users/agus/.lmstudio/models/lmstudio-community/DeepSeek-R1-0528-Qwen3-8B-MLX-4bit/model.safetensors";
    let tokenizer_path = "/Users/agus/.lmstudio/models/lmstudio-community/DeepSeek-R1-0528-Qwen3-8B-MLX-4bit/tokenizer.json";
    let device = Device::cuda_if_available(0)?;

    // Load tokenizer
    let tokenizer = Tokenizer::from_file(tokenizer_path)?;

    // Automatically detect and load the model based on file type
    let (model, config) = load_model(model_path, &device)?;

    // Example prompt
    let prompt = "Explain Rust lifetimes in simple terms.";
    let tokens = tokenizer.encode(prompt, true)?;
    let mut output_ids = tokens.get_ids().to_vec();
    let mut cache = model.empty_cache();

    // Generate up to 128 tokens
    for _ in 0..128 {
        let input_tensor = Tensor::new(output_ids.as_slice(), &device)?.unsqueeze(0)?;
        let logits = model.forward(&input_tensor, 0, &mut cache)?;
        let logits = logits.squeeze(0)?.get(output_ids.len() - 1)?;
        let next_token_id = logits.argmax(0)?.to_scalar::<u32>()?;

        // Stop on EOS token
        if next_token_id == tokenizer.token_to_id("</s>").unwrap_or(0) {
            break;
        }
        output_ids.push(next_token_id);
    }

    // Decode generated tokens
    let generated_text = tokenizer.decode(&output_ids, true)?;
    println!("Generated Text:\n{}", generated_text);

    Ok(())
}

// Function to automatically detect and load model
fn load_model<P: AsRef<Path>>(path: P, device: &Device) -> Result<(Llama, Config)> {
    let path_ref = path.as_ref();
    let ext = path_ref.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "gguf" => {
            println!("Detected GGUF model format.");
            let mut file = File::open(path_ref)?;
            let gguf = gguf_file::Content::read(&mut file)?;
            let config = Config::from_gguf(&gguf)?;
            let vb = gguf.var_builder(device)?;
            let model = Llama::load(vb, &config)?;
            Ok((model, config))
        }
        "safetensors" => {
            println!("Detected SafeTensors model format.");
            let tensors = SafeTensors::load(path_ref, device)?;
            let config = load_config_for_safetensors(path_ref)?;
            let vb = tensors.var_builder();
            let model = Llama::load(vb, &config)?;
            Ok((model, config))
        }
        _ => Err(anyhow!("Unsupported file type: {}", ext)),
    }
}

// Simple function to load or define config for SafeTensors models
fn load_config_for_safetensors<P: AsRef<Path>>(_path: P) -> Result<Config> {
    // You'd typically load this config from a separate JSON file or define it manually.
    // Here's a minimal example for a LLaMA-7B:
    Ok(Config {
        hidden_size: 4096,
        intermediate_size: 11008,
        n_heads: 32,
        n_layers: 32,
        vocab_size: 32000,
        rms_norm_eps: 1e-6,
        rope_theta: 10000.0,
        rope_traditional: true,
    })
}
