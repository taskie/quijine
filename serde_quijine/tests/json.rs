use quijine::{EvalFlags, ExternalResult, Result as QjResult};
use serde_json::Value;
use serde_quijine::{from_qj, to_qj};

#[test]
fn test_serde_json_compatibility() -> QjResult<()> {
    quijine::context(|ctx| {
        // keys must be sorted
        let json = r#"{"a":[2,3],"b":true,"f":0.25,"i":42,"n":null,"o":{"k":"v"},"s":"hello"}"#;
        let json_value: Value = serde_json::from_str(json).map_err_to_qj()?;
        let json_qj = to_qj(ctx, json_value)?;
        ctx.global_object()?.set("json", json_qj.clone())?;
        let code = r#"
            const assertEq = (a, b) => { if (a !== b) { throw Error(`${a} !== ${b}`); } };
            assertEq(2, json.a[0]);
            assertEq(3, json.a[1]);
            assertEq(true, json.b);
            assertEq(0.25, json.f);
            assertEq(42, json.i);
            assertEq(null, json.n);
            assertEq("v", json.o.k);
            assertEq("hello", json.s);
        "#;
        ctx.eval(code, "<input>", EvalFlags::TYPE_GLOBAL)?;
        let json_qj_json: String = ctx.json_stringify_into(json_qj.clone(), ctx.undefined(), ctx.undefined())?;
        assert_eq!(json, &json_qj_json);
        let json_qj_value: Value = from_qj(json_qj)?;
        let json_qj_value_json = serde_json::to_string(&json_qj_value).map_err_to_qj()?;
        assert_eq!(json, &json_qj_value_json);
        Ok(())
    })?;
    Ok(())
}
