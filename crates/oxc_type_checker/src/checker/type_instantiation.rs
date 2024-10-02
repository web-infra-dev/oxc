use oxc_semantic::AstNode;
use oxc_syntax::types::TypeId;

use super::{check::CheckMode, Checker};

impl<'a> Checker<'a> {
    /// ```typescript
    /// function instantiateTypeWithSingleGenericCallSignature(node: Expression | MethodDeclaration | QualifiedName, type: Type, checkMode?: CheckMode) {
    ///     if (checkMode && checkMode & (CheckMode.Inferential | CheckMode.SkipGenericFunctions)) {
    ///         const callSignature = getSingleSignature(type, SignatureKind.Call, /*allowMembers*/ true);
    ///         const constructSignature = getSingleSignature(type, SignatureKind.Construct, /*allowMembers*/ true);
    ///         const signature = callSignature || constructSignature;
    ///         if (signature && signature.typeParameters) {
    ///             const contextualType = getApparentTypeOfContextualType(node as Expression, ContextFlags.NoConstraints);
    ///             if (contextualType) {
    ///                 const contextualSignature = getSingleSignature(getNonNullableType(contextualType), callSignature ? SignatureKind.Call : SignatureKind.Construct, /*allowMembers*/ false);
    ///                 if (contextualSignature && !contextualSignature.typeParameters) {
    ///                     if (checkMode & CheckMode.SkipGenericFunctions) {
    ///                         skippedGenericFunction(node, checkMode);
    ///                         return anyFunctionType;
    ///                     }
    ///                     const context = getInferenceContext(node)!;
    ///                     // We have an expression that is an argument of a generic function for which we are performing
    ///                     // type argument inference. The expression is of a function type with a single generic call
    ///                     // signature and a contextual function type with a single non-generic call signature. Now check
    ///                     // if the outer function returns a function type with a single non-generic call signature and
    ///                     // if some of the outer function type parameters have no inferences so far. If so, we can
    ///                     // potentially add inferred type parameters to the outer function return type.
    ///                     const returnType = context.signature && getReturnTypeOfSignature(context.signature);
    ///                     const returnSignature = returnType && getSingleCallOrConstructSignature(returnType);
    ///                     if (returnSignature && !returnSignature.typeParameters && !every(context.inferences, hasInferenceCandidates)) {
    ///                         // Instantiate the signature with its own type parameters as type arguments, possibly
    ///                         // renaming the type parameters to ensure they have unique names.
    ///                         const uniqueTypeParameters = getUniqueTypeParameters(context, signature.typeParameters);
    ///                         const instantiatedSignature = getSignatureInstantiationWithoutFillingInTypeArguments(signature, uniqueTypeParameters);
    ///                         // Infer from the parameters of the instantiated signature to the parameters of the
    ///                         // contextual signature starting with an empty set of inference candidates.
    ///                         const inferences = map(context.inferences, info => createInferenceInfo(info.typeParameter));
    ///                         applyToParameterTypes(instantiatedSignature, contextualSignature, (source, target) => {
    ///                             inferTypes(inferences, source, target, /*priority*/ 0, /*contravariant*/ true);
    ///                         });
    ///                         if (some(inferences, hasInferenceCandidates)) {
    ///                             // We have inference candidates, indicating that one or more type parameters are referenced
    ///                             // in the parameter types of the contextual signature. Now also infer from the return type.
    ///                             applyToReturnTypes(instantiatedSignature, contextualSignature, (source, target) => {
    ///                                 inferTypes(inferences, source, target);
    ///                             });
    ///                             // If the type parameters for which we produced candidates do not have any inferences yet,
    ///                             // we adopt the new inference candidates and add the type parameters of the expression type
    ///                             // to the set of inferred type parameters for the outer function return type.
    ///                             if (!hasOverlappingInferences(context.inferences, inferences)) {
    ///                                 mergeInferences(context.inferences, inferences);
    ///                                 context.inferredTypeParameters = concatenate(context.inferredTypeParameters, uniqueTypeParameters);
    ///                                 return getOrCreateTypeFromSignature(instantiatedSignature);
    ///                             }
    ///                         }
    ///                     }
    ///                     // TODO: The signature may reference any outer inference contexts, but we map pop off and then apply new inference contexts, and thus get different inferred types.
    ///                     // That this is cached on the *first* such attempt is not currently an issue, since expression types *also* get cached on the first pass. If we ever properly speculate, though,
    ///                     // the cached "isolatedSignatureType" signature field absolutely needs to be included in the list of speculative caches.
    ///                     return getOrCreateTypeFromSignature(instantiateSignatureInContextOf(signature, contextualSignature, context), flatMap(inferenceContexts, c => c && map(c.inferences, i => i.typeParameter)).slice());
    ///                 }
    ///             }
    ///         }
    ///     }
    ///     return type;
    /// }
    /// ```
    pub(crate) fn instantiate_type_with_single_generic_call_signature(
        &mut self,
        node: &AstNode<'a>,
        type_id: TypeId,
        check_mode: CheckMode,
    ) -> TypeId {
        todo!()
    }
}
