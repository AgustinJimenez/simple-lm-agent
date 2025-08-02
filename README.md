# Simple LM Agent

A desktop chat application built with Tauri and React that provides a modern GUI interface for interacting with local GGUF language models. This project aims to run GGUF models directly in Rust for complete privacy and performance.

## Current Status

**🚧 In Development**: This project currently uses intelligent mock responses while the GGUF model integration is being developed. The application validates your GGUF model file exists and loads the necessary dependencies, but actual model inference is not yet implemented.

## Features

- 💬 **Modern Chat Interface**: Clean, responsive chat UI with message history
- 🔄 **Conversation Management**: Reset conversations and maintain context
- 🌙 **Dark/Light Mode**: Automatic theme switching based on system preferences
- ⚡ **High Performance**: Built with Rust and Tauri for maximum speed
- 🖥️ **Cross-platform**: Works on Windows, macOS, and Linux
- 🚀 **Self-contained**: Designed to run models locally with no external dependencies
- 🔒 **Privacy-First**: All processing stays on your device
- 📁 **GGUF Model Support**: Validates and prepares to load GGUF format models
- 🧠 **Intelligent Responses**: Context-aware responses while model integration is completed

## Prerequisites

Before running this project, make sure you have the following installed:

### Required Software
1. **Node.js** (v16 or higher) - [Download here](https://nodejs.org/)
2. **Rust** - [Install from rustup.rs](https://rustup.rs/)

### Package Managers
- **npm** (comes with Node.js) or **bun** (faster alternative)

## Installation

### 1. Clone the Repository
```bash
git clone <your-repo-url>
cd simple-lm-agent
```

### 2. Install Node.js Dependencies
```bash
npm install
# or if you prefer bun:
# bun install
```

### 3. Configure Your Model Path

**Important**: You need a GGUF model file for the application to initialize properly.

```bash
cp .env.example .env
```

Then edit the `.env` file and update the `MODEL_PATH` to point to your GGUF model:

```env
MODEL_PATH="E:\path\to\your\model.gguf"
```

**Where to get GGUF models:**
- [Hugging Face GGUF models](https://huggingface.co/models?library=gguf)
- [LM Studio model directory](https://lmstudio.ai/)
- Convert models using [llama.cpp](https://github.com/ggerganov/llama.cpp)

### 4. Add Tokenizer Files (Required for Future Model Loading)

For future model integration, you'll need tokenizer files in the same directory as your GGUF model:
- `tokenizer.json`
- `tokenizer_config.json` (optional)

These usually come with downloaded models or can be found in the original model repositories.

## Running the Application

### Development Mode
```bash
npm run tauri dev
# or with bun:
# bun run tauri dev
```

**Important:** You must use `npm run tauri dev` (not `npm run dev`) to run the Tauri app with the Rust backend.

This will:
1. Start the Vite development server for the React frontend
2. Launch the Tauri development window with Rust backend
3. The Rust backend will validate your GGUF model when you click "Initialize Model"

### Building for Production
```bash
npm run build
# or with bun:
# bun run build
```

The built application will be available in the `src-tauri/target/release/` directory.

## Usage

1. **Launch the app** using `npm run tauri dev`
2. **Initialize the model** by clicking the "Initialize Model" button
   - The app will validate your GGUF model file exists
   - You'll see a confirmation message when initialization is complete
3. **Start chatting** by typing in the input field and pressing Enter
   - The app will respond with intelligent, context-aware messages
   - Responses will mention your specific model name
4. **Reset conversation** using the "Reset Chat" button to clear history

## Current Response System

While GGUF integration is being completed, the app provides intelligent responses that:
- Are context-aware based on your input
- Mention your specific model name
- Provide helpful information about different topics
- Maintain conversation context
- Indicate the model file is validated and ready

## Troubleshooting

### Common Issues
- **Model not found**: Ensure your `.env` file has the correct path to a valid GGUF file
- **"Initialize Model" fails**: Check that the file path exists and points to a `.gguf` file
- **Compilation errors**: Make sure Rust is properly installed

### Build Issues
- **Rust not found**: Install Rust from [rustup.rs](https://rustup.rs/)
- **Node modules**: Delete `node_modules` and run `npm install` again
- **Tauri build fails**: Check the [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites/)

### Model Integration Issues
- **Tokenizer not found**: Ensure `tokenizer.json` is in the same directory as your GGUF model
- **Candle compilation errors**: The GGUF integration is complex and under development

## Project Structure
```
simple-lm-agent/
├── src/                    # React frontend
│   ├── App.tsx            # Main chat interface
│   ├── App.css            # Styling
│   └── main.tsx           # React entry point
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Tauri commands and LLM agent
│   │   └── main.rs        # Entry point
│   └── Cargo.toml         # Rust dependencies (includes Candle-RS)
├── .env.example           # Environment variables template
├── .env                   # Your local configuration
├── TODO.md                # Current development status and tasks
├── package.json           # Node.js dependencies
└── README.md              # This file
```

## Development Status

The project is structured to support full GGUF model integration using Candle-RS. Current implementation:

✅ **Completed:**
- Modern Tauri + React chat interface
- GGUF model file validation
- Candle-RS dependencies and setup
- Intelligent context-aware response system
- Environment configuration
- Conversation management

🚧 **In Progress:**
- Direct GGUF model loading with Candle-RS
- Tokenizer integration
- Model inference pipeline

❌ **Not Yet Implemented:**
- Actual GGUF model inference
- GPU acceleration support
- Advanced sampling methods
- Model switching at runtime

See `TODO.md` for detailed development status and next steps.

## Configuration

### Environment Variables
Create a `.env` file in the project root with:
```env
MODEL_PATH="E:\path\to\your\model.gguf"
```

### Model Requirements
- **Format**: GGUF (GGML Unified Format)
- **Location**: Anywhere on your system (specify in `.env`)
- **Tokenizer**: `tokenizer.json` in same directory (for future integration)
- **Size**: Any size (larger models require more RAM)

### UI Customization
Edit `src/App.css` to customize the chat interface appearance.

## Technical Details

### Backend
- **Language**: Rust
- **Framework**: Tauri 2.0
- **ML Library**: Candle-RS (pure Rust)
- **Model Format**: GGUF via candle-core
- **Tokenization**: HuggingFace tokenizers

### Frontend
- **Language**: TypeScript
- **Framework**: React 18
- **Build Tool**: Vite
- **Styling**: CSS with system theme support

## Contributing

This project is actively being developed. The main challenge is implementing robust GGUF model loading and inference with Candle-RS. Contributions to the model integration are especially welcome.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

This project is open source. Please check the license file for details.