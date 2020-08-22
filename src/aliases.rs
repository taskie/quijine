use crate::{
    instance::{Qj, QjVec},
    types::{
        QjAnyTag, QjBigDecimalTag, QjBigFloatTag, QjBigIntTag, QjBoolTag, QjCatchOffsetTag, QjExceptionTag,
        QjFloat64Tag, QjFunctionBytecodeTag, QjIntTag, QjModuleTag, QjNullTag, QjObjectTag, QjReferenceTag,
        QjStringTag, QjSymbolTag, QjUndefinedTag, QjUninitializedTag, QjValueTag,
    },
};

macro_rules! qj_define_aliases {
    ($tag: ident, $single: ident) => {
        pub type $single<'q> = Qj<'q, $tag>;
    };
    ($tag: ident, $single: ident, $vec: ident) => {
        pub type $single<'q> = Qj<'q, $tag>;
        pub type $vec<'q> = QjVec<'q, $tag>;
    };
}

// any
qj_define_aliases!(QjAnyTag, QjAny, QjAnyVec);

// references
qj_define_aliases!(QjReferenceTag, QjReference);

qj_define_aliases!(QjBigDecimalTag, QjBigDecimal);
qj_define_aliases!(QjBigIntTag, QjBigInt);
qj_define_aliases!(QjBigFloatTag, QjBigFloat);
qj_define_aliases!(QjSymbolTag, QjSymbol);
qj_define_aliases!(QjStringTag, QjString);
qj_define_aliases!(QjModuleTag, QjModule);
qj_define_aliases!(QjFunctionBytecodeTag, QjFunctionBytecode);
qj_define_aliases!(QjObjectTag, QjObject);

// values
qj_define_aliases!(QjValueTag, QjValue);

qj_define_aliases!(QjIntTag, QjInt);
qj_define_aliases!(QjBoolTag, QjBool);
qj_define_aliases!(QjNullTag, QjNull);
qj_define_aliases!(QjUndefinedTag, QjUndefined);
qj_define_aliases!(QjUninitializedTag, QjUninitialized);
qj_define_aliases!(QjCatchOffsetTag, QjCatchOffset);
qj_define_aliases!(QjExceptionTag, QjException);
qj_define_aliases!(QjFloat64Tag, QjFloat64);
