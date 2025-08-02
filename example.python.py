"""
An interactive command‑line chatbot using a local LLaMa/Qwen model.

This script wraps a locally hosted large language model via the `llama_cpp` library
and exposes a conversational interface with several quality‑of‑life commands.

Features
--------
* Maintains conversation history so that the model can produce contextually aware responses.
* Allows updating the system prompt at runtime to steer the model’s behavior.
* Supports resetting the conversation while keeping the current system prompt.
* Persists the chat transcript to a text file on demand.
* Provides a simple file‑editing workflow: read a file, ask the model to apply
  modifications according to a user‑supplied description, show the proposed
  changes, and ask for confirmation before writing back.  Editing a file is an
  external side effect, so the user must explicitly approve it.

Usage
-----
Run this file directly:

    python interactive_llm_agent.py

Within the chat, the following slash commands are available:

    /system <message>   Set or update the system prompt.  This message is
                        prepended to the conversation and guides the model’s
                        responses.
    /reset              Clear the conversation history (but keep the current
                        system prompt).
    /save [filename]    Save the current conversation to a text file.  If
                        filename is omitted, defaults to ``conversation.txt``.
    /edit <filepath>    Initiate an interactive file editing session.  The
                        script will read the file, prompt you for a
                        description of the desired changes, solicit the model’s
                        suggestions, and ask for confirmation before writing.
    /exit               End the chat session.

Before running, ensure that you have installed the `llama_cpp` Python
package and have access to the specified GGUF model file on your system.

"""

import os
from llama_cpp import Llama


def initialize_model() -> Llama:
    """Initialize the LLaMa model with sensible defaults.

    Adjust `model_path` and other parameters here according to your local
    environment and hardware capabilities.  For example, on a machine with
    sufficient VRAM (e.g. an RTX 4090 with 24 GB of memory) you can leave
    ``n_gpu_layers=-1`` to offload as many layers as possible to the GPU.  If
    you encounter out‑of‑memory errors, experiment with lowering
    ``n_gpu_layers``.

    Returns
    -------
    Llama
        An initialized model ready for chat completions.
    """
    return Llama(
        model_path=r"E:\repo\ai_models\Qwen3-Coder-30B-A3B-Instruct-1M-UD-Q8_K_XL.gguf",
        n_ctx=4096,
        n_gpu_layers=-1,  # offload as many layers to GPU as VRAM permits
        n_threads=8,
        chat_format="chatml",
    )


def edit_file_with_model(llm: Llama, filepath: str) -> None:
    """Interactively edit a file using the language model.

    The function reads the specified file, asks the user to describe the
    desired changes, queries the model for an updated version, displays the
    suggested content, and finally asks the user whether to overwrite the
    original file.  It only writes to disk if the user confirms.

    Parameters
    ----------
    llm : Llama
        The instantiated language model used to generate suggestions.
    filepath : str
        Path to the file that should be edited.
    """
    if not os.path.isfile(filepath):
        print(f"[error] File not found: {filepath}")
        return

    try:
        with open(filepath, "r", encoding="utf-8") as f:
            original_content = f.read()
    except Exception as exc:
        print(f"[error] Unable to read file: {exc}")
        return

    print("\nCurrent contents of the file:\n")
    print("--- BEGIN ORIGINAL CONTENT ---")
    print(original_content)
    print("--- END ORIGINAL CONTENT ---\n")

    # Ask the user to describe the desired changes
    description = input(
        "Describe the changes you would like the model to apply to this file:\n"
    ).strip()
    if not description:
        print("No description provided; aborting edit.")
        return

    # Construct a system prompt guiding the model to act as a code refactoring assistant
    system_message = {
        "role": "system",
        "content": (
            "You are a helpful code refactoring assistant. "
            "Given a code file and a set of instructions, you propose an updated "
            "version of the code reflecting those instructions. "
            "Respond with only the full updated file content, without commentary."
        ),
    }
    # Compose the messages for the model
    messages = [
        system_message,
        {
            "role": "user",
            "content": (
                f"Here is the current file content:\n\n{original_content}\n\n"
                f"Please apply the following changes:\n{description}"
            ),
        },
    ]

    # Query the model for the updated content
    try:
        response = llm.create_chat_completion(
            messages=messages,
            max_tokens=2048,
            temperature=0.3,
        )
    except Exception as exc:
        print(f"[error] Model failed to generate updated content: {exc}")
        return

    updated_content = response["choices"][0]["message"]["content"].strip()

    print("\n--- BEGIN SUGGESTED UPDATED CONTENT ---")
    print(updated_content)
    print("--- END SUGGESTED UPDATED CONTENT ---\n")

    # Confirm before writing the file
    confirm = input(
        f"Overwrite '{filepath}' with the updated content? (yes/no): "
    ).strip().lower()
    if confirm == "yes":
        try:
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(updated_content)
            print(f"[info] File '{filepath}' updated successfully.")
        except Exception as exc:
            print(f"[error] Failed to write file: {exc}")
    else:
        print("Edit cancelled; no changes were made.")


