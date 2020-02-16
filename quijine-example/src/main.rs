use env_logger;

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use std::cell::RefCell;

use quijine::{Qj, QjAny, QjEvalFlags, QjVec};

fn main() {
    env_logger::init();
    // main01();
    main02();
    // main03();
}

fn main01() {
    log::info!("main01");
    quijine::run_with_context(|ctx| {
        ctx.eval(
            "function foo(x, y) { return x + y; }",
            "<input>",
            QjEvalFlags::TYPE_GLOBAL,
        )
        .unwrap();
        let global = ctx.global_object();
        let foo = global.get("foo");
        let args =
            QjVec::<QjAny>::from_qj_ref_slice(&[ctx.new_int32(5).as_ref(), ctx.new_int32(3).as_ref()], ctx).unwrap();
        let result = ctx.call(&foo, &global, &args);
        println!("Result: {}", result.unwrap().to_i32().unwrap());
        println!("END");
    });
    println!("OK");
}

fn main02() {
    log::info!("main02");
    quijine::run_with_context(|ctx| {
        log::debug!("global");
        let global = ctx.global_object();
        log::debug!("console");
        let console = ctx.new_object();
        let console_log = ctx.new_function(
            |ctx, _this, args| {
                let args: Vec<Qj<QjAny>> = args.into();
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        print!(" ");
                    }
                    let s = arg.to_c_string().unwrap();
                    print!("{}", s.to_str().unwrap())
                }
                println!();
                Ok(ctx.undefined().into())
            },
            "log",
            0,
        );
        log::debug!("globalThis = console");
        global.set("console", &console);
        log::debug!("console = log");
        console.set("log", &console_log);
        log::debug!("call console.log");
        let r = ctx
            .eval("console.log('Hello, world!')", "<input>", QjEvalFlags::TYPE_GLOBAL)
            .unwrap();
        println!("END")
    });
    println!("OK");
}

fn main03() {
    log::info!("main03");
    let rng = Box::new(RefCell::new(XorShiftRng::from_seed([0; 16])));
    let sum = quijine::run_with_context(|ctx| {
        let r = ctx.new_function(
            move |ctx, _this, _args| Ok(ctx.new_int32((*rng.as_ref().borrow_mut()).gen()).into()),
            "f",
            0,
        );
        let t = ctx.new_object();
        let args = &QjVec::empty(ctx);
        let mut sum = 0i64;
        for _i in 1..10 {
            let x = ctx.call(&r, &t, &args).unwrap();
            let x = x.to_i32().unwrap();
            eprintln!("{}", x);
            sum += x as i64
        }
        println!("END");
        sum
    });
    eprintln!("sum: {}", sum);
    println!("OK");
}
