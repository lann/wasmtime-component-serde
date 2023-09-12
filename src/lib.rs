use serde::{de::DeserializeSeed, Deserializer, Serialize, Serializer};
use wasmtime::component::{Type, Val};

mod de;
mod ser;

pub use de::DeserializeVal;
pub use ser::SerializeVal;

/// Deserialize a [`Val`] of the given [`Type`] from a [`Deserializer`].
pub fn deserialize_val<'de, D: Deserializer<'de>>(
    deserializer: D,
    ty: &Type,
) -> Result<Val, D::Error> {
    de::DeserializeVal(ty).deserialize(deserializer)
}

/// Serialize a [`Val`] with a [`Serializer`].
pub fn serialize_val<S: Serializer>(serializer: S, val: &Val) -> Result<S::Ok, S::Error> {
    SerializeVal(val).serialize(serializer)
}

/// Deserialize a [`Val`] of the given [`Type`] from JSON.
#[cfg(feature = "json")]
pub fn from_json(ty: &Type, json: impl AsRef<[u8]>) -> serde_json::Result<Val> {
    let mut d = serde_json::Deserializer::from_slice(json.as_ref());
    deserialize_val(&mut d, ty)
}

/// Serialize a [`Val`] to JSON.
#[cfg(feature = "json")]
pub fn to_json(val: &Val) -> serde_json::Result<String> {
    serde_json::to_string(&SerializeVal(val))
}
