use crate::{
    atom::Atom,
    class::ClassId,
    convert::{AsJsClassId, AsJsValue, AsMutPtr, AsPtr},
    enums::CFunctionEnum,
    ffi::{self, c_size_t},
    flags::{EvalFlags, ParseJSONFlags, ReadObjFlags, WriteObjFlags},
    function::{convert_function_arguments, convert_function_result},
    internal::ref_sized_to_slice,
    marker::Covariant,
    runtime::Runtime,
    string::CString as QcCString,
    value::Value,
    AsJsAtom,
};
use std::{
    ffi::{c_void, CStr, CString},
    fmt,
    marker::PhantomData,
    mem::size_of,
    os::raw::{c_char, c_int},
    ptr::{null_mut, NonNull},
    slice,
};

macro_rules! def_throw_error {
    ($name: ident, $f: expr) => {
        #[inline]
        pub fn $name(self, message: &str) -> Value<'q> {
            unsafe {
                let fmt = CStr::from_bytes_with_nul_unchecked(b"%s\0");
                let message = message.to_owned() + "\0";
                let message = CStr::from_bytes_with_nul_unchecked(message.as_bytes());
                let value = $f(self.0.as_ptr(), fmt.as_ptr(), message.as_ptr());
                Value::from_raw(value, self)
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Context<'q>(NonNull<ffi::JSContext>, Covariant<'q>);

impl<'q> Context<'q> {
    def_throw_error!(throw_syntax_error, ffi::JS_ThrowSyntaxError);

    def_throw_error!(throw_type_error, ffi::JS_ThrowTypeError);

    def_throw_error!(throw_reference_error, ffi::JS_ThrowReferenceError);

    def_throw_error!(throw_range_error, ffi::JS_ThrowRangeError);

    def_throw_error!(throw_internal_error, ffi::JS_ThrowInternalError);

    // lifecycle

    /// # Safety
    /// The pointer of a context must have a valid lifetime.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut ffi::JSContext) -> Context<'q> {
        Context(NonNull::new(ptr).unwrap(), PhantomData)
    }

    #[inline]
    pub fn new(mut rt: Runtime<'q>) -> Context<'q> {
        unsafe { Self::from_raw(ffi::JS_NewContext(rt.as_mut_ptr())) }
    }

    #[inline]
    pub fn new_raw(mut rt: Runtime<'q>) -> Context<'q> {
        unsafe { Self::from_raw(ffi::JS_NewContextRaw(rt.as_mut_ptr())) }
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
        unsafe { Runtime::from_raw(ffi::JS_GetRuntime(self.0.as_ptr())) }
    }

    #[inline]
    pub fn set_class_proto<P>(self, clz: ClassId, proto: P)
    where
        P: AsJsValue<'q>,
    {
        unsafe { ffi::JS_SetClassProto(self.0.as_ptr(), clz.as_js_class_id(), proto.as_js_value()) }
    }

    #[inline]
    pub fn class_proto(self, clz: ClassId) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_GetClassProto(self.0.as_ptr(), clz.as_js_class_id()), self) }
    }

    // value

    #[inline]
    pub fn new_bool(self, v: bool) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewBool(self.0.as_ptr(), v as ffi::JS_BOOL);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewInt32(self.0.as_ptr(), v);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewInt64(self.0.as_ptr(), v);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_float64(self, v: f64) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewFloat64(self.0.as_ptr(), v);
            Value::from_raw(value, self)
        }
    }

    // exception

    #[inline]
    pub fn throw(self, obj: Value<'q>) -> Value<'q> {
        let value = unsafe { ffi::JS_Throw(self.0.as_ptr(), obj.as_js_value()) };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn exception(self) -> Value<'q> {
        unsafe {
            let value = ffi::JS_GetException(self.0.as_ptr());
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn reset_uncacheable_error(self) {
        unsafe { ffi::JS_ResetUncatchableError(self.0.as_ptr()) }
    }

    #[inline]
    pub fn new_error(self) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewError(self.0.as_ptr());
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn throw_out_of_memory(self) -> Value<'q> {
        unsafe {
            let value = ffi::JS_ThrowOutOfMemory(self.0.as_ptr());
            Value::from_raw(value, self)
        }
    }

    // lifecycle (value)

    /// # Safety
    /// You must free a value only once.
    #[inline]
    pub unsafe fn free_value(self, value: Value<'q>) {
        ffi::JS_FreeValue(self.0.as_ptr(), value.as_js_value());
    }

    #[inline]
    pub fn dup_value(self, value: Value<'q>) {
        unsafe {
            ffi::JS_DupValue(self.0.as_ptr(), value.as_js_value());
        }
    }

    // string

    /// This function throws an exception if v is not UTF-8 buffer.
    #[inline]
    pub(crate) fn new_string_len(self, v: &[u8]) -> Value<'q> {
        let value = unsafe { ffi::JS_NewStringLen(self.0.as_ptr(), v.as_ptr() as *const c_char, v.len() as c_size_t) };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn new_string(self, s: &str) -> Value<'q> {
        self.new_string_len(s.as_bytes())
    }

    /// # Safety
    /// You must free a string only once.
    #[inline]
    pub unsafe fn free_c_string(self, str: QcCString<'q>) {
        ffi::JS_FreeCString(self.0.as_ptr(), str.as_ptr());
    }

    // atom

    #[inline]
    pub(crate) fn new_atom_len(self, v: &[u8]) -> Atom<'q> {
        let atom = unsafe { ffi::JS_NewAtomLen(self.0.as_ptr(), v.as_ptr() as *const c_char, v.len() as c_size_t) };
        unsafe { Atom::from_raw(atom, self) }
    }

    #[inline]
    pub fn new_atom(self, s: &str) -> Atom<'q> {
        self.new_atom_len(s.as_bytes())
    }

    #[inline]
    pub fn new_atom_u_int32(self, n: u32) -> Atom<'q> {
        let atom = unsafe { ffi::JS_NewAtomUInt32(self.0.as_ptr(), n) };
        unsafe { Atom::from_raw(atom, self) }
    }

    #[inline]
    pub fn dup_atom(self, atom: Atom<'q>) -> Atom<'q> {
        let atom = unsafe { ffi::JS_DupAtom(self.0.as_ptr(), atom.as_js_atom()) };
        unsafe { Atom::from_raw(atom, self) }
    }

    #[inline]
    pub fn free_atom(self, atom: Atom<'q>) {
        unsafe { ffi::JS_FreeAtom(self.0.as_ptr(), atom.as_js_atom()) };
    }

    #[inline]
    pub fn atom_to_value(self, atom: Atom<'q>) -> Value<'q> {
        let v = unsafe { ffi::JS_AtomToValue(self.0.as_ptr(), atom.as_js_atom()) };
        unsafe { Value::from_raw(v, self) }
    }

    #[inline]
    pub fn atom_to_string(self, atom: Atom<'q>) -> Value<'q> {
        let v = unsafe { ffi::JS_AtomToString(self.0.as_ptr(), atom.as_js_atom()) };
        unsafe { Value::from_raw(v, self) }
    }

    #[inline]
    pub fn value_to_atom(self, v: Value<'q>) -> Atom<'q> {
        let atom = unsafe { ffi::JS_ValueToAtom(self.0.as_ptr(), v.as_js_value()) };
        unsafe { Atom::from_raw(atom, self) }
    }

    // object

    #[inline]
    pub fn new_object_proto_class<P>(self, proto: P, clz: ClassId) -> Value<'q>
    where
        P: AsJsValue<'q>,
    {
        unsafe {
            let value = ffi::JS_NewObjectProtoClass(self.0.as_ptr(), proto.as_js_value(), clz.as_js_class_id());
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_object_class(self, clz: ClassId) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewObjectClass(self.0.as_ptr(), clz.as_js_class_id() as i32);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_object_proto<P>(self, proto: P) -> Value<'q>
    where
        P: AsJsValue<'q>,
    {
        unsafe {
            let value = ffi::JS_NewObjectProto(self.0.as_ptr(), proto.as_js_value());
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_object(self) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_NewObject(self.0.as_ptr()), self) }
    }

    #[inline]
    pub fn new_array(self) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_NewArray(self.0.as_ptr()), self) }
    }

    // call

    #[inline]
    pub fn call<F, T, A>(self, func_obj: F, this_obj: T, args: &[A]) -> Value<'q>
    where
        F: AsJsValue<'q>,
        T: AsJsValue<'q>,
        A: AsJsValue<'q>,
    {
        let mut c_args: Vec<_> = args.as_ref().iter().map(|v| v.as_js_value()).collect();
        let value = unsafe {
            ffi::JS_Call(
                self.0.as_ptr(),
                func_obj.as_js_value(),
                this_obj.as_js_value(),
                c_args.len() as i32,
                c_args.as_mut_ptr(),
            )
        };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Value<'q> {
        let c_code = CString::new(code).expect("code");
        let c_filename = CString::new(filename).expect("filename");
        let value = unsafe {
            ffi::JS_Eval(
                self.0.as_ptr(),
                c_code.as_ptr(),
                c_code.as_bytes().len() as c_size_t,
                c_filename.as_ptr(),
                eval_flags.bits() as i32,
            )
        };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn eval_function(self, func_obj: Value) -> Value<'q> {
        let value = unsafe { ffi::JS_EvalFunction(self.0.as_ptr(), func_obj.as_js_value()) };
        unsafe { Value::from_raw(value, self) }
    }

    #[inline]
    pub fn global_object(self) -> Value<'q> {
        unsafe { Value::from_raw(ffi::JS_GetGlobalObject(self.0.as_ptr()), self) }
    }

    // json

    #[inline]
    pub fn parse_json(self, buf: &str, filename: &str) -> Value<'q> {
        let c_buf = CString::new(buf).expect("buf");
        let c_filename = CString::new(filename).expect("filename");
        unsafe {
            let value = ffi::JS_ParseJSON(
                self.0.as_ptr(),
                c_buf.as_ptr(),
                c_buf.as_bytes().len() as c_size_t,
                c_filename.as_ptr(),
            );
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn parse_json2(self, buf: &str, filename: &str, flags: ParseJSONFlags) -> Value<'q> {
        let c_buf = CString::new(buf).expect("buf");
        let c_filename = CString::new(filename).expect("filename");
        unsafe {
            let value = ffi::JS_ParseJSON2(
                self.0.as_ptr(),
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
                self.0.as_ptr(),
                obj.as_js_value(),
                replacer.as_js_value(),
                space0.as_js_value(),
            );
            Value::from_raw(value, self)
        }
    }

    // array buffer

    /// # Safety
    /// `buf`, `len` and `free_func` must match
    #[inline]
    pub unsafe fn new_array_buffer(
        self,
        buf: *mut u8,
        len: usize,
        free_func: ffi::JSFreeArrayBufferDataFunc,
        opaque: *mut c_void,
        is_shared: bool,
    ) -> Value<'q> {
        let value = ffi::JS_NewArrayBuffer(
            self.0.as_ptr(),
            buf,
            len as c_size_t,
            free_func,
            opaque,
            is_shared as i32,
        );
        Value::from_raw(value, self)
    }

    #[inline]
    pub fn new_array_buffer_from_boxed<T>(self, boxed: Box<T>) -> Value<'q> {
        let len = size_of::<T>();
        let ptr = Box::into_raw(boxed);
        unsafe extern "C" fn free_func<T>(_rrt: *mut ffi::JSRuntime, _opaque: *mut c_void, ptr: *mut c_void) {
            let _box = Box::from_raw(ptr as *mut T);
            // dropped
        }
        unsafe { self.new_array_buffer(ptr as *mut u8, len, Some(free_func::<T>), null_mut(), false) }
    }

    #[inline]
    pub fn new_array_buffer_copy(self, buf: &[u8]) -> Value<'q> {
        unsafe {
            let value = ffi::JS_NewArrayBufferCopy(self.0.as_ptr(), buf.as_ptr(), buf.len() as c_size_t);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_array_buffer_copy_from_ref<T>(self, t: &T) -> Value<'q> {
        self.new_array_buffer_copy(ref_sized_to_slice(t))
    }

    // callback

    #[inline]
    pub fn new_function<F>(self, func: F, length: i32) -> Value<'q>
    where
        F: Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> Value<'q> + 'q,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), length)
    }

    #[inline]
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
            let func = cb.array_buffer_as_ref::<*mut Callback>(ctx).unwrap();
            let any = (**func)(ctx, this, args.as_slice());
            convert_function_result(any)
        }
        log::trace!("save pointer to ArrayBuffer");
        let cb = self.new_array_buffer_copy_from_ref(&func.as_mut());
        if cb.is_exception() {
            return cb;
        }
        log::trace!("new c function data");
        self.new_c_function_data(Some(call), length, 0, &[cb])
    }

    #[inline]
    pub fn new_c_function(self, func: ffi::JSCFunction, name: &str, length: i32) -> Value<'q> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let value = ffi::JS_NewCFunction(self.0.as_ptr(), func, c_name.as_ptr(), length);
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_c_function2(
        self,
        func: ffi::JSCFunction,
        name: &str,
        length: i32,
        cproto: CFunctionEnum,
        magic: i32,
    ) -> Value<'q> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let value = ffi::JS_NewCFunction2(
                self.0.as_ptr(),
                func,
                c_name.as_ptr(),
                length,
                cproto as u32,
                magic as c_int,
            );
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_c_function_magic(
        self,
        func: ffi::JSCFunctionMagic,
        name: &str,
        length: i32,
        cproto: CFunctionEnum,
        magic: i32,
    ) -> Value<'q> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let value = ffi::JS_NewCFunctionMagic(
                self.0.as_ptr(),
                func,
                c_name.as_ptr(),
                length,
                cproto as u32,
                magic as c_int,
            );
            Value::from_raw(value, self)
        }
    }

    #[inline]
    pub fn new_c_function_data(
        self,
        func: ffi::JSCFunctionData,
        length: i32,
        magic: i32,
        data: &[Value<'q>],
    ) -> Value<'q> {
        let mut js_data: Vec<_> = data.iter().map(|v| v.as_js_value()).collect();
        unsafe {
            let value = ffi::JS_NewCFunctionData(
                self.0.as_ptr(),
                func,
                length,
                magic,
                js_data.len() as i32,
                js_data.as_mut_ptr(),
            );
            Value::from_raw(value, self)
        }
    }

    // intrinsics

    #[inline]
    pub fn add_intrinsic_base_objects(self) {
        unsafe { ffi::JS_AddIntrinsicBaseObjects(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_date(self) {
        unsafe { ffi::JS_AddIntrinsicDate(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_eval(self) {
        unsafe { ffi::JS_AddIntrinsicEval(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_string_normalize(self) {
        unsafe { ffi::JS_AddIntrinsicStringNormalize(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_reg_exp_compiler(self) {
        unsafe { ffi::JS_AddIntrinsicRegExpCompiler(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_reg_exp(self) {
        unsafe { ffi::JS_AddIntrinsicRegExp(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_json(self) {
        unsafe { ffi::JS_AddIntrinsicJSON(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_proxy(self) {
        unsafe { ffi::JS_AddIntrinsicProxy(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_map_set(self) {
        unsafe { ffi::JS_AddIntrinsicMapSet(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_typed_arrays(self) {
        unsafe { ffi::JS_AddIntrinsicTypedArrays(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_promise(self) {
        unsafe { ffi::JS_AddIntrinsicPromise(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_big_int(self) {
        unsafe { ffi::JS_AddIntrinsicBigInt(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_big_float(self) {
        unsafe { ffi::JS_AddIntrinsicBigFloat(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_big_decimal(self) {
        unsafe { ffi::JS_AddIntrinsicBigDecimal(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_intrinsic_operators(self) {
        unsafe { ffi::JS_AddIntrinsicOperators(self.0.as_ptr()) }
    }

    #[inline]
    pub fn add_enable_bigint_ext(self, enable: bool) {
        unsafe { ffi::JS_EnableBignumExt(self.0.as_ptr(), enable as c_int) }
    }

    // Object Writer/Reader

    #[inline]
    pub fn write_object(self, obj: Value, flags: WriteObjFlags) -> Option<Vec<u8>> {
        let mut size: c_size_t = 0;
        unsafe {
            let buf = ffi::JS_WriteObject(self.0.as_ptr(), &mut size, obj.as_js_value(), flags.bits() as i32);
            if buf.is_null() {
                return None;
            }
            let vec = slice::from_raw_parts(buf, size as usize).to_vec();
            ffi::js_free(self.0.as_ptr(), buf as *mut _);
            Some(vec)
        }
    }

    #[inline]
    pub fn read_object(self, buf: &[u8], flags: ReadObjFlags) -> Value<'q> {
        unsafe {
            let value = ffi::JS_ReadObject(self.0.as_ptr(), buf.as_ptr(), buf.len() as u64, flags.bits() as i32);
            Value::from_raw(value, self)
        }
    }
}

impl fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Context({:p})", self.0).as_str())
    }
}

impl<'q> AsPtr<ffi::JSContext> for Context<'q> {
    #[inline]
    fn as_ptr(&self) -> *const ffi::JSContext {
        self.0.as_ptr()
    }
}

impl<'q> AsMutPtr<ffi::JSContext> for Context<'q> {
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut ffi::JSContext {
        self.0.as_ptr()
    }
}

pub(crate) type Callback<'q, 'a> = dyn Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> Value<'q> + 'a;
