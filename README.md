<div align="center">

# 💬 Lumen Chat

A lightweight, monolithic native chat client that connects to
[OpenRouter](https://openrouter.ai), giving you access to a wide range of
language models through a single, clean, and extremely lightweight interface.

![Rust](https://img.shields.io/badge/Rust-2024-000599C?logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Windows-x64-0078D6?logo=windows&logoColor=white)

</div>

## Features

- **Secure API key storage.** API keys are stored in the Windows
  Credential Manager (via DPAPI), never in a plain text file or environment
  variables.
- **Key validation.** The key is verified against the OpenRouter API before
  being accepted.
- **Chat interface.** A simple message thread with a model picker, an
  attachment button, and a persistent input bar.
- **Super optimized.** Built in Rust, the whole application uses *80%* less
  system memory than a ChatGPT tab.

### Supported Models

| Provider | Models |
| :--- | :--- |
| **OpenAI** | GPT-5.5, GPT-5.4 Thinking, GPT-5.3 Instant |
| **Google** | Gemini 3.1 Pro, Gemini 3.5 Flash, Gemini 3.1 Flash-Lite |
| **Anthropic** | Claude Opus 4.8, Claude Sonnet 4.6, Claude Haiku 4.5 |
| **xAI** | Grok 4.20 Multi-Agent, Grok 4.3, Grok 4.20 |
| **Alibaba** | Qwen3.7-Max, Qwen3.6-Plus, Qwen3.6-Flash |
| **DeepSeek** | DeepSeek V4 Pro, DeepSeek V4 Flash, DeepSeek V3.2 |
| **Z.ai** | GLM-5.1, GLM-5V-Turbo, GLM-5-Turbo |
| **MoonshotAI** | Kimi K2.6, Kimi K2.5 |
| **MiniMax** | MiniMax-M3, MiniMax-M2.7, MiniMax-M2-her |
| **Xiaomi** | MiMo-V2.5-Pro, MiMo-V2.5, MiMo-V2-Flash |

## Requirements

- Windows 10 or later
- A valid [OpenRouter API key](https://openrouter.ai/keys)
- [Visual C++ for Visual Studio 2015-2022 x64](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- [Rust toolchain](https://rustup.rs) (only for building from source)

## Building

```sh
cargo build --release
```