//! JSON parsing with duplicate-key, depth, size and container limits.
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

pub const MAX_LINE_BYTES: usize = 1024 * 1024;
pub const MAX_DEPTH: usize = 64;
pub const MAX_CONTAINER_ITEMS: usize = 4096;

pub fn parse(bytes: &[u8]) -> Result<Value, &'static str> {
    if bytes.len() > MAX_LINE_BYTES {
        return Err("request_too_large");
    }
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let value = Seed { depth: 0 }
        .deserialize(&mut parser)
        .map_err(|_| "invalid_json")?;
    parser.end().map_err(|_| "invalid_json")?;
    Ok(value)
}

struct Seed {
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for Seed {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON with unique keys")
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.into()))
    }
    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        if self.depth >= MAX_DEPTH {
            return Err(de::Error::custom("nesting limit"));
        }
        let mut result = Vec::new();
        while let Some(value) = seq.next_element_seed(Seed {
            depth: self.depth + 1,
        })? {
            if result.len() >= MAX_CONTAINER_ITEMS {
                return Err(de::Error::custom("container limit"));
            }
            result.push(value);
        }
        Ok(Value::Array(result))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        if self.depth >= MAX_DEPTH {
            return Err(de::Error::custom("nesting limit"));
        }
        let mut result = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if result.len() >= MAX_CONTAINER_ITEMS || result.contains_key(&key) {
                return Err(de::Error::custom("duplicate key or container limit"));
            }
            let value = map.next_value_seed(Seed {
                depth: self.depth + 1,
            })?;
            result.insert(key, value);
        }
        Ok(Value::Object(result))
    }
}
