#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt::{self, Debug};
use super::number::Number;
use super::index::Index;

/// The Value enum, a loosely typed way of representing any valid value.
///
/// It is used to contains the parsing result of [serde_json] or [serde_yaml].
///
/// [serde_json]: https://docs.serde.rs/serde_json/
/// [serde_yaml]: https://docs.serde.rs/serde_yaml/
#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// Represents a null value.
    Null,

    /// Represents a boolean.
    Bool(bool),

    /// Represents a number, whether integer or floating point.
    Number(Number),

    /// Represents a string.
    String(String),

    /// Represents an array.
    Array(Vec<Value>),

    /// Represents an object.
    Object(BTreeMap<String, Value>),
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Value::Null => formatter.debug_tuple("Null").finish(),
            Value::Bool(v) => formatter.debug_tuple("Bool").field(&v).finish(),
            Value::Number(ref v) => Debug::fmt(v, formatter),
            Value::String(ref v) => formatter.debug_tuple("String").field(v).finish(),
            Value::Array(ref v) => formatter.debug_tuple("Array").field(v).finish(),
            Value::Object(ref v) => formatter.debug_tuple("Object").field(v).finish(),
        }
    }
}

impl Value {
    /// Index into an array or map. A string index can be used to access a
    /// value in a map, and a usize index can be used to access an element of an
    /// array.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is an array or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the array.
    pub fn get<I: Index>(&self, index: I) -> Option<&Self> {
        index.index_into(self)
    }

    /// Index into a JSON array or map. A string index can be used to access a
    /// value in a map, and a usize index can be used to access an element of an
    /// array.
    ///
    /// Returns `None` if the type of `self` does not match the type of the
    /// index, for example if the index is a string and `self` is an array or a
    /// number. Also returns `None` if the given key does not exist in the map
    /// or the given index is not within the bounds of the array.
    pub fn get_mut<I: Index>(&mut self, index: I) -> Option<&mut Self> {
        index.index_into_mut(self)
    }

