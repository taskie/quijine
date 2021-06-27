use quijine::{Object, ReadObjFlags, WriteObjFlags};
use std::convert::TryInto;

#[test]
fn test() {
    quijine::context(|ctx| {
        let buf = {
            let a = ctx.new_object()?;
            let b = ctx.new_object()?;
            let c = ctx.new_object()?;
            a.set("b", b.clone())?;
            b.set("c", c.clone())?;
            c.set("a", a.clone())?;
            ctx.write_object(a.into(), WriteObjFlags::REFERENCE)?
        };
        eprintln!("{:?}", buf);
        let a: Object = ctx.read_object(&buf, ReadObjFlags::REFERENCE)?.try_into()?;
        let b: Object = a.get("b")?;
        let c: Object = b.get("c")?;
        let _a2: Object = c.get("a")?;
        Ok(())
    })
    .unwrap();
}
