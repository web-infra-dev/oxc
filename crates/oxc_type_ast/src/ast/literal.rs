use oxc_syntax::types::TypeId;

#[derive(Debug)]
pub enum LiteralType<'a> {
    String(StringLiteralType<'a>),
    Number(NumberLiteralType),
    BigInt(BigIntLiteralType<'a>),
}

#[derive(Debug)]
pub struct StringLiteralType<'a> {
    pub id: TypeId,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct NumberLiteralType {
    pub id: TypeId,
    pub value: f64,
}

#[derive(Debug)]
pub struct BigIntLiteralType<'a> {
    pub id: TypeId,
    /// base-10 string representation of the BigInt
    pub raw: &'a str,
}
