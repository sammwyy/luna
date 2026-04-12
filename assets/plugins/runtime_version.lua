local function get_version(cmd)
    local out = luna.exec_stdout(cmd)
    if out and out ~= "" then
        -- Try to extract version (usually X.Y.Z)
        return out:match("(%d+%.[%w%.%-]+)") or out:sub(1, 10)
    end
    return nil
end

local function update_runtimes()
    local files_str = luna.exec_stdout("ls -a1")
    local files = {}
    for f in files_str:gmatch("[^\r\n]+") do
        files[f] = true
    end

    local runtimes = {}

    -- Rust
    if files["Cargo.toml"] then
        table.insert(runtimes, { name = "Rust", ver = get_version("rustc --version") })
    end

    -- Node / JS
    if files["package.json"] then
        table.insert(runtimes, { name = "Node", ver = get_version("node --version") })
        
        if files["package-lock.json"] then
            table.insert(runtimes, { name = "NPM", ver = get_version("npm --version") })
        elseif files["yarn.lock"] then
            table.insert(runtimes, { name = "Yarn", ver = get_version("yarn --version") })
        elseif files["pnpm-lock.yaml"] then
            table.insert(runtimes, { name = "PNPM", ver = get_version("pnpm --version") })
        end
    end

    -- Python
    if files["requirements.txt"] or files["Pipfile"] or files["poetry.lock"] or files["pyproject.toml"] then
        table.insert(runtimes, { name = "Python", ver = get_version("python3 --version") })
    end

    -- Go
    if files["go.mod"] then
        table.insert(runtimes, { name = "Go", ver = get_version("go version") })
    end

    -- Java / Kotlin
    if files["pom.xml"] then
        table.insert(runtimes, { name = "Maven", ver = get_version("mvn --version") })
    elseif files["build.gradle"] or files["build.gradle.kts"] then
        table.insert(runtimes, { name = "Gradle", ver = get_version("gradle --version") })
    end

    -- C++ / CMake
    if files["CMakeLists.txt"] then
        table.insert(runtimes, { name = "CMake", ver = get_version("cmake --version") })
    end

    -- Clear previous variables (set up to 5)
    for i = 1, 5 do
        luna.vars.set("ws_runtime_" .. i .. "_name", "")
        luna.vars.set("ws_runtime_" .. i .. "_ver", "")
    end

    -- Set new variables
    for i, rt in ipairs(runtimes) do
        if i > 5 then break end
        luna.vars.set("ws_runtime_" .. i .. "_name", rt.name)
        luna.vars.set("ws_runtime_" .. i .. "_ver", rt.ver or "unknown")
    end
end

luna.hook("on_directory_change", function()
    update_runtimes()
end)

luna.hook("on_prompt", function()
    -- We can keep it cached or refresh. Detection is fast, but command --version might be slow.
    -- To keep it snappy, maybe ONLY run on dir change?
    -- But let's run on prompt too in case someone created a file.
    update_runtimes()
end)

update_runtimes()
