#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::types::TypeId;

use super::{Checker, UnionReduction};

/// See: checker.ts, line 19871, getTypeFromTypeNodeWorker
pub(crate) trait GetTypeFromTypeNode<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId;
}

impl<'a> GetTypeFromTypeNode<'a> for TSType<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        match self {
            Self::TSAnyKeyword(_) => checker.intrinsics.any,
            Self::TSUnknownKeyword(_) => checker.intrinsics.unknown,
            Self::TSStringKeyword(_) => checker.intrinsics.string,
            Self::TSNumberKeyword(_) => checker.intrinsics.number,
            Self::TSBigIntKeyword(_) => checker.intrinsics.bigint,
            Self::TSBooleanKeyword(_) => todo!("get_type_from_type_node(boolean)"), //checker.intrinsics.boolean,
            Self::TSSymbolKeyword(_) => checker.intrinsics.es_symbol,
            Self::TSVoidKeyword(_) => checker.intrinsics.void,
            Self::TSUndefinedKeyword(_) => checker.intrinsics.undefined,
            Self::TSNullKeyword(_) => checker.intrinsics.null,
            Self::TSNeverKeyword(_) => checker.intrinsics.never,
            Self::TSObjectKeyword(_) => {
                if checker.semantic.source_type().is_javascript()
                    && !checker.settings.no_implicit_any
                {
                    checker.intrinsics.any
                } else {
                    checker.intrinsics.non_primitive
                }
            }
            Self::TSIntrinsicKeyword(_) => checker.intrinsics.intrinsic_marker,
            Self::TSThisType(this) => this.get_type_from_type_node(checker),
            Self::TSLiteralType(lit) => lit.get_type_from_type_node(checker),
            Self::TSTypeReference(ty) => ty.get_type_from_type_node(checker),
            Self::TSTypePredicate(pred) => pred.get_type_from_type_node(checker),
            // ExpressionWithTypeArguments
            Self::TSTypeQuery(query) => query.get_type_from_type_node(checker),
            Self::TSUnionType(union) => union.get_type_from_type_node(checker),
            _ => todo!("get_type_from_type_node: {:?}", self),
        }
    }
}

impl<'a> GetTypeFromTypeNode<'a> for TSThisType {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        todo!("get_type_from_type_node(TSThisType): {:?}", self)
    }
}

impl<'a> GetTypeFromTypeNode<'a> for TSLiteralType<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        todo!("get_type_from_type_node(TSLiteralType): {:?}", self)
    }
}

impl<'a> GetTypeFromTypeNode<'a> for TSTypeReference<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        todo!("get_type_from_type_node(TSTypeReference): {:?}", self)
    }
}

impl<'a> GetTypeFromTypeNode<'a> for TSTypePredicate<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        todo!("get_type_from_type_node(TSTypePredicate): {:?}", self)
    }
}

impl<'a> GetTypeFromTypeNode<'a> for TSTypeQuery<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        todo!("get_type_from_type_node(TSTypeQuery): {:?}", self)
    }
}

// function getTypeFromUnionTypeNode(node: UnionTypeNode): Type {
//     const links = getNodeLinks(node);
//     if (!links.resolvedType) {
//         const aliasSymbol = getAliasSymbolForTypeNode(node);
//         links.resolvedType = getUnionType(map(node.types, getTypeFromTypeNode), UnionReduction.Literal, aliasSymbol, getTypeArgumentsForAliasSymbol(aliasSymbol));
//     }
//     return links.resolvedType;
// }
impl<'a> GetTypeFromTypeNode<'a> for TSUnionType<'a> {
    fn get_type_from_type_node(&self, checker: &Checker<'a>) -> TypeId {
        let types =
            self.types.iter().map(|ty| ty.get_type_from_type_node(checker)).collect::<Vec<_>>();
        // TODO
        // let type_alias_arguments = checker.get_type_arguments_for_alias_symbol();
        checker.get_union_type(
            &types,
            UnionReduction::Literal,
            /* todo: aliasSymbol */ None,
            /* todo: typeAliasArguments */ None,
            None,
        )
    }
}
