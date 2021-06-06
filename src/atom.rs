use crate::{
    context::Context,
    convert::{AsData, FromQj, IntoQjAtom},
    data::Data,
    error::Result,
    types::String as QjString,
};
use qc::AsJsAtom;
use qjncore as qc;
use std::{fmt, result::Result as StdResult};

/// `Atom` is a atom holder with a context.
pub struct Atom<'q> {
    atom: qc::Atom<'q>,
    context: qc::Context<'q>,
}

impl<'q> Atom<'q> {
    pub(crate) fn from_raw_parts(atom: qc::Atom<'q>, context: qc::Context<'q>) -> Atom<'q> {
        Atom { atom, context }
    }

    // property

    #[inline]
    pub(crate) fn as_raw(&self) -> &qc::Atom<'q> {
        &self.atom
    }

    #[inline]
    pub(crate) fn context(&self) -> Context<'q> {
        Context::from_raw(self.context)
    }

    // memory

    #[inline]
    pub(crate) unsafe fn free(this: &Self) {
        this.context.free_atom(this.atom);
    }

    #[inline]
    pub(crate) fn dup(this: &Self) {
        this.context.dup_atom(this.atom);
    }

    // data

    pub fn to_data(&self) -> Result<Data<'q>> {
        self.context().atom_to_data(self)
    }

    pub fn to_string(&self) -> Result<QjString<'q>> {
        self.context().atom_to_string(self)
    }
}

impl Drop for Atom<'_> {
    fn drop(&mut self) {
        unsafe { Self::free(self) }
    }
}

impl Clone for Atom<'_> {
    fn clone(&self) -> Self {
        let atom = Atom {
            atom: self.atom,
            context: self.context,
        };
        Atom::dup(&atom);
        atom
    }
}

impl fmt::Debug for Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_str(&format!(
            "Atom({}; {:?})",
            self.atom.as_js_atom(),
            self.to_string()
                .and_then(|v| String::from_qj(v.clone().into()))
                .unwrap()
        ))
    }
}

impl<'q, T: AsData<'q>> IntoQjAtom<'q> for T {
    fn into_qj_atom(self, _ctx: Context<'q>) -> Result<Atom<'q>> {
        self.as_data().to_atom()
    }
}

impl<'q> IntoQjAtom<'q> for &str {
    fn into_qj_atom(self, ctx: Context<'q>) -> Result<Atom<'q>> {
        ctx.new_atom(self)
    }
}

impl<'q> IntoQjAtom<'q> for String {
    fn into_qj_atom(self, ctx: Context<'q>) -> Result<Atom<'q>> {
        ctx.new_atom(&self)
    }
}

impl<'q> IntoQjAtom<'q> for i32 {
    fn into_qj_atom(self, ctx: Context<'q>) -> Result<Atom<'q>> {
        ctx.new_int32(self).to_atom()
    }
}

pub struct PropertyEnum<'q> {
    property_enum: qc::PropertyEnum<'q>,
    context: qc::Context<'q>,
}

impl<'q> PropertyEnum<'q> {
    pub fn from_raw_parts(property_enum: qc::PropertyEnum<'q>, context: qc::Context<'q>) -> Self {
        PropertyEnum { property_enum, context }
    }

    // properties

    pub fn is_enumerable(&self) -> bool {
        self.property_enum.is_enumerable()
    }

    pub fn atom(&self) -> Atom<'q> {
        Atom::from_raw_parts(self.property_enum.atom(), self.context)
    }

    // memory

    #[inline]
    pub(crate) unsafe fn free(this: &Self) {
        this.context.free_atom(this.property_enum.atom());
    }

    #[inline]
    pub(crate) fn dup(this: &Self) {
        this.context.dup_atom(this.property_enum.atom());
    }
}

impl Drop for PropertyEnum<'_> {
    fn drop(&mut self) {
        unsafe { Self::free(self) }
    }
}

impl Clone for PropertyEnum<'_> {
    fn clone(&self) -> Self {
        let property_enum = PropertyEnum {
            property_enum: self.property_enum.clone(),
            context: self.context,
        };
        PropertyEnum::dup(&property_enum);
        property_enum
    }
}

impl fmt::Debug for PropertyEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_str(&format!(
            "PropertyEnum({:?}, is_enumerable={})",
            self.atom(),
            self.is_enumerable(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{run_with_context, Result};
    use qjncore::{EvalFlags, GPNFlags};
    #[test]
    fn test() -> Result<()> {
        run_with_context(|ctx| {
            let global = ctx.global_object()?;
            let k_foo = ctx.new_atom("foo")?;
            let k_bar = "bar";
            let foo = ctx.new_object()?;
            assert!(!foo.has_key(k_bar)?);
            foo.set(k_bar, 42)?;
            assert!(foo.has_key(k_bar)?);
            let v: i32 = foo.get(k_bar)?;
            assert_eq!(42, v);
            let prop_enums = foo.own_property_names(GPNFlags::STRING_MASK)?;
            assert_eq!(1, prop_enums.len());
            let k_bar2: String = prop_enums[0].atom().to_string()?.into();
            assert_eq!("bar", &k_bar2);
            assert!(prop_enums[0].is_enumerable());
            // set global
            global.set(k_foo.clone(), foo)?;
            let b: bool = ctx.eval_into("foo.bar === 42", "<input>", EvalFlags::TYPE_GLOBAL)?;
            assert!(b);
            Ok(())
        })?;
        Ok(())
    }
}
