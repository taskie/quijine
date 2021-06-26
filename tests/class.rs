use std::{cell::RefCell, sync::Arc};

use quijine::{Class, ClassProperties, Context, EvalFlags, Result, Value};

#[derive(Clone, Debug, Default)]
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

    fn constructor<'q>(&mut self, _ctx: Context<'q>, _this: Value, args: &[Value]) -> Result<()> {
        self.name = args[0].to_string()?;
        Ok(())
    }

    fn constructor_length<'q>() -> i32 {
        1
    }

    fn define_properties<'q, P: ClassProperties<'q, Self>>(properties: &mut P) -> Result<()> {
        properties.define_get_set_mut(
            "name",
            |v, _ctx, _this: Value| Ok(v.name.clone()),
            |v, _ctx, _this: Value, name: Value| {
                v.name = name.to_string()?;
                Ok(name)
            },
        )?;
        properties.define_get("pos", |v, ctx, _this: Value| {
            let obj = ctx.new_object()?;
            obj.set("x", v.pos.0)?;
            obj.set("y", v.pos.1)?;
            Ok(obj)
        })?;
        properties.define_method_mut("move", |v, _ctx, _this: Value, (x, y): (i32, i32)| {
            v.move_(x, y);
            Ok(())
        })?;
        Ok(())
    }
}

#[test]
fn new_object_class() -> Result<()> {
    quijine::context(|ctx| {
        ctx.new_global_constructor::<S1>()?;
        let mut s1 = ctx.eval("const s1 = new S1('foo'); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
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
    quijine::run(|rt| {
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            ctx.new_global_constructor::<S1>()?;
            let result = ctx.eval("const s1 = new S1('foo'); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(!result.prototype()?.is_null());
            let name: String = ctx.eval_into("s1.name", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("foo", name);
        }
        {
            let ctxs = rt.new_context_scope();
            let ctx = ctxs.get();
            ctx.new_global_constructor::<S1>()?;
            let result = ctx.eval("const s1 = new S1('foo'); s1", "<input>", EvalFlags::TYPE_GLOBAL)?;
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

    fn define_properties<'q, T: ClassProperties<'q, Self>>(properties: &mut T) -> Result<()> {
        properties.define_get_set_mut(
            "name",
            |v, _ctx, _this: Value| {
                let v = v.0.borrow();
                Ok(v.name.clone())
            },
            |v, _ctx, _this: Value, name: Value| {
                let mut v = v.0.borrow_mut();
                v.name = name.to_string()?;
                Ok(name)
            },
        )?;
        properties.define_get("pos", |v, ctx, _this: Value| {
            let v = v.0.borrow();
            let obj = ctx.new_object()?;
            obj.set("x", v.pos.0)?;
            obj.set("y", v.pos.1)?;
            Ok(obj)
        })?;
        properties.define_method_mut("move", |v, _ctx, _this: Value, (x, y): (i32, i32)| {
            let mut v = v.0.borrow_mut();
            v.move_(x, y);
            Ok(())
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
