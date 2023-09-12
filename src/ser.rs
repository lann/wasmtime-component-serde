use serde::{
    ser::{self, SerializeMap, SerializeSeq, SerializeTuple},
    Serialize,
};
use wasmtime::component::Val;

/// A [`serde::Serialize`] implementation for [`Val`]s.
pub struct SerializeVal<'a>(pub &'a Val);

impl<'a> Serialize for SerializeVal<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Val::Bool(v) => serializer.serialize_bool(*v),
            Val::S8(v) => serializer.serialize_i8(*v),
            Val::U8(v) => serializer.serialize_u8(*v),
            Val::S16(v) => serializer.serialize_i16(*v),
            Val::U16(v) => serializer.serialize_u16(*v),
            Val::S32(v) => serializer.serialize_i32(*v),
            Val::U32(v) => serializer.serialize_u32(*v),
            // TODO: consider (configurably?) serializing large numbers as strings
            Val::S64(v) => serializer.serialize_i64(*v),
            Val::U64(v) => serializer.serialize_u64(*v),

            Val::Float32(v) => match v.classify() {
                std::num::FpCategory::Nan => serializer.serialize_str("NaN"),
                std::num::FpCategory::Infinite if v.is_sign_negative() => {
                    serializer.serialize_str("-Infinity")
                }
                std::num::FpCategory::Infinite => serializer.serialize_str("Infinity"),
                _ => serializer.serialize_f32(*v),
            },
            Val::Float64(v) => match v.classify() {
                std::num::FpCategory::Nan => serializer.serialize_str("NaN"),
                std::num::FpCategory::Infinite if v.is_sign_negative() => {
                    serializer.serialize_str("-Infinity")
                }
                std::num::FpCategory::Infinite => serializer.serialize_str("Infinity"),
                _ => serializer.serialize_f64(*v),
            },

            Val::Char(v) => serializer.serialize_char(*v),
            Val::String(v) => serializer.serialize_str(v),

            Val::List(vlst) => {
                let mut seq = serializer.serialize_seq(Some(vlst.len()))?;
                for v in vlst.iter() {
                    seq.serialize_element(&SerializeVal(v))?;
                }
                seq.end()
            }

            Val::Record(vrec) => {
                let mut map = serializer.serialize_map(None)?;
                for (name, v) in vrec.fields() {
                    if let Val::Option(opt) = v {
                        if opt.value().is_none() {
                            continue;
                        }
                    }
                    map.serialize_entry(name, &SerializeVal(v))?;
                }
                map.end()
            }

            Val::Tuple(vtup) => {
                let mut tup = serializer.serialize_tuple(vtup.values().len())?;
                for v in vtup.values() {
                    tup.serialize_element(&SerializeVal(v))?;
                }
                tup.end()
            }

            Val::Variant(vvar) => {
                // Note: While it would be natural to `serialize_*_variant` below,
                // they require a variant index which might not be stable.
                single_entry_map(serializer, vvar.discriminant(), vvar.payload())
            }

            // re: `serialize_unit_variant`: see `Val::Variant` arm comment above.
            Val::Enum(venu) => serializer.serialize_str(venu.discriminant()),

            Val::Option(vopt) => {
                if let Some(v) = vopt.value() {
                    if let Val::Option(_) = v {
                        // Serialize `Some::<Option<_>>` as `{"value": ...}` to
                        // avoid ambiguity in serde_json.
                        single_entry_map(serializer, "value", Some(v))
                    } else {
                        serializer.serialize_some(&SerializeVal(v))
                    }
                } else {
                    serializer.serialize_none()
                }
            }

            Val::Result(vres) => match vres.value() {
                Ok(maybe_val) => single_entry_map(serializer, "result", maybe_val),
                Err(maybe_val) => single_entry_map(serializer, "error", maybe_val),
            },

            Val::Flags(vflg) => {
                let mut seq = serializer.serialize_seq(None)?;
                for flag in vflg.flags() {
                    seq.serialize_element(flag)?;
                }
                seq.end()
            }

            Val::Resource(_) => Err(ser::Error::custom("cannot serialize resources")),
        }
    }
}

fn single_entry_map<S: serde::Serializer>(
    serializer: S,
    key: &str,
    val: Option<&Val>,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(1))?;
    match val {
        Some(v) => map.serialize_entry(key, &SerializeVal(v))?,
        None => map.serialize_entry(key, &())?,
    }
    map.end()
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use super::*;
    use serde_json::json;

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
        let serialized = serde_json::to_value(SerializeVal(&val)).unwrap();
        assert_eq!(serialized, json);
    }
}
