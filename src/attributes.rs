use iref::IriBuf;
use snafu::ResultExt;
use syn::{Attribute, LitStr};

use crate::{Error, InvalidIriSnafu};

mod ast;
pub mod field;
mod parse;
pub mod r#type;
pub mod variant;

fn parse_ld_attributes<T: syn::parse::Parse>(attrs: &[Attribute]) -> Result<Vec<T>, Error> {
  Ok(
    attrs
      .iter()
      .filter(|attr| attr.path().is_ident("ld"))
      .map(|attr| attr.parse_args::<T>())
      .collect::<Result<_, _>>()?,
  )
}

fn parse_iri(lit_iri: LitStr) -> Result<IriBuf, Error> {
  IriBuf::new(lit_iri.value()).context(InvalidIriSnafu {
    span: lit_iri.span(),
  })
}
