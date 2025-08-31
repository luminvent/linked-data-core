# linked-data-core

This library provides the infrastructure for derive macros to extract RDF metadata from attributes and generate serialization/deserialization code.

- Defines [RDF attributes](./src/attributes/ast.rs) for Rust types
- Parses RDF attributes to a structured RDF metadata representation [`metadata::RdfType`](./src/rdf_metadata.rs)
- Provides a trait [`TokenGenerator`](./src/lib.rs) that authors can implement in a derive macro to generate code based on the metadata from `metadata::RdfType`

## Usage

The [linked-data-sparql](https://github.com/luminvent/linked-data-sparql) repository has an [implementation](https://github.com/luminvent/linked-data-sparql/blob/main/derive/src/lib.rs).

1. Annotate a type:

    ```Rust
    #[ld(type = "http://schema.org/Person")]
    #[ld(prefix("schema" = "http://schema.org/"))]
    struct Person {
        #[ld("schema:name")]
        name: String,
    }
    ```

2. To generate code:

    ```rust
    use linked_data_core::{
        PredicatePath, RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant,
        TokenGenerator,
    };
    use proc_macro_error::proc_macro_error;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::DeriveInput;

    #[proc_macro_derive(MyDerive, attributes(ld))]
    pub fn derive_my_trait(
        item: proc_macro::TokenStream,
    ) -> proc_macro::TokenStream {
        let raw_input = syn::parse_macro_input!(item as DeriveInput);
        let linked_data_type: RdfType<MyGenerator> = RdfType::from_derive(raw_input);

        let mut output = TokenStream::new();
        linked_data_type.to_tokens(&mut output);
        output.into()
    }

    struct MyGenerator;

    impl TokenGenerator for MyGenerator {
        fn generate_struct_tokens(rdf_struct: &RdfStruct<Self>, tokens: &mut TokenStream) {
            // Use rdf_struct metadata to generate serialization code
            let type_iri = rdf_struct.type_iri();
            let fields = &rdf_struct.fields;
            // Generate your custom code...
        }

        fn generate_enum_tokens(rdf_enum: &RdfEnum<Self>, tokens: &mut TokenStream) {
            // Handle enum types
            let variants = &rdf_enum.variants;
            // Generate your custom code...
        }
    }
    ```

## Attributes

**Type-Level Attributes**

```rust
#[ld(type = "http://schema.org/Person")]        // RDF type for instances
#[ld(prefix("schema" = "http://schema.org/"))]  // Define prefix mappings
```

**Field Attributes**

```rust
#[ld("http://schema.org/name")]  // RDF predicate IRI
#[ld("schema:name")]             // Using prefix (if defined)
#[ld(id)]                        // Mark as subject/ID field
#[ld(flatten)]                   // Inline nested properties
#[ld(graph)]                     // Contains graph data
#[ld(ignore)]                    // Exclude from RDF
```

```rust
#[ld("http://example.org/hasRole")]  // Simple predicate
```
