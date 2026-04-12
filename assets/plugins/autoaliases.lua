-- autoaliases.lua
-- Automatically loads aliases from a `.aliases` file in the current directory.
-- When leaving the directory, these temporary aliases are cleaned up automatically.

local temporary_aliases = {}

local function apply_aliases(filepath)
    local f = io.open(filepath, "r")
    if not f then return end

    for line in f:lines() do
        line = line:match("^%s*(.-)%s*$")
        if line ~= "" and not line:match("^#") then
            local idx = line:find("=")
            if idx then
                local k = line:sub(1, idx - 1):match("^%s*(.-)%s*$")
                local v = line:sub(idx + 1):match("^%s*(.-)%s*$")

                if v:match("^[\"'].-[\"']$") then
                    v = v:sub(2, -2)
                end

                if k and v then
                    luna.alias.set(k, v)
                    table.insert(temporary_aliases, k)
                end
            end
        end
    end
    f:close()
end

luna.hook("on_directory_change", function(old_cwd, new_cwd)
    -- Clean up previous directory's aliases
    for _, alias_key in ipairs(temporary_aliases) do
        luna.alias.remove(alias_key)
    end
    temporary_aliases = {}

    -- Attempt to load .aliases from the new directory
    local target = new_cwd .. "/.aliases"
    apply_aliases(target)
end)
