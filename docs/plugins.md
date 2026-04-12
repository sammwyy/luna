# Plugins

Plugins in Luna allow you to extend the shell's functionality using Lua. They are stored in `~/.luna/plugins/`.

## Plugin API

Plugins have access to the `luna` global table.

### `luna.hook(event, callback)`

Registers a function to be called when a specific event occurs.

**Events:**
- `on_pre_command(line)`: Called before a command is executed.
- `on_post_command(line, exit_code, duration_ms)`: Called after a command finishes.
- `on_directory_change(old_cwd, new_cwd)`: Called when the CWD changes.
- `on_prompt()`: Called just before the prompt is rendered.

### `luna.env`

Access and modify environment variables.

- `luna.env.get(key)`: Get an env var.
- `luna.env.set(key, value)`: Set an env var.
- `luna.env.all()`: Get a table of all env vars.

### `luna.vars`

Access and modify plugin variables. These variables are shared with the theme engine and other plugins.

- `luna.vars.get(key)`: Get a variable.
- `luna.vars.set(key, value)`: Set a variable.
- `luna.vars.all()`: Get all variables.

### `luna.exec(command)`

Executes a system shell command and returns its output.

**Returns:** A table with `code`, `stdout`, and `stderr`.

```lua
local res = luna.exec("git rev-parse --abbrev-ref HEAD")
if res.code == 0 then
    print("Full branch: " .. res.stdout)
end
```

### `luna.exec_stdout(command)`

Short-hand for `luna.exec(command).stdout` (trimmed).

### `luna.text` & `luna.ansi`

Rich text and ANSI utilities.

- `luna.text.render(str)`: Renders rich-text to ANSI.
- `luna.text.strip(str)`: Removes all tags.
- `luna.text.bold(str)`, `luna.text.color(col, str)`, etc.

## Example: Git Plugin

```lua
luna.hook("on_prompt", function()
  local branch = luna.exec_stdout("git branch --show-current 2>/dev/null")
  if branch ~= "" then
    luna.vars.set("git_branch", branch)
  else
    luna.vars.set("git_branch", "")
  end
end)
```

Usage in theme:
```lua
function render_prompt(ctx)
    local git = ctx.vars.git_branch
    if git ~= "" then
        return "(" .. git .. ") > "
    end
    return "> "
end
```
