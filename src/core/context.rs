use std::{
    ffi::CString,
    fmt,
    marker::PhantomData,
    mem::size_of,
    os::raw::{c_char, c_int},
    ptr::NonNull,
    slice,
};

use crate::core::{
    class::ClassID,
    conversion::AsJSValue,
    ffi,
    marker::Covariant,
    runtime::{AsJSRuntimePointer, Runtime},
    string::CString as CoreCString,
    util,
    value::Value,
};

pub trait AsJSContextPointer<'q> {
    fn as_ptr(&self) -> *mut ffi::JSContext;
}

impl AsJSContextPointer<'_> for *mut ffi::JSContext {
    fn as_ptr(&self) -> *mut ffi::JSContext {
        *self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Context<'q>(NonNull<ffi::JSContext>, Covariant<'q>);

impl<'q> Context<'q> {
    #[inline]
    pub unsafe fn from_ptr(ptr: *mut ffi::JSContext) -> Context<'q> {
        Context(NonNull::new(ptr).unwrap(), PhantomData)
    }

    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut ffi::JSContext) -> Context<'q> {
        Context(NonNull::new_unchecked(ptr), PhantomData)
    }

    #[inline]
    pub fn new(rt: Runtime<'q>) -> Context<'q> {
        unsafe { Self::from_ptr(ffi::JS_NewContext(rt.as_ptr())) }
    }

    #[inline]
    pub unsafe fn free(this: Self) {
        ffi::JS_FreeContext(this.0.as_ptr());
    }

    #[inline]
    pub unsafe fn raw(this: Self) -> *mut ffi::JSContext {
        this.0.as_ptr()
    }

    #[inline]
    pub fn runtime(self) -> Runtime<'q> {
        unsafe { Runtime::from_ptr(ffi::JS_GetRuntime(self.as_ptr())) }
    }

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

    #[inline]
    pub unsafe fn free_c_string(self, str: CoreCString<'q>) {
        ffi::JS_FreeCString(self.as_ptr(), CoreCString::raw(str));
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Value<'q> {
        let c_code = CString::new(code).expect("code");
        let c_filename = CString::new(filename).expect("filename");
        let value = unsafe {
            ffi::JS_Eval(
                self.as_ptr(),
                c_code.as_ptr(),
                c_code.as_bytes().len() as u64,
                c_filename.as_ptr(),
                eval_flags.bits() as i32,
            )
        };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn call(self, func_obj: Value, this_obj: Value, args: &[Value]) -> Value<'q> {
        let mut c_args = Vec::with_capacity(args.len());
        for arg in args {
            c_args.push(arg.as_js_value())
        }
        let value = unsafe {
            ffi::JS_Call(
                self.as_ptr(),
                func_obj.as_js_value(),
                this_obj.as_js_value(),
                c_args.len() as i32,
                c_args.as_mut_ptr(),
            )
        };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn global_object(self) -> Value<'q> {
        Value::from_raw(unsafe { ffi::JS_GetGlobalObject(self.as_ptr()) }, self)
    }

    #[inline]
    pub fn new_object(self) -> Value<'q> {
        Value::from_raw(unsafe { ffi::JS_NewObject(self.as_ptr()) }, self)
    }

    #[inline]
    pub fn new_object_class(self, clz: ClassID) -> Value<'q> {
        let value = unsafe { ffi::JS_NewObjectClass(self.as_ptr(), ClassID::raw(clz) as i32) };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn set_class_proto(self, clz: ClassID, proto: Value<'q>) {
        unsafe { ffi::JS_SetClassProto(self.as_ptr(), ClassID::raw(clz), proto.as_js_value()) }
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Value<'q> {
        let value = unsafe { ffi::JS_NewInt32(self.as_ptr(), v) };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Value<'q> {
        let value = unsafe { ffi::JS_NewInt64(self.as_ptr(), v) };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn new_string_from_bytes(self, v: &[u8]) -> Value<'q> {
        let value = unsafe { ffi::JS_NewStringLen(self.as_ptr(), v.as_ptr() as *const c_char, v.len() as u64) };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn new_string<T>(self, s: T) -> Value<'q>
    where
        T: AsRef<str>,
    {
        self.new_string_from_bytes(s.as_ref().as_bytes())
    }

    // exception

    #[inline]
    pub fn exception(self) -> Value<'q> {
        let value = unsafe { ffi::JS_GetException(self.as_ptr()) };
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn throw(self, obj: Value<'q>) -> Value<'q> {
        let value = unsafe { ffi::JS_Throw(self.as_ptr(), obj.as_js_value()) };
        Value::from_raw(value, self)
    }

    // callback

    pub fn new_function<F>(self, func: F, name: &str, length: i32) -> Value<'q>
    where
        F: 'static + Send + Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> Value<'q>,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), name, length)
    }

    pub fn new_callback(self, func: Callback<'q, 'static>, _name: &str, length: i32) -> Value<'q> {
        unsafe extern "C" fn call(
            ctx: *mut ffi::JSContext,
            js_this: ffi::JSValue,
            argc: c_int,
            argv: *mut ffi::JSValue,
            _magic: c_int,
            func_data: *mut ffi::JSValue,
        ) -> ffi::JSValue {
            let ctx = Context::from_ptr(ctx);
            let this = Value::from_raw(js_this, ctx);
            let mut args: Vec<Value> = Vec::with_capacity(argc as usize);
            for i in 0..argc {
                let p = argv.offset(i as isize);
                let any = Value::from_raw(*p, ctx);
                args.push(any);
            }
            let cb = Value::from_raw(*func_data, ctx);
            log::debug!("load pointer from ArrayBuffer");
            let func = ctx.array_buffer_to_sized::<Callback>(&cb).unwrap();
            let any = (*func)(ctx, this, args.as_slice());
            (&any).as_js_value()
        };
        unsafe {
            log::debug!("save pointer to ArrayBuffer");
            let cb = self.new_array_buffer_copy_from_sized::<Callback>(func);
            log::debug!("new c function data");
            self.new_c_function_data(Some(call), length, 0, vec![cb])
        }
    }

    pub(crate) unsafe fn new_c_function(self, func: ffi::JSCFunction, name: &str, length: i32) -> Value<'q> {
        let c_name = CString::new(name).unwrap();
        let value = ffi::JS_NewCFunction(self.as_ptr(), func, c_name.as_ptr(), length);
        Value::from_raw(value, self)
    }

    pub(crate) unsafe fn new_c_function_data(
        self,
        func: ffi::JSCFunctionData,
        length: i32,
        magic: i32,
        data: Vec<Value<'q>>,
    ) -> Value<'q> {
        let mut js_data = Vec::with_capacity(data.len());
        for datum in &data {
            js_data.push(datum.as_js_value());
        }
        let value = ffi::JS_NewCFunctionData(
            self.as_ptr(),
            func,
            length,
            magic,
            data.len() as i32,
            js_data.as_mut_ptr(),
        );
        Value::from_raw(value, self)
    }

    pub(crate) unsafe fn new_array_buffer_copy(self, t: &[u8]) -> Value<'q> {
        let value = ffi::JS_NewArrayBufferCopy(self.as_ptr(), t.as_ptr(), t.len() as u64);
        Value::from_raw(value, self)
    }

    pub(crate) unsafe fn new_array_buffer_copy_from_sized<T>(self, t: T) -> Value<'q> {
        let buf = util::to_vec(t);
        self.new_array_buffer_copy(buf.as_slice())
    }

    pub(crate) unsafe fn array_buffer<'v>(self, v: &'v Value<'q>) -> Option<&'v [u8]> {
        let mut len = 0;
        let bs: *const u8 = ffi::JS_GetArrayBuffer(self.as_ptr(), &mut len, v.as_js_value());
        if bs.is_null() {
            return None;
        }
        Some(slice::from_raw_parts(bs, len as usize))
    }

    pub(crate) unsafe fn array_buffer_to_sized<'v, T>(self, v: &'v Value<'q>) -> Option<&'v T> {
        let mut len = 0;
        let bs: *const u8 = ffi::JS_GetArrayBuffer(self.as_ptr(), &mut len, v.as_js_value());
        if bs.is_null() {
            return None;
        }
        assert_eq!(size_of::<T>(), len as usize);
        Some(&*(bs as *const T))
    }
}

