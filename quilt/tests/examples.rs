use quilt::{
    js_c_function, js_class_finalizer, CFunctionListEntry, ClassDef, ClassId, Context, EvalFlags, Runtime, Value,
};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::{
    cell::RefCell,
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

thread_local! {
    static PRNG_CLASS_ID: RefCell<ClassId> = RefCell::new(ClassId::none());
}

struct PRNG {
    rng: XorShiftRng,
}

unsafe fn prng_generate<'q, 'a>(ctx: Context<'q>, this: Value<'q>, _values: &'a [Value<'q>]) -> Value<'q> {
    let mut prng = NonNull::new(PRNG_CLASS_ID.with(|id| this.opaque(*id.borrow()) as *mut PRNG)).unwrap();
    let res: i32 = prng.as_mut().rng.gen();
    ctx.new_int32(res)
}

unsafe fn prng_new<'q, 'a>(ctx: Context<'q>, this: Value<'q>, _values: &'a [Value<'q>]) -> Value<'q> {
    let obj = PRNG_CLASS_ID.with(|id| ctx.new_object_class(*id.borrow()));
    let prng = Box::new(PRNG {
        rng: XorShiftRng::from_seed([0; 16]),
    });
    obj.set_opaque(Box::into_raw(prng) as *mut c_void);
    obj
}

unsafe fn prng_finalizer<'q, 'a>(rt: Runtime<'q>, val: Value<'q>) {
    let obj = PRNG_CLASS_ID.with(|id| val.opaque(*id.borrow())) as *mut PRNG;
    Box::from_raw(obj);
}

#[test]
fn test() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    let prng_class = ClassDef {
        class_name: "PRNG".to_string(),
        finalizer: js_class_finalizer!(prng_finalizer),
        ..Default::default()
    };
    let prng_proto_funcs = &[CFunctionListEntry::new("generate", 0, js_c_function!(prng_generate))];
    PRNG_CLASS_ID.with(|id| {
        *id.borrow_mut() = ClassId::generate();
        rt.new_class(*id.borrow(), &prng_class);
        let prng_proto = ctx.new_object();
        prng_proto.set_property_function_list(ctx, prng_proto_funcs);
        prng_proto.set_property_str(ctx, "answer", ctx.new_int32(42));
        ctx.set_class_proto(*id.borrow(), prng_proto);
        eprintln!("{:?}", prng_proto);
    });
    let global = ctx.global_object();
    global.set_property_str(ctx, "PRNG", unsafe {
        ctx.new_c_function(js_c_function!(prng_new), "PRNG", 0)
    });
    let ret = ctx.eval("var prng = PRNG(); prng", "<input>", EvalFlags::TYPE_GLOBAL);
    assert_eq!(false, ctx.exception().is_exception(), "no exception");
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
    // assert!(!ret.is_exception());
    unsafe {
        ctx.free_value(ret);
        ctx.free_value(global);
        Context::free(ctx);
        Runtime::free(rt);
    }
}
