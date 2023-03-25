use maplit::{btreemap, hashmap};
use quijine::{EvalFlags, FromQj, Result};
use std::collections::{BTreeMap, HashMap};

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
        ctx.call_into_void(js_assert_eq.clone(), (), ("hello", "\"hello\""))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (true, "true"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (42, "42"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (0.25, "0.25"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (Some(42), "42"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (None as Option<i32>, "null"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (Vec::<i32>::new(), "[]"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (vec![2, 3, 5, 7], "[2,3,5,7]"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (HashMap::<String, i32>::new(), "{}"))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (hashmap! {"H" => 1}, r#"{"H":1}"#))?;
        ctx.call_into_void(js_assert_eq.clone(), (), (BTreeMap::<String, i32>::new(), "{}"))?;
        ctx.call_into_void(
            js_assert_eq.clone(),
            (),
            (btreemap! {"H" => 1, "He" => 2}, r#"{"H":1,"He":2}"#),
        )?;
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
        let v = Option::from_qj(ctx.parse_json("42", "<input>")?)?;
        assert_eq!(Some(42), v);
        let v = Option::from_qj(ctx.parse_json("null", "<input>")?)?;
        assert_eq!(None as Option<i32>, v);
        let v = Vec::from_qj(ctx.parse_json("[]", "<input>")?)?;
        assert_eq!(Vec::<i32>::new(), v);
        let v = Vec::from_qj(ctx.parse_json("[2, 3, 5, 7]", "<input>")?)?;
        assert_eq!(vec![2, 3, 5, 7], v);
        let v = HashMap::from_qj(ctx.parse_json("{}", "<input>")?)?;
        assert_eq!(HashMap::<String, i32>::new(), v);
        let v = HashMap::from_qj(ctx.parse_json(r#"{"H":1}"#, "<input>")?)?;
        assert_eq!(hashmap! {"H".to_owned() => 1}, v);
        let v = BTreeMap::from_qj(ctx.parse_json("{}", "<input>")?)?;
        assert_eq!(BTreeMap::<String, i32>::new(), v);
        let v = BTreeMap::from_qj(ctx.parse_json(r#"{"H":1,"He":2}"#, "<input>")?)?;
        assert_eq!(btreemap! {"H".to_owned() => 1, "He".to_owned() => 2}, v);
        Ok(())
    })?;
    Ok(())
}

#[test]
fn js_to_rust_iterable() -> Result<()> {
    quijine::context(|ctx| {
        let v = ctx.eval(
            r#"
            const a = [0];
            a.x = 42;
            a;
        "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        let json: String = ctx.json_stringify_into(v.clone(), (), ())?;
        assert_eq!(r#"[0]"#, json);
        assert_eq!(vec![0], Vec::from_qj(v.clone())?);
        assert_eq!(
            btreemap! {"0".to_owned() => 0, "x".to_owned() => 42},
            BTreeMap::from_qj(v.clone())?
        );
        assert_eq!(btreemap! {0 => 42}, BTreeMap::from_qj(v)?);
        Ok(())
    })?;
    Ok(())
}

#[test]
fn js_to_rust_proto() -> Result<()> {
    quijine::context(|ctx| {
        let v = ctx.eval(
            r#"
            const c = Object.create(null);
            c.c = 3;
            const b = Object.create(c);
            b.b = 2;
            const a = Object.create(b);
            a.a = 1;
            a;
        "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        )?;
        let json: String = ctx.json_stringify_into(v.clone(), (), ())?;
        assert_eq!(r#"{"a":1}"#, json);
        assert_eq!(btreemap! {"a".to_owned() => 1}, BTreeMap::from_qj(v)?);
        Ok(())
    })?;
    Ok(())
}