impl fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Context({:p})", self.0).as_str())
    }
}

impl<'q> AsJSContextPointer<'q> for Context<'q> {
    fn as_ptr(&self) -> *mut ffi::JSContext {
        self.0.as_ptr()
    }
}

bitflags! {
    pub struct EvalFlags: u32 {
        /// global code (default)
        const TYPE_GLOBAL = ffi::JS_EVAL_TYPE_GLOBAL;
        /// module code
        const TYPE_MODULE = ffi::JS_EVAL_TYPE_MODULE;
        // direct call (internal use)
        // const TypeDirect = ffi::JS_EVAL_TYPE_DIRECT;
        // indirect call (internal use)
        // const TypeInDirect = ffi::JS_EVAL_TYPE_INDIRECT;
        const TYPE_MASK = ffi::JS_EVAL_TYPE_MASK;

        /// force 'strict' mode
        const FLAG_STRICT = ffi::JS_EVAL_FLAG_STRICT;
        /// force 'strip' mode
        const FLAG_STRIP = ffi::JS_EVAL_FLAG_STRIP;
        /// compile but do not run. The result is an object with a
        /// JS_TAG_FUNCTION_BYTECODE or JS_TAG_MODULE tag. It can be executed
        /// with JS_EvalFunction().
        const FLAG_COMPILE_ONLY = ffi::JS_EVAL_FLAG_COMPILE_ONLY;
    }
}

pub(crate) type Callback<'q, 'a> = Box<dyn Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> Value<'q> + 'a>;
