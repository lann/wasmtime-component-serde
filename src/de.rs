use std::collections::HashMap;

use serde::{
    de::{self, DeserializeSeed, Unexpected, Visitor},
    Deserializer,
};
use wasmtime::component::{Type, Val};

/// A [`serde::de::DeserializeSeed`] implementation for deserializing [`Val`]s
/// of a given dynamic [`Type`].
pub struct DeserializeVal<'a>(pub &'a Type);

impl<'a, 'de> DeserializeSeed<'de> for DeserializeVal<'a> {
    type Value = Val;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.0 {
            Type::Bool => deserializer.deserialize_bool(self),
            Type::Char => deserializer.deserialize_char(self),
            Type::String => deserializer.deserialize_string(self),
            Type::List(_) => deserializer.deserialize_seq(self),
            Type::Record(_) => deserializer.deserialize_map(self),
            Type::Tuple(tuple) => deserializer.deserialize_tuple(tuple.types().len(), self),
            Type::Variant(_) => deserializer.deserialize_map(self),
            Type::Enum(_) => deserializer.deserialize_str(self),
            Type::Option(opt) => match opt.ty() {
                Type::Option(_) => deserializer.deserialize_any(self),
                _ => deserializer.deserialize_option(self),
            },
            Type::Result(_) => deserializer.deserialize_map(self),
            Type::Flags(_) => deserializer.deserialize_seq(self),
            _ => deserializer.deserialize_any(self),
        }
    }
}

