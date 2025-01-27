// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_gen/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::operator::*;

impl Serialize for AssignmentOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AssignmentOperator::Assign(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::Addition(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::Subtraction(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::Multiplication(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::Division(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::Remainder(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::Exponential(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::ShiftLeft(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::ShiftRight(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::ShiftRightZeroFill(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::BitwiseOR(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::BitwiseXOR(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::BitwiseAnd(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::LogicalOr(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::LogicalAnd(x) => Serialize::serialize(x, serializer),
            AssignmentOperator::LogicalNullish(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for BinaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BinaryOperator::Equality(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Inequality(x) => Serialize::serialize(x, serializer),
            BinaryOperator::StrictEquality(x) => Serialize::serialize(x, serializer),
            BinaryOperator::StrictInequality(x) => Serialize::serialize(x, serializer),
            BinaryOperator::LessThan(x) => Serialize::serialize(x, serializer),
            BinaryOperator::LessEqualThan(x) => Serialize::serialize(x, serializer),
            BinaryOperator::GreaterThan(x) => Serialize::serialize(x, serializer),
            BinaryOperator::GreaterEqualThan(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Addition(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Subtraction(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Multiplication(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Division(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Remainder(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Exponential(x) => Serialize::serialize(x, serializer),
            BinaryOperator::ShiftLeft(x) => Serialize::serialize(x, serializer),
            BinaryOperator::ShiftRight(x) => Serialize::serialize(x, serializer),
            BinaryOperator::ShiftRightZeroFill(x) => Serialize::serialize(x, serializer),
            BinaryOperator::BitwiseOR(x) => Serialize::serialize(x, serializer),
            BinaryOperator::BitwiseXOR(x) => Serialize::serialize(x, serializer),
            BinaryOperator::BitwiseAnd(x) => Serialize::serialize(x, serializer),
            BinaryOperator::In(x) => Serialize::serialize(x, serializer),
            BinaryOperator::Instanceof(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for LogicalOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LogicalOperator::Or(x) => Serialize::serialize(x, serializer),
            LogicalOperator::And(x) => Serialize::serialize(x, serializer),
            LogicalOperator::Coalesce(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for UnaryOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            UnaryOperator::UnaryPlus(x) => Serialize::serialize(x, serializer),
            UnaryOperator::UnaryNegation(x) => Serialize::serialize(x, serializer),
            UnaryOperator::LogicalNot(x) => Serialize::serialize(x, serializer),
            UnaryOperator::BitwiseNot(x) => Serialize::serialize(x, serializer),
            UnaryOperator::Typeof(x) => Serialize::serialize(x, serializer),
            UnaryOperator::Void(x) => Serialize::serialize(x, serializer),
            UnaryOperator::Delete(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for UpdateOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            UpdateOperator::Increment(x) => Serialize::serialize(x, serializer),
            UpdateOperator::Decrement(x) => Serialize::serialize(x, serializer),
        }
    }
}
