use std::cell::Cell;

use bitflags::bitflags;

use oxc_allocator::Vec;
use oxc_semantic::{NodeId, SymbolId};
use oxc_syntax::types::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignatureKind {
    Call,
    Construct,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SignatureFlags: u16 {
        const None = 0;

        // Propagating flags

        /// Indicates last parameter is rest parameter
        const HasRestParameter = 1 << 0;
        /// Indicates signature is specialized
        const HasLiteralTypes = 1 << 1;
        /// Indicates signature comes from an abstract class, abstract construct signature, or abstract constructor type
        const Abstract = 1 << 2;

        // Non-propagating flags

        /// Indicates signature comes from a CallChain nested in an outer OptionalChain
        const IsInnerCallChain = 1 << 3;
        /// Indicates signature comes from a CallChain that is the outermost chain of an optional expression
        const IsOuterCallChain = 1 << 4;
        /// Indicates signature is from a js file and has no types
        const IsUntypedSignatureInJSFile = 1 << 5;
        /// Indicates signature comes from a non-inferrable type
        const IsNonInferrable = 1 << 6;
        const IsSignatureCandidateForOverloadFailure = 1 << 7;

        /// We do not propagate `IsInnerCallChain` or `IsOuterCallChain` to instantiated signatures, as that would result in us
        /// attempting to add `| undefined` on each recursive call to `getReturnTypeOfSignature` when
        /// instantiating the return type.
        const PropagatingFlags = Self::HasRestParameter.bits() | Self::HasLiteralTypes.bits() | Self::Abstract.bits() | Self::IsUntypedSignatureInJSFile.bits() | Self::IsSignatureCandidateForOverloadFailure.bits();

        const CallChainFlags = Self::IsInnerCallChain.bits() | Self::IsOuterCallChain.bits();
    }
}

// compiler/types.ts, line 6953
// export interface Signature {
//     /** @internal */ flags: SignatureFlags;
//     /** @internal */ checker?: TypeChecker;
//     declaration?: SignatureDeclaration | JSDocSignature; // Originating declaration
//     typeParameters?: readonly TypeParameter[];   // Type parameters (undefined if non-generic)
//     parameters: readonly Symbol[];               // Parameters
//     thisParameter?: Symbol;             // symbol of this-type parameter
//     /** @internal */
//     // See comment in `instantiateSignature` for why these are set lazily.
//     resolvedReturnType?: Type;          // Lazily set by `getReturnTypeOfSignature`.
//     /** @internal */
//     // Lazily set by `getTypePredicateOfSignature`.
//     // `undefined` indicates a type predicate that has not yet been computed.
//     // Uses a special `noTypePredicate` sentinel value to indicate that there is no type predicate. This looks like a TypePredicate at runtime to avoid polymorphism.
//     resolvedTypePredicate?: TypePredicate;
//     /** @internal */
//     minArgumentCount: number;           // Number of non-optional parameters
//     /** @internal */
//     resolvedMinArgumentCount?: number;  // Number of non-optional parameters (excluding trailing `void`)
//     /** @internal */
//     target?: Signature;                 // Instantiation target
//     /** @internal */
//     mapper?: TypeMapper;                // Instantiation mapper
//     /** @internal */
//     compositeSignatures?: Signature[];  // Underlying signatures of a union/intersection signature
//     /** @internal */
//     compositeKind?: TypeFlags;          // TypeFlags.Union if the underlying signatures are from union members, otherwise TypeFlags.Intersection
//     /** @internal */
//     erasedSignatureCache?: Signature;   // Erased version of signature (deferred)
//     /** @internal */
//     canonicalSignatureCache?: Signature; // Canonical version of signature (deferred)
//     /** @internal */
//     baseSignatureCache?: Signature;      // Base version of signature (deferred)
//     /** @internal */
//     optionalCallSignatureCache?: { inner?: Signature, outer?: Signature }; // Optional chained call version of signature (deferred)
//     /** @internal */
//     isolatedSignatureType?: ObjectType; // A manufactured type that just contains the signature for purposes of signature comparison
//     /** @internal */
//     instantiations?: Map<string, Signature>;    // Generic signature instantiation cache
//     /** @internal */
//     implementationSignatureCache?: Signature;  // Copy of the signature with fresh type parameters to use in checking the body of a potentially self-referential generic function (deferred)
// }

#[derive(Debug)]
pub struct Signature<'a> {
    /// Originating declaration
    pub declaration: Option<NodeId>,
    // type_parameters: Option<Vec<'a, TypeParameter<'a>>>,
    /// Parameters
    pub parameters: Vec<'a, SymbolId>,
    /// Lazily set by `getReturnTypeOfSignature`.
    pub(crate) resolved_return_type: Cell<Option<TypeId>>,
}
