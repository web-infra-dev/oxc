#![allow(unused_variables, clippy::wildcard_imports)]
use oxc_span::{GetSpan, Span};
use walk::walk_pattern;

use crate::ast::{
    Alternative, BoundaryAssertion, CapturingGroup, Character, CharacterClass,
    CharacterClassContents, CharacterClassEscape, CharacterClassRange, ClassString,
    ClassStringDisjunction, Disjunction, Dot, IgnoreGroup, IndexedReference, LookAroundAssertion,
    NamedReference, Pattern, Quantifier, Term, UnicodePropertyEscape,
};
use walk::*;

#[derive(Copy, Clone, Debug)]
pub enum RegExpAstKind<'a> {
    Pattern(&'a Pattern<'a>),
    Disjunction(&'a Disjunction<'a>),
    Alternative(&'a Alternative<'a>),
    Term(&'a Term<'a>),
    LookAroundAssertion(&'a LookAroundAssertion<'a>),
    Quantifier(&'a Quantifier<'a>),
    CapturingGroup(&'a CapturingGroup<'a>),
    IgnoreGroup(&'a IgnoreGroup<'a>),
    BoundaryAssertion(&'a BoundaryAssertion),
    Character(&'a Character),
    Dot(&'a Dot),
    CharacterClassEscape(&'a CharacterClassEscape),
    UnicodePropertyEscape(&'a UnicodePropertyEscape<'a>),
    CharacterClass(&'a CharacterClass<'a>),
    CharacterClassContents(&'a CharacterClassContents<'a>),
    CharacterClassRange(&'a CharacterClassRange),
    CharacterClassStringDisjunction(&'a ClassStringDisjunction<'a>),
    CharacterClassString(&'a ClassString<'a>),
    IndexedReference(&'a IndexedReference),
    NamedReference(&'a NamedReference<'a>),
}

impl<'a> GetSpan for RegExpAstKind<'a> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Pattern(it) => it.span,
            Self::Disjunction(it) => it.span,
            Self::Alternative(it) => it.span,
            Self::Term(it) => GetSpan::span(*it),
            Self::LookAroundAssertion(it) => it.span,
            Self::Quantifier(it) => it.span,
            Self::CapturingGroup(it) => it.span,
            Self::IgnoreGroup(it) => it.span,
            Self::BoundaryAssertion(it) => it.span,
            Self::Character(it) => it.span,
            Self::Dot(it) => it.span,
            Self::CharacterClassEscape(it) => it.span,
            Self::UnicodePropertyEscape(it) => it.span,
            Self::CharacterClass(it) => it.span,
            Self::CharacterClassContents(it) => GetSpan::span(*it),
            Self::CharacterClassRange(it) => it.span,
            Self::CharacterClassStringDisjunction(it) => it.span,
            Self::CharacterClassString(it) => it.span,
            Self::IndexedReference(it) => it.span,
            Self::NamedReference(it) => it.span,
        }
    }
}

/// RegEx syntax tree traversal
pub trait Visit<'a>: Sized {
    #[inline]
    fn enter_node(&mut self, kind: RegExpAstKind<'a>) {}
    #[inline]
    fn leave_node(&mut self, kind: RegExpAstKind<'a>) {}

    #[inline]
    fn alloc<T>(&self, t: &T) -> &'a T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        unsafe { std::mem::transmute(t) }
    }

    #[inline]
    fn visit_pattern(&mut self, it: &Pattern<'a>) {
        walk_pattern(self, it);
    }

    #[inline]
    fn visit_disjunction(&mut self, it: &Disjunction<'a>) {
        walk_disjunction(self, it);
    }

    #[inline]
    fn visit_alternative(&mut self, it: &Alternative<'a>) {
        walk_alternative(self, it);
    }

    #[inline]
    fn visit_term(&mut self, it: &Term<'a>) {
        walk_term(self, it);
    }

    #[inline]
    fn visit_lookaround_assertion(&mut self, it: &LookAroundAssertion<'a>) {
        walk_lookaround_assertion(self, it);
    }

    #[inline]
    fn visit_quantifier(&mut self, it: &Quantifier<'a>) {
        walk_quantifier(self, it);
    }

    #[inline]
    fn visit_capturing_group(&mut self, it: &CapturingGroup<'a>) {
        walk_capturing_group(self, it);
    }

    #[inline]
    fn visit_ignore_group(&mut self, it: &IgnoreGroup<'a>) {
        walk_ignore_group(self, it);
    }

    #[inline]
    fn visit_boundary_assertion(&mut self, it: &BoundaryAssertion) {
        walk_boundary_assertion(self, it);
    }

    #[inline]
    fn visit_character(&mut self, it: &Character) {
        walk_character(self, it);
    }

    #[inline]
    fn visit_dot(&mut self, it: &Dot) {
        walk_dot(self, it);
    }

    #[inline]
    fn visit_character_class_escape(&mut self, it: &CharacterClassEscape) {
        walk_character_class_escape(self, it);
    }

    #[inline]
    fn visit_unicode_property_escape(&mut self, it: &UnicodePropertyEscape<'a>) {
        walk_unicode_property_escape(self, it);
    }

    #[inline]
    fn visit_character_class(&mut self, it: &CharacterClass<'a>) {
        walk_character_class(self, it);
    }

    #[inline]
    fn visit_character_class_contents(&mut self, it: &CharacterClassContents<'a>) {
        walk_character_class_contents(self, it);
    }

    #[inline]
    fn visit_character_class_range(&mut self, it: &CharacterClassRange) {
        walk_character_class_range(self, it);
    }

    #[inline]
    fn visit_character_class_string_disjunction(&mut self, it: &ClassStringDisjunction<'a>) {
        walk_character_class_string_disjunction(self, it);
    }

    #[inline]
    fn visit_character_class_string(&mut self, it: &ClassString<'a>) {
        walk_character_class_string(self, it);
    }

    #[inline]
    fn visit_indexed_reference(&mut self, it: &IndexedReference) {
        walk_indexed_reference(self, it);
    }

    #[inline]
    fn visit_named_reference(&mut self, it: &NamedReference<'a>) {
        walk_named_reference(self, it);
    }
}

pub mod walk {
    use super::*;

    #[inline]
    pub fn walk_pattern<'a, V: Visit<'a>>(visitor: &mut V, it: &Pattern<'a>) {
        let kind = RegExpAstKind::Pattern(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_disjunction(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_disjunction<'a, V: Visit<'a>>(visitor: &mut V, it: &Disjunction<'a>) {
        let kind = RegExpAstKind::Disjunction(visitor.alloc(it));
        visitor.enter_node(kind);
        for alt in &it.body {
            visitor.visit_alternative(alt);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_alternative<'a, V: Visit<'a>>(visitor: &mut V, it: &Alternative<'a>) {
        let kind = RegExpAstKind::Alternative(visitor.alloc(it));
        visitor.enter_node(kind);
        for term in &it.body {
            visitor.visit_term(term);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_term<'a, V: Visit<'a>>(visitor: &mut V, it: &Term<'a>) {
        let kind = RegExpAstKind::Term(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            Term::LookAroundAssertion(lookaround) => {
                visitor.visit_lookaround_assertion(lookaround);
            }
            Term::Quantifier(quant) => {
                visitor.visit_quantifier(quant);
            }
            Term::CapturingGroup(group) => {
                visitor.visit_capturing_group(group);
            }
            Term::IgnoreGroup(group) => {
                visitor.visit_ignore_group(group);
            }
            Term::BoundaryAssertion(boundary_assertion) => {
                visitor.visit_boundary_assertion(boundary_assertion);
            }
            Term::Character(character) => {
                visitor.visit_character(character);
            }
            Term::Dot(dot) => {
                visitor.visit_dot(dot);
            }
            Term::CharacterClassEscape(character_class_escape) => {
                visitor.visit_character_class_escape(character_class_escape);
            }
            Term::UnicodePropertyEscape(unicode_property_escape) => {
                visitor.visit_unicode_property_escape(unicode_property_escape);
            }
            Term::CharacterClass(character_class) => {
                visitor.visit_character_class(character_class);
            }
            Term::IndexedReference(indexed_reference) => {
                visitor.visit_indexed_reference(indexed_reference);
            }
            Term::NamedReference(named_reference) => {
                visitor.visit_named_reference(named_reference);
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_lookaround_assertion<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &LookAroundAssertion<'a>,
    ) {
        let kind = RegExpAstKind::LookAroundAssertion(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_disjunction(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_quantifier<'a, V: Visit<'a>>(visitor: &mut V, it: &Quantifier<'a>) {
        let kind = RegExpAstKind::Quantifier(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_term(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_capturing_group<'a, V: Visit<'a>>(visitor: &mut V, it: &CapturingGroup<'a>) {
        let kind = RegExpAstKind::CapturingGroup(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_disjunction(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_ignore_group<'a, V: Visit<'a>>(visitor: &mut V, it: &IgnoreGroup<'a>) {
        let kind = RegExpAstKind::IgnoreGroup(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_disjunction(&it.body);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_boundary_assertion<'a, V: Visit<'a>>(visitor: &mut V, it: &BoundaryAssertion) {
        let kind = RegExpAstKind::BoundaryAssertion(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character<'a, V: Visit<'a>>(visitor: &mut V, it: &Character) {
        let kind = RegExpAstKind::Character(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_dot<'a, V: Visit<'a>>(visitor: &mut V, it: &Dot) {
        let kind = RegExpAstKind::Dot(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character_class_escape<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &CharacterClassEscape,
    ) {
        let kind = RegExpAstKind::CharacterClassEscape(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_unicode_property_escape<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &UnicodePropertyEscape<'a>,
    ) {
        let kind = RegExpAstKind::UnicodePropertyEscape(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character_class<'a, V: Visit<'a>>(visitor: &mut V, it: &CharacterClass<'a>) {
        let kind = RegExpAstKind::CharacterClass(visitor.alloc(it));
        visitor.enter_node(kind);
        for content in &it.body {
            visitor.visit_character_class_contents(content);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character_class_contents<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &CharacterClassContents<'a>,
    ) {
        let kind = RegExpAstKind::CharacterClassContents(visitor.alloc(it));
        visitor.enter_node(kind);
        match it {
            CharacterClassContents::CharacterClassRange(character_class_range) => {
                visitor.visit_character_class_range(character_class_range);
            }
            CharacterClassContents::CharacterClassEscape(character_class_escape) => {
                visitor.visit_character_class_escape(character_class_escape);
            }
            CharacterClassContents::UnicodePropertyEscape(unicode_property_escape) => {
                visitor.visit_unicode_property_escape(unicode_property_escape);
            }
            CharacterClassContents::Character(character) => {
                visitor.visit_character(character);
            }
            CharacterClassContents::NestedCharacterClass(character_class) => {
                visitor.visit_character_class(character_class);
            }
            CharacterClassContents::ClassStringDisjunction(class_string_disjunction) => {
                visitor.visit_character_class_string_disjunction(class_string_disjunction);
            }
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character_class_range<'a, V: Visit<'a>>(visitor: &mut V, it: &CharacterClassRange) {
        let kind = RegExpAstKind::CharacterClassRange(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.visit_character(&it.min);
        visitor.visit_character(&it.max);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character_class_string_disjunction<'a, V: Visit<'a>>(
        visitor: &mut V,
        it: &ClassStringDisjunction<'a>,
    ) {
        let kind = RegExpAstKind::CharacterClassStringDisjunction(visitor.alloc(it));
        visitor.enter_node(kind);
        for string in &it.body {
            visitor.visit_character_class_string(string);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_character_class_string<'a, V: Visit<'a>>(visitor: &mut V, it: &ClassString<'a>) {
        let kind = RegExpAstKind::CharacterClassString(visitor.alloc(it));
        visitor.enter_node(kind);
        for character in &it.body {
            visitor.visit_character(character);
        }
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_indexed_reference<'a, V: Visit<'a>>(visitor: &mut V, it: &IndexedReference) {
        let kind = RegExpAstKind::IndexedReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }

    #[inline]
    pub fn walk_named_reference<'a, V: Visit<'a>>(visitor: &mut V, it: &NamedReference<'a>) {
        let kind = RegExpAstKind::NamedReference(visitor.alloc(it));
        visitor.enter_node(kind);
        visitor.leave_node(kind);
    }
}
