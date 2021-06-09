use anyhow::Result;
use quijine::{self, Context, Data, EvalFlags, ExternalResult, FunctionBytecode, Result as QjResult};
use serde_json::Value;
use serde_quijine::to_qj;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    process::exit,
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
    #[structopt(short = "c", long)]
    compact_output: bool,

    #[structopt(short = "n", long)]
    silent: bool,

    #[structopt(short = "r", long)]
    raw_output: bool,

    #[structopt(name = "SCRIPT")]
    script: String,

    #[structopt(name = "FILE")]
    files: Vec<String>,
}

type Handler<'q> = Box<dyn Fn(Context<'q>, Data, &[Data]) -> QjResult<Data<'q>> + Send + 'static>;

fn define_print<'q>(opt: Arc<Opt>) -> Handler<'q> {
    Box::new(move |ctx: Context<'q>, _this, args| {
        let ret = Ok(ctx.undefined().into());
        let arg = args.get(0);
        let arg = match arg {
            Some(v) => v,
            None => return ret,
        };
        if arg.is_null() || arg.is_undefined() {
            return ret;
        }
        if opt.raw_output {
            if let Ok(s) = arg.to_string() {
                println!("{}", s);
                return ret;
            }
        }
        let space: Data = if opt.compact_output {
            ctx.undefined().into()
        } else {
            ctx.new_string("  ")?.into()
        };
        let v: String = ctx.json_stringify_into(arg.clone(), ctx.undefined(), space)?;
        println!("{}", v);
        ret
    })
}

fn process<'q, R: Read>(
    opt: Arc<Opt>,
    ctx: Context<'q>,
    bytecode: FunctionBytecode<'q>,
    r: R,
    filename: &str,
) -> QjResult<()> {
    let de = serde_json::Deserializer::from_reader(r);
    let stream = de.into_iter::<Value>();
    let global = ctx.global_object()?;
    for (i, value) in stream.enumerate() {
        let value = match value {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ParseError: {}: {}", filename, e);
                exit(4);
            }
        };
        let result: Data = to_qj(ctx, value)?;
        global.set("_", result)?;
        global.set("_F", filename)?;
        global.set("_I", i as i32)?;
        global.set("_P", ctx.new_function(define_print(opt.clone()), "_P", 0)?)?;
        let result = ctx.eval_function(bytecode.clone().into());
        let result = match result {
            Ok(v) => v,
            Err(e) => {
                eprint!("{}", e);
                continue;
            }
        };
        if !opt.silent {
            define_print(opt.clone())(ctx, global.clone().into(), &[result])?;
        }
    }
    Ok(())
}

fn process_stdin<'q>(opt: Arc<Opt>, ctx: Context<'q>, bytecode: FunctionBytecode<'q>) -> QjResult<()> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let buf = BufReader::new(stdin);
    process(opt, ctx, bytecode, buf, "<stdin>")
}

fn process_file<'q>(opt: Arc<Opt>, ctx: Context<'q>, bytecode: FunctionBytecode<'q>, file: &str) -> QjResult<()> {
    let fp = File::open(file).map_err_to_qj()?;
    let buf = BufReader::new(fp);
    process(opt, ctx, bytecode, buf, file)
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Arc::new(Opt::from_args());
    quijine::context(move |ctx| {
        let script = opt.script.as_str();
        // check a syntax error
        let bytecode: FunctionBytecode = ctx
            .eval_into(script, "<input>", EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_COMPILE_ONLY)
            .unwrap_or_else(|e| {
                eprint!("{}", e);
                exit(3)
            });
        // read stdin
        if opt.files.is_empty() {
            process_stdin(opt, ctx, bytecode)?;
        } else {
            for file in opt.files.iter() {
                if file == "-" {
                    process_stdin(opt.clone(), ctx, bytecode.clone())?;
                } else {
                    process_file(opt.clone(), ctx, bytecode.clone(), file)?;
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}
