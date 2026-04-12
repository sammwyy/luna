# 🌙 Luna

Luna is a modern, fast, and highly customizable shell developed in Rust. It features a rich-text rendering engine, modular built-in commands, real-time IDE-like feedback, and a powerful glob and configuration system.

## ✨ Features

### 🚦 Next-Gen Real-Time Engine
- **Live Syntax Highlighting**: Visual feedback as you type for commands, flags, strings, numbers, and booleans.
- **Instant Dry-Run Validations**: Catches errors (like missing files or invalid flags) *before* you hit Enter, showing you exactly what will fail.
- **Context-Aware Autocompletion**: Smart hints for files, commands, and arguments that understand pipelines (`|`), logical operators (`&&`, `||`), and nested paths.
- **Intelligent Command Corrector**: Dynamic suggestions using Damerau-Levenshtein distance to fix typos instantly (e.g., `git sttaus` → `git status`).

### 🎨 Advanced Rendering & UI
- **Rich Text Markup**: An XML-like syntax for terminal output supporting hex colors, named colors, gradients, and text styles (`<bold>`, `<italic>`, `<#ff0000>`).
- **Data-First Visualization**: Automatic **Rich Table** rendering for data-heavy commands like `ls` and `stat`.
- **Integrated Image Viewer**: High-quality image rendering directly in the terminal using the `view` command.
- **Dynamic Theming**: Full control over shell colors and prompt styles via a flexible Lua-based theme engine.

### 📂 Superior File & Path Management
- **Universal Glob Expansion**: Native support for complex patterns (`*`, `?`, `[]`) across all built-in commands.
- **Powerful Brace Expansion**: Efficient path generation with native brace support (e.g., `touch src/{lib,main}.rs`).
- **Interactive Fuzzy Navigation**: Use `cdi` for a fuzzy-finding directory navigator or `back`/`next` for browser-like history traversal.
- **Recursive Operations**: Native recursive flags for common tools (`ls -R`, `rm -r`, `cp -r`) with optimized performance.

### 🛠️ Modular Built-in Ecosystem
Luna includes a suite of high-performance native commands, each configurable via a central TOML file:
- **`cat`**: Feature-rich file viewer with syntax highlighting and line numbering.
- **`grep`**: Fast text searching with colorized results and multi-file support.
- **`jq`**: Built-in JSON processor for quick data manipulation.
- **`math`**: Inline mathematical evaluator for quick calculations.
- **`tree`**: Beautiful recursive directory visualization.
- **`stat`**: Detailed file metadata and permission explorer.
- **`sed`**: Stream editor for quick text transformations.
- **`view`**: Image viewer for the terminal.
- **Standard POSIX Tools**: Enhanced versions of `mkdir`, `cp`, `mv`, `rm`, `wc`, `sort`, `uniq`, and more.

### 🌙 Extensibility via Lua
Luna is a platform, not just a shell. Its behavior can be completely transformed using Lua:
- **Hooks**: Listen for command execution, directory changes, or prompt requests.
- **Environment Injection**: Plugins like `autoenv.lua` manage project-specific variables automatically.
- **Dynamic Aliases**: Unload or load aliases based on your current directory with `autoaliases.lua`.
- **Git Integration**: First-class support for repository status and branch tracking in your prompt.

## 🧩 Default Plugins

Default plugins are located in `~/.luna/plugins/` when you run `luna init`.

- **`autoenv.lua`**: Seamlessly manages `.env` files exclusively for the current command scope.
- **`autoaliases.lua`**: Context-aware aliases that change based on your project directory.
- **`git.lua`**: Dynamic Git metadata provider for prompt templates.
- **`runtime_version.lua`**: Auto-detects project runtimes (Node, Rust, Go, Python) and exposes versions.
- **`windows-aliases.lua`**: Translation layer for CMD/PowerShell users (`dir` → `ls`, etc.).

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

*Note: Use `luna init --force` to restore defaults or fix a broken configuration.*

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
