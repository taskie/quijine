// This is an experimental feature.
#![cfg(feature = "c_function_list")]

use quijine::{
    js_c_function, js_c_getter, js_c_setter, js_module_init_func, CFunctionListBuilder, CStringArena, Class, Context,
    DefArena, EvalFlags, Object, Result, Value,
};
use std::cell::RefCell;

thread_local! {
    pub static DEF_ARENA: RefCell<DefArena> = RefCell::new(DefArena::new());
}

struct Counter(i32);

impl Class for Counter {
    fn name() -> &'static str {
        "Counter"
    }

    fn setup_proto<'q>(_ctx: Context<'q>, proto: Object<'q>) -> Result<()> {
        let mut arena = CStringArena::new();
        let func_list = CFunctionListBuilder::new(&mut arena)
            .cfunc_def(
                "increment",
                0,
                js_c_function!(|ctx, mut this, _args| {
                    this.opaque_mut::<Counter>().unwrap().0 += 1;
                    ctx.undefined().into()
                }),
            )
            .cgetset_def(
                "count",
                js_c_getter!(|ctx, this| { ctx.new_int32(this.opaque::<Counter>().unwrap().0).into() }),
                js_c_setter!(|_ctx, mut this, val| {
                    this.opaque_mut::<Counter>().unwrap().0 = val.to_i32().unwrap();
                    val
                }),
            )
            .build();
        DEF_ARENA.with(|arena| {
            let mut arena = arena.borrow_mut();
            let val: Value = proto.into();
            val.set_property_function_list(arena.registered_function_list("classes/Counter", func_list));
        });
        Ok(())
    }
}

fn init_modules<'q>(ctx: Context<'q>, c_string_arena: &mut CStringArena) {
    {
        // console
        let func_list = CFunctionListBuilder::new(c_string_arena)
            .cfunc_def(
                "log",
                1,
                js_c_function!(|ctx, _this, args| {
                    for (i, arg) in args.iter().enumerate() {
                        if i != 0 {
                            print!(" ");
                        }
                        print!("{}", arg.to_string().unwrap())
                    }
                    println!();
                    ctx.undefined().into()
                }),
            )
            .cfunc_def(
                "error",
                1,
                js_c_function!(|ctx, _this, args| {
                    for (i, arg) in args.iter().enumerate() {
                        if i != 0 {
                            eprint!(" ");
                        }
                        eprint!("{}", arg.to_string().unwrap())
                    }
                    eprintln!();
                    ctx.undefined().into()
                }),
            )
            .build();
        let m = ctx.new_c_module(
            "console",
            js_module_init_func!(|_ctx, m| {
                DEF_ARENA
                    .with(|arena| m.set_module_export_list(arena.borrow().function_list("modules/console").unwrap()))
            }),
        );
        DEF_ARENA.with(|arena| {
            m.add_module_export_list(
                arena
                    .borrow_mut()
                    .registered_function_list("modules/console", func_list),
            );
        });
    }
    {
        // counter
        let func_list = CFunctionListBuilder::new(c_string_arena)
            .cfunc_constructor_def(
                "Counter",
                1,
                js_c_function!(|ctx, _this, args| {
                    let v = Counter(args.get(0).and_then(|v| v.to_i32().ok()).unwrap_or(0));
                    ctx.new_object_with_opaque(v).unwrap().into()
                }),
            )
            .build();
        let m = ctx.new_c_module(
            "counter",
            js_module_init_func!(|_ctx, m| {
                DEF_ARENA
                    .with(|arena| m.set_module_export_list(arena.borrow().function_list("modules/counter").unwrap()))
            }),
        );
        DEF_ARENA.with(|arena| {
            m.add_module_export_list(
                arena
                    .borrow_mut()
                    .registered_function_list("modules/counter", func_list),
            );
        });
    }
}

#[test]
fn modules() -> Result<()> {
    quijine::context(|ctx| {
        let mut arena = CStringArena::new();
        init_modules(ctx, &mut arena);

        let ret = ctx.eval(
            r#"
                import { Counter } from "counter";
                import * as console from "console";
                console.error("BEGIN");
                const counter = new Counter(1);
                console.log(counter.count);
                counter.increment();
                console.log(counter.count);
                counter.count = 42;
                console.log(counter.count);
                counter.increment();
                console.log(counter.count);
                console.error("END");
                counter
            "#,
            "<input>",
            EvalFlags::TYPE_MODULE | EvalFlags::FLAG_STRICT,
        )?;
        assert!(ret.is_undefined());
        Ok(())
    })?;
    Ok(())
}
