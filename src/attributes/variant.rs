use iref::IriBuf;
use snafu::ResultExt;
use syn::spanned::Spanned;

use crate::{Error, InvalidMappingSnafu};
use crate::attributes::ast::VariantAttribute;
use crate::attributes::parse_ld_attributes;
use crate::prefix_mappings::PrefixMappings;

pub struct RdfVariantAttributes {
    pub predicate_path: PredicatePath,
}

pub enum PredicatePath {
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

impl RdfVariantAttributes {
    pub fn try_from_attrs(
        variant: &syn::Variant,
        inner_attrs: Vec<syn::Attribute>,
        outer_attrs: Vec<syn::Attribute>,
        prefix_mappings: &PrefixMappings,
    ) -> Result<Self, Error> {
        let inner_attrs: Vec<VariantAttribute> =
            parse_ld_attributes(&inner_attrs)?;
        let outer_attrs: Vec<VariantAttribute> =
            parse_ld_attributes(&outer_attrs)?;

        let unpack_variant_attrs =
            |attrs: &[VariantAttribute]| -> Result<Option<IriBuf>, Error> {
                if let Some(VariantAttribute::Iri(iri)) = attrs.get(1) {
                    Err(Error::MultipleIris { span: iri.span() })
                } else {
                    attrs
                        .first()
                        .map(|variant_attr| match variant_attr {
                            VariantAttribute::Iri(lit_str) => lit_str,
                        })
                        .map(|lit_str| {
                            prefix_mappings.expand(lit_str.value()).context(
                                InvalidMappingSnafu {
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
            (None, None) => Err(Error::MissingIriAttribute {
                span: variant.span(),
            }),
            (None, Some(outer_iri)) => Ok(RdfVariantAttributes {
                predicate_path: PredicatePath::Predicate(outer_iri),
            }),
            (Some(inner_iri), None) => Ok(RdfVariantAttributes {
                predicate_path: PredicatePath::Predicate(inner_iri),
            }),
            (Some(to_blank), Some(from_blank)) => Ok(RdfVariantAttributes {
                predicate_path: PredicatePath::ChainedPath {
                    to_blank,
                    from_blank,
                },
            }),
        }
    }
}
