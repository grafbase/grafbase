use std::fmt::Write;

use engine_value::ConstValue;

use crate::{Error, InputValueError, InputValueResult};

mod string;
pub use string::StringScalar;
mod id;
pub use id::IDScalar;
mod boolean;
pub use boolean::BooleanScalar;
mod float;
pub use float::FloatScalar;
mod int;
pub use int::IntScalar;
mod datetime;
pub use datetime::DateTimeScalar;
mod date;
pub use date::DateScalar;
mod email;
pub use email::EmailScalar;
mod json;
pub use json::JSONScalar;
mod url;
pub use self::url::URLScalar;
mod ipaddr;
pub use ipaddr::IPAddressScalar;
mod timestamp;
pub use timestamp::TimestampScalar;
mod phone;
pub use phone::PhoneNumberScalar;
mod decimal;
pub use decimal::DecimalScalar;
mod bigint;
pub use bigint::BigIntScalar;
mod unsigned_bigint;
pub use unsigned_bigint::UnsignedBigIntScalar;
mod bytes;
pub use self::bytes::BytesScalar;
mod hex_bytes;
pub use self::hex_bytes::HexBytesScalar;
mod time;
pub use self::time::TimeScalar;
mod naive_datetime;
pub use naive_datetime::NaiveDateTimeScalar;
mod uuid;
pub use self::uuid::UuidScalar;
mod federation;
pub use self::federation::*;

/// ` SDLDefinitionScalar` trait is to be implemented for every custom scalar we add into `engine`
///
/// The purpose of this trait is to give a definition of a Scalar based on a SDL.
/// It's part of the definition of the `OutputType` but without a `Registry`
pub trait SDLDefinitionScalar<'a> {
    /// Name of the Scalar
    fn name() -> Option<&'a str> {
        None
    }

    /// If internal it means we won't generate the SDL of this scalar
    fn internal() -> bool {
        false
    }

    /// Url to describe the scalar if needed
    fn specified_by() -> Option<&'a str> {
        None
    }

    /// Description of the scalar
    fn description() -> Option<&'a str> {
        None
    }

    /// Write the scalar into SDL
    fn sdl() -> String {
        if Self::internal() {
            return String::new();
        }

        if let Some(name) = Self::name() {
            let mut sdl = String::new();
            if let Some(desc) = Self::description() {
                writeln!(sdl, "\"\"\"\n{desc}\n\"\"\"").ok();
            }
            let directive = Self::specified_by()
                .map(|directive| format!("@specifiedBy(url: \"{directive}\")"))
                .unwrap_or_default();
            writeln!(sdl, "scalar {name} {directive}").ok();
            sdl
        } else {
            String::new()
        }
    }
}

pub trait DynamicParse {
    /// Parse a scalar value and execute an Input Coercion
    /// When a value is given, depending on the scalar we can transform this value into something
    /// else before doing any transformation or saving it inside the database, it's called `Input
    /// coercion`.
    /// TODO: We need to change a little the parsing workflow to allow this kind of coercion.
    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value>;

    /// Checks for a valid scalar value.
    ///
    /// Implementing this function can find incorrect input values during the verification phase.
    /// TODO: We are actually parsing the value to check if the value is coherent, when the input
    /// coercion will be working, we can change it inside the expected validators.
    fn is_valid(_value: &ConstValue) -> bool;

    /// Result Coercion
    /// We take the value from the database and depending on the scalar we convert it into the
    /// format described by the Result Coercion.
    ///
    /// Can fail if the data can't be coerced
    ///
    /// TODO: We need to change a little the fetching workflow to allow this kind of coercion.
    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error>;
}

/// Dynamic Scalar allow us to have a proper check over the expected type
pub trait DynamicScalar: Sized {
    /// Parse a scalar value.
    /// Input Coercion
    fn parse<S: AsRef<str>>(expected_ty: S, value: ConstValue) -> InputValueResult<serde_json::Value>;

    /// Checks for a valid scalar value.
    ///
    /// Implementing this function can find incorrect input values during the verification phase, which can improve performance.
    fn is_valid<S: AsRef<str>>(expected_ty: S, _value: &ConstValue) -> bool;

    /// Result coercion
    fn to_value<S: AsRef<str>>(expected_ty: S, value: serde_json::Value) -> Result<ConstValue, Error>;

    fn test_scalar_name<S: AsRef<str>>(expected_ty: S) -> bool;
    fn test_scalar_name_recursive<S: AsRef<str>>(expected_ty: S) -> bool;
}

pub struct PossibleScalarNil;

impl PossibleScalarNil {
    #[allow(dead_code)]
    pub(crate) const fn with<V>(self, visitor: V) -> PossibleScalarCons<V, Self> {
        PossibleScalarCons(visitor, self)
    }
}

