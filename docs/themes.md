# Themes

Luna uses Lua for its theme system. Themes are stored in `~/.luna/themes/` as `.lua` files.

## Theme Structure

A theme is a Lua script that defines a `render_prompt` function and optional variables.

### Variables

You can define theme variables using the `theme` global table. These variables can be used in your rich-text strings (e.g., `<color_primary>`).

```lua
theme = {
    color_primary = "#3b82f6",
    color_secondary = "#10b981",
    color_text = "#f3f4f6",
    color_bg = "#1f2937",
    color_error = "#ef4444"
}
```

### Rendering the Prompt

The `render_prompt` function is called every time a new prompt is needed. It receives a `ctx` table with shell information.

```lua
function render_prompt(ctx)
    local user = "<color_primary>" .. ctx.user .. "</color_primary>"
    local host = "<color_secondary>" .. ctx.hostname .. "</color_secondary>"
    local cwd = "<color_text>" .. ctx.cwd_short .. "</color_text>"
    
    return user .. "@" .. host .. " " .. cwd .. " > "
end
```

### Error Rendering

You can also customize how errors are rendered:

```lua
function render_error(ctx, err)
    return "<color_error>Error: " .. err .. "</color_error>"
end
```

## Prompt Context (`ctx`)

The `ctx` table contains the following fields:

- `user`: Current username.
- `hostname`: Machine hostname.
- `cwd`: Current working directory (full path).
- `cwd_short`: Shortened current working directory (e.g., `~/projects/luna`).
- `cwd_home`: CWD with home directory replaced by `~`.
- `last_exit_code`: Exit code of the last command.
- `last_duration_ms`: Execution time of the last command in milliseconds.
- `time_h`, `time_m`, `time_s`: Current time components.
- `env`: Table of environment variables.
- `vars`: Table of plugin-defined variables.

## Rich Text Tag Support

Luna supports a custom rich-text syntax in the strings returned by the theme:

- `<color_name>`: Use a named color or a variable defined in `theme`.
- `<#rrggbb>`: Use a hex color.
- `<bg:color>`: Use a background color.
- `<bold>`, `<italic>`, `<underline>`, `<strike>`: Text styles.
- `<gradient from=#color to=#color>`: Text gradient.
- `<reset>`: Reset all styles.
- `</color>`: Pop last color.
- `</bold>`, `</italic>`, etc.: Close specific tag.

## Example Theme

```lua
theme = {
    color_primary = "#58a6ff",
    color_success = "#3fb950"
}

function render_prompt(ctx)
    local status = ctx.last_exit_code == 0 and "<color_success>➜</color_success>" or "<red>➜</red>"
    return status .. " <color_primary>" .. ctx.cwd_home .. "</color_primary> "
end
```
