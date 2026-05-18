# OpenChat

A lightweight, Windows-native desktop chat client that connects to
[OpenRouter](https://openrouter.ai), giving you access to a wide range of
language models through a single, clean interface.

## Features

- **Secure API key storage.** API keys are stored in the Windows
  Credential Manager (via DPAPI), never in a plain text file or environment
  variables.
- **Key validation.** The key is verified against the OpenRouter API before
  being accepted.
- **Chat interface.** A simple message thread with a model picker, an
  attachment button, and a persistent input bar.

## Requirements

- Windows 10 or later
- A valid [OpenRouter API key](https://openrouter.ai/keys)
- [Rust toolchain](https://rustup.rs) (only for building from source)

## Building

```sh
cargo build --release