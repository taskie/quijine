use quijine::{self, QjAnyTag, QjEvalFlags, QjVec};
use std::{
    convert::From,
    fmt::Formatter,
    io::{self, BufRead},
};
use structopt::{clap, StructOpt};

#[derive(Debug)]
pub struct JjError {
    kind: JjErrorKind,
}

#[derive(Debug)]
pub enum JjErrorKind {
    Io(io::Error),
    Other(String),
    #[doc(hidden)]
    __Nonexhaustive,
}

impl std::error::Error for JjError {}

impl std::fmt::Display for JjError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            JjErrorKind::Io(ref e) => e.fmt(f),
            _ => f.write_str("internal error"),
        }
    }
}

impl From<std::io::Error> for std::boxed::Box<JjError> {
    fn from(e: io::Error) -> Self {
        Box::new(JjError {
            kind: JjErrorKind::Io(e),
        })
    }
}

impl From<Box<dyn std::error::Error>> for std::boxed::Box<JjError> {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Box::new(JjError {
            kind: JjErrorKind::Other(e.to_string()),
        })
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "jj", about = "JavaScript JSON processor")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(short = "n")]
    silent: bool,

    #[structopt(short = "r")]
    raw_output: bool,

    // #[structopt(short, long, default_value = "jsonl")]
    // from: String,

    // #[structopt(short, long, default_value = "jsonl")]
    // to: String,
    #[structopt(name = "SCRIPT")]
    script: String,
}

fn main() -> Result<(), Box<JjError>> {
    env_logger::init();
    let opt = Opt::from_args();
    quijine::run_with_context(move |ctx| {
        let json = ctx.global_object().get("JSON");
        let json_parse = json.get("parse");
        let json_stringify = json.get("stringify");
        let stdin = io::stdin();
        for (i, line) in stdin.lock().lines().enumerate() {
            let line = line?;
            let args = QjVec::<QjAnyTag>::from_qj_ref_slice(&[ctx.new_string(&line).as_ref()], ctx).unwrap();
            let result = match ctx.call(&json_parse, ctx.undefined(), &args) {
                Ok(v) => v,
                Err(e) => {
                    return Err(Box::new(JjError {
                        kind: JjErrorKind::Other(format!("Input error: {}", e.to_string())),
                    }));
                }
            };
            ctx.global_object().set("$_", &result);
            ctx.global_object().set("$L", ctx.new_int64(i as i64));
            let result = match ctx.eval(&opt.script.as_str(), "<input>", QjEvalFlags::TYPE_GLOBAL) {
                Ok(v) => v,
                Err(e) => {
                    return Err(Box::new(JjError {
                        kind: JjErrorKind::Other(format!("Script error: {}", e.to_string())),
                    }));
                }
            };
            if !opt.silent {
                if result.is_null() || result.is_undefined() {
                    continue;
                } else if opt.raw_output {
                    if let Some(s) = result.to_string() {
                        println!("{}", s);
                        continue;
                    }
                }
                let args = QjVec::<QjAnyTag>::from_qj_ref_slice(&[result.as_ref()], ctx).unwrap();
                match ctx.call(&json_stringify, ctx.undefined(), &args) {
                    Ok(v) => println!("{}", v.to_string().unwrap()),
                    Err(e) => {
                        return Err(Box::new(JjError {
                            kind: JjErrorKind::Other(format!("Script error: {}", e.to_string())),
                        }));
                    }
                };
            }
        }
        Ok(())
    })
}
