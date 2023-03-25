use anyhow::Result;
use clap::Parser;
use colored_json::ColoredFormatter;
use quijine::{self, Context, EvalFlags, ExternalResult, FunctionBytecode, Result as QjResult, Value as QjValue};
use serde::Serialize;
use serde_json::{
    ser::{CompactFormatter, PrettyFormatter},
    Value,
};
use serde_quijine::to_qj;
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    process::exit,
    sync::Arc,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JjError {
    #[error("IOError: {0}")]
    Io(io::Error),
    #[error(transparent)]
    Qj(#[from] quijine::Error),
}

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Opt {
    /// Colorize JSON output
    #[clap(short = 'C', long)]
    color_output: bool,

    /// Produce compact output instead of pretty-printed output
    #[clap(short = 'c', long)]
    compact_output: bool,

    /// Use JavaScript from a file instead of SCRIPT argument
    #[clap(short = 'f', long)]
    from_file: Option<String>,

    /// Iterate over the result and print each item as an individual JSON
    #[clap(short = 'i', long)]
    iter: bool,

    /// Don't colorize JSON output
    #[clap(short = 'M', long, overrides_with = "color_output")]
    monochrome_output: bool,

    /// Don't print results automatically
    #[clap(short = 'n', long)]
    silent: bool,

    /// Treat input as raw strings instead of JSON
    #[clap(short = 'R', long)]
    raw_input: bool,

    /// Output results as raw strings instead of JSON
    #[clap(short = 'r', long)]
    raw_output: bool,

    /// Read (slurp) all inputs into an array
    #[clap(short = 's', long)]
    slurp: bool,

    /// Flush the output buffers more often
    #[clap(long)]
    unbuffered: bool,

    /// JavaScript code to process JSON
    #[clap(name = "SCRIPT")]
    script: Option<String>,

    /// JSON files
    #[clap(name = "FILE")]
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

struct ProcessOneArgs<'q, 'a> {
    opt: &'a Arc<Opt>,
    ctx: Context<'q>,
    bytecode: &'a FunctionBytecode<'q>,
    filename: &'a str,
    global: &'a QjValue<'q>,
    print_obj: &'a QjValue<'q>,
    i: usize,
    result: QjValue<'q>,
}

fn process_one(a: &ProcessOneArgs) -> QjResult<()> {
    a.global.set("_", a.result.clone())?;
    a.global.set("_F", a.filename)?;
    a.global.set("_I", a.i as i32)?;
    a.global.set("_P", a.print_obj.clone())?;
    let result = a.ctx.eval_function(a.bytecode.clone().into());
    let result = match result {
        Ok(v) => v,
        Err(e) => {
            eprint!("{}", e);
            return Ok(());
        }
    };
    if !a.opt.silent {
        if a.opt.iter {
            for v in result.iterator()? {
                print(a.opt, a.ctx, a.global.clone(), &[v?])?;
            }
        } else {
            print(a.opt, a.ctx, a.global.clone().into(), &[result])?;
        }
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
    let mut args = ProcessOneArgs {
        opt: &opt,
        ctx,
        bytecode: &bytecode,
        filename,
        global: &global,
        print_obj: &print_obj,
        i: 0,
        result: ctx.undefined().into(),
    };
    if opt.raw_input {
        if opt.slurp {
            let mut value = String::new();
            if let Err(e) = r.read_to_string(&mut value) {
                eprintln!("IOError: {}: {}", filename, e);
                exit(4);
            };
            args.i = 0;
            args.result = ctx.new_string(&value)?.into();
            process_one(&args)?;
        } else {
            for (i, value) in r.lines().enumerate() {
                let value = match value {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("IOError: {}: {}", filename, e);
                        exit(4);
                    }
                };
                args.i = i;
                args.result = ctx.new_string(&value)?.into();
                process_one(&args)?;
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
            let result = to_qj(ctx, value)?;
            if opt.slurp {
                ctx.call(array_push.clone(), values.clone(), &[result])?;
            } else {
                args.i = i;
                args.result = result;
                process_one(&args)?;
            }
        }
        if opt.slurp {
            args.i = 0;
            args.result = values;
            process_one(&args)?;
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
    let opt = Arc::new(Opt::parse());
    let script = match opt.from_file {
        // jj -f FILTERFILE FILE [FILES...]
        Some(ref s) => fs::read_to_string(s)?,
        None => match opt.script {
            // jj FILTER [FILES...]
            Some(ref s) => s.clone(),
            // jj
            None => "_".to_owned(),
        },
    };
    let files = match opt.from_file {
        Some(_) => match opt.script.as_ref() {
            // jj -f FILTERFILE FILE [FILES...]
            Some(s) => [vec![s.clone()], opt.files.clone()].concat(),
            // jj -f FILTERFILE
            None => vec![],
        },
        // jj FILTER [FILES...]
        None => opt.files.clone(),
    };
    quijine::context(move |ctx| {
        // check a syntax error
        let bytecode: FunctionBytecode = ctx
            .eval_into(
                &script,
                "<input>",
                EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_COMPILE_ONLY,
            )
            .unwrap_or_else(|e| {
                eprint!("{}", e);
                exit(3)
            });
        // read stdin
        if files.is_empty() {
            process_stdin(opt, ctx, bytecode)?;
        } else {
            for file in files.iter() {
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
