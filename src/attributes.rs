use iref::IriBuf;
use proc_macro2::Span;
use snafu::{ResultExt, Snafu};
use syn::{Attribute, LitStr};

mod ast;
mod parse;
pub mod r#type;
pub mod variant;
pub mod field;

#[derive(Debug, Snafu)]
pub enum AttributeError {
    #[snafu(display("invalid iri attribute"))]
    MalformedAttribute { source: syn::Error },

    #[snafu(display("invalid iri"))]
    InvalidIri {
        source: iref::InvalidIri<String>,
        span: Span,
    },

    #[snafu(display("invalid expansion"))]
    InvalidExpansion {
        source: crate::prefix_mappings::Error,
        span: Span,
    },

    #[snafu(display("invalid prefix attribute"))]
    InvalidPrefix {
        source: crate::prefix_mappings::Error,
        span: Span,
    },

    #[snafu(display("multiple types"))]
    MultipleTypes { span: Span },

    #[snafu(display("multiple IRIs"))]
    MultipleIris { span: Span },

    #[snafu(display("missing IRIs"))]
    MissingIri,

    /// remove

    #[snafu(display("invalid shape"))]
    InvalidShape { span: Span },

    #[snafu(display("expected string literal"))]
    ExpectedString { span: Span },

    #[snafu(display("unknown attribute name"))]
    UnknownIdent { span: Span },

    #[snafu(display("empty"))]
    Empty { span: Span },

    #[snafu(display("unexpected token"))]
    UnexpectedToken { span: Span },

    #[snafu(display("invalid compact IRI"))]
    InvalidCompactIri { span: Span },

    #[snafu(display("missing `=`"))]
    MissingEq { span: Span },

    #[snafu(display("missing suffix string"))]
    MissingSuffix { span: Span },

    #[snafu(display("missing prefix binding"))]
    MissingPrefixBinding { span: Span },

    #[snafu(display("missing type iri format should be type = 'http'"))]
    MissingTypeIri { span: Span },

    #[snafu(display("enum cannot have a type"))]
    DisallowedEnumType,

    // #[snafu(display("enum cannot have a type"))]
    // ParseError { span: Span, message: String },
    #[snafu(display("invalid type"))]
    InvalidType { span: Span },

    #[snafu(display("invalid type attribute"))]
    InvalidTypeAttribute { span: Span },
}

fn parse_ld_attributes<T: syn::parse::Parse>(
    attrs: &[Attribute],
) -> Result<Vec<T>, AttributeError> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("ld"))
        .map(|attr| attr.parse_args::<T>())
        .collect::<Result<_, _>>()
        .context(MalformedAttributeSnafu)
}

fn parse_iri(lit_iri: LitStr) -> Result<IriBuf, AttributeError> {
    IriBuf::new(lit_iri.value()).context(InvalidIriSnafu {
        span: lit_iri.span(),
    })
}
