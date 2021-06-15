use quijine::{Class, Data, EvalFlags, Object, Result};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::cell::RefCell;

#[test]
fn example_call_js_func_from_rust() -> Result<()> {
    quijine::context(|ctx| {
        ctx.eval(
            "function foo(x, y) { return x + y; }",
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        let global = ctx.global_object()?;
        let foo: Object = global.get("foo")?;
        let result: i32 = ctx.call_into(foo, global, (5, 3))?;
        assert_eq!(8, result, "call foo (JS) from Rust");
        Ok(())
    })
}

#[test]
fn example_call_rust_func_from_js() -> Result<()> {
    quijine::context(|ctx| {
        let global = ctx.global_object()?;
        let foo = ctx.new_function_with(|_ctx, _this: Data, (x, y): (i32, i32)| Ok(x + y), "foo", 2)?;
        global.set("foo", foo)?;
        let result: i32 = ctx.eval_into("foo(5, 3)", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(8, result, "call foo (Rust) from JS");
        Ok(())
    })
}

#[test]
fn example_use_rust_rand_from_js() -> Result<()> {
    let rng = RefCell::new(XorShiftRng::from_seed([0; 16]));
    let sum = quijine::context(|ctx| {
        let rand = ctx.new_function_with(
            move |_ctx, _this: Data, _args: ()| Ok(rng.borrow_mut().gen::<u16>() as i32),
            "rand",
            0,
        )?;
        ctx.global_object()?.set("rand", rand)?;
        let sum: i32 = ctx.eval_into(
            r#"
                let sum = 0;
                for (let i = 0; i < 10; ++i) {
                    sum += rand();
                }
                sum;
            "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        Ok(sum)
    })?;
    assert_eq!(176820, sum, "call PRNG from JS");
    Ok(())
}

#[test]
fn example_use_rust_struct_from_js() -> Result<()> {
    struct Random {
        rng: XorShiftRng,
    }

    impl Random {
        fn gen_u16(&mut self) -> u16 {
            self.rng.gen::<u16>()
        }
    }

    impl Default for Random {
        fn default() -> Self {
            Random {
                rng: XorShiftRng::from_seed([0; 16]),
            }
        }
    }

    impl Class for Random {
        fn name() -> &'static str {
            "Random"
        }

        fn add_methods<'q, M: quijine::ClassMethods<'q, Self>>(methods: &mut M) -> Result<()> {
            methods.add_method("genU16", |_ctx, t, _this: Data, _args: ()| Ok(t.gen_u16() as i32))?;
            Ok(())
        }
    }

    let sum = quijine::context(|ctx| {
        let random = ctx.new_function_with(
            |ctx, _this: Data, _args: ()| Ok(ctx.new_object_with_opaque(Random::default())?),
            "Random",
            0,
        )?;
        random.set_constructor_bit(true)?;
        ctx.global_object()?.set("Random", random)?;
        let sum: i32 = ctx.eval_into(
            r#"
                const rand = new Random();
                let sum = 0;
                for (let i = 0; i < 10; ++i) {
                    sum += rand.genU16();
                }
                sum;
            "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        Ok(sum)
    })?;
    assert_eq!(176820, sum, "use PRNG object from JS");
    Ok(())
}
