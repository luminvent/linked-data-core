use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use snafu::Snafu;

mod attributes;
mod prefix_mappings;
mod rdf_metadata;

pub use crate::attributes::variant::PredicatePath;
pub use crate::rdf_metadata::{
    RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant,
};

pub trait TokenGenerator: Sized {
    fn generate_type_tokens(
        linked_data_type: &RdfType<Self>,
        tokens: &mut TokenStream,
    );

    fn generate_struct_tokens(
        r#struct: &RdfStruct<Self>,
        tokens: &mut TokenStream,
    );

    fn generate_enum_tokens(r#enum: &RdfEnum<Self>, tokens: &mut TokenStream);

    fn generate_variant_tokens(
        variant: &RdfVariant<Self>,
        tokens: &mut TokenStream,
    );

    fn generate_field_tokens(field: &RdfField<Self>, tokens: &mut TokenStream);
}

impl<G: TokenGenerator> ToTokens for RdfType<G> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        G::generate_type_tokens(self, tokens)
    }
}

impl<F: TokenGenerator> ToTokens for RdfEnum<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_enum_tokens(self, tokens)
    }
}

impl<F: TokenGenerator> ToTokens for RdfStruct<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_struct_tokens(self, tokens)
    }
}

impl<F: TokenGenerator> ToTokens for RdfVariant<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_variant_tokens(self, tokens)
    }
}

impl<F: TokenGenerator> ToTokens for RdfField<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_field_tokens(self, tokens)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("union types are not supported"))]
    UnionType { span: Span },

    #[snafu(display("unit variants are not supported"))]
    UnitVariant { span: Span },

    #[snafu(display("struct variants are not supported"))]
    StructVariant { span: Span },

    #[snafu(transparent)]
    MalformedAttribute { source: syn::Error },

    #[snafu(display("type attribute is only allowed once"))]
    MultipleTypes { span: Span },

    #[snafu(display("multiple path IRIs defined"))]
    MultipleIris { span: Span },

    #[snafu(display("missing IRI path"))]
    MissingIriAttribute { span: Span },

    #[snafu(display("{source}"))]
    InvalidIri {
        source: iref::InvalidIri<String>,
        span: Span,
    },

    #[snafu(display("{source}"))]
    InvalidMapping {
        source: crate::prefix_mappings::Error,
        span: Span,
    },
}

impl Error {
    fn span(&self) -> Span {
        match self {
            Error::UnionType { span } => *span,
            Error::UnitVariant { span } => *span,
            Error::StructVariant { span } => *span,
            Error::MalformedAttribute { source } => source.span(),
            Error::InvalidIri { span, .. } => *span,
            Error::InvalidMapping { span, .. } => *span,
            Error::MultipleTypes { span } => *span,
            Error::MultipleIris { span } => *span,
            Error::MissingIriAttribute { span } => *span,
        }
    }
}
