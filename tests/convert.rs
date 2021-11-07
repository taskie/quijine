use quijine::{EvalFlags, FromQj, Result};

#[test]
fn rust_to_js() -> Result<()> {
    quijine::context(|ctx| {
        let js_assert_eq = ctx.eval(
            r#"
                (a, b) => {
                    if (typeof a === "object") {
                        const as = JSON.stringify(a);
                        if (as !== b) {
                            throw Error(`${as} !== ${b}`);
                        }
                    } else {
                        const bv = JSON.parse(b);
                        if (a !== bv) {
                            throw Error(`${a} !== ${bv}`);
                        }
                    }
                };
            "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        ctx.call(js_assert_eq.clone(), (), ("hello", "\"hello\""))?;
        ctx.call(js_assert_eq.clone(), (), (true, "true"))?;
        ctx.call(js_assert_eq.clone(), (), (42, "42"))?;
        ctx.call(js_assert_eq.clone(), (), (0.25, "0.25"))?;
        ctx.call(js_assert_eq.clone(), (), (Some(42), "42"))?;
        ctx.call(js_assert_eq.clone(), (), (None as Option<i32>, "null"))?;
        ctx.call(js_assert_eq.clone(), (), (Vec::<i32>::new(), "[]"))?;
        ctx.call(js_assert_eq.clone(), (), (vec![2, 3, 5, 7], "[2,3,5,7]"))?;
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
        let v = Option::from_qj(ctx.parse_json("42", "<input>")?)?;
        assert_eq!(Some(42), v);
        let v = Option::from_qj(ctx.parse_json("null", "<input>")?)?;
        assert_eq!(None as Option<i32>, v);
        let v = Vec::from_qj(ctx.parse_json("[]", "<input>")?)?;
        assert_eq!(Vec::<i32>::new(), v);
        let v = Vec::from_qj(ctx.parse_json("[2, 3, 5, 7]", "<input>")?)?;
        assert_eq!(vec![2, 3, 5, 7], v);
        Ok(())
    })?;
    Ok(())
}
