use iref::IriBuf;
use snafu::ResultExt;

use crate::attributes::ast::FieldAttribute;
use crate::attributes::parse_ld_attributes;
use crate::prefix_mappings::PrefixMappings;
use crate::{Error, InvalidMappingSnafu};

#[derive(Debug, Default)]
pub struct RdfFieldAttributes {
  pub flatten: bool,
  pub is_graph: bool,
  pub ignore: bool,
  pub predicate: Option<IriBuf>,
  pub is_id: bool,
}

impl RdfFieldAttributes {
  pub fn try_from_attrs(
    attrs: Vec<syn::Attribute>,
    prefix_mappings: &PrefixMappings,
  ) -> Result<Self, Error> {
    let field_attrs = parse_ld_attributes(&attrs)?;

    let mut attributes = RdfFieldAttributes::default();

    for attr in field_attrs {
      match attr {
        FieldAttribute::Ignore => {
          attributes.ignore = true;
        }
        FieldAttribute::Iri(lit_str) => {
          if attributes.predicate.is_some() {
            return Err(Error::MultipleIris {
              span: lit_str.span(),
            });
          }
          let iri = prefix_mappings
            .expand(lit_str.value())
            .context(InvalidMappingSnafu {
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
