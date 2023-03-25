use crate::error::{Error, Result};
use quijine::{Atom, Context, GpnFlags, Value};
use serde::{
    de::{self, Error as _},
    Deserialize,
};
use std::convert::TryInto;

pub fn from_qj<'q, 'de, T>(input: Value<'q>) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(input);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub struct Deserializer<'q> {
    input: Value<'q>,
}

impl<'q> Deserializer<'q> {
    pub fn new(input: Value<'q>) -> Self {
        Deserializer { input }
    }

    #[inline]
    fn context(&self) -> Context<'q> {
        self.input.context()
    }
}

impl<'q, 'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'q> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.input.clone().to_variant() {
            quijine::Variant::BigDecimal(_) => unimplemented!(),
            quijine::Variant::BigInt(_) => unimplemented!(),
            quijine::Variant::BigFloat(_) => unimplemented!(),
            quijine::Variant::Symbol(_) => unimplemented!(),
            quijine::Variant::String(_) => self.deserialize_string(visitor),
            quijine::Variant::Module(_) => unimplemented!(),
            quijine::Variant::FunctionBytecode(_) => unimplemented!(),
            quijine::Variant::Object(v) => {
                if v.is_array() {
                    self.deserialize_seq(visitor)
                } else {
                    self.deserialize_map(visitor)
                }
            }
            quijine::Variant::Int(_) => self.deserialize_i32(visitor),
            quijine::Variant::Bool(_) => self.deserialize_bool(visitor),
            quijine::Variant::Null => self.deserialize_unit(visitor),
            quijine::Variant::Undefined => self.deserialize_unit(visitor),
            quijine::Variant::Uninitialized => self.deserialize_unit(visitor),
            quijine::Variant::CatchOffset(_) => unimplemented!(),
            quijine::Variant::Exception => unimplemented!(),
            quijine::Variant::Float64(_) => self.deserialize_f64(visitor),
            _ => unimplemented!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.input.to_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.input.to_i32()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.input.to_i32()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.input.to_i32()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.input.to_i64()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.input.to_i32()? as u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.input.to_i32()? as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.input.to_i64()? as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.input.to_i64()? as u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.input.to_f64()? as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.input.to_f64()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.input.to_string()?;
        if let Some(c) = s.chars().next() {
            visitor.visit_char(c)
        } else {
            Err(Error::custom("not a single char"))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.input.to_string()?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.input.is_null() || self.input.is_undefined() || self.input.is_uninitialized() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.input.is_null() || self.input.is_undefined() || self.input.is_uninitialized() {
            visitor.visit_none()
        } else {
            Err(Error::custom("not nullish"))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len: i32 = self.input.get("length")?;
        let seq = SeqAccess {
            object: self.input.clone(),
            len,
            pos: 0,
        };
        visitor.visit_seq(seq)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let seq = SeqAccess {
            object: self.input.clone(),
            len: len as i32,
            pos: 0,
        };
        visitor.visit_seq(seq)
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let prop_names = self
            .input
            .own_property_names(GpnFlags::STRING_MASK | GpnFlags::ENUM_ONLY)?;
        let keys: Vec<_> = prop_names
            .into_iter()
            .map(|v| v.atom())
            .filter(|k| !self.input.property(k.clone()).unwrap().is_undefined())
            .collect();
        let map = MapAccess {
            object: self.input.clone(),
            keys,
            pos: 0,
        };
        visitor.visit_map(map)
    }

    fn deserialize_struct<V>(self, _name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let map = ObjectAccess {
            object: self.input.clone(),
            fields,
            pos: 0,
        };
        visitor.visit_map(map)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let tag: quijine::Result<quijine::String> = self.input.clone().try_into();
        match tag {
            Ok(tag) => visitor.visit_enum(EnumAccess {
                tag: tag.into(),
                payload: self.context().undefined().into(),
            }),
            Err(_) => {
                let prop_names = self
                    .input
                    .own_property_names(GpnFlags::STRING_MASK | GpnFlags::ENUM_ONLY)?;
                if prop_names.len() != 1 {
                    return Err(Error::custom("multiple keys in variant object"));
                }
                let tag = prop_names[0].atom();
                let payload = self.input.property(tag.clone())?;
                visitor.visit_enum(EnumAccess {
                    tag: tag.to_value()?,
                    payload,
                })
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_none()
    }
}

// Map

struct MapAccess<'q> {
    object: Value<'q>,
    keys: Vec<Atom<'q>>,
    pos: usize,
}

impl<'de> de::MapAccess<'de> for MapAccess<'_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        Ok(match self.keys.get(self.pos) {
            Some(key) => {
                let mut deserializer = Deserializer::new(key.to_value()?);
                Some(seed.deserialize(&mut deserializer)?)
            }
            None => None,
        })
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let key = self.keys[self.pos].clone();
        self.pos += 1;
        let value = self.object.get(key).unwrap();
        let mut deserializer = Deserializer::new(value);
        seed.deserialize(&mut deserializer)
    }
}

struct ObjectAccess<'q> {
    object: Value<'q>,
    fields: &'static [&'static str],
    pos: usize,
}

fn str_deserializer(s: &str) -> de::value::StrDeserializer<Error> {
    de::IntoDeserializer::into_deserializer(s)
}

impl<'de> de::MapAccess<'de> for ObjectAccess<'_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        Ok(match self.fields.get(self.pos) {
            Some(&field) => Some(seed.deserialize(str_deserializer(field))?),
            None => None,
        })
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let field = self.fields[self.pos];
        self.pos += 1;
        let value = self.object.get(field).unwrap();
        let mut deserializer = Deserializer::new(value);
        seed.deserialize(&mut deserializer)
    }
}

// Seq

struct SeqAccess<'q> {
    object: Value<'q>,
    len: i32,
    pos: i32,
}

impl<'de> de::SeqAccess<'de> for SeqAccess<'_> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let pos = self.pos;
        self.pos += 1;

        if pos < self.len {
            let value = self.object.get(pos).unwrap();
            let mut deserializer = Deserializer::new(value);
            Ok(Some(seed.deserialize(&mut deserializer)?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some((self.len - self.pos) as usize)
    }
}

// Variant

struct VariantAccess<'q> {
    value: Value<'q>,
}

impl<'de, 'q> de::VariantAccess<'de> for VariantAccess<'q> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        let mut d = Deserializer::new(self.value);
        de::Deserialize::deserialize(&mut d)
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let mut d = Deserializer::new(self.value);
        seed.deserialize(&mut d)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        let mut d = Deserializer::new(self.value);
        de::Deserializer::deserialize_tuple(&mut d, len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value> {
        let mut d = Deserializer::new(self.value);
        de::Deserializer::deserialize_struct(&mut d, "", fields, visitor)
    }
}

struct EnumAccess<'q> {
    tag: Value<'q>,
    payload: Value<'q>,
}

impl<'de, 'q> de::EnumAccess<'de> for EnumAccess<'q> {
    type Error = Error;
    type Variant = VariantAccess<'q>;

    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let seed = {
            let mut dtag = Deserializer::new(self.tag);
            seed.deserialize(&mut dtag)
        };
        let dpayload = VariantAccess::<'q> { value: self.payload };

        Ok((seed?, dpayload))
    }
}