impl<'a, 'de> Visitor<'de> for DeserializeVal<'a> {
    type Value = Val;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: better messages
        write!(formatter, "a {:?}", self.0)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0 {
            Type::Bool => Ok(Val::Bool(v)),
            _ => Err(de::Error::invalid_type(de::Unexpected::Bool(v), &self)),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0 {
            Type::U8 => u8::try_from(v).map(Val::U8),
            Type::S8 => i8::try_from(v).map(Val::S8),
            Type::U16 => u16::try_from(v).map(Val::U16),
            Type::S16 => i16::try_from(v).map(Val::S16),
            Type::U32 => u32::try_from(v).map(Val::U32),
            Type::S32 => i32::try_from(v).map(Val::S32),
            Type::U64 => u64::try_from(v).map(Val::U64),
            Type::S64 => Ok(Val::S64(v)),
            _ => return Err(de::Error::invalid_type(de::Unexpected::Signed(v), &self)),
        }
        .map_err(|_| de::Error::invalid_value(de::Unexpected::Signed(v), &self))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0 {
            Type::U8 => u8::try_from(v).map(Val::U8),
            Type::S8 => i8::try_from(v).map(Val::S8),
            Type::U16 => u16::try_from(v).map(Val::U16),
            Type::S16 => i16::try_from(v).map(Val::S16),
            Type::U32 => u32::try_from(v).map(Val::U32),
            Type::S32 => i32::try_from(v).map(Val::S32),
            Type::U64 => Ok(Val::U64(v)),
            Type::S64 => i64::try_from(v).map(Val::S64),
            _ => return Err(de::Error::invalid_type(de::Unexpected::Unsigned(v), &self)),
        }
        .map_err(|_| de::Error::invalid_value(de::Unexpected::Unsigned(v), &self))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // TODO: should this try to deserialize ints?
        match self.0 {
            // TODO: Should this do some precision check?
            Type::Float32 => Ok(Val::Float32(v as f32)),
            Type::Float64 => Ok(Val::Float64(v)),
            _ => return Err(de::Error::invalid_type(de::Unexpected::Float(v), &self)),
        }
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0 {
            Type::Char => Ok(Val::Char(v)),
            _ => Err(de::Error::invalid_type(de::Unexpected::Char(v), &self)),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0 {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                let v = v
                    .parse()
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &self))?;
                self.visit_u64(v)
            }
            Type::S8 | Type::S16 | Type::S32 | Type::S64 => {
                let v = v
                    .parse()
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(v), &self))?;
                self.visit_i64(v)
            }
            Type::Float32 => {
                let val = match v {
                    "NaN" => f32::NAN,
                    "Infinity" => f32::INFINITY,
                    "-Infinity" => f32::NEG_INFINITY,
                    _ => return Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
                };
                Ok(Val::Float32(val))
            }
            Type::Float64 => {
                let val = match v {
                    "NaN" => f64::NAN,
                    "Infinity" => f64::INFINITY,
                    "-Infinity" => f64::NEG_INFINITY,
                    _ => return Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
                };
                Ok(Val::Float64(val))
            }
            Type::String => Ok(Val::String(v.into())),
            Type::Char if v.chars().take(2).count() == 1 => {
                Ok(Val::Char(v.chars().next().unwrap()))
            }
            Type::Enum(enum_) => enum_.new_val(v).map_err(de::Error::custom),
            _ => Err(de::Error::invalid_type(de::Unexpected::Str(v), &self)),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0 {
            Type::String => Ok(Val::String(v.into_boxed_str())),
            _ => self.visit_str(&v),
        }
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match &self.0 {
            Type::List(list) if matches!(list.ty(), Type::U8) => {
                let bytes = v.iter().map(|b| Val::U8(*b)).collect();
                list.new_val(bytes).map_err(de::Error::custom)
            }
            _ => Err(de::Error::invalid_type(de::Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_none()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match &self.0 {
            Type::Option(opt) => opt.new_val(None).map_err(de::Error::custom),
            _ => Err(de::Error::invalid_type(de::Unexpected::Option, &self)),
        }
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match &self.0 {
            Type::Option(opt) => {
                let v = if let Type::Option(_) = opt.ty() {
                    deserializer.deserialize_map(DeserializeVal(&opt.ty()))?
                } else {
                    deserializer.deserialize_any(DeserializeVal(&opt.ty()))?
                };
                opt.new_val(Some(v)).map_err(de::Error::custom)
            }
            _ => Err(de::Error::invalid_type(de::Unexpected::Option, &self)),
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        match &self.0 {
            Type::List(list) => {
                let ty = list.ty();
                let mut values = Vec::with_capacity(seq.size_hint().unwrap_or_default());
                while let Some(v) = seq.next_element_seed(DeserializeVal(&ty))? {
                    values.push(v);
                }
                list.new_val(values.into()).map_err(de::Error::custom)
            }
            Type::Tuple(tuple) => {
                let len = tuple.types().len();
                let mut values = Vec::with_capacity(len);
                for ty in tuple.types() {
                    let v = seq
                        .next_element_seed(DeserializeVal(&ty))?
                        .ok_or_else(|| de::Error::invalid_length(values.len(), &self))?;
                    values.push(v);
                }
                if seq.next_element::<()>()?.is_some() {
                    return Err(de::Error::invalid_length(len + 1, &self));
                }

                tuple
                    .new_val(values.into_boxed_slice())
                    .map_err(de::Error::custom)
            }
            Type::Flags(flags) => {
                let mut names = Vec::with_capacity(seq.size_hint().unwrap_or_default());
                while let Some(name) = seq.next_element()? {
                    names.push(name);
                }
                flags.new_val(&names).map_err(de::Error::custom)
            }
            _ => Err(de::Error::invalid_type(de::Unexpected::Seq, &self)),
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        match &self.0 {
            Type::Record(rec) => {
                let field_tys = rec
                    .fields()
                    .map(|f| (f.name, f.ty))
                    .collect::<HashMap<_, _>>();
                let mut field_vals = HashMap::with_capacity(rec.fields().len());
                while let Some(name) = map.next_key::<Box<str>>()? {
                    let ty = field_tys
                        .get(&*name)
                        .ok_or_else(|| de::Error::custom(format!("unknown field `{name}`")))?;
                    let val = map.next_value_seed(DeserializeVal(ty))?;
                    if field_vals.contains_key(&name) {
                        return Err(de::Error::custom(format!("duplicate field `{name}`")));
                    }
                    field_vals.insert(name, val);
                }
                let values = rec
                    .fields()
                    .map(|field| {
                        if let Some(v) = field_vals.remove(field.name) {
                            Ok(v)
                        } else if let Type::Option(opt) = field.ty {
                            opt.new_val(None).map_err(de::Error::custom)
                        } else {
                            Err(de::Error::custom(format!("missing field `{}`", field.name)))
                        }
                    })
                    .collect::<Result<Vec<_>, A::Error>>()?;
                let values = rec.fields().map(|f| f.name).zip(values);
                rec.new_val(values).map_err(de::Error::custom)
            }

            Type::Variant(var) => single_entry_map(map, |map, name| {
                let ty = var
                    .cases()
                    .find_map(|case| (case.name == name).then_some(case.ty))
                    .ok_or_else(|| de::Error::custom(format!("unknown variant `{name}`")))?;
                let v = next_value_maybe(map, ty)?;
                var.new_val(name, v).map_err(de::Error::custom)
            }),

            Type::Option(opt) => single_entry_map(map, |map, name| {
                if name != "value" {
                    return Err(de::Error::unknown_field("name", &["value"]));
                }
                let v = map.next_value_seed(DeserializeVal(&opt.ty()))?;
                opt.new_val(Some(v)).map_err(de::Error::custom)
            }),

            Type::Result(res) => single_entry_map(map, |map, name| {
                let (ty, is_ok) = match name {
                    "result" => (res.ok(), true),
                    "error" => (res.err(), false),
                    _ => return Err(de::Error::unknown_variant(name, &["result", "error"])),
                };
                let v = next_value_maybe(map, ty)?;
                if is_ok {
                    res.new_val(Ok(v))
                } else {
                    res.new_val(Err(v))
                }
                .map_err(de::Error::custom)
            }),

            _ => Err(de::Error::invalid_type(de::Unexpected::Map, &self)),
        }
    }
}

fn single_entry_map<'de, A>(
    mut map: A,
    f: impl FnOnce(&mut A, &str) -> Result<Val, A::Error>,
) -> Result<Val, A::Error>
where
    A: de::MapAccess<'de>,
{
    let name: &str = map
        .next_key()?
        .ok_or_else(|| de::Error::invalid_length(0, &"exactly one field"))?;
    let v = f(&mut map, name)?;
    if map.next_key::<&str>()?.is_some() {
        return Err(de::Error::invalid_length(
            map.size_hint().unwrap_or(2),
            &"exactly one field",
        ));
    }
    Ok(v)
}

fn next_value_maybe<'de, A>(map: &mut A, ty: Option<Type>) -> Result<Option<Val>, A::Error>
where
    A: de::MapAccess<'de>,
{
    Ok(match ty {
        Some(t) => Some(map.next_value_seed(DeserializeVal(&t))?),
        None => {
            map.next_value::<()>()?;
            None
        }
    })
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn basic_types_smoke_tests() {
        assert_val_json(Val::Bool(true), json!(true));
        assert_val_json(Val::U16(123), json!(123));
        assert_val_json(Val::S32(-123456), json!(-123456));
        assert_val_json(Val::Float32(1.5), json!(1.5));
        assert_val_json(Val::Float32(f32::INFINITY), json!("Infinity"));
        assert_val_json(Val::Float32(f32::NEG_INFINITY), json!("-Infinity"));
        assert_val_json(Val::Float32(f32::NAN), json!("NaN"));
        assert_val_json(Val::Float64(1.5), json!(1.5));
        assert_val_json(Val::Char('☃'), json!("☃"));
        assert_val_json(Val::String("☃☃☃".into()), json!("☃☃☃"));
    }

    fn assert_val_json(val: Val, json: serde_json::Value) {
        let ty = val.ty();
        let deserialized = DeserializeVal(&ty).deserialize(json).unwrap();
        assert_eq!(deserialized, val)
    }
}
