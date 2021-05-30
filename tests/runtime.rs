use quijine::{QjError, QjEvalFlags};

#[test]
fn multiple_runtimes() {
    use std::sync::mpsc::channel;
    let (tx, rx) = channel::<String>();
    let th = std::thread::spawn(move || {
        quijine::run_with_context(move |ctx| {
            let recv = ctx.new_function(
                move |ctx, _this, _args| {
                    let message = rx.recv().map_err(|e| QjError::with_str(e.to_string().as_str()))?;
                    Ok(ctx.new_string(message.as_str()).into())
                },
                "recv",
                0,
            );
            ctx.global_object().set("recv", recv);
            let result = ctx.eval("recv();", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
            assert_eq!("Hello, world!".to_owned(), result.to_string().unwrap(), "received");
            let result = ctx.eval("recv();", "<input>", QjEvalFlags::TYPE_GLOBAL).unwrap();
            assert_eq!("Goodbye, world!".to_owned(), result.to_string().unwrap(), "received");
        });
    });
    quijine::run_with_context(move |ctx| {
        let send = ctx.new_function(
            move |ctx, _this, args| {
                let message = args[0].to_string().unwrap();
                tx.send(message)
                    .map_err(|e| QjError::with_str(e.to_string().as_str()))?;
                Ok(ctx.undefined().into())
            },
            "send",
            1,
        );
        ctx.global_object().set("send", send);
        ctx.eval("send('Hello, world!');", "<input>", QjEvalFlags::TYPE_GLOBAL)
            .unwrap();
        ctx.eval("send('Goodbye, world!');", "<input>", QjEvalFlags::TYPE_GLOBAL)
            .unwrap();
    });
    th.join().expect("joined");
}
