use quijine_core::{Context, ReadObjFlags, Runtime, WriteObjFlags};

#[test]
fn test() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    let a = ctx.new_object();
    let b = ctx.new_object();
    let c = ctx.new_object();
    ctx.dup_value(b);
    a.set_property_str(ctx, "b", b).unwrap();
    ctx.dup_value(c);
    b.set_property_str(ctx, "c", c).unwrap();
    ctx.dup_value(a);
    c.set_property_str(ctx, "a", a).unwrap();
    let buf = ctx.write_object(a, WriteObjFlags::REFERENCE).unwrap();
    eprintln!("{:?}", buf);
    unsafe {
        ctx.free_value(c);
        ctx.free_value(b);
        ctx.free_value(a);
    }
    let a = ctx.read_object(buf.as_slice(), ReadObjFlags::REFERENCE);
    assert!(a.is_object());
    let b = a.property_str(ctx, "b");
    assert!(b.is_object());
    let c = b.property_str(ctx, "c");
    assert!(c.is_object());
    let a2 = c.property_str(ctx, "a");
    assert!(a2.is_object());
    unsafe {
        ctx.free_value(a2);
        ctx.free_value(c);
        ctx.free_value(b);
        ctx.free_value(a);
        Context::free(ctx);
        Runtime::free(rt);
    }
}
