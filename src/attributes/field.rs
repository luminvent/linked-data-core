use iref::IriBuf;
use snafu::ResultExt;
use syn::Attribute;

use crate::attributes::ast::FieldAttribute;
use crate::attributes::{AttributeError, InvalidExpansionSnafu, parse_iri, parse_ld_attributes};
use crate::prefix_mappings::PrefixMappings;

#[derive(Debug, Default)]
pub struct FieldAttributes {
    pub flatten: bool,
    pub is_graph: bool,
    pub ignore: bool,
    pub predicate: Option<IriBuf>,
    pub is_id: bool,
}

impl FieldAttributes {
    pub fn try_from_attrs(
        attrs: Vec<Attribute>,
        prefix_mappings: &PrefixMappings,
    ) -> Result<Self, AttributeError> {
        let field_attrs = parse_ld_attributes(&attrs)?;

        let mut attributes = FieldAttributes::default();

        for attr in field_attrs {
            match attr {
                FieldAttribute::Ignore => {
                    attributes.ignore = true;
                }
                FieldAttribute::Iri(lit_str) => {
                    if attributes.predicate.is_some() {
                        return Err(AttributeError::MultipleIris {
                            span: lit_str.span(),
                        });
                    }
                    // TODO function for parsing and expanding
                    let _iri = parse_iri(lit_str.clone())?;
                    let iri =
                        prefix_mappings
                            .expand(lit_str.value())
                            .context(InvalidExpansionSnafu {
                                span: lit_str.span(),
                            })?;
                    attributes.predicate = Some(iri);
                }
                FieldAttribute::Flatten => {
                    attributes.flatten = true;
                }
                FieldAttribute::Id => {
                    attributes.is_id = true;
                }

                FieldAttribute::Graph => {
                    attributes.is_graph = true;
                }
            }
        }

        Ok(attributes)
    }
}