def summarize_file_with_model(llm: Llama, filepath: str) -> None:
    """Read a file and request a summary from the model.

    This helper opens the given file, constructs a prompt to the model
    asking for a high-level description of its purpose and key
    components, and prints the resulting summary.  Unlike editing, this
    operation does not change the file on disk and therefore does not
    require user confirmation.

    Parameters
    ----------
    llm : Llama
        The language model used to generate the summary.
    filepath : str
        Path to the file whose contents should be summarized.
    """
    if not os.path.isfile(filepath):
        print(f"[error] File not found: {filepath}")
        return

    try:
        with open(filepath, "r", encoding="utf-8") as f:
            contents = f.read()
    except Exception as exc:
        print(f"[error] Unable to read file: {exc}")
        return

    # Build messages instructing the model to summarize the code
    system_message = {
        "role": "system",
        "content": (
            "You are a code analysis assistant. Given a source code file, "
            "provide a concise summary describing what the file does, the "
            "language it is written in, and any notable functions or classes. "
            "Do not repeat the entire code; focus on its purpose."
        ),
    }
    user_message = {
        "role": "user",
        "content": (
            "Here is the file content:\n\n" + contents + "\n\nPlease provide a summary."
        ),
    }
    try:
        response = llm.create_chat_completion(
            messages=[system_message, user_message],
            max_tokens=512,
            temperature=0.3,
        )
    except Exception as exc:
        print(f"[error] Failed to generate summary: {exc}")
        return
    summary = response["choices"][0]["message"]["content"].strip()
    print("\n--- FILE SUMMARY ---")
    print(summary)
    print("--- END OF SUMMARY ---\n")


def save_conversation(conversation: list[dict[str, str]], filename: str) -> None:
    """Save the conversation history to a text file.

    Parameters
    ----------
    conversation : list of dict
        The full conversation, including roles and messages.
    filename : str
        Destination filename.  If the file already exists, it will be
        overwritten.
    """
    try:
        with open(filename, "w", encoding="utf-8") as f:
            for entry in conversation:
                role = entry.get("role", "unknown").capitalize()
                content = entry.get("content", "")
                f.write(f"{role}: {content}\n\n")
        print(f"[info] Conversation saved to '{filename}'.")
    except Exception as exc:
        print(f"[error] Failed to save conversation: {exc}")


