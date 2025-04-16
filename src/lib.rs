use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use snafu::{ResultExt, Snafu};
use syn::DeriveInput;

use crate::attributes::AttributeError;
use crate::r#enum::{Enum, Variant, extract};

pub mod attributes;
pub mod r#enum;
mod prefix_mappings;

#[derive(Debug, Snafu)]
pub enum Error {
    // #[snafu(display("union types are not supported"))]
    // UnionType(Span),
    #[snafu(display("invalid `ld` attribute: {source}"))]
    InvalidAttribute { source: AttributeError },

    #[snafu(display("missing field"))]
    MissingField { span: Span },

    #[snafu(display("multiple fields"))]
    MultipleFields { span: Span },
    // #[snafu(display("missing field serialization method"))]
    // UnknownFieldSerializationMethod(Span),
    //
    // #[snafu(display("invalid IRI `{0}`"))]
    // InvalidIri(String, Span),
    //
    // #[snafu(display("missing variant IRI"))]
    // MissingVariantIri(Span),
}

impl Error {
    fn span(&self) -> Span {
        match self {
            Error::InvalidAttribute { source } => todo!(),
            Error::MissingField { span } => todo!(),
            Error::MultipleFields { span } => todo!(),
        }
    }
}

pub enum LinkedDataType<F> {
    Enum(Enum<F>),
}

pub trait TokenGenerator: Sized {
    fn generate_type_tokens(
        linked_data_type: &LinkedDataType<Self>,
        tokens: &mut TokenStream,
    );

    fn generate_enum_tokens(r#enum: &Enum<Self>, tokens: &mut TokenStream);

    fn generate_variant_tokens(
        variant: &Variant<Self>,
        tokens: &mut TokenStream,
    );
}

impl<F: TokenGenerator> ToTokens for LinkedDataType<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_type_tokens(self, tokens)
    }
}

impl<F: TokenGenerator> TryFrom<DeriveInput> for LinkedDataType<F> {
    type Error = Error;

    fn try_from(derive_input: DeriveInput) -> Result<Self, Self::Error> {
        // derive_input.attrs.get(0).unwrap().parse_args_with
        // let type_attributes: StructAttributes = derive_input
        //     .attrs
        //     .try_into()
        //     .context(InvalidAttributeSnafu)?;
        match derive_input.data {
            syn::Data::Struct(data_struct) => todo!(),
            syn::Data::Enum(data_enum) => {
                extract::<F>(derive_input.attrs, derive_input.ident, data_enum)
                    .map(LinkedDataType::Enum)
                    .context(InvalidAttributeSnafu)
            }
            syn::Data::Union(data_union) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {}
