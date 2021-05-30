use quijine::{QjAny, QjEvalFlags};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::cell::RefCell;

#[test]
fn example_call_js_func_from_rust() {
    quijine::run_with_context(|ctx| {
        ctx.eval(
            "function foo(x, y) { return x + y; }",
            "<input>",
            QjEvalFlags::TYPE_GLOBAL,
        )
        .unwrap();
        let global = ctx.global_object();
        let foo = global.get("foo");
        let args = &[ctx.new_int32(5).into(), ctx.new_int32(3).into()];
        let result = ctx.call(&foo, &global, &args).unwrap();
        assert_eq!(8, result.to_i32().unwrap(), "call foo (JS) from Rust");
    });
}

#[test]
fn example_call_rust_func_from_js() {
    quijine::run_with_context(|ctx| {
        let global = ctx.global_object();
        let foo = ctx.new_function(
            |ctx, _this, args| {
                let args: Vec<QjAny> = args.into();
                let x = args[0].to_f64().unwrap();
                let y = args[1].to_f64().unwrap();
                Ok(ctx.new_float64(x + y).into())
            },
            "foo",
            2,
        );
        global.set("foo", &foo);
        let result = ctx.eval("foo(5, 3)", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        assert_eq!(8, result.to_i32().unwrap(), "call foo (Rust) from JS");
    });
}

#[test]
fn example_use_rust_rand_from_js() {
    let rng = Box::new(RefCell::new(XorShiftRng::from_seed([0; 16])));
    let sum = quijine::run_with_context(|ctx| {
        let r = ctx.new_function(
            move |ctx, _this, _args| Ok(ctx.new_int32((*rng.as_ref().borrow_mut()).gen()).into()),
            "f",
            0,
        );
        let t = ctx.new_object();
        let args = &[];
        let mut sum = 0i64;
        for _i in 1..10 {
            let x = ctx.call(&r, &t, &args).unwrap();
            let x = x.to_i32().unwrap();
            sum += x as i64
        }
        sum
    });
    assert_eq!(3967332714, sum, "call PRNG from JS");
}
