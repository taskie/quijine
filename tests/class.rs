use quijine::{Class, ClassMethods, Context, Data, EvalFlags, Result};

struct S1 {
    name: String,
    pos: (i32, i32),
}

impl Class for S1 {
    fn name() -> &'static str {
        "S1"
    }

    fn add_methods<'q, T: ClassMethods<'q, Self>>(methods: &mut T) -> Result<()> {
        methods.add_method("name", |_ctx, t, _this: Data, _args: ()| Ok(t.name.clone()))?;
        methods.add_method("setName", |ctx, t, _this: Data, (name,): (String,)| {
            t.name = name;
            Ok(ctx.undefined())
        })?;
        methods.add_method("pos", |ctx, t, _this: Data, _args: ()| {
            let obj = ctx.new_object()?;
            obj.set("x", t.pos.0)?;
            obj.set("y", t.pos.1)?;
            Ok(obj)
        })?;
        methods.add_method("move", |ctx, t, _this: Data, args: (i32, i32)| {
            t.pos.0 += args.0;
            t.pos.1 += args.1;
            Ok(ctx.undefined())
        })?;
        Ok(())
    }
}

#[test]
fn new_object_class() -> Result<()> {
    quijine::run_with_context(|ctx| {
        let global = ctx.global_object()?;
        global.set(
            "S1",
            ctx.new_function_with(
                |ctx, _this: Data, _args: ()| {
                    let obj = ctx.new_object_with_opaque(Box::new(S1 {
                        name: "foo".to_owned(),
                        pos: (3, 4),
                    }))?;
                    Ok(obj)
                },
                "S1",
                0,
            )?,
        )?;
        let result = ctx.eval("const s1 = S1(); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert!(!result.prototype()?.is_null());
        let name: String = ctx.eval_into("s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("foo", name);
        let name: String = ctx.eval_into("s1.setName('bar'); s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("bar", name);
        let x: i32 = ctx.eval_into("s1.pos().x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(3, x);
        let y: i32 = ctx.eval_into("s1.pos().y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(4, y);
        ctx.eval("s1.move(2, 3)", "<input>", EvalFlags::TYPE_GLOBAL)?;
        let x: i32 = ctx.eval_into("s1.pos().x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(5, x);
        let y: i32 = ctx.eval_into("s1.pos().y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(7, y);
        Ok(())
    })
}

#[test]
fn multiple_context() -> Result<()> {
    fn register(ctx: Context) -> Result<()> {
        let global = ctx.global_object()?;
        global.set(
            "S1",
            ctx.new_function_with(
                |ctx, _this: Data, _args: ()| {
                    let obj = ctx.new_object_with_opaque(Box::new(S1 {
                        name: "foo".to_owned(),
                        pos: (3, 4),
                    }))?;
                    Ok(obj)
                },
                "S1",
                0,
            )?,
        )?;
        Ok(())
    }
    quijine::run(|rt| {
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            register(ctx)?;
            let result = ctx.eval("const s1 = S1(); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype()?.is_null());
            let name: String = ctx.eval_into("s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name);
        }
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            register(ctx)?;
            let result = ctx.eval("const s1 = S1(); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype()?.is_null());
            let name: String = ctx.eval_into("s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name);
        }
        Ok(())
    })
}
