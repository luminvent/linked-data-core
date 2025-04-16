use iref::IriBuf;
use snafu::ResultExt;
use syn::Attribute;

use crate::attributes::ast::VariantAttribute;
use crate::attributes::{
    AttributeError, InvalidExpansionSnafu, parse_ld_attributes,
};
use crate::prefix_mappings::PrefixMappings;

pub struct VariantAttributes {
    predicate_path: PredicatePath,
}

enum PredicatePath {
    // Represents a path with an intermediate blank node
    // :s <to_blank> _:blank .
    // _:blank <from_blank> :o .
    ChainedPath {
        to_blank: IriBuf,
        from_blank: IriBuf,
    },

    // For the direct case:
    // :s :predicate :o .
    Predicate(IriBuf),
}

impl VariantAttributes {
    pub fn try_from_attrs(
        inner_attrs: Vec<Attribute>,
        outer_attrs: Vec<Attribute>,
        prefix_mappings: &PrefixMappings,
    ) -> Result<Self, AttributeError> {
        let inner_attrs: Vec<VariantAttribute> =
            parse_ld_attributes(&inner_attrs)?;
        let outer_attrs: Vec<VariantAttribute> =
            parse_ld_attributes(&outer_attrs)?;

        let unpack_variant_attrs = |attrs: &[VariantAttribute]| -> Result<
            Option<IriBuf>,
            AttributeError,
        > {
            if let Some(VariantAttribute::Iri(iri)) = attrs.get(1) {
                Err(AttributeError::MultipleIris { span: iri.span() })
            } else {
                attrs
                    .first()
                    .map(|variant_attr| match variant_attr {
                        VariantAttribute::Iri(lit_str) => lit_str,
                    })
                    .map(|lit_str| {
                        prefix_mappings.expand(lit_str.value()).context(
                            InvalidExpansionSnafu {
                                span: lit_str.span(),
                            },
                        )
                    })
                    .transpose()
            }
        };

        let inner_attr = unpack_variant_attrs(&inner_attrs)?;
        let outer_attr = unpack_variant_attrs(&outer_attrs)?;

        match (inner_attr, outer_attr) {
            (None, None) => Err(AttributeError::MissingIri),
            (None, Some(outer_iri)) => Ok(VariantAttributes {
                predicate_path: PredicatePath::Predicate(outer_iri),
            }),
            (Some(inner_iri), None) => Ok(VariantAttributes {
                predicate_path: PredicatePath::Predicate(inner_iri),
            }),
            (Some(to_blank), Some(from_blank)) => Ok(VariantAttributes {
                predicate_path: PredicatePath::ChainedPath {
                    to_blank,
                    from_blank,
                },
            }),
        }
    }
}

// impl TryFrom<Vec<Attribute>> for VariantAttributes {
//     type Error = AttributeError;
//
//     fn try_from(attrs: Vec<Attribute>) -> Result<Self, Self::Error> {
//         let variant_attr: Vec<VariantAttribute> = parse_ld_attributes(&attrs)?;
//         if let Some(type_attr) = type_attrs.get(1) {
//             return Err(AttributeError::MultipleTypes {
//                 span: type_attr.identifier.span(),
//             });
//         }
//     }
// }

// impl TryFrom<Vec<Attribute>> for VariantAttributes {
//     type Error = AttributeError;
//
//     fn try_from(attrs: Vec<Attribute>) -> Result<Self, Self::Error> {
//         let variant_attrs = parse_ld_attributes(&attrs)?;
//
//         // For now, we only expect a single VariantAttribute
//         if let Some(variant_attr) = variant_attrs.into_iter().next() {
//             match variant_attr {
//                 VariantAttribute::Iri(lit_iri) => {
//                     // Convert the string literal to an IriBuf
//                     let iri = IriBuf::new(lit_iri.value()).context(InvalidIriSnafu {
//                         span: lit_iri.span(),
//                     })?;
//
//                     Ok(VariantAttributes {
//                         predicate_path: PredicatePath::Predicate(iri),
//                     })
//                 }
//             }
//         } else {
//             // We need to handle the case when no attributes are provided
//             // For now, let's throw an error
//             Err(AttributeError::Empty {
//                 span: attrs.first().map_or_else(
//                     || proc_macro2::Span::call_site(),
//                     |attr| attr.span(),
//                 ),
//             })
//         }
//     }
// }
