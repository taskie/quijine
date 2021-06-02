use anyhow::Result;
use quijine::{self, Context, Data, EvalFlags, ExternalResult, Result as QjResult};
use serde_json::Value;
use std::{
    io::{self, BufReader},
    sync::Arc,
};
use structopt::{clap, StructOpt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JjError {
    #[error("IOError: {0}")]
    Io(io::Error),
    #[error(transparent)]
    Qj(#[from] quijine::Error),
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "jj", about = "Genuine JavaScript Object Notation processor")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(short = "c")]
    compact_output: bool,

    #[structopt(short = "n")]
    silent: bool,

    #[structopt(short = "r")]
    raw_output: bool,

    #[structopt(name = "SCRIPT")]
    script: String,
}

type Handler<'q> = Box<dyn Fn(Context<'q>, Data, &[Data]) -> QjResult<Data<'q>> + Send + 'static>;

fn define_print<'q>(opt: Arc<Opt>) -> Handler<'q> {
    Box::new(move |ctx: Context<'q>, _this, args| {
        for arg in args {
            if arg.is_null() || arg.is_undefined() {
                continue;
            } else if opt.raw_output {
                if let Ok(s) = arg.to_string() {
                    println!("{}", s);
                    continue;
                }
            }
            let space: Data = if opt.compact_output {
                ctx.undefined().into()
            } else {
                ctx.new_string("  ")?.into()
            };
            let v = ctx.json_stringify(arg, ctx.undefined(), space)?;
            println!("{}", v.to_string()?)
        }
        Ok(ctx.undefined().into())
    })
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Arc::new(Opt::from_args());
    quijine::run_with_context(move |ctx| {
        let global = ctx.global_object()?;
        let script = opt.script.as_str();
        // check a syntax error
        ctx.eval(script, "<input>", EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_COMPILE_ONLY)?;
        // read stdin
        let stdin = io::stdin();
        let stdin = stdin.lock();
        let buf = BufReader::new(stdin);
        let de = serde_json::Deserializer::from_reader(buf);
        let stream = de.into_iter::<Value>();
        for (i, value) in stream.enumerate() {
            // TODO: direct conversion
            let json = value.and_then(|v| serde_json::to_string(&v)).map_err_to_qj()?;
            let result = ctx.parse_json(&json, "<input>")?;
            global.set("$_", &result)?;
            global.set("$I", ctx.new_int32(i as i32))?;
            global.set("$P", ctx.new_function(define_print(opt.clone()), "$P", 0)?)?;
            let result = ctx.eval(script, "<input>", EvalFlags::TYPE_GLOBAL)?;
            if !opt.silent {
                define_print(opt.clone())(ctx, global.clone().into(), &[result])?;
            }
        }
        Ok(())
    })?;
    Ok(())
}
