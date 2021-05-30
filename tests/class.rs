use quijine::{Class, ClassMethods, Context, EvalFlags, Result};

struct S1 {
    name: String,
    pos: (i32, i32),
}

impl Class for S1 {
    fn name() -> &'static str {
        "S1"
    }

    fn add_methods<'q, T: ClassMethods<'q, Self>>(methods: &mut T) {
        methods.add_method("name", |ctx, t, _this, _args| Ok(ctx.new_string(t.name.as_str())));
        methods.add_method("setName", |ctx, t, _this, args| {
            let name = args[0].to_string()?;
            t.name = name;
            Ok(ctx.undefined())
        });
        methods.add_method("pos", |ctx, t, _this, _args| {
            let obj = ctx.new_object();
            obj.set("x", ctx.new_int32(t.pos.0));
            obj.set("y", ctx.new_int32(t.pos.1));
            Ok(obj)
        });
        methods.add_method("move", |ctx, t, _this, args| {
            t.pos.0 += args[0].to_i32()?;
            t.pos.1 += args[1].to_i32()?;
            Ok(ctx.undefined())
        });
    }
}

#[test]
fn new_object_class() -> Result<()> {
    quijine::run_with_context(|ctx| {
        let global = ctx.global_object();
        global.set(
            "S1",
            ctx.new_function(
                |ctx, _this, _args| {
                    let mut obj = ctx.new_object_class::<S1>();
                    let s1 = Box::new(S1 {
                        name: "foo".to_owned(),
                        pos: (3, 4),
                    });
                    obj.set_opaque(s1);
                    Ok(obj)
                },
                "S1",
                0,
            ),
        );
        let result = ctx.eval("const s1 = S1(); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert!(!result.prototype().is_null());
        let name = ctx.eval("s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("foo", name.to_string()?);
        let name = ctx.eval("s1.setName('bar'); s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("bar", name.to_string()?);
        let x = ctx.eval("s1.pos().x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(3, x.to_i32()?);
        let y = ctx.eval("s1.pos().y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(4, y.to_i32()?);
        ctx.eval("s1.move(2, 3)", "<input>", EvalFlags::TYPE_GLOBAL)?;
        let x = ctx.eval("s1.pos().x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(5, x.to_i32()?);
        let y = ctx.eval("s1.pos().y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(7, y.to_i32()?);
        Ok(())
    })
}

#[test]
fn multiple_context() -> Result<()> {
    fn register(ctx: Context) {
        let global = ctx.global_object();
        global.set(
            "S1",
            ctx.new_function(
                |ctx, _this, _args| {
                    let mut obj = ctx.new_object_class::<S1>();
                    let s1 = Box::new(S1 {
                        name: "foo".to_owned(),
                        pos: (3, 4),
                    });
                    obj.set_opaque(s1);
                    Ok(obj)
                },
                "S1",
                0,
            ),
        );
    }
    quijine::run(|rt| {
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            register(ctx);
            let result = ctx.eval("const s1 = S1(); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype().is_null());
            let name = ctx.eval("s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name.to_string()?);
        }
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            register(ctx);
            let result = ctx.eval("const s1 = S1(); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype().is_null());
            let name = ctx.eval("s1.name()", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name.to_string()?);
        }
        Ok(())
    })
}
