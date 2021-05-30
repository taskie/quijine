use quijine::{self, Error, ErrorValue, EvalFlags};
use std::{
    convert::From,
    fmt::Formatter,
    io::{self, BufRead},
    process::exit,
};
use structopt::{clap, StructOpt};

#[derive(Debug)]
pub struct JjError {
    kind: JjErrorKind,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum JjErrorKind {
    Io(io::Error),
    Qj(String),
    Other(String),
}

impl std::error::Error for JjError {}

impl std::fmt::Display for JjError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            JjErrorKind::Io(ref e) => e.fmt(f),
            JjErrorKind::Qj(ref s) => f.write_str(s.as_str()),
            JjErrorKind::Other(ref s) => f.write_str(s.as_str()),
            #[allow(unreachable_patterns)]
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

impl<'q> From<Error> for std::boxed::Box<JjError> {
    fn from(e: Error) -> Self {
        let s: Option<String> = match e.value {
            ErrorValue::String(s) => Some(s),
            ErrorValue::JsError(e) => Some(format!("{}", e)),
            _ => None,
        };
        Box::new(JjError {
            kind: JjErrorKind::Qj(s.unwrap_or_else(|| "internal error".to_owned())),
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
#[structopt(name = "jj", about = "Genuine JavaScript Object Notation processor")]
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

fn check_error<T>(result: Result<T, quijine::Error>) -> T {
    match result {
        Ok(result) => result,
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    }
}

fn main() -> Result<(), Box<JjError>> {
    env_logger::init();
    let opt = Opt::from_args();
    quijine::run_with_context(move |ctx| {
        let script = opt.script.as_str();
        // check a syntax error
        check_error(ctx.eval(script, "<input>", EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_COMPILE_ONLY));
        let stdin = io::stdin();
        for (i, line) in stdin.lock().lines().enumerate() {
            let line = line?;
            let result = check_error(ctx.parse_json(&line, "<input>"));
            ctx.global_object().set("$_", &result);
            ctx.global_object().set("$L", ctx.new_int64(i as i64));
            let result = check_error(ctx.eval(script, "<input>", EvalFlags::TYPE_GLOBAL));
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
