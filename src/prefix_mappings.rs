use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use oxiri::Iri;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("{}", reason))]
  InvalidPrefix { reason: String },
  #[snafu(transparent)]
  InvalidIri { source: oxiri::IriParseError },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix(String);

#[derive(Clone, Debug, Default)]
pub struct PrefixMappings(HashMap<Prefix, Iri<String>>);

impl PrefixMappings {
  // TODO this sucks!
  pub fn expand(&self, iri_or_prefixed_name: String) -> Result<Iri<String>, Error> {
    let iri = Iri::parse(iri_or_prefixed_name.clone())?;

    if let Some((prefix, name)) = iri_or_prefixed_name.split_once(":") {
      match Prefix::from_str(prefix) {
        Ok(prefix) => match self.get(prefix) {
          Some(ns_iri) => {
            Ok(Iri::parse(format!("{ns_iri}{name}")).expect("was already parsed as Iri"))
          }
          None => Ok(iri),
        },
        Err(_) => Ok(iri),
      }
    } else {
      Ok(iri)
    }
  }

  pub fn insert_prefix_mapping(&mut self, prefix: Prefix, iri: Iri<String>) -> Option<Iri<String>> {
    self.0.insert(prefix, iri)
  }

  pub fn get(&self, prefix: Prefix) -> Option<&Iri<String>> {
    self.0.get(&prefix)
  }
}

impl IntoIterator for PrefixMappings {
  type IntoIter = Box<dyn Iterator<Item = Self::Item>>;
  type Item = (Prefix, Iri<String>);

  fn into_iter(self) -> Self::IntoIter {
    Box::new(self.0.into_iter())
  }
}

impl Extend<(Prefix, Iri<String>)> for PrefixMappings {
  fn extend<T: IntoIterator<Item = (Prefix, Iri<String>)>>(&mut self, iter: T) {
    for (prefix, iri) in iter {
      self.insert_prefix_mapping(prefix, iri);
    }
  }
}

impl Extend<PrefixMappings> for PrefixMappings {
  fn extend<T: IntoIterator<Item = PrefixMappings>>(&mut self, iter: T) {
    iter.into_iter().for_each(|mapping| {
      for (prefix, iri) in mapping {
        self.insert_prefix_mapping(prefix, iri);
      }
    });
  }
}

impl FromIterator<(Prefix, Iri<String>)> for PrefixMappings {
  fn from_iter<I: IntoIterator<Item = (Prefix, Iri<String>)>>(iter: I) -> Self {
    let mut mappings = PrefixMappings::default();
    mappings.extend(iter);
    mappings
  }
}

impl FromIterator<PrefixMappings> for PrefixMappings {
  fn from_iter<I: IntoIterator<Item = PrefixMappings>>(iter: I) -> Self {
    let mut mappings = PrefixMappings::default();
    mappings.extend(iter);
    mappings
  }
}

impl Prefix {
  pub fn new(prefix: &str) -> Result<Self, Error> {
    let contains_colon = |s: &str| s.contains(':');
    let is_empty = |s: &str| s.chars().next().is_none();
    let has_valid_first_char = |c: char| c.is_alphabetic() || c == '_';
    let is_valid_subsequent_char =
      |c: char| c.is_alphabetic() || c.is_ascii_digit() || c == '_' || c == '-' || c == '.';

    if contains_colon(prefix) {
      return InvalidPrefixSnafu {
        reason: "prefix cannot contain colons".to_string(),
      }
      .fail();
    }

    if is_empty(prefix) {
      return InvalidPrefixSnafu {
        reason: "prefix cannot be empty".to_string(),
      }
      .fail();
    }

    let first_char = prefix.chars().next().unwrap(); // Safe due to previous check
    if !has_valid_first_char(first_char) {
      return InvalidPrefixSnafu {
        reason: "prefix must start with a letter or underscore".to_string(),
      }
      .fail();
    }

    for (i, c) in prefix.chars().enumerate() {
      if i == 0 {
        continue;
      }
      if !is_valid_subsequent_char(c) {
        return InvalidPrefixSnafu {
          reason: format!("prefix has invalid character '{}' at position {}", c, i),
        }
        .fail();
      }
    }
    Ok(Prefix(prefix.to_owned()))
  }
}

impl FromStr for Prefix {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Prefix::new(s)
  }
}

impl Display for Prefix {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}
