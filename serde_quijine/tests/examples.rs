use quijine::{EvalFlags, Result as QjResult};
use serde::{Deserialize, Serialize};
use serde_quijine::{from_qj, to_qj};

#[derive(Serialize, Deserialize)]
struct S1 {
    name: String,
    pos: (i32, i32),
}

#[test]
fn example_ser() -> QjResult<()> {
    quijine::context(|ctx| {
        let s1 = S1 {
            name: "foo".to_owned(),
            pos: (3, 4),
        };
        let s1_qj = to_qj(ctx, s1)?;
        ctx.global_object()?.set("s1", s1_qj.clone())?;
        let code = r#"
            const assertEq = (a, b) => { if (a !== b) { throw Error(`${a} !== ${b}`); } };
            assertEq("string", typeof s1.name);
            assertEq("foo", s1.name);
            assertEq("object", typeof s1.pos);
            assertEq(true, Array.isArray(s1.pos));
            assertEq(2, s1.pos.length);
            assertEq(3, s1.pos[0]);
            assertEq(4, s1.pos[1]);
        "#;
        ctx.eval(code, "<input>", EvalFlags::TYPE_GLOBAL)?;
        let s1_json: String = ctx.json_stringify(s1_qj, ctx.undefined(), ctx.undefined())?.into();
        assert_eq!(r#"{"name":"foo","pos":[3,4]}"#, s1_json);
        Ok(())
    })?;
    Ok(())
}

#[test]
fn example_de() -> QjResult<()> {
    quijine::context(|ctx| {
        let code = r#"
            const s1 = {
                name: "foo",
                pos: [3, 4],
            };
            s1;
        "#;
        let s1_value = ctx.eval(code, "<input>", EvalFlags::TYPE_GLOBAL)?;
        let s1: S1 = from_qj(s1_value)?;
        assert_eq!("foo", s1.name);
        assert_eq!((3, 4), s1.pos);
        Ok(())
    })?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct S2 {
    v: Option<String>,
}

#[test]
fn example_option_ser() -> QjResult<()> {
    quijine::context(|ctx| {
        // Some
        let s2 = S2 {
            v: Some("hello".to_owned()),
        };
        ctx.global_object()?.set("s2", to_qj(ctx, s2)?)?;
        let code = r#"
            const assertEq = (a, b) => { if (a !== b) { throw Error(`${a} !== ${b}`); } };
            assertEq("string", typeof s2.v);
            assertEq("hello", s2.v);
        "#;
        ctx.eval(code, "<input>", EvalFlags::TYPE_GLOBAL)?;
        // None
        let s2 = S2 { v: None };
        ctx.global_object()?.set("s2", to_qj(ctx, s2)?)?;
        let code = r#"
            assertEq("object", typeof s2.v);
            assertEq(null, s2.v);
        "#;
        ctx.eval(code, "<input>", EvalFlags::TYPE_GLOBAL)?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn example_option_de() -> QjResult<()> {
    quijine::context(|ctx| {
        // Some
        let s2_value = ctx.eval(r#"({"v":"hello"})"#, "<input>", EvalFlags::TYPE_GLOBAL)?;
        let s2: S2 = from_qj(s2_value)?;
        assert_eq!(Some("hello".to_owned()), s2.v);
        // None (null)
        let s2_value = ctx.eval(r#"({"v":null})"#, "<input>", EvalFlags::TYPE_GLOBAL)?;
        let s2: S2 = from_qj(s2_value)?;
        assert_eq!(None, s2.v);
        // None (uninitialized)
        let s2_value = ctx.eval(r#"({})"#, "<input>", EvalFlags::TYPE_GLOBAL)?;
        let s2: S2 = from_qj(s2_value)?;
        assert_eq!(None, s2.v);
        Ok(())
    })?;
    Ok(())
}
