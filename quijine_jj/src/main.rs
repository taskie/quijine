use anyhow::Result;
use colored_json::ColoredFormatter;
use quijine::{
    self, Context, EvalFlags, ExternalResult, FunctionBytecode, Object, Result as QjResult, Value as QjValue,
};
use serde::Serialize;
use serde_json::{
    ser::{CompactFormatter, PrettyFormatter},
    Value,
};
use serde_quijine::to_qj;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
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
    #[structopt(short = "C", long)]
    color_output: bool,

    #[structopt(short = "c", long)]
    compact_output: bool,

    #[structopt(short = "M", long, overrides_with = "color-output")]
    monochrome_output: bool,

    #[structopt(short = "n", long)]
    silent: bool,

    #[structopt(short = "R", long)]
    raw_input: bool,

    #[structopt(short = "r", long)]
    raw_output: bool,

    #[structopt(short = "s", long)]
    slurp: bool,

    #[structopt(long)]
    unbuffered: bool,

    #[structopt(name = "SCRIPT")]
    script: Option<String>,

    #[structopt(name = "FILE")]
    files: Vec<String>,
}

impl Opt {
    fn is_colored(&self) -> bool {
        self.color_output || (!self.monochrome_output && atty::is(atty::Stream::Stdout))
    }
}

type Handler<'q> = Box<dyn Fn(Context<'q>, QjValue<'q>, &[QjValue<'q>]) -> QjResult<QjValue<'q>> + 'static>;

fn print<'q, 'a, 'b>(
    opt: &'a Opt,
    ctx: Context<'q>,
    _this: QjValue<'q>,
    args: &'b [QjValue<'q>],
) -> QjResult<QjValue<'q>> {
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
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut buf = BufWriter::new(stdout);
    let val: Value = serde_quijine::from_qj(arg.clone()).map_err_to_qj()?;
    #[allow(clippy::collapsible_else_if)]
    if opt.is_colored() {
        if opt.compact_output {
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, ColoredFormatter::new(CompactFormatter));
            val.serialize(&mut ser).map_err_to_qj()?;
        } else {
            let mut ser =
                serde_json::Serializer::with_formatter(&mut buf, ColoredFormatter::new(PrettyFormatter::new()));
            val.serialize(&mut ser).map_err_to_qj()?;
        }
    } else {
        if opt.compact_output {
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, CompactFormatter);
            val.serialize(&mut ser).map_err_to_qj()?;
        } else {
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, PrettyFormatter::new());
            val.serialize(&mut ser).map_err_to_qj()?;
        }
    }
    writeln!(buf).map_err_to_qj()?;
    if opt.unbuffered || atty::is(atty::Stream::Stdout) {
        buf.flush().map_err_to_qj()?;
    }
    ret
}

fn define_print<'q>(opt: Arc<Opt>) -> Handler<'q> {
    Box::new(move |ctx: Context<'q>, this, args| print(&opt, ctx, this, args))
}

fn process_one<'q>(
    opt: &Arc<Opt>,
    ctx: Context<'q>,
    bytecode: &FunctionBytecode<'q>,
    filename: &str,
    global: &Object,
    print_obj: &Object,
    i: usize,
    result: QjValue,
) -> QjResult<()> {
    global.set("_", result)?;
    global.set("_F", filename)?;
    global.set("_I", i as i32)?;
    global.set("_P", print_obj.clone())?;
    let result = ctx.eval_function(bytecode.clone().into());
    let result = match result {
        Ok(v) => v,
        Err(e) => {
            eprint!("{}", e);
            return Ok(());
        }
    };
    if !opt.silent {
        print(opt, ctx, global.clone().into(), &[result])?;
    }
    Ok(())
}

fn process<'q, R: BufRead>(
    opt: Arc<Opt>,
    ctx: Context<'q>,
    bytecode: FunctionBytecode<'q>,
    mut r: R,
    filename: &str,
) -> QjResult<()> {
    let global = ctx.global_object()?;
    let print_obj = ctx.new_function(define_print(opt.clone()), "_P", 0)?;
    if opt.raw_input {
        if opt.slurp {
            let mut value = String::new();
            if let Err(e) = r.read_to_string(&mut value) {
                eprintln!("IOError: {}: {}", filename, e);
                exit(4);
            };
            let result: QjValue = ctx.new_string(&value)?.into();
            process_one(&opt, ctx, &bytecode, filename, &global, &print_obj, 0, result)?;
        } else {
            for (i, value) in r.lines().enumerate() {
                let value = match value {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("IOError: {}: {}", filename, e);
                        exit(4);
                    }
                };
                let result: QjValue = ctx.new_string(&value)?.into();
                process_one(&opt, ctx, &bytecode, filename, &global, &print_obj, i, result)?;
            }
        }
    } else {
        let de = serde_json::Deserializer::from_reader(r);
        let stream = de.into_iter::<Value>();
        let values: QjValue = ctx.new_array()?.into();
        let array_push: QjValue = values.get("push")?;
        for (i, value) in stream.enumerate() {
            let value = match value {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ParseError: {}: {}", filename, e);
                    exit(4);
                }
            };
            let result: QjValue = to_qj(ctx, value)?;
            if opt.slurp {
                ctx.call(array_push.clone(), values.clone(), &[result])?;
            } else {
                process_one(&opt, ctx, &bytecode, filename, &global, &print_obj, i, result)?;
            }
        }
        if opt.slurp {
            process_one(&opt, ctx, &bytecode, filename, &global, &print_obj, 0, values)?;
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
    #[cfg(windows)]
    let _enabled = colored_json::enable_ansi_support();
    let opt = Arc::new(Opt::from_args());
    quijine::context(move |ctx| {
        let script = match opt.script {
            Some(ref s) => s.as_str(),
            None => "_",
        };
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
