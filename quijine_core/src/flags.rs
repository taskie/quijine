use crate::ffi;
use bitflags::bitflags;

bitflags! {
    /// flags for object properties
    pub struct PropFlags: u32 {
        const CONFIGURABLE = ffi::JS_PROP_CONFIGURABLE;
        const WRITABLE = ffi::JS_PROP_WRITABLE;
        const ENUMERABLE = ffi::JS_PROP_ENUMERABLE;
        const C_W_E = ffi::JS_PROP_C_W_E;
        // used internally in Arrays
        // const LENGTH = ffi::JS_PROP_LENGTH;
        /// mask for NORMAL, GETSET, VARREF, AUTOINIT
        const TMASK = ffi::JS_PROP_TMASK;
        const NORMAL = ffi::JS_PROP_NORMAL;
        const GETSET = ffi::JS_PROP_GETSET;
        // used internally
        // const VARREF = ffi::JS_PROP_VARREF;
        // used internally
        // const AUTOINIT = ffi::JS_PROP_AUTOINIT;
        /// flags for JS_DefineProperty
        const HAS_CONFIGURABLE = ffi::JS_PROP_HAS_CONFIGURABLE;
        /// flags for JS_DefineProperty
        const HAS_WRITABLE = ffi::JS_PROP_HAS_WRITABLE;
        /// flags for JS_DefineProperty
        const HAS_ENUMERABLE = ffi::JS_PROP_HAS_ENUMERABLE;
        /// flags for JS_DefineProperty
        const HAS_GET = ffi::JS_PROP_HAS_GET;
        /// flags for JS_DefineProperty
        const HAS_SET = ffi::JS_PROP_HAS_SET;
        /// flags for JS_DefineProperty
        const HAS_VALUE = ffi::JS_PROP_HAS_VALUE;
        /// throw an exception if false would be returned (JS_DefineProperty/JS_SetProperty)
        const THROW = ffi::JS_PROP_THROW;
        /// throw an exception if false would be returned in strict mode (JS_SetProperty)
        const THROW_STRICT = ffi::JS_PROP_THROW_STRICT;
        // internal use
        // const NO_ADD = ffi::JS_PROP_NO_ADD;
        // internal use
        // const NO_EXOTIC = ffi::JS_PROP_NO_EXOTIC;
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
        /// don't include the stack frames before this eval in the Error() backtraces
        const FLAG_BACKTRACE_BARRIER = ffi::JS_EVAL_FLAG_BACKTRACE_BARRIER;
    }
}

bitflags! {
    pub struct ParseJSONFlags: u32 {
        const EXT = 0b0001;
    }
}

bitflags! {
    pub struct GPNFlags: u32 {
        const STRING_MASK = ffi::JS_GPN_STRING_MASK;
        const SYMBOL_MASK = ffi::JS_GPN_SYMBOL_MASK;
        const PRIVATE_MASK = ffi::JS_GPN_PRIVATE_MASK;
        /// only include the enumerable properties
        const ENUM_ONLY = ffi::JS_GPN_ENUM_ONLY;
        /// set the JSPropertyEnum.is_enumerable field
        const SET_ENUM = ffi::JS_GPN_SET_ENUM;
    }
}

bitflags! {
    pub struct WriteObjFlags: u32 {
        /// allow function/module
        const BYTECODE = ffi::JS_WRITE_OBJ_BYTECODE;
        ///  byte swapped output
        const BSWAP = ffi::JS_WRITE_OBJ_BSWAP;
        /// allow SharedArrayBuffer
        const SAB = ffi::JS_WRITE_OBJ_SAB;
        /// allow object references to encode arbitrary object graph
        const REFERENCE = ffi::JS_WRITE_OBJ_REFERENCE;
    }
}

bitflags! {
    pub struct ReadObjFlags: u32 {
        /// allow function/module
        const BYTECODE = ffi::JS_READ_OBJ_BYTECODE;
        /// avoid duplicating 'buf' data
        const ROM_DATA = ffi::JS_READ_OBJ_ROM_DATA;
        /// allow SharedArrayBuffer
        const SAB = ffi::JS_READ_OBJ_SAB;
        /// allow object references
        const REFERENCE = ffi::JS_READ_OBJ_REFERENCE;
    }
}
