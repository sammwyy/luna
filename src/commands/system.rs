use crate::shell::config::LunaConfig;
use crate::shell::state::LunaState;
use shellframe::{Context, Output};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum FlagType {
    Bool,
    String,
    Integer,
    Enum(Vec<&'static str>),
}

#[derive(Clone, Debug)]
pub enum FlagValue {
    Bool(bool),
    String(String),
    Integer(i64),
    Enum(String),
}

#[derive(Clone)]
pub struct FlagDef {
    pub name: &'static str,
    pub short: Option<char>,
    pub desc: &'static str,
    pub flag_type: FlagType,
    pub required: bool,
}

#[derive(Debug)]
pub struct ParsedArgs {
    pub flags: HashMap<String, FlagValue>,
    pub positionals: Vec<String>,
}

impl ParsedArgs {
    pub fn get_bool(&self, name: &str) -> bool {
        match self.flags.get(name) {
            Some(FlagValue::Bool(b)) => *b,
            _ => false,
        }
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        match self.flags.get(name) {
            Some(FlagValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.flags.get(name) {
            Some(FlagValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }
}

pub trait BuiltinCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn flags(&self) -> Vec<FlagDef> {
        Vec::new()
    }
    fn run(
        &self,
        ctx: &mut Context<LunaState>,
        args: ParsedArgs,
        stdin: &str,
    ) -> anyhow::Result<Output>;

    fn dry_run(&self, _config: &LunaConfig, _args: &ParsedArgs) -> Result<(), String> {
        Ok(())
    }

    fn parse_args(&self, raw_args: &[String]) -> Result<ParsedArgs, String> {
        let mut parsed = ParsedArgs {
            flags: HashMap::new(),
            positionals: Vec::new(),
        };

        let flags = self.flags();

        for f in &flags {
            if f.flag_type == FlagType::Bool {
                parsed
                    .flags
                    .insert(f.name.to_string(), FlagValue::Bool(false));
            }
        }

        let mut i = 0;
        while i < raw_args.len() {
            let arg = &raw_args[i];

            if arg.starts_with("--") {
                let name = &arg[2..];
                if let Some(def) = flags.iter().find(|f| f.name == name) {
                    i = self.parse_flag_value(def, &mut parsed, raw_args, i)?;
                } else {
                    return Err(format!("Unknown flag: --{}", name));
                }
            } else if arg.starts_with('-') && arg.len() > 1 {
                let short = arg.chars().nth(1).unwrap();
                if let Some(def) = flags.iter().find(|f| f.short == Some(short)) {
                    i = self.parse_flag_value(def, &mut parsed, raw_args, i)?;
                } else {
                    return Err(format!("Unknown short flag: -{}", short));
                }
            } else {
                parsed.positionals.push(arg.clone());
            }
            i += 1;
        }

        Ok(parsed)
    }

    fn parse_flag_value(
        &self,
        def: &FlagDef,
        parsed: &mut ParsedArgs,
        raw_args: &[String],
        mut i: usize,
    ) -> Result<usize, String> {
        match def.flag_type {
            FlagType::Bool => {
                parsed
                    .flags
                    .insert(def.name.to_string(), FlagValue::Bool(true));
            }
            _ => {
                if i + 1 >= raw_args.len() {
                    return Err(format!("Missing value for flag: {}", def.name));
                }
                let val_str = &raw_args[i + 1];
                i += 1;

                match &def.flag_type {
                    FlagType::String => {
                        parsed
                            .flags
                            .insert(def.name.to_string(), FlagValue::String(val_str.clone()));
                    }
                    FlagType::Integer => {
                        let val: i64 = val_str.parse().map_err(|_| {
                            format!("Invalid integer for flag {}: {}", def.name, val_str)
                        })?;
                        parsed
                            .flags
                            .insert(def.name.to_string(), FlagValue::Integer(val));
                    }
                    FlagType::Enum(ref allowed) => {
                        if !allowed.contains(&val_str.as_str()) {
                            return Err(format!(
                                "Invalid value for flag {}. Allowed values: {:?}",
                                def.name, allowed
                            ));
                        }
                        parsed
                            .flags
                            .insert(def.name.to_string(), FlagValue::Enum(val_str.clone()));
                    }
                    _ => {}
                }
            }
        }
        Ok(i)
    }
}

pub struct Registry {
    pub commands: HashMap<String, std::sync::Arc<dyn BuiltinCommand>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, cmd: std::sync::Arc<dyn BuiltinCommand>) {
        self.commands.insert(cmd.name().to_string(), cmd);
    }

    pub fn register_with_name(&mut self, name: &str, cmd: std::sync::Arc<dyn BuiltinCommand>) {
        self.commands.insert(name.to_string(), cmd);
    }
}
