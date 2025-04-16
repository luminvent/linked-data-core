use proc_macro2::Span;
use snafu::{ResultExt, Snafu};
use syn::DeriveInput;

use crate::attributes::AttributeError;
use crate::r#enum::{Enum, generate};

mod attributes;
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

pub enum LinkedDataType {
    Enum(Enum),
}

impl TryFrom<DeriveInput> for LinkedDataType {
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
                generate(derive_input.attrs, derive_input.ident, data_enum)
                    .map(LinkedDataType::Enum)
                    .context(InvalidAttributeSnafu)?;
            }
            syn::Data::Union(data_union) => todo!(),
        }

        // Complete the implementation here
        // For now, returning a placeholder error
        todo!("Complete the Type construction using prefix_mappings and others")
    }
}

#[cfg(test)]
mod tests {}
