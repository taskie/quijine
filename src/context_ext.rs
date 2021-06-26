use crate::context::Context;

pub trait ContextAddIntrinsicExt {
    fn add_intrinsic_base_objects(self);
    fn add_intrinsic_date(self);
    fn add_intrinsic_eval(self);
    fn add_intrinsic_string_normalize(self);
    fn add_intrinsic_reg_exp_compiler(self);
    fn add_intrinsic_reg_exp(self);
    fn add_intrinsic_json(self);
    fn add_intrinsic_proxy(self);
    fn add_intrinsic_map_set(self);
    fn add_intrinsic_typed_arrays(self);
    fn add_intrinsic_promise(self);
    fn add_intrinsic_big_int(self);
    fn add_intrinsic_big_float(self);
    fn add_intrinsic_big_decimal(self);
    fn add_intrinsic_operators(self);
    fn add_enable_bigint_ext(self, enable: bool);
}

macro_rules! fn_add_intrinsic {
    ($f:ident) => {
        #[inline]
        fn $f(self) {
            self.as_raw().$f();
        }
    };
}

impl ContextAddIntrinsicExt for Context<'_> {
    fn_add_intrinsic! { add_intrinsic_base_objects }

    fn_add_intrinsic! { add_intrinsic_date }

    fn_add_intrinsic! { add_intrinsic_eval }

    fn_add_intrinsic! { add_intrinsic_string_normalize }

    fn_add_intrinsic! { add_intrinsic_reg_exp_compiler }

    fn_add_intrinsic! { add_intrinsic_reg_exp }

    fn_add_intrinsic! { add_intrinsic_json }

    fn_add_intrinsic! { add_intrinsic_proxy }

    fn_add_intrinsic! { add_intrinsic_map_set }

    fn_add_intrinsic! { add_intrinsic_typed_arrays }

    fn_add_intrinsic! { add_intrinsic_promise }

    fn_add_intrinsic! { add_intrinsic_big_int }

    fn_add_intrinsic! { add_intrinsic_big_float }

    fn_add_intrinsic! { add_intrinsic_big_decimal }

    fn_add_intrinsic! { add_intrinsic_operators }

    fn add_enable_bigint_ext(self, enable: bool) {
        self.as_raw().add_enable_bigint_ext(enable);
    }
}

#[cfg(test)]
mod tests {
    use quijine_core::EvalFlags;

    use crate::{ContextScope, Result, RuntimeScope};
    #[test]
    fn test() -> Result<()> {
        use crate::ContextAddIntrinsicExt;
        let rts = RuntimeScope::new();
        let ctxs = ContextScope::new_raw(rts.get());
        let ctx = ctxs.get();
        assert!(ctx.eval("", "<input>", EvalFlags::TYPE_GLOBAL).is_err());
        ctx.add_intrinsic_base_objects();
        ctx.add_intrinsic_eval();
        let v: String = ctx.eval_into("typeof Date", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("undefined", &v);
        ctx.add_intrinsic_date();
        let v: String = ctx.eval_into("typeof new Date()", "<input>", EvalFlags::TYPE_GLOBAL)?;
        assert_eq!("object", &v);
        Ok(())
    }
}
