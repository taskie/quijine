use quijine::{EvalFlags, Result};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::cell::RefCell;

#[test]
fn example_call_js_func_from_rust() -> Result<()> {
    quijine::run_with_context(|ctx| {
        ctx.eval(
            "function foo(x, y) { return x + y; }",
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        let global = ctx.global_object();
        let foo = global.get("foo");
        let args = &[ctx.new_int32(5).into(), ctx.new_int32(3).into()];
        let result = ctx.call(foo, global, args)?;
        assert_eq!(8, result.to_i32()?, "call foo (JS) from Rust");
        Ok(())
    })
}

#[test]
fn example_call_rust_func_from_js() -> Result<()> {
    quijine::run_with_context(|ctx| {
        let global = ctx.global_object();
        let foo = ctx.new_function(
            |ctx, _this, args| {
                let x = args[0].to_f64()?;
                let y = args[1].to_f64()?;
                Ok(ctx.new_float64(x + y))
            },
            "foo",
            2,
        );
        global.set("foo", &foo);
        let result = ctx.eval("foo(5, 3)", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(8, result.to_i32()?, "call foo (Rust) from JS");
        Ok(())
    })
}

#[test]
fn example_use_rust_rand_from_js() -> Result<()> {
    let rng = Box::new(RefCell::new(XorShiftRng::from_seed([0; 16])));
    let sum = quijine::run_with_context(|ctx| {
        let r = ctx.new_function(
            move |ctx, _this, _args| Ok(ctx.new_int32((*rng.as_ref().borrow_mut()).gen())),
            "f",
            0,
        );
        let t = ctx.new_object();
        let args = &[];
        let mut sum = 0i64;
        for _i in 1..10 {
            let x = ctx.call(&r, &t, &args)?;
            let x = x.to_i32()?;
            sum += x as i64
        }
        Ok(sum)
    })?;
    assert_eq!(3967332714, sum, "call PRNG from JS");
    Ok(())
}
