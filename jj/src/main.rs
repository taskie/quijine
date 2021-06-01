use anyhow::Result;
use quijine::{self, Error as QjError, EvalFlags};
use serde_json::Value;
use std::io::{self, BufReader};
use structopt::{clap, StructOpt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JjError {
    #[error("IOError: {0}")]
    Io(io::Error),
    #[error(transparent)]
    Qj(#[from] quijine::Error),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "jj", about = "Genuine JavaScript Object Notation processor")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(short = "n")]
    silent: bool,

    #[structopt(short = "r")]
    raw_output: bool,

    #[structopt(name = "SCRIPT")]
    script: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    quijine::run_with_context(move |ctx| {
        let script = opt.script.as_str();
        // check a syntax error
        ctx.eval(script, "<input>", EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_COMPILE_ONLY)?;
        let stdin = io::stdin();
        let stdin = stdin.lock();
        let buf = BufReader::new(stdin);
        let de = serde_json::Deserializer::from_reader(buf);
        let stream = de.into_iter::<Value>();
        for value in stream {
            let json = value
                .and_then(|v| serde_json::to_string(&v))
                .map_err(QjError::external)?;
            let result = ctx.parse_json(&json, "<input>")?;
            ctx.global_object().set("$_", &result);
            let result = ctx.eval(script, "<input>", EvalFlags::TYPE_GLOBAL)?;
            if !opt.silent {
                if result.is_null() || result.is_undefined() {
                    continue;
                } else if opt.raw_output {
                    if let Ok(s) = result.to_string() {
                        println!("{}", s);
                        continue;
                    }
                }
                let v = ctx.json_stringify(result, ctx.undefined(), ctx.undefined())?;
                println!("{}", v.to_string()?)
            }
        }
        Ok(())
    })?;
    Ok(())
}
