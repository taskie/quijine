use quijine::{EvalFlags, Result};

#[test]
fn rust_to_js() -> Result<()> {
    quijine::context(|ctx| {
        let js_assert_eq = ctx.eval(
            r#"
                (a, b) => {
                    if (a !== JSON.parse(b)) {
                        throw Error(`${a} !== ${b}`);
                    }
                };
            "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        ctx.call(js_assert_eq.clone(), ctx.undefined(), ("hello", "\"hello\""))?;
        ctx.call(js_assert_eq.clone(), ctx.undefined(), (true, "true"))?;
        ctx.call(js_assert_eq.clone(), ctx.undefined(), (42, "42"))?;
        ctx.call(js_assert_eq.clone(), ctx.undefined(), (0.25, "0.25"))?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn js_to_rust() -> Result<()> {
    quijine::context(|ctx| {
        let v: String = ctx.parse_json("\"hello\"", "<input>")?.try_into()?;
        assert_eq!("hello", v);
        let v: bool = ctx.parse_json("true", "<input>")?.try_into()?;
        assert_eq!(true, v);
        let v: i32 = ctx.parse_json("42", "<input>")?.try_into()?;
        assert_eq!(42, v);
        let v: f64 = ctx.parse_json("0.25", "<input>")?.try_into()?;
        assert_eq!(0.25, v);
        Ok(())
    })?;
    Ok(())
}
