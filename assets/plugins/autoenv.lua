-- autoenv.lua
-- Automatically loads environment variables from .env / .env.local files in the current directory
-- for the duration of the command execution, without polluting the global shell context.

local temporary_envs = {}

local function load_env_file(filepath)
    local f = io.open(filepath, "r")
    if not f then return end

    for line in f:lines() do
        line = line:match("^%s*(.-)%s*$")
        if line ~= "" and not line:match("^#") then
            -- KEY=VALUE or KEY="VALUE"
            local idx = line:find("=")
            if idx then
                local k = line:sub(1, idx - 1):match("^%s*(.-)%s*$")
                local v = line:sub(idx + 1):match("^%s*(.-)%s*$")

                if v:match("^[\"'].-[\"']$") then
                    v = v:sub(2, -2)
                end

                if k and v then
                    local prev = luna.env.get(k)
                    table.insert(temporary_envs, {k, prev})
                    luna.env.set(k, v)
                end
            end
        end
    end
    f:close()
end

luna.hook("on_pre_command", function(cmd)
    temporary_envs = {}
    load_env_file(".env")
    load_env_file(".env.local")
end)

luna.hook("on_post_command", function(cmd, code, ms)
    for i = #temporary_envs, 1, -1 do
        local item = temporary_envs[i]
        if item[2] == nil then
            luna.env.remove(item[1])
        else
            luna.env.set(item[1], item[2])
        end
    end
    temporary_envs = {}
end)