impl DynamicScalar for PossibleScalarNil {
    fn test_scalar_name<S: AsRef<str>>(_expected_ty: S) -> bool {
        false
    }

    fn test_scalar_name_recursive<S: AsRef<str>>(_expected_ty: S) -> bool {
        false
    }

    fn parse<S: AsRef<str>>(expected_ty: S, _value: ConstValue) -> InputValueResult<serde_json::Value> {
        Err(InputValueError::ty_custom(
            expected_ty.as_ref(),
            "Internal error while parsing this scalar.",
        ))
    }

    fn is_valid<S: AsRef<str>>(_expected_ty: S, _value: &ConstValue) -> bool {
        false
    }

    fn to_value<S: AsRef<str>>(expected_ty: S, _value: serde_json::Value) -> Result<ConstValue, Error> {
        Err(Error::new(format!(
            "Internal error: unknown type '{}'",
            expected_ty.as_ref()
        )))
    }
}

pub struct PossibleScalarCons<A, B>(A, B);

impl<A, B> PossibleScalarCons<A, B> {
    #[allow(dead_code)]
    pub(crate) const fn with<V>(self, visitor: V) -> PossibleScalarCons<V, Self> {
        PossibleScalarCons(visitor, self)
    }
}

impl<'a, A, B> DynamicScalar for PossibleScalarCons<A, B>
where
    A: DynamicParse + SDLDefinitionScalar<'a>,
    B: DynamicScalar,
{
    fn test_scalar_name<S: AsRef<str>>(expected_ty: S) -> bool {
        A::name().unwrap_or_default() == expected_ty.as_ref()
    }

    fn test_scalar_name_recursive<S: AsRef<str>>(expected_ty: S) -> bool {
        A::name().unwrap_or_default() == expected_ty.as_ref() || B::test_scalar_name_recursive(&expected_ty)
    }

    fn parse<S: AsRef<str>>(expected_ty: S, value: ConstValue) -> InputValueResult<serde_json::Value> {
        if Self::test_scalar_name(&expected_ty) {
            A::parse(value)
        } else {
            B::parse(expected_ty, value)
        }
    }

    fn is_valid<S: AsRef<str>>(expected_ty: S, value: &ConstValue) -> bool {
        if Self::test_scalar_name(&expected_ty) {
            A::is_valid(value)
        } else {
            B::is_valid(expected_ty, value)
        }
    }

    fn to_value<S: AsRef<str>>(expected_ty: S, value: serde_json::Value) -> Result<ConstValue, Error> {
        if Self::test_scalar_name(&expected_ty) {
            A::to_value(value)
        } else {
            B::to_value(expected_ty, value)
        }
    }
}

const SPECIFIED_BY_DIRECTIVE: &str = r#"
directive @specifiedBy(url: String!) on SCALAR
"#;

impl<'a> SDLDefinitionScalar<'a> for PossibleScalarNil {
    fn sdl() -> String {
        SPECIFIED_BY_DIRECTIVE.to_string()
    }
}

impl<'a, A, B> SDLDefinitionScalar<'a> for PossibleScalarCons<A, B>
where
    A: SDLDefinitionScalar<'a>,
    B: SDLDefinitionScalar<'a>,
{
    fn sdl() -> String {
        let mut sdl = String::new();
        writeln!(sdl, "{}", <A as SDLDefinitionScalar>::sdl()).ok();
        writeln!(sdl, "{}", <B as SDLDefinitionScalar>::sdl()).ok();
        sdl
    }
}

macro_rules! merge_scalar {
    ($a:ty) => {
        PossibleScalarCons<$a, PossibleScalarNil>
    };

    ($a:ty, $ab:ty) => {
        PossibleScalarCons<$a, merge_scalar!($ab)>
    };

    ($a:ty, $ab:ty, $($bc:ty),+) => {
        PossibleScalarCons<$a, merge_scalar!($ab, $($bc),*)>
    };
}

/// Public type which expose every available dynamic Scalar
pub type PossibleScalar = merge_scalar!(
    IDScalar,
    StringScalar,
    BooleanScalar,
    FloatScalar,
    IntScalar,
    EmailScalar,
    DateTimeScalar,
    DateScalar,
    JSONScalar,
    IPAddressScalar,
    URLScalar,
    TimestampScalar,
    PhoneNumberScalar,
    DecimalScalar,
    BigIntScalar,
    BytesScalar,
    HexBytesScalar,
    UnsignedBigIntScalar,
    TimeScalar,
    NaiveDateTimeScalar,
    UuidScalar,
    FederationAnyScalar
);
