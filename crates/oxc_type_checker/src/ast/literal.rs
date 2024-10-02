use std::{hash::Hash, ops::Deref};

use oxc_ast::ast::BigintBase;
use oxc_span::Atom;

use super::FreshableType;

#[derive(Debug, Clone)]
pub struct PseudoBigInt<'a> {
    pub raw: Atom<'a>,
    pub base: BigintBase,
}

#[derive(Debug, Default, Clone, Copy, PartialOrd)]
#[repr(transparent)]
pub struct Number(f64);

impl Number {
    #[inline]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn is_int(self) -> bool {
        Self::eq(&self, &Number(self.round()))
    }
}

impl Deref for Number {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<f64> for Number {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if (self.0 - self.round()) < f64::EPSILON {
            if self.0 < i64::MAX as f64 {
                (self.0 as i64).hash(state);
            } else if self.0 < i128::MAX as f64 {
                (self.0 as i128).hash(state);
            } else {
                self.0.to_bits().hash(state);
            }
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl Eq for Number {}

pub type LiteralType<'a> = FreshableType<FreshLiteralType<'a>>;

#[derive(Debug)]
pub enum FreshLiteralType<'a> {
    String(StringLiteralType<'a>),
    Number(NumberLiteralType),
    BigInt(BigIntLiteralType<'a>),
}

#[derive(Debug)]
pub struct StringLiteralType<'a> {
    pub value: Atom<'a>,
}

#[derive(Debug)]
pub struct NumberLiteralType {
    pub value: Number,
}

#[derive(Debug)]
pub struct BigIntLiteralType<'a> {
    /// base-10 string representation of the BigInt
    pub value: PseudoBigInt<'a>,
}

impl<'a> PseudoBigInt<'a> {
    pub fn new(raw: Atom<'a>, base: BigintBase) -> Self {
        Self { raw, base }
    }
}

impl<'a> From<&'a str> for FreshLiteralType<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(StringLiteralType { value: Atom::from(value) })
    }
}

impl<'a> From<Atom<'a>> for FreshLiteralType<'a> {
    fn from(value: Atom<'a>) -> Self {
        Self::String(StringLiteralType { value })
    }
}

impl From<f64> for FreshLiteralType<'static> {
    fn from(value: f64) -> Self {
        Self::Number(NumberLiteralType { value: value.into() })
    }
}

impl From<Number> for FreshLiteralType<'_> {
    fn from(value: Number) -> Self {
        Self::Number(NumberLiteralType { value })
    }
}

impl<'a> From<PseudoBigInt<'a>> for FreshLiteralType<'a> {
    fn from(value: PseudoBigInt<'a>) -> Self {
        Self::BigInt(BigIntLiteralType { value })
    }
}
