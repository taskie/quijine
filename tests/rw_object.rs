use quijine::{ReadObjFlags, WriteObjFlags};

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
        let a = ctx.read_object(&buf, ReadObjFlags::REFERENCE)?;
        let b = a.get("b")?;
        let c = b.get("c")?;
        let _a2 = c.get("a")?;
        Ok(())
    })
    .unwrap();
}
