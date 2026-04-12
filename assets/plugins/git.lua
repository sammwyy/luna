local function split(str, sep)
    local result = {}
    for match in (str .. sep):gmatch("(.-)" .. sep) do
        table.insert(result, match)
    end
    return result
end

local function join_path(parts)
    local res = ""
    for i, p in ipairs(parts) do
        if p ~= "" then
            res = res .. "/" .. p
        end
    end
    if res == "" then
        return "/"
    end
    return res
end

local function find_git_root(cwd)
    local parts = split(cwd, "/")
    while #parts >= 0 do
        local path = join_path(parts)
        local git_dir = path .. "/.git"

        -- Check if .git exists (cheap way via luna.exec)
        local res = luna.exec("test -d " .. git_dir)
        if res.code == 0 then
            return path
        end

        if #parts == 0 then
            break
        end
        table.remove(parts)
    end
    return nil
end

local function update_git_status()
    local cwd = luna.env.get("PWD") or ""
    local root = find_git_root(cwd)

    if root then
        luna.vars.set("git_repo_directory", root)

        -- Get branch from .git/HEAD
        local head_res = luna.exec("cat " .. root .. "/.git/HEAD")
        if head_res.code == 0 then
            local head = head_res.stdout:gsub("[\r\n]", "")
            if head:match("ref: refs/heads/(.*)") then
                luna.vars.set("git_branch", head:match("ref: refs/heads/(.*)"))
            else
                luna.vars.set("git_branch", head:sub(1, 7)) -- Detached HEAD
            end
        else
            luna.vars.set("git_branch", "unknown")
        end

        -- Dirty check using git porcelain
        local dirty_res = luna.exec("git -C " .. root .. " status --porcelain")
        if dirty_res.code == 0 then
            local is_dirty = dirty_res.stdout ~= ""
            luna.vars.set("git_dirty", tostring(is_dirty))
        else
            luna.vars.set("git_dirty", "false")
        end
    else
        luna.vars.set("git_repo_directory", "")
        luna.vars.set("git_branch", "")
        luna.vars.set("git_dirty", "false")
    end
end

-- Refresh on directory change
luna.hook("on_directory_change", function(old, new)
    update_git_status()
end)

-- Refresh on prompt to ensure status is live
luna.hook("on_prompt", function()
    update_git_status()
end)

-- Run once on load
update_git_status()
