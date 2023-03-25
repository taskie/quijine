use quijine::{EvalFlags, IntoQjAtom, PropFlags, Result, Value};

#[test]
fn prevent_extensions() -> Result<()> {
    quijine::context(|ctx| {
        let obj = ctx.new_object()?;
        obj.set("x", "hello")?;
        ctx.global_object()?.set("obj", obj.clone())?;
        assert!(obj.is_extensible()?);
        ctx.eval(
            "obj.y = 'foo'",
            "<input>",
            EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_STRICT,
        )?;
        obj.prevent_extensions()?;
        assert!(!obj.is_extensible()?);
        ctx.eval(
            "obj.z = 'bar'",
            "<input>",
            EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_STRICT,
        )
        .expect_err("not extensible");
        Ok(())
    })
}

#[test]
fn define_property_value() -> Result<()> {
    quijine::context(|ctx| {
        let obj = ctx.new_object()?;
        let val = ctx.new_string("hello")?;
        obj.define_property_value_from("x", val, PropFlags::empty())?;
        ctx.global_object()?.set("obj", obj.clone())?;
        let x = ctx.eval("obj.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("hello", x.to_string()?);
        ctx.eval(
            "obj.x = 'foo'",
            "<input>",
            EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_STRICT,
        )
        .expect_err("not writable");
        Ok(())
    })
}

#[test]
fn define_property_value_raw() -> Result<()> {
    quijine::context(|ctx| {
        let obj = ctx.new_object()?;
        let val = ctx.new_string("hello")?;
        obj.define_property(
            "x".into_qj_atom(ctx)?,
            val.into(),
            ctx.undefined().into(),
            ctx.undefined().into(),
            PropFlags::HAS_VALUE | PropFlags::HAS_CONFIGURABLE | PropFlags::HAS_WRITABLE | PropFlags::HAS_ENUMERABLE,
        )?;
        ctx.global_object()?.set("obj", obj.clone())?;
        let x = ctx.eval("obj.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("hello", x.to_string()?);
        ctx.eval(
            "obj.x = 'foo'",
            "<input>",
            EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_STRICT,
        )
        .expect_err("not writable");
        Ok(())
    })
}

#[test]
fn define_property_get_set() -> Result<()> {
    quijine::context(|ctx| {
        let obj = ctx.new_object()?;
        obj.set("_x", "hello")?;
        ctx.global_object()?.set("obj", obj.clone())?;
        let getter = ctx.new_function_from(
            |_ctx, this: Value, _args: ()| {
                let x: String = this.get("_x")?;
                Ok(x.to_ascii_uppercase())
            },
            "x",
            0,
        )?;
        let setter = ctx.new_function_from(
            |_ctx, this: Value, args: (String,)| {
                this.set("_x", args.0.to_ascii_lowercase())?;
                Ok(())
            },
            "x",
            1,
        )?;
        obj.define_property_get_set_from("x", getter, setter, PropFlags::empty())?;
        let x = ctx.eval("obj.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("HELLO", x.to_string()?);
        ctx.eval("obj.x = 'foo'", "<input>", EvalFlags::TYPE_GLOBAL)?;
        let x = ctx.eval("obj.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("FOO", x.to_string()?);
        let x: String = obj.get("_x")?;
        assert_eq!("foo", x);
        Ok(())
    })
}

#[test]
fn define_property_get_set_raw() -> Result<()> {
    quijine::context(|ctx| {
        let obj = ctx.new_object()?;
        obj.set("_x", "hello")?;
        ctx.global_object()?.set("obj", obj.clone())?;
        let getter = ctx.new_function(
            |ctx, this, _args| {
                let x: String = this.get("_x")?;
                Ok(ctx.new_string(&x.to_ascii_uppercase())?.into())
            },
            "x",
            0,
        )?;
        let setter = ctx.new_function(
            |ctx, this, args| {
                this.set("_x", args[0].to_string()?.to_ascii_lowercase())?;
                Ok(ctx.undefined().into())
            },
            "x",
            1,
        )?;
        obj.define_property(
            "x".into_qj_atom(ctx)?,
            ctx.undefined().into(),
            getter.into(),
            setter.into(),
            PropFlags::HAS_GET | PropFlags::HAS_SET | PropFlags::HAS_CONFIGURABLE | PropFlags::HAS_ENUMERABLE,
        )?;
        let x = ctx.eval("obj.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("HELLO", x.to_string()?);
        ctx.eval("obj.x = 'foo'", "<input>", EvalFlags::TYPE_GLOBAL)?;
        let x = ctx.eval("obj.x", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("FOO", x.to_string()?);
        let x: String = obj.get("_x")?;
        assert_eq!("foo", x);
        Ok(())
    })
}
