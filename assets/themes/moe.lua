-- luna Theme: moe

-- Theme colors
primary   = "#f5c2e7" -- pink
secondary = "#cba6f7" -- lavender
warn      = "#f9e2af" -- soft yellow
error     = "#f38ba8" -- soft red/pink
text      = "#f2cdcd" -- light pastel text
border    = "#6c7086" -- muted

-- Optional error formatter
function on_error(msg)
  print("<color_error><bold>oops:</bold></color_error> " .. msg)
end

function run()
  -- User @ Host (cute)
  print("<color_primary>" .. ctx.user .. "</color_primary>")
  print("<color_text>@</color_text>")
  print("<color_secondary>" .. ctx.hostname .. "</color_secondary> ")

  -- CWD (soft highlight)
  print("<color_warn>" .. ctx.cwd_home .. "</color_warn>")

  -- Git Info (kawaii style)
  local branch = ctx.vars.git_branch
  if branch and branch ~= "" then
    local dirty = ctx.vars.git_dirty == "true"
    local color = dirty and "color_error" or "color_secondary"
    local icon = dirty and "!!" or "ok"
    print(" <color_text>~</color_text> <" .. color .. ">" .. branch .. " " .. icon .. "</" .. color .. ">")
  end

  -- Runtimes (cute + soft)
  local runtimes_str = ""
  for i = 1, 5 do
    local name = ctx.vars["ws_runtime_" .. i .. "_name"]
    local ver = ctx.vars["ws_runtime_" .. i .. "_ver"]
    if name and name ~= "" then
      runtimes_str = runtimes_str ..
        " <color_text>~</color_text> " ..
        "<color_primary>" .. name .. "</color_primary>" ..
        "<color_text>@</color_text>" ..
        "<color_secondary>" .. ver .. "</color_secondary>"
    end
  end
  if runtimes_str ~= "" then
    print(runtimes_str)
  end

  -- Error code (cute but visible)
  if ctx.last_exit_code ~= 0 then
    print(" <color_text>~</color_text> <color_error>!" .. ctx.last_exit_code .. "!</color_error>")
  end

  print("\n")

  -- Prompt symbol
  local symbol = luna.ansi.bold .. "♡ ❯ " .. luna.ansi.reset
  return symbol
end