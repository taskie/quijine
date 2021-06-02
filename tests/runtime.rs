use quijine::{Error, ErrorKind, EvalFlags, Result};

#[test]
fn multiple_runtimes() -> Result<()> {
    use std::sync::mpsc::channel;
    let (tx, rx) = channel::<String>();
    let th = std::thread::spawn(move || {
        quijine::run_with_context(move |ctx| {
            let recv = ctx.new_function(
                move |ctx, _this, _args| {
                    let message = rx
                        .recv()
                        .map_err(|e| Error::with_external(ErrorKind::InternalError, Box::new(e.clone())))?;
                    Ok(ctx.new_string(message.as_str())?)
                },
                "recv",
                0,
            )?;
            ctx.global_object()?.set("recv", recv)?;
            let result = ctx.eval("recv();", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("Hello, world!".to_owned(), result.to_string()?, "received");
            let result = ctx.eval("recv();", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("Goodbye, world!".to_owned(), result.to_string()?, "received");
            Ok(())
        })
        .unwrap();
    });
    quijine::run_with_context(move |ctx| {
        let send = ctx.new_function(
            move |ctx, _this, args| {
                let message = args[0].to_string()?;
                tx.send(message)
                    .map_err(|e| Error::with_external(ErrorKind::InternalError, Box::new(e.clone())))?;
                Ok(ctx.undefined())
            },
            "send",
            1,
        )?;
        ctx.global_object()?.set("send", send)?;
        ctx.eval("send('Hello, world!');", "<input>", EvalFlags::TYPE_GLOBAL)?;
        ctx.eval("send('Goodbye, world!');", "<input>", EvalFlags::TYPE_GLOBAL)?;
        Ok(())
    })?;
    th.join().expect("joined");
    Ok(())
}
