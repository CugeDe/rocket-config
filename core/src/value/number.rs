#![allow(dead_code)]

use std::fmt::{self, Debug};

/// Represents a number, whether integer or floating point.
#[derive(Clone, PartialEq, PartialOrd)]
pub struct Number {
    n: N,
}

#[cfg(not(feature = "arbitrary_precision"))]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
enum N {
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
    /// Always finite.
    Float(f64),
}

#[cfg(feature = "arbitrary_precision")]
type N = String;

impl Debug for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_tuple("Number");
        match self.n {
            N::PosInt(i) => {
                debug.field(&i);
            }
            N::NegInt(i) => {
                debug.field(&i);
            }
            N::Float(f) => {
                debug.field(&f);
            }
        }
        debug.finish()
    }
}

impl Number {
    /// Returns true if the `Number` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Number on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    #[inline]
    pub fn is_i64(&self) -> bool {
        #[cfg(not(feature = "arbitrary_precision"))]
        match self.n {
            N::PosInt(v) => v <= i64::max_value() as u64,
            N::NegInt(_) => true,
            N::Float(_) => false,
        }
        #[cfg(feature = "arbitrary_precision")]
        self.as_i64().is_some()
    }

    /// Returns true if the `Number` is an integer between zero and `u64::MAX`.
    ///
    /// For any Number on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    #[inline]
    pub fn is_u64(&self) -> bool {
        #[cfg(not(feature = "arbitrary_precision"))]
        match self.n {
            N::PosInt(_) => true,
            N::NegInt(_) | N::Float(_) => false,
        }
        #[cfg(feature = "arbitrary_precision")]
        self.as_u64().is_some()
    }

    /// Returns true if the `Number` can be represented by f64.
    ///
    /// For any Number on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    ///
    /// Currently this function returns true if and only if both `is_i64` and
    /// `is_u64` return false but this is not a guarantee in the future.
    #[inline]
    pub fn is_f64(&self) -> bool {
        #[cfg(not(feature = "arbitrary_precision"))]
        match self.n {
            N::Float(_) => true,
            N::PosInt(_) | N::NegInt(_) => false,
        }
        #[cfg(feature = "arbitrary_precision")]
        {
            for c in self.n.chars() {
                if c == '.' || c == 'e' || c == 'E' {
                    return self.n.parse::<f64>().ok().map_or(false, |f| f.is_finite());
                }
            }
            false
        }
    }

    /// If the `Number` is an integer, represent it as i64 if possible. Returns
    /// None otherwise.
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        #[cfg(not(feature = "arbitrary_precision"))]
        match self.n {
            N::PosInt(n) => {
                if n <= i64::max_value() as u64 {
                    Some(n as i64)
                } else {
                    None
                }
            }
            N::NegInt(n) => Some(n),
            N::Float(_) => None,
        }
        #[cfg(feature = "arbitrary_precision")]
        self.n.parse().ok()
    }

    /// If the `Number` is an integer, represent it as u64 if possible. Returns
    /// None otherwise.
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        #[cfg(not(feature = "arbitrary_precision"))]
        match self.n {
            N::PosInt(n) => Some(n),
            N::NegInt(_) | N::Float(_) => None,
        }
        #[cfg(feature = "arbitrary_precision")]
        self.n.parse().ok()
    }

    /// Represents the number as f64 if possible. Returns None otherwise.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        #[cfg(not(feature = "arbitrary_precision"))]
        match self.n {
            N::PosInt(n) => Some(n as f64),
            N::NegInt(n) => Some(n as f64),
            N::Float(n) => Some(n),
        }
        #[cfg(feature = "arbitrary_precision")]
        self.n.parse().ok()
    }

    /// Converts a finite `f64` to a `Number`. Infinite or NaN values are not
    /// numbers.
    #[inline]
    pub fn from_f64(f: f64) -> Option<Number> {
        if f.is_finite() {
            let n = {
                #[cfg(not(feature = "arbitrary_precision"))]
                {
                    N::Float(f)
                }
                #[cfg(feature = "arbitrary_precision")]
                {
                    ryu::Buffer::new().format_finite(f).to_owned()
                }
            };
            Some(Number { n })
        } else {
            None
        }
    }

    #[cfg(feature = "arbitrary_precision")]
    /// Not public API. Only tests use this.
    #[doc(hidden)]
    #[inline]
    pub fn from_string_unchecked(n: String) -> Self {
        Number { n }
    }
}

