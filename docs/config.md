# Configuration

Luna is configured via a `config.toml` file located in `~/.luna/config.toml`.

## Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `theme` | `string` | `"default.lua"` | The name of the active theme file in `~/.luna/themes/`. |
| `bashrc` | `boolean` | `true` | Whether to load and execute `~/.bashrc` on startup. |
| `lunarc` | `boolean` | `true` | Whether to load and execute `~/.lunarc` or `~/.luna/.lunarc` on startup. |
| `newline` | `boolean` | `true` | Whether to print a newline after each command output. |
| `system_env`| `boolean` | `true` | Whether to inherit environment variables from the parent shell. |

## Example `config.toml`

```toml
theme = "default.lua"
bashrc = true
lunarc = true
newline = true
system_env = true

[builtins]
# You can disable specific built-in commands if you prefer system ones.
# ls = false
# grep = false
```

## Built-ins

You can enable or disable built-in commands in the `[builtins]` section. By default, all implemented commands are enabled.

```toml
[builtins]
ls = true
cat = true
# etc...
```
