use crate::{
    class::ClassId,
    convert::{AsJsCString, AsJsClassId, AsJsContextPointer, AsJsRuntimePointer, AsJsValue},
    ffi::{self, c_size_t},
    flags::{EvalFlags, ParseJSONFlags},
    function::{convert_function_arguments, convert_function_result},
    marker::Covariant,
    runtime::Runtime,
    string::CString as QcCString,
    util,
    value::Value,
};
use std::{
    ffi::{c_void, CString},
    fmt,
    marker::PhantomData,
    os::raw::{c_char, c_int},
    ptr::NonNull,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Context<'q>(NonNull<ffi::JSContext>, Covariant<'q>);

impl<'q> Context<'q> {
    // lifecycle

    /// # Safety
    /// The pointer of a context must have a valid lifetime.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut ffi::JSContext) -> Context<'q> {
        Context(NonNull::new(ptr).unwrap(), PhantomData)
    }

    #[inline]
    pub fn new(rt: Runtime<'q>) -> Context<'q> {
        unsafe { Self::from_raw(ffi::JS_NewContext(rt.as_ptr())) }
    }

    /// # Safety
    /// You must free a context only once.
    #[inline]
    pub unsafe fn free(this: Self) {
        ffi::JS_FreeContext(this.0.as_ptr());
    }

    #[inline]
    pub fn dup(this: Self) -> Context<'q> {
        unsafe { Context::from_raw(ffi::JS_DupContext(this.0.as_ptr())) }
    }

    // basic

    #[inline]
    pub fn opaque(self) -> *mut c_void {
        unsafe { ffi::JS_GetContextOpaque(self.0.as_ptr()) }
    }

    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetContextOpaque(self.0.as_ptr(), opaque) }
    }

    #[inline]
    pub fn runtime(self) -> Runtime<'q> {
        unsafe { Runtime::from_raw(ffi::JS_GetRuntime(self.as_ptr())) }
    }

    #[inline]
    pub fn set_class_proto<P>(self, clz: ClassId, proto: P)
    where
        P: AsJsValue<'q>,
    {
        unsafe { ffi::JS_SetClassProto(self.as_ptr(), clz.as_js_class_id(), proto.as_js_value()) }
    }

    #[inline]
    pub fn class_proto(self, clz: ClassId) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_GetClassProto(self.as_ptr(), clz.as_js_class_id()), self) }
    }

    // value

    #[inline]
    pub fn new_bool(self, v: bool) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewBool(self.as_ptr(), v as ffi::JS_BOOL);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewInt32(self.as_ptr(), v);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewInt64(self.as_ptr(), v);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_float64(self, v: f64) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewFloat64(self.as_ptr(), v);
            Value::from_raw(value, self)
        }
    }

    // exception

    #[inline]
    pub fn throw(self, obj: Value<'q>) -> Value<'q> {
        let value = unsafe { ffi::JS_Throw(self.as_ptr(), obj.as_js_value()) };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn exception(self) -> Value<'q> {
        unsafe {
            let value = ffi::JS_GetException(self.as_ptr());
            Value::from_raw(value, self)
        }
    }

    // lifecycle (value)

    /// # Safety
    /// You must free a value only once.
    #[inline]
    pub unsafe fn free_value(self, value: Value<'q>) {
        ffi::JS_FreeValue(self.as_ptr(), value.as_js_value());
    }

    #[inline]
    pub fn dup_value(self, value: Value<'q>) {
        unsafe {
            ffi::JS_DupValue(self.as_ptr(), value.as_js_value());
        }
    }

    // string

    /// This function throws an exception if v is not UTF-8 buffer.
    #[inline]
    pub(crate) fn new_string_len(self, v: &[u8]) -> Value<'q> {
        let value = unsafe { ffi::JS_NewStringLen(self.as_ptr(), v.as_ptr() as *const c_char, v.len() as c_size_t) };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn new_string<T>(self, s: T) -> Value<'q>
    where
        T: AsRef<str>,
    {
        self.new_string_len(s.as_ref().as_bytes())
    }

    /// # Safety
    /// You must free a string only once.
    #[inline]
    pub unsafe fn free_c_string(self, str: QcCString<'q>) {
        ffi::JS_FreeCString(self.as_ptr(), str.as_js_c_string());
    }

    // object

    #[inline]
    pub fn new_object_proto_class<P>(self, proto: P, clz: ClassId) -> Value<'q>
    where
        P: AsJsValue<'q>,
    {
        unsafe {
            let value = ffi::JS_NewObjectProtoClass(self.as_ptr(), proto.as_js_value(), clz.as_js_class_id());
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_object_class(self, clz: ClassId) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewObjectClass(self.as_ptr(), clz.as_js_class_id() as i32);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_object_proto<P>(self, proto: P) -> Value<'q>
    where
        P: AsJsValue<'q>,
    {
        unsafe {
            let value = ffi::JS_NewObjectProto(self.as_ptr(), proto.as_js_value());
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_object(self) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_NewObject(self.as_ptr()), self) }
    }

    // call

    #[inline]
    pub fn call<F, T, A>(self, func_obj: F, this_obj: T, args: A) -> Value<'q>
    where
        F: AsJsValue<'q>,
        T: AsJsValue<'q>,
        A: AsRef<[Value<'q>]>,
    {
        let mut c_args: Vec<_> = args.as_ref().iter().map(|v| v.as_js_value()).collect();
        let value = unsafe {
            ffi::JS_Call(
                self.as_ptr(),
                func_obj.as_js_value(),
                this_obj.as_js_value(),
                c_args.len() as i32,
                c_args.as_mut_ptr(),
            )
        };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn eval<C, F>(self, code: C, filename: F, eval_flags: EvalFlags) -> Value<'q>
    where
        C: AsRef<str>,
        F: AsRef<str>,
    {
        let c_code = CString::new(code.as_ref()).expect("code");
        let c_filename = CString::new(filename.as_ref()).expect("filename");
        let value = unsafe {
            ffi::JS_Eval(
                self.as_ptr(),
                c_code.as_ptr(),
                c_code.as_bytes().len() as c_size_t,
                c_filename.as_ptr(),
                eval_flags.bits() as i32,
            )
        };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn global_object(self) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_GetGlobalObject(self.as_ptr()), self) }
    }

    // json

    #[inline]
    pub fn parse_json<B, F>(self, buf: B, filename: F) -> Value<'q>
    where
        B: AsRef<str>,
        F: AsRef<str>,
    {
        let c_buf = CString::new(buf.as_ref()).expect("buf");
        let c_filename = CString::new(filename.as_ref()).expect("filename");
        unsafe {
            let value = ffi::JS_ParseJSON(
                self.as_ptr(),
                c_buf.as_ptr(),
                c_buf.as_bytes().len() as c_size_t,
                c_filename.as_ptr(),
            );
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn parse_json2<B, F>(self, buf: B, filename: F, flags: ParseJSONFlags) -> Value<'q>
    where
        B: AsRef<str>,
        F: AsRef<str>,
    {
        let c_buf = CString::new(buf.as_ref()).expect("buf");
        let c_filename = CString::new(filename.as_ref()).expect("filename");
        unsafe {
            let value = ffi::JS_ParseJSON2(
                self.as_ptr(),
                c_buf.as_ptr(),
                c_buf.as_bytes().len() as c_size_t,
                c_filename.as_ptr(),
                flags.bits() as i32,
            );
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn json_stringify(self, obj: Value<'q>, replacer: Value<'q>, space0: Value<'q>) -> Value<'q> {
        unsafe {
            let value = ffi::JS_JSONStringify(
                self.as_ptr(),
                obj.as_js_value(),
                replacer.as_js_value(),
                space0.as_js_value(),
            );
            Value::from_raw(value, self)
        }
    }

    // array buffer

    #[inline]
    pub fn new_array_buffer_copy<B>(self, buf: B) -> Value<'q>
    where
        B: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        unsafe {
            let value = ffi::JS_NewArrayBufferCopy(self.as_ptr(), buf.as_ptr(), buf.len() as c_size_t);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_array_buffer_copy_from_sized<T: 'q>(self, t: T) -> Value<'q> {
        let buf = util::to_vec(t);
        self.new_array_buffer_copy(buf.as_slice())
    }

    // callback

    pub fn new_function<F>(self, func: F, length: i32) -> Value<'q>
    where
        F: Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> Value<'q> + Send + 'q,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), length)
    }

    fn new_callback(self, mut func: Box<Callback<'q, 'q>>, length: i32) -> Value<'q> {
        unsafe extern "C" fn call(
            ctx: *mut ffi::JSContext,
            js_this: ffi::JSValue,
            argc: c_int,
            argv: *mut ffi::JSValue,
            _magic: c_int,
            func_data: *mut ffi::JSValue,
        ) -> ffi::JSValue {
            let (ctx, this, args) = convert_function_arguments(ctx, js_this, argc, argv);
            let cb = Value::from_raw(*func_data, ctx);
            log::trace!("load pointer from ArrayBuffer");
            let func = cb.array_buffer_to_sized::<*mut Callback>(ctx).unwrap();
            let any = (**func)(ctx, this, args.as_slice());
            convert_function_result(&any)
        }
        log::trace!("save pointer to ArrayBuffer");
        let cb = self.new_array_buffer_copy_from_sized::<*mut Callback>(func.as_mut());
        if cb.is_exception() {
            return cb;
        }
        log::trace!("new c function data");
        self.new_c_function_data(Some(call), length, 0, vec![cb])
    }

    pub fn new_c_function(self, func: ffi::JSCFunction, name: &str, length: i32) -> Value<'q> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let value = ffi::JS_NewCFunction(self.as_ptr(), func, c_name.as_ptr(), length);
            Value::from_raw(value, self)
        }
    }

    pub fn new_c_function_data<D>(self, func: ffi::JSCFunctionData, length: i32, magic: i32, data: D) -> Value<'q>
    where
        D: Into<Vec<Value<'q>>>,
    {
        let mut js_data: Vec<_> = data.into().into_iter().map(|v| v.as_js_value()).collect();
        unsafe {
            let value = ffi::JS_NewCFunctionData(
                self.as_ptr(),
                func,
                length,
                magic,
                js_data.len() as i32,
                js_data.as_mut_ptr(),
            );
            Value::from_raw(value, self)
        }
    }
}

impl fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Context({:p})", self.0).as_str())
    }
}

impl<'q> AsJsContextPointer<'q> for Context<'q> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::JSContext {
        self.0.as_ptr()
    }
}

pub(crate) type Callback<'q, 'a> = dyn Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> Value<'q> + Send + 'a;