    /// Returns true if the `Value` is an Object. Returns false otherwise.
    ///
    /// For any Value on which `is_object` returns true, `as_object` and
    /// `as_object_mut` are guaranteed to return the map representation of the
    /// object.
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// If the `Value` is an Object, returns the associated Map. Returns None
    /// otherwise.
    pub fn as_object(&self) -> Option<&BTreeMap<String, Self>> {
        match *self {
            Self::Object(ref map) => Some(map),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> {
        match *self {
            Self::Object(ref mut map) => Some(map),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    ///
    /// For any Value on which `is_array` returns true, `as_array` and
    /// `as_array_mut` are guaranteed to return the vector representing the
    /// array.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// If the `Value` is an Array, returns the associated vector. Returns None
    /// otherwise.
    pub fn as_array(&self) -> Option<&Vec<Self>> {
        match *self {
            Self::Array(ref array) => Some(&*array),
            _ => None,
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match *self {
            Self::Array(ref mut list) => Some(list),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// For any Value on which `is_string` returns true, `as_str` is guaranteed
    /// to return the string slice.
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// If the `Value` is a String, returns the associated str. Returns None
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Self::String(ref s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        match *self {
            Self::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Value on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    pub fn is_i64(&self) -> bool {
        match *self {
            Self::Number(ref n) => n.is_i64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between zero and `u64::MAX`.
    ///
    /// For any Value on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    pub fn is_u64(&self) -> bool {
        match *self {
            Self::Number(ref n) => n.is_u64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number that can be represented by f64.
    ///
    /// For any Value on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    ///
    /// Currently this function returns true if and only if both `is_i64` and
    /// `is_u64` return false but this is not a guarantee in the future.
    pub fn is_f64(&self) -> bool {
        match *self {
            Self::Number(ref n) => n.is_f64(),
            _ => false,
        }
    }

    /// If the `Value` is an integer, represent it as i64 if possible. Returns
    /// None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Self::Number(ref n) => n.as_i64(),
            _ => None,
        }
    }

    /// If the `Value` is an integer, represent it as u64 if possible. Returns
    /// None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Self::Number(ref n) => n.as_u64(),
            _ => None,
        }
    }

    /// If the `Value` is a number, represent it as f64 if possible. Returns
    /// None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Self::Number(ref n) => n.as_f64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// For any Value on which `is_boolean` returns true, `as_bool` is
    /// guaranteed to return the boolean value.
    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool. Returns None
    /// otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Returns true if the `Value` is a Null. Returns false otherwise.
    ///
    /// For any Value on which `is_null` returns true, `as_null` is guaranteed
    /// to return `Some(())`.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the `Value` is a Null, returns (). Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match *self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    /// Takes the value out of the `Value`, leaving a `Null` in its place.
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Null)
    }
}

/// The default value is `Value::Null`.
///
/// This is useful for handling omitted `Value` fields when deserializing.
impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl From<&serde_json::Value> for Value
{
    /// Converts [serde_json] deserialization results under a common value:
    /// [Value].
    /// 
    /// [serde_yaml](https://docs.serde.rs/serde_json/index.html)
    /// [Value](./struct.Value.html)
    fn from(json: &serde_json::Value) -> Self
    {
        match json {
            serde_json::Value::Null         => {
                Self::Null
            },
            serde_json::Value::String(ref str)  => {
                Self::String(str.to_string())
            },
            serde_json::Value::Bool(ref bool)   => {
                Self::Bool(*bool)
            },
            serde_json::Value::Number(ref n)    => {
                Self::Number(Number::from(n))
            },
            serde_json::Value::Array(json)   => {
                let vec: Vec<Self> = json.iter().map(|each| {
                    // Dangerous recusivity
                    Self::from(each)
                }).collect();

                Self::Array(vec)
            },
            serde_json::Value::Object(json)   => {
                let map: BTreeMap<String, Self> = json.iter()
                .map(|(key, each)| {
                    // Dangerous recusivity
                    (key.to_string(), Self::from(each))
                }).collect();

                Self::Object(map)
            },
        }
    }
}

impl From<&serde_yaml::Value> for Value
{
    /// Converts [serde_yaml] deserialization results under a common value:
    /// [Value].
    /// 
    /// [serde_yaml](https://docs.serde.rs/serde_yaml/index.html)
    /// [Value](./struct.Value.html)
    fn from(yaml: &serde_yaml::Value) -> Self
    {
        match yaml {
            serde_yaml::Value::Null             => {
                Self::Null
            },
            serde_yaml::Value::String(ref str)  => {
                Self::String(str.to_string())
            },
            serde_yaml::Value::Bool(ref bool)   => {
                Self::Bool(*bool)
            },
            serde_yaml::Value::Number(ref n)    => {
                Self::Number(Number::from(n))
            },
            serde_yaml::Value::Sequence(yaml)   => {
                let vec: Vec<Self> = yaml.iter().map(|each| {
                    // Dangerous recusivity
                    Self::from(each)
                }).collect();

                Self::Array(vec)
            },
            serde_yaml::Value::Mapping(yaml)    => {
                let map: BTreeMap<String, Self> = yaml.iter()
                .map(|(key, each)| {
                    let key = {
                        if !key.is_string() {
                            unimplemented!();
                        }

                        key.as_str().unwrap().to_owned()
                    };

                    // Dangerous recusivity
                    (key, Self::from(each))
                }).collect();

                Self::Object(map)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_value() {
        let value = Value::Null;

        // Checks if the good value is attributed
        assert_eq!(value, Value::Null);

        // Checks if tester works fine
        assert!(value.is_null());

        // Checks if converter works fine
        assert!(value.as_null().is_some());
        assert_eq!(value.as_null().unwrap(), ());
    }

    #[test]
    fn bool_value() {
        let value = Value::Bool(true);

        // Checks if the good value is attributed
        assert_eq!(value, Value::Bool(true));

        // Checks if tester works fine
        assert!(value.is_boolean());

        // Checks if converter works fine
        assert!(value.as_bool().is_some());
        assert_eq!(value.as_bool().unwrap(), true);
    }

    #[test]
    fn string_value() {
        let value = Value::String("test string".to_owned());

        // Checks if the good value is attributed
        assert_eq!(value, Value::String("test string".to_owned()));

        // Checks if tester works fine
        assert!(value.is_string());

        // Checks if converter works fine
        assert!(value.as_str().is_some());
        assert_eq!(value.as_str().unwrap(), "test string".to_owned());
    }

    #[test]
    fn number_value() {
        let value_unsigned = Value::Number(Number::from(10u8));
        let value_signed = Value::Number(Number::from(10i8));
        let value_float = Value::Number(Number::from_f64(10.12).expect("failed to create number from float"));

        // Checks if the good value is attributed
        assert_eq!(value_unsigned, Value::Number(Number::from(10u8)));
        assert_eq!(value_signed, Value::Number(Number::from(10i8)));

        // Checks if tester works fine
        assert!(value_unsigned.is_number());
        assert!(value_unsigned.is_u64());
        assert!(value_signed.is_number());
        assert!(value_unsigned.is_i64());
        assert!(value_float.is_number());
        assert!(value_float.is_f64());

        // Checks if converters work fine
        assert!(value_unsigned.as_u64().is_some());
        assert_eq!(value_unsigned.as_u64().unwrap(), 10u64);

        assert!(value_unsigned.as_i64().is_some());
        assert_eq!(value_unsigned.as_i64().unwrap(), 10i64);

        assert!(value_float.as_f64().is_some());
        assert_eq!(value_float.as_f64().unwrap(), 10.12);
    }

    #[test]
    fn array_value() {
        let value = Value::Array(vec!(
            Value::String("test".to_owned()),
            Value::String("test 2".to_owned())
        ));

        // Checks if the good value is attributed
        assert_eq!(value, Value::Array(vec!(
            Value::String("test".to_owned()),
            Value::String("test 2".to_owned())
        )));

        // Checks if tester works fine
        assert!(value.is_array());

        // Checks if converter works fine
        assert!(value.as_array().is_some());
        assert_eq!(value.as_array().unwrap(), &vec!(
            Value::String("test".to_owned()),
            Value::String("test 2".to_owned())
        ));
    }

    #[test]
    fn object_value() {
        let value = Value::Object({
            let mut map = std::collections::BTreeMap::new();

            map.insert("name".to_owned(), Value::String("Doe".to_owned()));
            map.insert("firstname".to_owned(), Value::String("John".to_owned()));
            map
        });

        // Checks if the good value is attributed
        assert_eq!(value, Value::Object({
            let mut map = std::collections::BTreeMap::new();

            map.insert("name".to_owned(), Value::String("Doe".to_owned()));
            map.insert("firstname".to_owned(), Value::String("John".to_owned()));
            map
        }));

        // Checks if tester works fine
        assert!(value.is_object());

        // Checks if converter works fine
        assert!(value.as_object().is_some()); 
        assert_eq!(value.as_object().unwrap(), &{
            let mut map = std::collections::BTreeMap::new();

            map.insert("name".to_owned(), Value::String("Doe".to_owned()));
            map.insert("firstname".to_owned(), Value::String("John".to_owned()));
            map
        });

        // Checks if mut converter works fine
        let mut cloned_value = value.clone();
        assert!(cloned_value.as_object_mut().is_some()); 
        assert_eq!(cloned_value.as_object_mut().unwrap(), &mut {
            let mut map = std::collections::BTreeMap::new();

            map.insert("name".to_owned(), Value::String("Doe".to_owned()));
            map.insert("firstname".to_owned(), Value::String("John".to_owned()));
            map
        });

    }

    #[test]
    fn from_json_value() {
        let json = json!({
            "house": {
                "rooms": [
                    "kitchen",
                    "living room",
                    "toilet",
                    "room 1",
                    "room 2"
                ],
                "inhabitant_number": 2,
                "inhabitants": [
                    {
                        "name": "Doe",
                        "firstname": "John",
                        "age": 37.5,
                        "job": true,
                    },
                    {
                        "name": "Doe",
                        "firstname": "Jane",
                        "age": 36.4,
                        "job": true,
                    }
                ],
                "cars": null,
            }
        });

        // If it does not panic, it worked
        let json_value = Value::from(&json);
        assert_eq!(
            format!("{:?}", json_value),
            "Object({\"house\": Object({\"cars\": Null, \"inhabitant_number\": Number(2), \"inhabitants\": Array([Object({\"age\": Number(37.5), \"firstname\": String(\"John\"), \"job\": Bool(true), \"name\": String(\"Doe\")}), Object({\"age\": Number(36.4), \"firstname\": String(\"Jane\"), \"job\": Bool(true), \"name\": String(\"Doe\")})]), \"rooms\": Array([String(\"kitchen\"), String(\"living room\"), String(\"toilet\"), String(\"room 1\"), String(\"room 2\")])})})"
        );
    }

    #[test]
    fn from_yaml_value() {
        let yaml = serde_yaml::Value::Mapping({
            let mut mapping = serde_yaml::Mapping::new();

            mapping.insert(
                serde_yaml::Value::String("house".to_owned()),
                serde_yaml::Value::Mapping({
                    let mut mapping = serde_yaml::Mapping::new();

                    mapping.insert(
                        serde_yaml::Value::String("cars".to_owned()),
                        serde_yaml::Value::Null
                    );

                    mapping.insert(
                        serde_yaml::Value::String("rooms".to_owned()),
                        serde_yaml::Value::Sequence({
                            let mut sequence = serde_yaml::Sequence::new();

                            sequence.push(serde_yaml::Value::String("kitchen".to_owned()));
                            sequence.push(serde_yaml::Value::String("living room".to_owned()));
                            sequence.push(serde_yaml::Value::String("toilet".to_owned()));
                            sequence.push(serde_yaml::Value::String("room 1".to_owned()));
                            sequence.push(serde_yaml::Value::String("room 2".to_owned()));

                            sequence
                        })
                    );

                    mapping.insert(
                        serde_yaml::Value::String("inhabitant_number".to_owned()),
                        serde_yaml::Value::Number(serde_yaml::Number::from(2u64))
                    );

                    mapping.insert(
                        serde_yaml::Value::String("inhabitants".to_owned()),
                        serde_yaml::Value::Sequence({
                            let mut sequence = serde_yaml::Sequence::new();

                            sequence.push(serde_yaml::Value::Mapping({
                                let mut mapping = serde_yaml::Mapping::new();

                                mapping.insert(
                                    serde_yaml::Value::String("firstname".to_owned()),
                                    serde_yaml::Value::String("John".to_owned())
                                );
                                mapping.insert(
                                    serde_yaml::Value::String("name".to_owned()),
                                    serde_yaml::Value::String("Doe".to_owned())
                                );
                                mapping.insert(
                                    serde_yaml::Value::String("age".to_owned()),
                                    serde_yaml::Value::Number(
                                        serde_yaml::Number::from(37.5)
                                    )
                                );

                                mapping
                            }));

                            sequence.push(serde_yaml::Value::Mapping({
                                let mut mapping = serde_yaml::Mapping::new();

                                mapping.insert(
                                    serde_yaml::Value::String("firstname".to_owned()),
                                    serde_yaml::Value::String("Jane".to_owned())
                                );
                                mapping.insert(
                                    serde_yaml::Value::String("name".to_owned()),
                                    serde_yaml::Value::String("Doe".to_owned())
                                );
                                mapping.insert(
                                    serde_yaml::Value::String("age".to_owned()),
                                    serde_yaml::Value::Number(
                                        serde_yaml::Number::from(36.4)
                                    )
                                );

                                mapping
                            }));

                            sequence
                        })
                    );

                    mapping
                })
            );

            mapping
        });

        // If it does not panic, it worked
        let yaml_value = Value::from(&yaml);
        assert_eq!(
            format!("{:?}", yaml_value),
            "Object({\"house\": Object({\"cars\": Null, \"inhabitant_number\": Number(2), \"inhabitants\": Array([Object({\"age\": Number(37.5), \"firstname\": String(\"John\"), \"name\": String(\"Doe\")}), Object({\"age\": Number(36.4), \"firstname\": String(\"Jane\"), \"name\": String(\"Doe\")})]), \"rooms\": Array([String(\"kitchen\"), String(\"living room\"), String(\"toilet\"), String(\"room 1\"), String(\"room 2\")])})})"
        );
    }
}
