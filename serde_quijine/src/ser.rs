use quijine::{Context, Data, Object};
use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub fn to_qj<T>(context: Context, input: T) -> Result<Data>
where
    T: Serialize,
{
    let serializer = Serializer::new(context);
    input.serialize(serializer)
}

pub struct Serializer<'q> {
    context: Context<'q>,
}

impl<'q> Serializer<'q> {
    fn new(context: Context<'q>) -> Self {
        Serializer { context }
    }
}

impl<'q, 'a> ser::Serializer for Serializer<'q> {
    type Error = Error;
    type Ok = Data<'q>;
    type SerializeMap = MapSerializer<'q>;
    type SerializeSeq = ArraySerializer<'q>;
    type SerializeStruct = ObjectSerializer<'q>;
    type SerializeStructVariant = VariantSerializer<'q, ObjectSerializer<'q>>;
    type SerializeTuple = ArraySerializer<'q>;
    type SerializeTupleStruct = ArraySerializer<'q>;
    type SerializeTupleVariant = VariantSerializer<'q, ArraySerializer<'q>>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(self.context.new_bool(v).into())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(self.context.new_int32(v as i32).into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(self.context.new_int32(v as i32).into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(self.context.new_int32(v).into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(self.context.new_int64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(self.context.new_int32(v as i32).into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(self.context.new_int32(v as i32).into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(self.context.new_int64(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(self.context.new_float64(v as f64).into())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(self.context.new_float64(v as f64).into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(self.context.new_float64(v).into())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(self.context.new_string(v).map(|v| v.into())?)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(self.context.null().into())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        let ctx = self.context;
        let x = self.serialize_newtype_struct(variant, value)?;
        VariantSerializer::new(ctx, variant, x).end(Ok)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(ArraySerializer::new(self.context, len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(VariantSerializer::new(
            self.context,
            variant,
            self.serialize_tuple_struct(variant, len)?,
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer::new(self.context.new_object()?, self.context))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(ObjectSerializer::new(self.context.new_object()?, self.context))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let ctx = self.context;
        let x = self.serialize_struct(variant, len)?;
        Ok(VariantSerializer::new(ctx, variant, x))
    }
}

// Array

pub struct ArraySerializer<'q> {
    pending: Vec<Data<'q>>,
    context: Context<'q>,
}

impl<'q> ArraySerializer<'q> {
    pub fn new(context: Context<'q>, len: Option<usize>) -> Self {
        let pending = match len {
            Some(len) => Vec::with_capacity(len),
            None => vec![],
        };
        Self { pending, context }
    }
}

impl<'q> ser::SerializeSeq for ArraySerializer<'q> {
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let x = value.serialize(Serializer::new(self.context))?;
        self.pending.push(x);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let arr = self.context.new_array()?;
        arr.set("length", self.pending.len() as i32)?;
        for (i, elem) in self.pending.iter().enumerate() {
            arr.set(i as i32, elem.clone())?;
        }
        Ok(arr.into())
    }
}

impl<'q> ser::SerializeTuple for ArraySerializer<'q> {
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl<'q> ser::SerializeTupleStruct for ArraySerializer<'q> {
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeTuple::end(self)
    }
}

// Object

pub struct ObjectSerializer<'q> {
    object: Object<'q>,
    context: Context<'q>,
}

impl<'q> ObjectSerializer<'q> {
    pub fn new(object: Object<'q>, context: Context<'q>) -> Self {
        Self { object, context }
    }
}

impl<'q> ser::SerializeStruct for ObjectSerializer<'q> {
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
        let value = value.serialize(Serializer::new(self.context))?;
        self.object.set(key, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.object.into())
    }
}

pub struct MapSerializer<'q> {
    object: Object<'q>,
    context: Context<'q>,
    next_key: Option<Data<'q>>,
}

impl<'q> MapSerializer<'q> {
    pub fn new(object: Object<'q>, context: Context<'q>) -> Self {
        Self {
            object,
            context,
            next_key: None,
        }
    }
}

impl<'q> ser::SerializeMap for MapSerializer<'q> {
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        debug_assert!(self.next_key.is_none());
        self.next_key = Some(key.serialize(Serializer::new(self.context))?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let value = value.serialize(Serializer::new(self.context))?;
        self.object.set(self.next_key.take().unwrap(), value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        debug_assert!(self.next_key.is_none());
        Ok(self.object.into())
    }
}

// Variant

pub struct VariantSerializer<'q, S> {
    inner: S,
    context: Context<'q>,
    variant: &'static str,
}

impl<'q, S> VariantSerializer<'q, S> {
    pub fn new(context: Context<'q>, variant: &'static str, inner: S) -> Self {
        Self {
            variant,
            context,
            inner,
        }
    }

    fn end(self, inner: impl FnOnce(S) -> Result<Data<'q>>) -> Result<Data<'q>> {
        let value = inner(self.inner)?;
        let obj = self.context.new_object()?;
        obj.set(self.variant, value)?;
        Ok(obj.into())
    }
}

impl<'q, S> ser::SerializeTupleVariant for VariantSerializer<'q, S>
where
    S: ser::SerializeTupleStruct<Ok = Data<'q>, Error = Error>,
{
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end(S::end)
    }
}

impl<'q, S> ser::SerializeStructVariant for VariantSerializer<'q, S>
where
    S: ser::SerializeStruct<Ok = Data<'q>, Error = Error>,
{
    type Error = Error;
    type Ok = Data<'q>;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end(S::end)
    }
}
