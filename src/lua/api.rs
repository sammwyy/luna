/// Shared API primitives for both theme and plugin Lua VMs.
///
/// Functions exposed here do NOT rely on mutable shell state at registration
/// time — they receive data by value via closures or userdata.
use crate::{renderer::markup, shell::context::ShellContext};
use mlua::prelude::*;

/// Register the `luna.text.*` rich-text helpers into a table.
/// Safe enough to expose in sandboxed themes too.
pub fn register_text_api(lua: &Lua, luna: &LuaTable) -> LuaResult<()> {
    let text = lua.create_table()?;

    // luna.text.render(str) -> str  (converts <red>…</red> to ANSI)
    let render = lua.create_function(|_, s: String| Ok(markup::render_ansi(&s)))?;
    text.set("render", render)?;

    // luna.text.strip(str) -> str  (remove all markup, return plain text)
    let strip = lua.create_function(|_, s: String| Ok(markup::strip_ansi(&s)))?;
    text.set("strip", strip)?;

    // luna.text.bold(str) -> str
    let bold = lua
        .create_function(|_, s: String| Ok(markup::render_ansi(&format!("<bold>{}</bold>", s))))?;
    text.set("bold", bold)?;

    // luna.text.italic(str) -> str
    let italic = lua.create_function(|_, s: String| {
        Ok(markup::render_ansi(&format!("<italic>{}</italic>", s)))
    })?;
    text.set("italic", italic)?;

    // luna.text.color(color, str) -> str  — e.g. luna.text.color("red", "hello")
    let color = lua.create_function(|_, (col, s): (String, String)| {
        Ok(markup::render_ansi(&format!("<{}>{}</{}>", col, s, col)))
    })?;
    text.set("color", color)?;

    // luna.text.gradient(from, to, str) -> str
    let gradient = lua.create_function(|_, (from, to, s): (String, String, String)| {
        Ok(markup::render_ansi(&format!(
            "<gradient from={} to={}>{}</gradient>",
            from, to, s
        )))
    })?;
    text.set("gradient", gradient)?;

    luna.set("text", text)?;
    Ok(())
}

/// Register `luna.ansi.*` raw ANSI constants. Safe for themes.
pub fn register_ansi_api(lua: &Lua, luna: &LuaTable) -> LuaResult<()> {
    let ansi = lua.create_table()?;
    ansi.set("reset", "\x1b[0m")?;
    ansi.set("bold", "\x1b[1m")?;
    ansi.set("italic", "\x1b[3m")?;
    ansi.set("underline", "\x1b[4m")?;
    ansi.set("strike", "\x1b[9m")?;
    luna.set("ansi", ansi)?;
    Ok(())
}

/// Register plugin-specific `luna.exec` and `luna.exec_stdout` utils.
pub fn register_plugin_exec_api(
    lua: &Lua,
    luna: &LuaTable,
    env_vars: std::sync::Arc<std::sync::Mutex<indexmap::IndexMap<String, String>>>,
) -> LuaResult<()> {
    let env_snap = env_vars.clone();
    let exec_fn = lua.create_function(move |lua_ctx, cmd: String| {
        let env_map = env_snap.lock().unwrap().clone();
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .envs(&env_map)
            .output();

        let t = lua_ctx.create_table()?;
        match result {
            Ok(out) => {
                t.set("code", out.status.code().unwrap_or(-1))?;
                t.set("stdout", String::from_utf8_lossy(&out.stdout).to_string())?;
                t.set("stderr", String::from_utf8_lossy(&out.stderr).to_string())?;
            }
            Err(e) => {
                t.set("code", -1)?;
                t.set("stdout", "")?;
                t.set("stderr", e.to_string())?;
            }
        }
        Ok(t)
    })?;
    luna.set("exec", exec_fn)?;

    let env_snap2 = env_vars.clone();
    let exec_stdout = lua.create_function(move |_, cmd: String| {
        let env_map = env_snap2.lock().unwrap().clone();
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .envs(&env_map)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        Ok(out)
    })?;
    luna.set("exec_stdout", exec_stdout)?;

    Ok(())
}

/// Register plugin-specific `luna.env` table.
pub fn register_plugin_env_api(
    lua: &Lua,
    luna: &LuaTable,
    env_vars: std::sync::Arc<std::sync::Mutex<indexmap::IndexMap<String, String>>>,
) -> LuaResult<()> {
    let env_tbl = lua.create_table()?;

    let ev1 = env_vars.clone();
    let env_get = lua.create_function(move |_, key: String| {
        let map = ev1.lock().unwrap();
        Ok(map.get(&key).cloned())
    })?;
    env_tbl.set("get", env_get)?;

    let ev2 = env_vars.clone();
    let env_set = lua.create_function(move |_, (key, val): (String, String)| {
        ev2.lock().unwrap().insert(key, val);
        Ok(())
    })?;
    env_tbl.set("set", env_set)?;

    let ev3 = env_vars.clone();
    let env_all = lua.create_function(move |lua_ctx, ()| {
        let map = ev3.lock().unwrap();
        let t = lua_ctx.create_table()?;
        for (k, v) in map.iter() {
            t.set(k.as_str(), v.as_str())?;
        }
        Ok(t)
    })?;
    env_tbl.set("all", env_all)?;

    let ev4 = env_vars.clone();
    let env_remove = lua.create_function(move |_, key: String| {
        ev4.lock().unwrap().shift_remove(&key);
        Ok(())
    })?;
    env_tbl.set("remove", env_remove)?;

    luna.set("env", env_tbl)?;
    Ok(())
}

