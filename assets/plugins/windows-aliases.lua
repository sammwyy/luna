-- windows-aliases.lua
-- This plugin registers common Windows commands as aliases for Luna's built-ins.
-- You can disable this by simply removing or renaming this file.

-- Format: luna.alias.set(new_name, original_command)

luna.alias.set("dir", "ls")
luna.alias.set("whereis", "which")
luna.alias.set("cls", "clear")
luna.alias.set("move", "mv")
luna.alias.set("copy", "cp")
luna.alias.set("quit", "exit")
