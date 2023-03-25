use quijine_core::{
    js_c_function, js_class_finalizer, raw, CFunctionListEntry, ClassDef, ClassId, Context, EvalFlags, Runtime, Value,
};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::{
    cell::RefCell,
    ffi::{c_void, CString},
    ptr::{null_mut, NonNull},
};

thread_local! {
    static PRNG_CLASS_ID: RefCell<ClassId> = RefCell::new(ClassId::none());
}

struct Prng {
    rng: XorShiftRng,
}

fn prng_generate<'q>(ctx: Context<'q>, this: Value<'q>, _values: &[Value<'q>]) -> Value<'q> {
    let mut prng = NonNull::new(PRNG_CLASS_ID.with(|id| this.opaque(*id.borrow()) as *mut Prng)).unwrap();
    let res: i32 = unsafe { prng.as_mut() }.rng.gen();
    ctx.new_int32(res)
}

fn prng_new<'q>(ctx: Context<'q>, _this: Value<'q>, _values: &[Value<'q>]) -> Value<'q> {
    let obj = PRNG_CLASS_ID.with(|id| ctx.new_object_class(*id.borrow()));
    let prng = Box::new(Prng {
        rng: XorShiftRng::from_seed([0; 16]),
    });
    obj.set_opaque(Box::into_raw(prng) as *mut c_void);
    obj
}

unsafe fn prng_finalizer<'q>(_rt: Runtime<'q>, val: Value<'q>) {
    let obj = PRNG_CLASS_ID.with(|id| val.opaque(*id.borrow())) as *mut Prng;
    drop(Box::from_raw(obj));
}

#[allow(non_snake_case)]
fn C(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[test]
fn test() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    let class_name = C("Prng");
    let prng_class = unsafe {
        ClassDef::from_raw(raw::JSClassDef {
            class_name: class_name.as_ptr(),
            finalizer: js_class_finalizer!(prng_finalizer),
            gc_mark: None,
            call: None,
            exotic: null_mut(),
        })
    };
    let generate_name = C("generate");
    let prng_proto_funcs = unsafe {
        &[CFunctionListEntry::cfunc_def(
            &generate_name,
            0,
            js_c_function!(prng_generate),
        )]
    };
    PRNG_CLASS_ID.with(|id| {
        *id.borrow_mut() = ClassId::generate();
        rt.new_class(*id.borrow(), &prng_class);
        let prng_proto = ctx.new_object();
        prng_proto.set_property_function_list(ctx, prng_proto_funcs);
        prng_proto.set_property_str(ctx, "answer", ctx.new_int32(42)).unwrap();
        ctx.set_class_proto(*id.borrow(), prng_proto);
        eprintln!("{:?}", prng_proto);
    });
    let global = ctx.global_object();
    global
        .set_property_str(ctx, "Prng", ctx.new_c_function(js_c_function!(prng_new), "Prng", 0))
        .unwrap();
    let ret = ctx.eval("var prng = Prng(); prng", "<input>", EvalFlags::TYPE_GLOBAL);
    assert!(!ctx.exception().is_exception(), "no exception");
    unsafe {
        ctx.free_value(ret);
    }
    PRNG_CLASS_ID.with(|id| {
        let pt = ret.prototype(ctx);
        assert!(pt.is_object(), "prototype is object");
        eprintln!("{:?}", pt);
        assert_ne!(null_mut(), ret.opaque(*id.borrow()), "valid class_id");
        unsafe {
            ctx.free_value(pt);
        }
    });
    let answer = ctx.eval("prng.answer", "<input>", EvalFlags::TYPE_GLOBAL);
    assert_eq!(Some(42), answer.to_i32(ctx), "property");
    unsafe {
        ctx.free_value(answer);
    };
    let ret = ctx.eval("prng.generate()", "<input>", EvalFlags::TYPE_GLOBAL);
    eprintln!("{:?}", ret);
    assert!(!ret.is_exception());
    unsafe {
        ctx.free_value(ret);
        ctx.free_value(global);
        Context::free(ctx);
        Runtime::free(rt);
    }
}
