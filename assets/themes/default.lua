-- luna Theme: Default (Minimal and clean)

-- Theme colors
primary   = "#89b4fa" -- soft blue
secondary = "#a6e3a1" -- soft green
warn      = "#f9e2af" -- soft yellow
error     = "#f38ba8" -- soft red
text      = "#cdd6f4" -- main text
border    = "#45475a" -- subtle border

-- Optional error formatter
function on_error(msg)
  print("<color_error><bold>error:</bold></color_error> " .. msg)
end

function run()
  -- User @ Host
  print("<color_primary>" .. ctx.user .. "</color_primary>")
  print("<color_text>@</color_text>")
  print("<color_secondary>" .. ctx.hostname .. "</color_secondary> ")

  -- CWD (main focus)
  print("<color_warn>" .. ctx.cwd_home .. "</color_warn>")

  -- Git Info (minimal, no noise)
  local branch = ctx.vars.git_branch
  if branch and branch ~= "" then
    local dirty = ctx.vars.git_dirty == "true"
    local color = dirty and "color_error" or "color_secondary"
    local symbol = dirty and "*" or ""
    print(" <color_text>·</color_text> <" .. color .. ">" .. branch .. symbol .. "</" .. color .. ">")
  end

  -- Runtimes (subtle + compact)
  local runtimes_str = ""
  for i = 1, 5 do
    local name = ctx.vars["ws_runtime_" .. i .. "_name"]
    local ver = ctx.vars["ws_runtime_" .. i .. "_ver"]
    if name and name ~= "" then
      runtimes_str = runtimes_str ..
        " <color_text>·</color_text> " ..
        "<color_secondary>" .. name .. "</color_secondary>" ..
        "<color_text>:</color_text>" ..
        "<color_primary>" .. ver .. "</color_primary>"
    end
  end
  if runtimes_str ~= "" then
    print(runtimes_str)
  end

  -- Error code (only if needed)
  if ctx.last_exit_code ~= 0 then
    print(" <color_text>·</color_text> <color_error>" .. ctx.last_exit_code .. "</color_error>")
  end

  print("\n")

  -- Prompt symbol (clean)
  local symbol = luna.ansi.bold .. "❯ " .. luna.ansi.reset
  return symbol
end