# 🌙 Luna

Luna is a modern, fast, and highly customizable shell developed in Rust. It features a rich-text rendering engine, modular built-in commands, real-time IDE-like feedback, and a powerful glob and configuration system.

![Luna Preview](https://github.com/sammwyy/luna/raw/main/assets/preview.png)

## ✨ Features

- 🦀 **Powered by Rust**: Fast, safe, and reliable.
- 🪄 **Interactive Command Corrector**: Suggests alternatives and corrections dynamically using Damerau-Levenshtein distance when you misspell a command.
- 🚦 **Real-time Engine**: 
  - **Syntax Highlighting**: Commands, flags, strings, numbers, and booleans are colored as you type.
  - **Dry-run Validations**: Real-time error messages (no such file, invalid flags) *before* you even hit enter.
  - **Smart Hints & Autocompletion**: Completes files, commands, and arguments dynamically (Context-aware inside pipelines `|` and `&&`).
- 📂 **Universal File Parsing**: 
  - Seamless expansion of globs (`*`, `?`, `[]`) across built-ins.
  - Native Brace Expansion (`path/to/{a,b}.txt`).
- 🧭 **Browser-like Navigation**: Uses `back` and `next` commands to easily traverse your directory history during the session.
- 🎨 **Rich Text Engine**: Support for colors, hex, gradients, and text styles via an easy XML-like syntax (`<#58a6ff>Text</#58a6ff>`).
- 🌙 **Lua Theming & Plugins**: Completely dynamic prompts and extendable behavior with Lua hooks.
- 🛠️ **Modular Built-ins**: Includes many essential commands (`ls` with rich tables, `cat` with syntax highlighting, `grep`, `jq`, etc.) configurable explicitly via TOML.
- 🔄 **Alias System**: Bash-compatible aliases, plus native POSIX/Windows built-in aliases out of the box (`dir`, `cls`, `move`, `whereis`).

## 🧩 Plugins

Default plugins are located in `~/.luna/plugins/` when you run `luna init`.

- **`autoenv.lua`**: Reads `.env` and `.env.local` files automatically without polluting the global environment, injecting them exclusively for the duration of the current command execution.
- **`autoaliases.lua`**: Temporarily reads and registers `.aliases` files contained in the current directory and cleanly unloads them when you navigate backwards or into another path.
- **`git.lua`**: Listens for directory changes and prompt events to export the current Git branch and repository status directly to your prompt template variables.
- **`runtime_version.lua`**: Detects if your current directory is a Rust, Node, Python, or Go project, and exposes the specific Language Version you are using to your prompt templates dynamically.
- **`windows-aliases.lua`**: Registers native Command Prompt translations into the shell mapping (e.g., `dir` → `ls`, `cls` → `clear`, `move` → `mv`).

> Delete those files inside your `.luna/plugins/` to disable them! (Or change file extension)

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/sammwyy/luna.git
cd luna

# Build and install
cargo install --path .
```

### Setup

Luna is completely decoupled from your disk by default. To install the default configuration file, sample themes, and useful plugins to your `~/.luna` directory, run:

```bash
luna init
```

*Note: If you broke your config or want to restore all default assets, you can run `luna init --force`.*

Luna is fully configurable. Check out your `~/.luna/config.toml` to customize components like the syntax highlighter, corrector, tab-completion behavior, and granular features per-builtin (like changing default lines for `tail` or making `cd` go home by default).


## 📖 Documentation

- [Configuration Guide](docs/config.md)
- [Creating Themes](docs/themes.md)
- [Writing Plugins](docs/plugins.md)

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## 📄 License

This project is licensed under the **MIT License**.

## 👤 Author

Developed with ❤️ by **[sammwy](https://github.com/sammwyy)**.
