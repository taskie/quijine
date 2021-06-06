use crate::ffi;
use bitflags::bitflags;

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