/// Register plugin-specific `luna.vars` table.
pub fn register_plugin_vars_api(
    lua: &Lua,
    luna: &LuaTable,
    plugin_vars: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
) -> LuaResult<()> {
    let vars_tbl = lua.create_table()?;

    let pv1 = plugin_vars.clone();
    let vars_get =
        lua.create_function(move |_, key: String| Ok(pv1.lock().unwrap().get(&key).cloned()))?;
    vars_tbl.set("get", vars_get)?;

    let pv2 = plugin_vars.clone();
    let vars_set = lua.create_function(move |_, (key, val): (String, String)| {
        pv2.lock().unwrap().insert(key, val);
        Ok(())
    })?;
    vars_tbl.set("set", vars_set)?;

    let pv3 = plugin_vars.clone();
    let vars_all = lua.create_function(move |lua_ctx, ()| {
        let map = pv3.lock().unwrap();
        let t = lua_ctx.create_table()?;
        for (k, v) in map.iter() {
            t.set(k.as_str(), v.as_str())?;
        }
        Ok(t)
    })?;
    vars_tbl.set("all", vars_all)?;

    luna.set("vars", vars_tbl)?;
    Ok(())
}

/// Register plugin-specific `luna.alias` table.
pub fn register_plugin_alias_api(
    lua: &Lua,
    luna: &LuaTable,
    aliases: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
) -> LuaResult<()> {
    let alias_tbl = lua.create_table()?;

    let a1 = aliases.clone();
    let alias_set = lua.create_function(move |_, (orig, alias): (String, String)| {
        a1.lock().unwrap().insert(orig, alias);
        Ok(())
    })?;
    alias_tbl.set("set", alias_set)?;

    let a2 = aliases.clone();
    let alias_remove = lua.create_function(move |_, orig: String| {
        a2.lock().unwrap().remove(&orig);
        Ok(())
    })?;
    alias_tbl.set("remove", alias_remove)?;

    luna.set("alias", alias_tbl)?;
    Ok(())
}

/// Register global functions like `print` and `println` into the Lua state.
/// For themes, these will buffer output or write to a target.
pub fn register_globals(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    // print(str)
    let print = lua.create_function(|_, s: String| {
        print!("{}", markup::render_ansi(&s));
        Ok(())
    })?;
    globals.set("print", print)?;

    // println(str)
    let println = lua.create_function(|_, s: String| {
        println!("{}", markup::render_ansi(&s));
        Ok(())
    })?;
    globals.set("println", println)?;

    Ok(())
}

/// Convert a Rust ShellContext into a Lua table.
pub fn convert_context(lua: &Lua, sc: &ShellContext) -> LuaResult<LuaTable> {
    let ctx = lua.create_table()?;

    ctx.set("last_exit_code", sc.last_exit_code)?;
    ctx.set("last_duration_ms", sc.last_duration_ms as i64)?;
    ctx.set("cwd", sc.cwd.as_str())?;
    ctx.set("cwd_home", sc.cwd_home.as_str())?;
    ctx.set("cwd_short", sc.cwd_short.as_str())?;
    ctx.set("user", sc.user.as_str())?;
    ctx.set("hostname", sc.hostname.as_str())?;
    ctx.set("shell", sc.shell.as_str())?;
    ctx.set("pid", sc.pid)?;

    ctx.set("time_h", sc.time_h.as_str())?;
    ctx.set("time_m", sc.time_m.as_str())?;
    ctx.set("time_s", sc.time_s.as_str())?;
    ctx.set("date_y", sc.date_y)?;
    ctx.set("date_mo", sc.date_mo)?;
    ctx.set("date_d", sc.date_d)?;
    ctx.set("date_iso", sc.date_iso.as_str())?;
    ctx.set("datetime", sc.datetime.as_str())?;

    // Env table
    let env_tbl = lua.create_table()?;
    for (k, v) in &sc.env {
        env_tbl.set(k.as_str(), v.as_str())?;
    }
    ctx.set("env", env_tbl)?;

    // Vars table
    let vars_tbl = lua.create_table()?;
    for (k, v) in &sc.vars {
        vars_tbl.set(k.as_str(), v.as_str())?;
    }
    ctx.set("vars", vars_tbl)?;

    Ok(ctx)
}
