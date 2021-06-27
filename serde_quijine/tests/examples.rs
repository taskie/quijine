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
        let s1_json: String = ctx.json_stringify_into(s1_qj, ctx.undefined(), ctx.undefined())?;
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
