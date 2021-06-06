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
        ctx.call_into_void(js_assert_eq.clone(), ctx.undefined(), ("hello", "\"hello\""))?;
        ctx.call_into_void(js_assert_eq.clone(), ctx.undefined(), (true, "true"))?;
        ctx.call_into_void(js_assert_eq.clone(), ctx.undefined(), (42, "42"))?;
        ctx.call_into_void(js_assert_eq.clone(), ctx.undefined(), (0.25, "0.25"))?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn js_to_rust() -> Result<()> {
    quijine::context(|ctx| {
        let v: String = ctx.parse_json_into("\"hello\"", "<input>")?;
        assert_eq!("hello", v);
        let v: bool = ctx.parse_json_into("true", "<input>")?;
        assert_eq!(true, v);
        let v: i32 = ctx.parse_json_into("42", "<input>")?;
        assert_eq!(42, v);
        let v: f64 = ctx.parse_json_into("0.25", "<input>")?;
        assert_eq!(0.25, v);
        Ok(())
    })?;
    Ok(())
}
