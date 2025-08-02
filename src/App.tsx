import { useState, useEffect, useRef } from "react";
import "./App.css";

// Extend Window interface for Tauri
declare global {
  interface Window {
    __TAURI__?: any;
  }
}

interface Message {
  id: number;
  content: string;
  isUser: boolean;
  timestamp: Date;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isModelInitialized, setIsModelInitialized] = useState(false);
  const [initializationStatus, setInitializationStatus] = useState("");
  const [debugInfo, setDebugInfo] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Debug info on component mount
  useEffect(() => {
    const checkTauriContext = async () => {
      const info = [];
      info.push(`window.__TAURI__: ${!!window.__TAURI__}`);
      info.push(`User Agent: ${navigator.userAgent}`);
      
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        info.push(`Tauri API imported: ${!!invoke}`);
      } catch (e) {
        info.push(`Tauri API import failed: ${e}`);
      }
      
      setDebugInfo(info.join(" | "));
    };
    
    // Wait a bit for Tauri to initialize, then check again
    const timeoutId = setTimeout(checkTauriContext, 100);
    checkTauriContext(); // Check immediately too
    
    return () => clearTimeout(timeoutId);
  }, []);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const initializeModel = async () => {
    setIsLoading(true);
    setInitializationStatus("Initializing model...");
    
    try {
      // Dynamic import of Tauri API
      const { invoke } = await import("@tauri-apps/api/core");
      
      // Try to invoke directly, sometimes it works even when window.__TAURI__ is false
      setInitializationStatus("Attempting to call Rust backend...");
      const result = await invoke<string>("initialize_model");
      
      setIsModelInitialized(true);
      setInitializationStatus(result);
      setTimeout(() => setInitializationStatus(""), 3000);
    } catch (error) {
      console.error("Initialize model error:", error);
      
      // If direct invoke fails, show detailed error info
      if (error instanceof Error) {
        if (error.message.includes('invoke')) {
          setInitializationStatus(`Error: Failed to communicate with Rust backend. ${error.message}`);
        } else {
          setInitializationStatus(`Error: ${error.message}`);
        }
      } else {
        setInitializationStatus(`Error: ${error}`);
      }
      setIsModelInitialized(false);
    } finally {
      setIsLoading(false);
    }
  };

  const sendMessage = async () => {
    if (!inputValue.trim() || !isModelInitialized || isLoading) return;

    const messageToSend = inputValue;
    const userMessage: Message = {
      id: Date.now(),
      content: messageToSend,
      isUser: true,
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputValue("");
    setIsLoading(true);

    try {
      // Dynamic import of Tauri API
      const { invoke } = await import("@tauri-apps/api/core");
      
      const response = await invoke<string>("send_message", { message: messageToSend });
      
      const assistantMessage: Message = {
        id: Date.now() + 1,
        content: response,
        isUser: false,
        timestamp: new Date(),
      };

      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error("Send message error:", error);
      const errorMessage: Message = {
        id: Date.now() + 1,
        content: `Error: ${error}`,
        isUser: false,
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const resetConversation = async () => {
    try {
      // Dynamic import of Tauri API
      const { invoke } = await import("@tauri-apps/api/core");
      
      await invoke<string>("reset_conversation");
      setMessages([]);
    } catch (error) {
      console.error("Failed to reset conversation:", error);
    }
  };

  return (
    <div className="chat-container">
      <header className="chat-header">
        <h1>Simple LM Agent</h1>
        <div className="header-controls">
          {!isModelInitialized ? (
            <button 
              onClick={initializeModel} 
              disabled={isLoading}
              className="init-button"
            >
              {isLoading ? "Initializing..." : "Initialize Model"}
            </button>
          ) : (
            <div className="status-controls">
              <span className="status-indicator">Model Ready</span>
              <button onClick={resetConversation} className="reset-button">
                Reset Chat
              </button>
            </div>
          )}
        </div>
        {initializationStatus && (
          <div className="status-message">{initializationStatus}</div>
        )}
        {debugInfo && (
          <div className="debug-info" style={{fontSize: '12px', color: '#666', marginTop: '0.5rem'}}>
            Debug: {debugInfo}
          </div>
        )}
      </header>

      <div className="messages-container">
        {messages.length === 0 && isModelInitialized && (
          <div className="empty-state">
            <p>Start a conversation with your local AI assistant!</p>
          </div>
        )}
        
        {messages.map((message) => (
          <div
            key={message.id}
            className={`message ${message.isUser ? "user-message" : "assistant-message"}`}
          >
            <div className="message-content">
              <div className="message-text">{message.content}</div>
              <div className="message-time">
                {message.timestamp.toLocaleTimeString()}
              </div>
            </div>
          </div>
        ))}
        
        {isLoading && (
          <div className="message assistant-message">
            <div className="message-content">
              <div className="loading-indicator">
                <span>.</span><span>.</span><span>.</span>
              </div>
            </div>
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>

      <div className="input-container">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            sendMessage();
          }}
          className="input-form"
        >
          <input
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder={isModelInitialized ? "Type your message..." : "Initialize the model first"}
            disabled={!isModelInitialized || isLoading}
            className="message-input"
          />
          <button
            type="submit"
            disabled={!inputValue.trim() || !isModelInitialized || isLoading}
            className="send-button"
          >
            Send
          </button>
        </form>
      </div>
    </div>
  );
}

export default App;
