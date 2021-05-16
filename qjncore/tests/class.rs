use std::ffi::CString;

use qjncore::{ClassDef, ClassId, Context, Runtime, conversion::AsJsValue};

#[test]
fn test() {
    let rt = Runtime::new();
    // Define Class
    let ctx = Context::new(rt);
    let test_class_def = ClassDef {
        class_name: CString::new("Test").unwrap(),
        ..Default::default()
    };
    let test_class_id = ClassId::generate();
    rt.new_class(test_class_id, &test_class_def);
    let test_proto = ctx.new_object();
    ctx.set_class_proto(test_class_id, test_proto);
    // Use Class
    let obj = ctx.new_object_class(test_class_id);
    let obj_proto = obj.prototype(ctx);
    assert!(obj_proto.is_object());
    assert!(! obj_proto.is_null());
    unsafe {
        assert_eq!(obj_proto.as_js_value().u.ptr, test_proto.as_js_value().u.ptr);
    }
    unsafe {
        ctx.free_value(obj_proto);
        ctx.free_value(obj);
        Context::free(ctx);
        Runtime::free(rt);
    }
}