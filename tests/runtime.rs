use quijine::{Data, Error, ErrorKind, EvalFlags, Result};

#[test]
fn multiple_runtimes() -> Result<()> {
    use std::sync::mpsc::channel;
    let (tx, rx) = channel::<String>();
    let th = std::thread::spawn(move || {
        quijine::context(move |ctx| {
            let recv = ctx.new_function_with(
                move |_ctx, _this: Data, _args: ()| {
                    let message = rx
                        .recv()
                        .map_err(|e| Error::with_external(ErrorKind::InternalError, Box::new(e.clone())))?;
                    Ok(message)
                },
                "recv",
                0,
            )?;
            ctx.global_object()?.set("recv", recv)?;
            let result: String = ctx.eval_into("recv();", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("Hello, world!".to_owned(), result, "received");
            let result: String = ctx.eval_into("recv();", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert_eq!("Goodbye, world!".to_owned(), result, "received");
            Ok(())
        })
        .unwrap();
    });
    quijine::context(move |ctx| {
        let send = ctx.new_function_with(
            move |_ctx, _this: Data, (message,): (String,)| {
                tx.send(message)
                    .map_err(|e| Error::with_external(ErrorKind::InternalError, Box::new(e.clone())))?;
                Ok(())
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
