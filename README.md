# Simple LM Agent

A desktop chat application built with Tauri and React that provides a modern GUI interface for interacting with AI models. Currently includes a mock implementation for testing the interface, with plans for real language model integration.

## Features

- 💬 **Modern Chat Interface**: Clean, responsive chat UI with message history
- 🔄 **Conversation Management**: Reset conversations and maintain context
- 🌙 **Dark/Light Mode**: Automatic theme switching based on system preferences
- ⚡ **High Performance**: Built with Rust and React for maximum speed
- 🖥️ **Cross-platform**: Works on Windows, macOS, and Linux
- 🚀 **Self-contained**: No external runtime dependencies
- 🧪 **Mock Implementation**: Currently includes demo responses for interface testing

## Current Status

**⚠️ Note**: This version includes a mock AI agent that provides demo responses to test the chat interface. The mock agent will respond with pre-written messages to demonstrate the functionality while you set up a real language model integration.

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

### 3. Configure Your Model Path (Optional)

The current version uses a mock AI agent, but you can still configure a model path for future use:

```bash
cp .env.example .env
```

Then edit the `.env` file and update the `MODEL_PATH`:

```env
MODEL_PATH=path/to/your/model.gguf
```

**Note**: The app will currently check if the model file exists but won't actually load it. Instead, it will provide demo responses.

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
3. The Rust backend with integrated LLM will be ready when you click "Initialize Model"

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
   - The mock agent will initialize quickly
   - You'll see a "Model Ready" indicator when initialization is complete
3. **Start chatting** by typing in the input field and pressing Enter
   - The app will respond with demo messages to test the interface
4. **Reset conversation** using the "Reset Chat" button to clear history
5. **Mock responses** are defined in `src-tauri/src/lib.rs` in the `generate_response` method

## Troubleshooting

### Common Issues
- **Model not found**: Currently expected behavior - the app uses mock responses
- **"Initialize Model" fails**: Check that you have a valid file path in your `.env` file
- **No AI responses**: This is expected - the app currently provides demo responses

### Build Issues
- **Rust not found**: Install Rust from [rustup.rs](https://rustup.rs/)
- **Node modules**: Delete `node_modules` and run `npm install` again
- **Tauri build fails**: Check the [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites/)

### Real LLM Integration (Future)
To integrate a real language model, you would need to:
1. Install LLVM/Clang for Windows compilation
2. Replace the mock implementation with actual LLM bindings
3. Handle the compilation dependencies for native C++ libraries

For now, the mock implementation lets you test the chat interface without these complexities.

## Project Structure
```
simple-lm-agent/
├── src/                    # React frontend
│   ├── App.tsx            # Main chat interface
│   ├── App.css            # Styling
│   └── main.tsx           # React entry point
├── src-tauri/             # Rust backend with mock AI agent
│   ├── src/
│   │   ├── lib.rs         # Tauri commands and mock AI agent
│   │   └── main.rs        # Entry point
│   └── Cargo.toml         # Rust dependencies
├── .env.example           # Environment variables template
├── .env                   # Your local configuration (create from .env.example)
├── package.json           # Node.js dependencies
└── README.md              # This file
```

## Configuration

### Mock Agent Settings
- **Model Path**: Set `MODEL_PATH` in your `.env` file (for validation only)
- **Demo Responses**: Edit the `responses` array in `src-tauri/src/lib.rs`
- **Conversation History**: Stored in memory during the session

### Environment Variables
Create a `.env` file in the project root with:
```env
MODEL_PATH=path/to/your/model.gguf
```

The mock agent will validate the file exists but won't load it.

### UI Customization
Edit `src/App.css` to customize the chat interface appearance.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

This project is open source. Please check the license file for details.
