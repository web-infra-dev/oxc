//! Intrinsic type constants. Near the top of `checker.ts`
use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};

use super::{TypeBuilder, TypeCache, TypeList};

use crate::{
    ast::{FreshableType, Type},
    CheckerSettings,
};

/// Intrinsics that checker.ts creates in createTypeChecker lol
///
/// checker.ts, line 2046 (commit `3386e943215613c40f68ba0b108cda1ddb7faee1`)
#[derive(Debug)]
pub(crate) struct Intrinsics {
    /// any
    pub any: TypeId,
    /// any
    pub auto: TypeId,
    /// any
    pub wildcard: TypeId,
    /// any
    pub blocked_string: TypeId,
    /// anyma
    pub error: TypeId,
    /// any
    pub unresolved: TypeId,
    /// any
    pub non_inferrable_any: TypeId,
    /// any
    pub intrinsic_marker: TypeId,
    /// unknown
    pub unknown: TypeId,
    /// undefined
    pub undefined: TypeId,
    /// undefined
    pub undefined_widening: TypeId,
    /// undefined
    pub missing: TypeId,
    /// missing or undefined, depending on exactOptionalPropertyTypes
    pub undefined_or_missing: TypeId,
    /// undefined
    pub optional: TypeId,
    /// null
    pub null: TypeId,
    /// null, strictNullChecks: false is not supported rn
    pub null_widening: TypeId,
    /// string
    pub string: TypeId,
    /// number
    pub number: TypeId,
    /// bigint
    pub bigint: TypeId,
    /// false (fresh)
    pub false_type: TypeId,
    /// false
    pub regular_false: TypeId,
    /// true (fresh)
    pub true_type: TypeId,
    /// true
    pub regular_true: TypeId,
    /// boolean
    pub boolean: TypeId,
    /// symbol
    pub es_symbol: TypeId,
    /// void
    pub void: TypeId,
    /// never
    pub never: TypeId,
    /// never
    pub silent_never: TypeId,
    /// never
    pub implicit_never: TypeId,
    /// never
    pub unreachable_never: TypeId,
    /// never
    pub non_primitive: TypeId,
    /// never
    pub unique_literal: TypeId,
}

impl Intrinsics {
    pub fn new(
        builder: &TypeBuilder<'_>,
        settings: &CheckerSettings,
        cache: &TypeCache<'_>,
    ) -> Self {
        let any = builder.create_intrinsic_type(TypeFlags::Any, "any", None);
        let auto = builder.create_intrinsic_type(TypeFlags::Any, "any", Some("auto"));
        let wildcard = builder.create_intrinsic_type(TypeFlags::Any, "any", Some("wildcard"));
        let blocked_string =
            builder.create_intrinsic_type(TypeFlags::Any, "any", Some("blocked string"));
        let error = builder.create_intrinsic_type(TypeFlags::Any, "any", Some("error"));
        let unresolved = builder.create_intrinsic_type(TypeFlags::Any, "any", Some("unresolved"));
        let non_inferrable_any =
            builder.create_intrinsic_type(TypeFlags::Any, "any", Some("non-inferrable"));
        let intrinsic_marker =
            builder.create_intrinsic_type(TypeFlags::Any, "any", Some("intrinsic"));
        let unknown = builder.create_intrinsic_type(TypeFlags::Unknown, "unknown", None);
        let undefined = builder.create_intrinsic_type(TypeFlags::Undefined, "undefined", None);
        let undefined_widening = if settings.strict_null_checks {
            undefined
        } else {
            // createIntrinsicType(TypeFlags.Undefined, "undefined", ObjectFlags.ContainsWideningType, "widening")
            todo!(
                "[checker::intrinsics::undefined_widening] strictNullChecks: false is not supported rn."
            );
        };
        let missing =
            builder.create_intrinsic_type(TypeFlags::Undefined, "undefined", Some("missing"));
        let undefined_or_missing =
            if settings.exact_optional_property_types { missing } else { undefined };
        let optional =
            builder.create_intrinsic_type(TypeFlags::Undefined, "undefined", Some("optional"));
        let null = builder.create_intrinsic_type(TypeFlags::Null, "null", None);
        let null_widening = if settings.strict_null_checks {
            null
        } else {
            // createIntrinsicType(TypeFlags.Null, "null", ObjectFlags.ContainsWideningType, "widening")
            todo!(
                "[checker::intrinsics::null_widening] strictNullChecks: false is not supported rn."
            );
        };
        let string = builder.create_intrinsic_type(TypeFlags::String, "string", None);
        let number = builder.create_intrinsic_type(TypeFlags::Number, "number", None);
        let bigint = builder.create_intrinsic_type(TypeFlags::BigInt, "bigint", None);

        let false_type = builder.create_fresh_freshable_intrinsic_type(
            TypeFlags::BooleanLiteral,
            "false",
            Some("fresh"),
            None,
        );
        let regular_false = builder.create_regular_freshable_intrinsic_type(
            TypeFlags::BooleanLiteral,
            "false",
            None,
            false_type,
        );
        let true_type = builder.create_fresh_freshable_intrinsic_type(
            TypeFlags::BooleanLiteral,
            "true",
            Some("fresh"),
            None,
        );
        let regular_true = builder.create_regular_freshable_intrinsic_type(
            TypeFlags::BooleanLiteral,
            "true",
            None,
            true_type,
        );
        Self::init_freshable_intrinsic(builder, true_type, regular_true);
        Self::init_freshable_intrinsic(builder, false_type, regular_false);

        let boolean_union = cache.type_list(&[regular_false, regular_true]);
        let boolean =
            builder.create_union_type(&boolean_union, ObjectFlags::empty(), None, None, None);
        cache.add_union(boolean_union, boolean);

        let es_symbol = builder.create_intrinsic_type(TypeFlags::ESSymbol, "symbol", None);
        let void = builder.create_intrinsic_type(TypeFlags::Void, "void", None);
        let never = builder.create_intrinsic_type(TypeFlags::Never, "never", None);
        let silent_never = builder.create_intrinsic_type(TypeFlags::Never, "never", Some("silent"));
        let implicit_never =
            builder.create_intrinsic_type(TypeFlags::Never, "never", Some("implicit"));
        let unreachable_never =
            builder.create_intrinsic_type(TypeFlags::Never, "never", Some("unreachable"));
        let non_primitive = builder.create_intrinsic_type(TypeFlags::NonPrimitive, "object", None);
        let unique_literal =
            builder.create_intrinsic_type(TypeFlags::Never, "never", Some("unique literal"));

        Self {
            any,
            auto,
            wildcard,
            blocked_string,
            error,
            unresolved,
            non_inferrable_any,
            intrinsic_marker,
            unknown,
            undefined,
            undefined_widening,
            missing,
            undefined_or_missing,
            optional,
            null,
            null_widening,
            string,
            number,
            bigint,
            false_type,
            regular_false,
            true_type,
            regular_true,
            boolean,
            es_symbol,
            void,
            never,
            silent_never,
            implicit_never,
            unreachable_never,
            non_primitive,
            unique_literal,
        }
    }

    fn init_freshable_intrinsic(
        builder: &TypeBuilder<'_>,
        fresh_type_id: TypeId,
        regular_type_id: TypeId,
    ) {
        let mut table = builder.table_mut();
        let fresh_type = table.get_type_mut(fresh_type_id);
        if let Type::FreshableIntrinsic(fresh_type) = fresh_type {
            if let FreshableType::Fresh(fresh_type, regular_type) = fresh_type.as_mut() {
                debug_assert!(regular_type.is_none());
                regular_type.replace(regular_type_id);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    }
}
