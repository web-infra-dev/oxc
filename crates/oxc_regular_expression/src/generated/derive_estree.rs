// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_gen/src/derives/estree.rs`

#![allow(unused_imports, unused_mut, clippy::match_same_arms)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::ast::*;

impl Serialize for Pattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Pattern")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Disjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Disjunction")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Alternative<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Alternative")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Term<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Term::BoundaryAssertion(x) => Serialize::serialize(x, serializer),
            Term::LookAroundAssertion(x) => Serialize::serialize(x, serializer),
            Term::Quantifier(x) => Serialize::serialize(x, serializer),
            Term::Character(x) => Serialize::serialize(x, serializer),
            Term::Dot(x) => Serialize::serialize(x, serializer),
            Term::CharacterClassEscape(x) => Serialize::serialize(x, serializer),
            Term::UnicodePropertyEscape(x) => Serialize::serialize(x, serializer),
            Term::CharacterClass(x) => Serialize::serialize(x, serializer),
            Term::CapturingGroup(x) => Serialize::serialize(x, serializer),
            Term::IgnoreGroup(x) => Serialize::serialize(x, serializer),
            Term::IndexedReference(x) => Serialize::serialize(x, serializer),
            Term::NamedReference(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for BoundaryAssertion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "BoundaryAssertion")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

impl Serialize for BoundaryAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BoundaryAssertionKind::Start(x) => Serialize::serialize(x, serializer),
            BoundaryAssertionKind::End(x) => Serialize::serialize(x, serializer),
            BoundaryAssertionKind::Boundary(x) => Serialize::serialize(x, serializer),
            BoundaryAssertionKind::NegativeBoundary(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for LookAroundAssertion<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "LookAroundAssertion")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for LookAroundAssertionKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LookAroundAssertionKind::Lookahead(x) => Serialize::serialize(x, serializer),
            LookAroundAssertionKind::NegativeLookahead(x) => Serialize::serialize(x, serializer),
            LookAroundAssertionKind::Lookbehind(x) => Serialize::serialize(x, serializer),
            LookAroundAssertionKind::NegativeLookbehind(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for Quantifier<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Quantifier")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.serialize_entry("greedy", &self.greedy)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Character {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Character")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for CharacterKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CharacterKind::ControlLetter(x) => Serialize::serialize(x, serializer),
            CharacterKind::HexadecimalEscape(x) => Serialize::serialize(x, serializer),
            CharacterKind::Identifier(x) => Serialize::serialize(x, serializer),
            CharacterKind::Null(x) => Serialize::serialize(x, serializer),
            CharacterKind::Octal1(x) => Serialize::serialize(x, serializer),
            CharacterKind::Octal2(x) => Serialize::serialize(x, serializer),
            CharacterKind::Octal3(x) => Serialize::serialize(x, serializer),
            CharacterKind::SingleEscape(x) => Serialize::serialize(x, serializer),
            CharacterKind::Symbol(x) => Serialize::serialize(x, serializer),
            CharacterKind::UnicodeEscape(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for CharacterClassEscape {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassEscape")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("kind", &self.kind)?;
        map.end()
    }
}

impl Serialize for CharacterClassEscapeKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CharacterClassEscapeKind::D(x) => Serialize::serialize(x, serializer),
            CharacterClassEscapeKind::NegativeD(x) => Serialize::serialize(x, serializer),
            CharacterClassEscapeKind::S(x) => Serialize::serialize(x, serializer),
            CharacterClassEscapeKind::NegativeS(x) => Serialize::serialize(x, serializer),
            CharacterClassEscapeKind::W(x) => Serialize::serialize(x, serializer),
            CharacterClassEscapeKind::NegativeW(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for UnicodePropertyEscape<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "UnicodePropertyEscape")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("negative", &self.negative)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

impl Serialize for Dot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Dot")?;
        map.serialize_entry("span", &self.span)?;
        map.end()
    }
}

impl Serialize for CharacterClass<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClass")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("negative", &self.negative)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CharacterClassContentsKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CharacterClassContentsKind::Union(x) => Serialize::serialize(x, serializer),
            CharacterClassContentsKind::Intersection(x) => Serialize::serialize(x, serializer),
            CharacterClassContentsKind::Subtraction(x) => Serialize::serialize(x, serializer),
        }
    }
}

impl Serialize for CharacterClassContents<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CharacterClassContents::CharacterClassRange(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::CharacterClassEscape(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::UnicodePropertyEscape(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::Character(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::NestedCharacterClass(x) => Serialize::serialize(x, serializer),
            CharacterClassContents::ClassStringDisjunction(x) => {
                Serialize::serialize(x, serializer)
            }
        }
    }
}

impl Serialize for CharacterClassRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CharacterClassRange")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("min", &self.min)?;
        map.serialize_entry("max", &self.max)?;
        map.end()
    }
}

impl Serialize for ClassStringDisjunction<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassStringDisjunction")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for ClassString<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "ClassString")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("strings", &self.strings)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for CapturingGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "CapturingGroup")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for IgnoreGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IgnoreGroup")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("modifiers", &self.modifiers)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl Serialize for Modifiers {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Modifiers")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("enabling", &self.enabling)?;
        map.serialize_entry("disabling", &self.disabling)?;
        map.end()
    }
}

impl Serialize for Modifier {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Modifier")?;
        map.serialize_entry("ignoreCase", &self.ignore_case)?;
        map.serialize_entry("multiline", &self.multiline)?;
        map.serialize_entry("sticky", &self.sticky)?;
        map.end()
    }
}

impl Serialize for IndexedReference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "IndexedReference")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("index", &self.index)?;
        map.end()
    }
}

impl Serialize for NamedReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "NamedReference")?;
        map.serialize_entry("span", &self.span)?;
        map.serialize_entry("name", &self.name)?;
        map.end()
    }
}