def interactive_chat() -> None:
    """Start an interactive chat session with the language model.

    This function sets up a persistent conversation list, handles special
    slash commands, sends user input to the model, and prints the model’s
    responses.  Commands are processed before model calls, enabling dynamic
    behavior such as adjusting the system prompt, resetting context, saving
    transcripts, and editing files.
    """
    llm = initialize_model()

    # Initialize conversation with a default system prompt
    conversation: list[dict[str, str]] = [
        {
            "role": "system",
            "content": "You are a helpful assistant.",
        }
    ]

    print(
        "\nWelcome to the interactive LLM agent!\n"
        "Type your messages and press Enter to receive a response.\n"
        "Available commands:\n"
        "  /system <message>   Set or update the system prompt\n"
        "  /reset              Clear conversation history (keep system prompt)\n"
        "  /save [filename]    Save conversation to a file (default: conversation.txt)\n"
        "  /read <filepath>    Summarize a file's contents using the model\n"
        "  /edit <filepath>    Edit a file using the model (requires confirmation)\n"
        "  /exit               Exit the chat session\n"
        "\n"
        "You can also reference a local file directly in a message. "
        "If the agent recognizes a valid file path, it will automatically "
        "summarize the file. Include words like 'edit' or 'modify' in your "
        "request to trigger an editing workflow."
    )

    while True:
        try:
            user_input = input("You: ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\n[info] Exiting chat.")
            break

        if not user_input:
            continue

        # Handle slash commands
        if user_input.lower().startswith("/system"):
            # Update system prompt
            new_system = user_input[len("/system"):].strip()
            conversation[0] = {
                "role": "system",
                "content": new_system,
            }
            print(f"[info] System prompt updated to: {new_system!r}")
            continue

        if user_input.lower() == "/reset":
            # Keep the system prompt but clear user/assistant messages
            system_message = conversation[0]
            conversation = [system_message]
            print("[info] Conversation history cleared; system prompt preserved.")
            continue

        if user_input.lower().startswith("/save"):
            # Save conversation
            parts = user_input.split(maxsplit=1)
            filename = "conversation.txt"
            if len(parts) > 1:
                filename = parts[1].strip() or filename
            save_conversation(conversation, filename)
            continue

        if user_input.lower().startswith("/read"):
            # Summarize a file without modifying it
            parts = user_input.split(maxsplit=1)
            if len(parts) < 2:
                print("[error] Please specify a file path to read.")
                continue
            filepath = parts[1].strip()
            summarize_file_with_model(llm, filepath)
            continue

        if user_input.lower().startswith("/edit"):
            # Initiate file editing
            parts = user_input.split(maxsplit=1)
            if len(parts) < 2:
                print("[error] Please specify a file path to edit.")
                continue
            filepath = parts[1].strip()
            edit_file_with_model(llm, filepath)
            continue

        if user_input.lower() in {"/exit", "exit", "quit"}:
            print("[info] Exiting chat.")
            break

        # Attempt to detect if the user is referring to a local file path and act automatically
        # This heuristic looks at each whitespace‑separated token and checks whether it
        # corresponds to a readable file on the local filesystem.  If so, the agent
        # will summarize or edit the file without requiring a special slash command.
        tokens = user_input.split()
        file_candidates: list[str] = []
        for tok in tokens:
            # Strip common punctuation and quoting characters
            cleaned = tok.strip("`'\"()[]{}<>.,;: ")
            if os.path.isfile(cleaned):
                file_candidates.append(cleaned)

        if file_candidates:
            # Determine whether the user intends to edit or merely inspect the file
            lower_input = user_input.lower()
            wants_edit = any(keyword in lower_input for keyword in ["edit", "modify", "change"])
            for path in file_candidates:
                if wants_edit:
                    edit_file_with_model(llm, path)
                else:
                    summarize_file_with_model(llm, path)
            # After handling the files, skip sending this message to the model
            continue

        # Normal chat: append user message, query model, and append reply
        conversation.append({"role": "user", "content": user_input})
        try:
            response = llm.create_chat_completion(
                messages=conversation,
                max_tokens=512,
                temperature=0.7,
            )
        except Exception as exc:
            print(f"[error] Failed to generate response: {exc}")
            # Remove the last user message to avoid polluting the conversation
            conversation.pop()
            continue

        reply = response["choices"][0]["message"]["content"].strip()
        print(f"Assistant: {reply}")
        conversation.append({"role": "assistant", "content": reply})


if __name__ == "__main__":
    interactive_chat()