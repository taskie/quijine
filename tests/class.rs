use std::{cell::RefCell, sync::Arc};

use quijine::{Class, ClassMethods, Context, Data, EvalFlags, Result};

#[derive(Clone, Debug)]
struct S1 {
    name: String,
    pos: (i32, i32),
}

impl S1 {
    fn move_(&mut self, x: i32, y: i32) {
        self.pos.0 += x;
        self.pos.1 += y;
    }
}

impl Class for S1 {
    fn name() -> &'static str {
        "S1"
    }

    fn add_methods<'q, T: ClassMethods<'q, Self>>(methods: &mut T) -> Result<()> {
        methods.add_get_set(
            "name",
            |_ctx, t, _this: Data| Ok(t.name.clone()),
            |_ctx, t, _this: Data, name: Data| {
                t.name = name.to_string()?;
                Ok(name)
            },
        )?;
        methods.add_get("pos", |ctx, t, _this: Data| {
            let obj = ctx.new_object()?;
            obj.set("x", t.pos.0)?;
            obj.set("y", t.pos.1)?;
            Ok(obj)
        })?;
        methods.add_method("move", |ctx, t, _this: Data, (x, y): (i32, i32)| {
            t.move_(x, y);
            Ok(ctx.undefined())
        })?;
        Ok(())
    }
}

#[test]
fn new_object_class() -> Result<()> {
    quijine::context(|ctx| {
        let global = ctx.global_object()?;
        global.set(
            "S1",
            ctx.new_function_with(
                |ctx, _this: Data, (name,): (String,)| {
                    let obj = ctx.new_object_with_opaque(S1 { name, pos: (0, 0) })?;
                    Ok(obj)
                },
                "S1",
                0,
            )?,
        )?;
        let mut s1 = ctx.eval("const s1 = S1('foo'); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
        let name: String = ctx.eval_into("s1.name", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("foo", name);
        let name: String = ctx.eval_into("s1.name = 'bar'; s1.name", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("bar", name);
        let x: i32 = ctx.eval_into("s1.pos.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(0, x);
        let y: i32 = ctx.eval_into("s1.pos.y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(0, y);
        ctx.eval("s1.move(2, 3)", "<input>", EvalFlags::TYPE_GLOBAL)?;
        let x: i32 = ctx.eval_into("s1.pos.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(2, x);
        let y: i32 = ctx.eval_into("s1.pos.y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(3, y);
        let s1 = s1.opaque_mut::<S1>().unwrap();
        assert_eq!("bar", s1.name);
        assert_eq!((2, 3), s1.pos);
        s1.name = "baz".to_owned();
        s1.pos = (4, 5);
        let name: String = ctx.eval_into("s1.name", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("baz", name);
        let x: i32 = ctx.eval_into("s1.pos.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(4, x);
        let y: i32 = ctx.eval_into("s1.pos.y", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!(5, y);
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
                |ctx, _this: Data, (name,): (String,)| {
                    let obj = ctx.new_object_with_opaque(S1 { name, pos: (0, 0) })?;
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
            let result = ctx.eval("const s1 = S1('foo'); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype()?.is_null());
            let name: String = ctx.eval_into("s1.name", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name);
        }
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            register(ctx)?;
            let result = ctx.eval("const s1 = S1('foo'); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype()?.is_null());
            let name: String = ctx.eval_into("s1.name", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name);
        }
        Ok(())
    })
}

#[derive(Clone, Debug)]
struct S2(Arc<RefCell<S1>>);

impl Class for S2 {
    fn name() -> &'static str {
        "S2"
    }

    fn add_methods<'q, T: ClassMethods<'q, Self>>(methods: &mut T) -> Result<()> {
        methods.add_get_set(
            "name",
            |_ctx, t, _this: Data| {
                let t = t.0.borrow();
                Ok(t.name.clone())
            },
            |_ctx, t, _this: Data, name: Data| {
                let mut t = t.0.borrow_mut();
                t.name = name.to_string()?;
                Ok(name)
            },
        )?;
        methods.add_get("pos", |ctx, t, _this: Data| {
            let t = t.0.borrow();
            let obj = ctx.new_object()?;
            obj.set("x", t.pos.0)?;
            obj.set("y", t.pos.1)?;
            Ok(obj)
        })?;
        methods.add_method("move", |ctx, t, _this: Data, (x, y): (i32, i32)| {
            let mut t = t.0.borrow_mut();
            t.move_(x, y);
            Ok(ctx.undefined())
        })?;
        Ok(())
    }
}

#[test]
fn new_object_class_arc() -> Result<()> {
    let s2 = S2(Arc::new(RefCell::new(S1 {
        name: "foo".to_owned(),
        pos: (0, 0),
    })));
    quijine::run(|rt| {
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            let global = ctx.global_object()?;
            let s2 = ctx.new_object_with_opaque(s2.clone())?;
            global.set("s2", s2.clone())?;
            let b: bool = ctx.eval_into("s2.name === 'foo'", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(b);
            ctx.eval("s2.name = 'bar'", "<input>", EvalFlags::TYPE_GLOBAL)?;
        }
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            let global = ctx.global_object()?;
            let s2 = ctx.new_object_with_opaque(s2.clone())?;
            global.set("s2", s2.clone())?;
            let b: bool = ctx.eval_into("s2.name === 'bar'", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(b);
            ctx.eval("s2.move(1, -1)", "<input>", EvalFlags::TYPE_GLOBAL)?;
        }
        Ok(())
    })?;
    assert_eq!(1, Arc::strong_count(&s2.0));
    let s2 = s2.0.borrow();
    assert_eq!("bar", s2.name);
    assert_eq!((1, -1), s2.pos);
    Ok(())
}