impl Eq for Number {}

impl Ord for Number 
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.n.cmp(&other.n)
    }
}

impl Eq for N {}

impl Ord for N 
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // PosInts
            (Self::PosInt(self_u64), Self::PosInt(other_u64))   => {
                // Real comparison
                self_u64.cmp(&other_u64)
            },
            (Self::PosInt(_self_u64), Self::NegInt(_other_i64))   => {
                // Faking by the order in enum (PosInt > NegInt > Float)
                std::cmp::Ordering::Greater
            },
            (Self::PosInt(_self_u64), Self::Float(_other_f64))   => {
                // Faking by the order in enum (PosInt > NegInt > Float)
                std::cmp::Ordering::Greater
            },

            // NegInts
            (Self::NegInt(self_i64), Self::NegInt(other_i64))   => {
                // Real comparison
                self_i64.cmp(&other_i64)
            },
            (Self::NegInt(_self_i64), Self::PosInt(_other_u64))   => {
                // Faking by the order in enum (PosInt > NegInt > Float)
                std::cmp::Ordering::Less
            },
            (Self::NegInt(_self_i64), Self::Float(_other_f64))   => {
                // Faking by the order in enum (PosInt > NegInt > Float)
                std::cmp::Ordering::Greater
            },

            // Floats
            (Self::Float(self_f64), Self::Float(other_f64))     => {
                // Pseudo-real comparison
                format!("{:e}", self_f64).cmp(&format!("{:e}", other_f64))
            },

            (Self::Float(_self_f64), Self::PosInt(_other_u64))     => {
                // Faking by the order in enum (PosInt > NegInt > Float)
                std::cmp::Ordering::Less
            },
            (Self::Float(_self_f64), Self::NegInt(_other_i64))     => {
                // Faking by the order in enum (PosInt > NegInt > Float)
                std::cmp::Ordering::Less
            },
        }
    }
}

impl From<&serde_json::Number> for Number
{
    fn from(json: &serde_json::Number) -> Self
    {
        match json {
            u64_val if u64_val.is_u64() => {
                Self { n: N::PosInt(u64_val.as_u64().unwrap()) }
            },
            i64_val if i64_val.is_i64() => {
                Self { n: N::NegInt(i64_val.as_i64().unwrap()) }
            },
            f64_val if f64_val.is_f64() => {
                Self { n: N::Float(f64_val.as_f64().unwrap()) }
            },
            _                           => {
                unimplemented!()
            }
        }
    }
}

impl From<&serde_yaml::Number> for Number
{
    fn from(yaml: &serde_yaml::Number) -> Self
    {
        match yaml {
            u64_val if u64_val.is_u64() => {
                Self { n: N::PosInt(u64_val.as_u64().unwrap()) }
            },
            i64_val if i64_val.is_i64() => {
                Self { n: N::NegInt(i64_val.as_i64().unwrap()) }
            },
            f64_val if f64_val.is_f64() => {
                Self { n: N::Float(f64_val.as_f64().unwrap()) }
            },
            _                           => {
                unimplemented!()
            }
        }
    }
}

macro_rules! impl_from_unsigned {
    (
        $($ty:ty),*
    ) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(u: $ty) -> Self {
                    let n = {
                        #[cfg(not(feature = "arbitrary_precision"))]
                        { N::PosInt(u as u64) }
                        #[cfg(feature = "arbitrary_precision")]
                        {
                            itoa::Buffer::new().format(u).to_owned()
                        }
                    };
                    Number { n: n }
                }
            }
        )*
    };
}

macro_rules! impl_from_signed {
    (
        $($ty:ty),*
    ) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(i: $ty) -> Self {
                    let n = {
                        #[cfg(not(feature = "arbitrary_precision"))]
                        {
                            if i < 0 {
                                N::NegInt(i as i64)
                            } else {
                                N::PosInt(i as u64)
                            }
                        }
                        #[cfg(feature = "arbitrary_precision")]
                        {
                            itoa::Buffer::new().format(i).to_owned()
                        }
                    };
                    Number { n: n }
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_signed!(i8, i16, i32, i64, isize);