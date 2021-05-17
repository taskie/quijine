use quijine::{QjClass, QjClassMethods, QjContext, QjEvalFlags};

struct S1 {
    name: String,
    pos: (i32, i32),
}

impl QjClass for S1 {
    fn name() -> &'static str {
        "S1"
    }

    fn add_methods<'q, T: QjClassMethods<'q, Self>>(methods: &mut T) {
        methods.add_method("name", |ctx, t, _this, _args| {
            Ok(ctx.new_string(t.name.as_str()).into())
        });
        methods.add_method("setName", |ctx, t, _this, args| {
            let name = args.get(0).to_string().unwrap();
            t.name = name;
            Ok(ctx.undefined().into())
        });
        methods.add_method("pos", |ctx, t, _this, _args| {
            let obj = ctx.new_object();
            obj.set("x", ctx.new_int32(t.pos.0));
            obj.set("y", ctx.new_int32(t.pos.1));
            Ok(obj.into())
        });
        methods.add_method("move", |ctx, t, _this, args| {
            t.pos.0 += args.get(0).to_i32().unwrap();
            t.pos.1 += args.get(1).to_i32().unwrap();
            Ok(ctx.undefined().into())
        });
    }
}

#[test]
fn new_object_class() {
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
                    Ok(obj.into())
                },
                "S1",
                0,
            ),
        );
        let result = ctx
            .eval("const s1 = S1(); s1", "<input>", QjEvalFlags::TYPE_GLOBAL)
            .unwrap();
        assert!(!result.prototype().is_null());
        let name = ctx.eval("s1.name()", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        assert_eq!("foo", name.to_string().unwrap());
        let name = ctx
            .eval("s1.setName('bar'); s1.name()", "<input>", QjEvalFlags::TYPE_GLOBAL)
            .unwrap();
        assert_eq!("bar", name.to_string().unwrap());
        let x = ctx.eval("s1.pos().x", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        assert_eq!(3, x.to_i32().unwrap());
        let y = ctx.eval("s1.pos().y", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        assert_eq!(4, y.to_i32().unwrap());
        ctx.eval("s1.move(2, 3)", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        let x = ctx.eval("s1.pos().x", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        assert_eq!(5, x.to_i32().unwrap());
        let y = ctx.eval("s1.pos().y", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
        assert_eq!(7, y.to_i32().unwrap());
    })
}

#[test]
fn multiple_context() {
    fn register(ctx: QjContext) {
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
                    Ok(obj.into())
                },
                "S1",
                0,
            ),
        );
    }
    quijine::run(|rt| {
        {
            let ctxg = rt.new_context_guard();
            let ctx = ctxg.get();
            register(ctx);
            let result = ctx
                .eval("const s1 = S1(); s1", "<input>", QjEvalFlags::TYPE_GLOBAL)
                .unwrap();
            assert!(!result.prototype().is_null());
            let name = ctx.eval("s1.name()", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
            assert_eq!("foo", name.to_string().unwrap());
        }
        {
            let ctxg = rt.new_context_guard();
            let ctx = ctxg.get();
            register(ctx);
            let result = ctx
                .eval("const s1 = S1(); s1", "<input>", QjEvalFlags::TYPE_GLOBAL)
                .unwrap();
            assert!(!result.prototype().is_null());
            let name = ctx.eval("s1.name()", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
            assert_eq!("foo", name.to_string().unwrap());
        }
    })
}
