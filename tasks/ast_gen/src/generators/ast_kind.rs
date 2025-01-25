//! Generator of code related to `AstKind`.
//!
//! * `AstType` type definition.
//! * `AstKind` type definition.
//! * `AstKind::ty` method.
//! * `AstKind::as_*` methods.
//! * `GetSpan` impl for `AstKind`.
//!
//! Variants of `AstKind` and `AstType` are not created for types listed in `BLACK_LIST` below.

use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::LitInt;

use crate::{
    output::{output_path, Output},
    schema::{Def, Schema, TypeDef},
    Generator,
};

use super::define_generator;

pub struct AstKindGenerator;

define_generator!(AstKindGenerator);

pub const BLACK_LIST: [&str; 62] = [
    "Span",
    "Expression",
    "ObjectPropertyKind",
    "TemplateElement",
    "ComputedMemberExpression",
    "StaticMemberExpression",
    "PrivateFieldExpression",
    "AssignmentTargetRest",
    "AssignmentTargetMaybeDefault",
    "AssignmentTargetProperty",
    "AssignmentTargetPropertyIdentifier",
    "AssignmentTargetPropertyProperty",
    "ChainElement",
    "Statement",
    "Declaration",
    "ForStatementLeft",
    "BindingPattern",
    "BindingPatternKind",
    "BindingProperty",
    "ClassElement",
    "AccessorProperty",
    "ImportDeclarationSpecifier",
    "WithClause",
    "ImportAttribute",
    "ImportAttributeKey",
    "ExportDefaultDeclarationKind",
    "ModuleExportName",
    "TSEnumMemberName",
    "TSLiteral",
    "TSType",
    "TSTypeOperator",
    "TSArrayType",
    "TSTupleType",
    "TSOptionalType",
    "TSRestType",
    "TSTupleElement",
    "TSInterfaceBody",
    "TSSignature",
    "TSIndexSignature",
    "TSCallSignatureDeclaration",
    "TSIndexSignatureName",
    "TSTypePredicate",
    "TSTypePredicateName",
    "TSModuleDeclarationName",
    "TSModuleDeclarationBody",
    "TSTypeQueryExprName",
    "TSImportAttribute",
    "TSImportAttributes",
    "TSImportAttributeName",
    "TSFunctionType",
    "TSConstructorType",
    "TSNamespaceExportDeclaration",
    "JSDocNullableType",
    "JSDocNonNullableType",
    "JSDocUnknownType",
    "JSXExpression",
    "JSXEmptyExpression",
    "JSXAttribute",
    "JSXAttributeName",
    "JSXAttributeValue",
    "JSXChild",
    "JSXSpreadChild",
];

impl Generator for AstKindGenerator {
    /// Set `has_kind` for structs and enums which are not on blacklist.
    fn modify(&self, schema: &mut Schema) {
        for type_def in &mut schema.types {
            match type_def {
                TypeDef::Struct(struct_def) => {
                    if struct_def.is_visited() && !BLACK_LIST.contains(&struct_def.name()) {
                        struct_def.has_kind = true;
                    }
                }
                TypeDef::Enum(enum_def) => {
                    if enum_def.is_visited() && !BLACK_LIST.contains(&enum_def.name()) {
                        enum_def.has_kind = true;
                    }
                }
                _ => {}
            }
        }
    }

    /// Generate `AstKind` etc definitions.
    fn generate(&self, schema: &Schema) -> Output {
        let mut type_variants = vec![];
        let mut kind_variants = vec![];
        let mut span_match_arms = vec![];
        let mut as_methods = vec![];

        let mut next_index = 0usize;
        for type_def in &schema.types {
            if !type_def.has_kind() {
                continue;
            }

            let type_ident = type_def.ident();
            let type_ty = type_def.ty(schema);

            let index = u8::try_from(next_index).unwrap();
            let index = LitInt::new(&index.to_string(), Span::call_site());
            type_variants.push(quote!( #type_ident = #index ));
            kind_variants.push(quote!( #type_ident(&'a #type_ty) = AstType::#type_ident as u8 ));

            span_match_arms.push(quote!( Self::#type_ident(it) => it.span() ));

            let as_method_name = format_ident!("as_{}", type_def.snake_name());
            as_methods.push(quote! {
                ///@@line_break
                #[inline]
                pub fn #as_method_name(self) -> Option<&'a #type_ty> {
                    if let Self::#type_ident(v) = self {
                        Some(v)
                    } else {
                        None
                    }
                }
            });

            next_index += 1;
        }

        let output = quote! {
            #![allow(missing_docs)] ///@ FIXME (in ast_tools/src/generators/ast_kind.rs)

            ///@@line_break
            use std::ptr;

            ///@@line_break
            use oxc_span::{GetSpan, Span};

            ///@@line_break
            use crate::ast::*;

            ///@@line_break
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[repr(u8)]
            pub enum AstType {
                #(#type_variants,)*
            }

            ///@@line_break
            /// Untyped AST Node Kind
            #[derive(Debug, Clone, Copy)]
            #[repr(C, u8)]
            pub enum AstKind<'a> {
                #(#kind_variants,)*
            }

            ///@@line_break
            impl AstKind<'_> {
                /// Get the [`AstType`] of an [`AstKind`].
                #[inline]
                pub fn ty(&self) -> AstType {
                    ///@ SAFETY: `AstKind` is `#[repr(C, u8)]`, so discriminant is stored in first byte,
                    ///@ and it's valid to read it.
                    ///@ `AstType` is also `#[repr(u8)]` and `AstKind` and `AstType` both have the same
                    ///@ discriminants, so it's valid to read `AstKind`'s discriminant as `AstType`.
                    unsafe { *ptr::from_ref(self).cast::<AstType>().as_ref().unwrap_unchecked() }
                }
            }

            ///@@line_break
            impl GetSpan for AstKind<'_> {
                fn span(&self) -> Span {
                    match self {
                        #(#span_match_arms,)*
                    }
                }
            }

            ///@@line_break
            impl<'a> AstKind<'a> {
                #(#as_methods)*
            }
        };

        Output::Rust { path: output_path(crate::AST_CRATE, "ast_kind.rs"), tokens: output }
    }
}
