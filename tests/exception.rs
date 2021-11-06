use quijine::{PropFlags, Result};

#[test]
fn exception() -> Result<()> {
    quijine::context(|ctx| {
        // 0
        let e = ctx.take_exception();
        assert!(e.is_null());
        // 1
        let error = ctx.new_error()?;
        error.define_property_value("message", "HELP!", PropFlags::CONFIGURABLE | PropFlags::WRITABLE)?;
        let exc = ctx.throw(error.into());
        assert!(exc.is_exception());
        let e = ctx.take_exception();
        assert!(e.is_error());
        assert_eq!("Error: HELP!", e.to_string()?);
        // 2
        let exc = ctx.throw_syntax_error("syntax error!");
        assert!(exc.is_exception());
        let e = ctx.take_exception();
        assert_eq!("SyntaxError: syntax error!", e.to_string()?);
        // 4, 5
        let error = ctx.new_error()?;
        ctx.throw(error.into());
        let error = ctx.new_error()?;
        ctx.throw(error.into());
        let e = ctx.take_exception();
        assert_eq!("Error", e.to_string()?);
        // 3
        let e = ctx.take_exception();
        assert!(e.is_null());
        Ok(())
    })
}
