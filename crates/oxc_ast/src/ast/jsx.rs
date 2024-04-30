//! [JSX](https://facebook.github.io/jsx)

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_allocator::{Box, Vec};
use oxc_span::{Atom, Span};
#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

use super::{js::*, literal::*, ts::*};

use super::inherit_variants;

// 1.2 JSX Elements

/// JSX Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub opening_element: Box<'a, JSXOpeningElement<'a>>,
    pub closing_element: Option<Box<'a, JSXClosingElement<'a>>>,
    pub children: Vec<'a, JSXChild<'a>>,
}

/// JSX Opening Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXOpeningElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub self_closing: bool,
    pub name: JSXElementName<'a>,
    pub attributes: Vec<'a, JSXAttributeItem<'a>>,
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
}

/// JSX Closing Element
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXClosingElement<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: JSXElementName<'a>,
}

/// JSX Fragment
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type", rename_all = "camelCase"))]
pub struct JSXFragment<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub opening_fragment: JSXOpeningFragment,
    pub closing_fragment: JSXClosingFragment,
    pub children: Vec<'a, JSXChild<'a>>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXOpeningFragment {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXClosingFragment {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

/// JSX Element Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXElementName<'a> {
    /// `<Apple />`
    Identifier(Box<'a, JSXIdentifier<'a>>),
    /// `<Apple:Orange />`
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
    /// `<Apple.Orange />`
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

/// JSX Namespaced Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXNamespacedName<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub namespace: JSXIdentifier<'a>,
    pub property: JSXIdentifier<'a>,
}

impl<'a> std::fmt::Display for JSXNamespacedName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace.name, self.property.name)
    }
}

/// JSX Member Expression
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXMemberExpression<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub object: JSXMemberExpressionObject<'a>,
    pub property: JSXIdentifier<'a>,
}

impl<'a> JSXMemberExpression<'a> {
    pub fn get_object_identifier(&self) -> &JSXIdentifier {
        match &self.object {
            JSXMemberExpressionObject::Identifier(ident) => ident,
            JSXMemberExpressionObject::MemberExpression(expr) => expr.get_object_identifier(),
        }
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXMemberExpressionObject<'a> {
    Identifier(Box<'a, JSXIdentifier<'a>>),
    MemberExpression(Box<'a, JSXMemberExpression<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXExpressionContainer<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: JSXExpression<'a>,
}

inherit_variants! {
/// JSX Expression
///
/// Inherits variants from [`Expression`].
#[repr(C, u8)]
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXExpression<'a> {
    EmptyExpression(JSXEmptyExpression) = 64,
    // `Expression` variants added here by `inherit_variants!` macro
    @inherit Expression
}
}

impl<'a> JSXExpression<'a> {
    /// Determines whether the given expr is a `undefined` literal
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "undefined")
    }
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXEmptyExpression {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
}

// 1.3 JSX Attributes

/// JSX Attributes
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeItem<'a> {
    Attribute(Box<'a, JSXAttribute<'a>>),
    SpreadAttribute(Box<'a, JSXSpreadAttribute<'a>>),
}

/// JSX Attribute
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: JSXAttributeName<'a>,
    pub value: Option<JSXAttributeValue<'a>>,
}

impl<'a> JSXAttribute<'a> {
    pub fn is_identifier(&self, name: &str) -> bool {
        matches!(&self.name, JSXAttributeName::Identifier(ident) if ident.name == name)
    }

    pub fn is_key(&self) -> bool {
        self.is_identifier("key")
    }
}

/// JSX Spread Attribute
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXSpreadAttribute<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub argument: Expression<'a>,
}

/// JSX Attribute Name
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeName<'a> {
    Identifier(Box<'a, JSXIdentifier<'a>>),
    NamespacedName(Box<'a, JSXNamespacedName<'a>>),
}

/// JSX Attribute Value
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXAttributeValue<'a> {
    StringLiteral(Box<'a, StringLiteral<'a>>),
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXIdentifier<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub name: Atom<'a>,
}

impl<'a> JSXIdentifier<'a> {
    pub fn new(span: Span, name: Atom<'a>) -> Self {
        Self { span, name }
    }
}

// 1.4 JSX Children

/// JSX Child
#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(untagged))]
pub enum JSXChild<'a> {
    Text(Box<'a, JSXText<'a>>),
    Element(Box<'a, JSXElement<'a>>),
    Fragment(Box<'a, JSXFragment<'a>>),
    ExpressionContainer(Box<'a, JSXExpressionContainer<'a>>),
    Spread(Box<'a, JSXSpreadChild<'a>>),
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXSpreadChild<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub expression: Expression<'a>,
}

#[derive(Debug, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub struct JSXText<'a> {
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub span: Span,
    pub value: Atom<'a>,
}
