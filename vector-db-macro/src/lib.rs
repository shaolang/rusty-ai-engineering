use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident, Type, parse_macro_input};

#[proc_macro_derive(VectorDbRecord, attributes(vector))]
pub fn vector_db_record(item: TokenStream) -> TokenStream {
    let ast = &parse_macro_input!(item as DeriveInput);
    let struct_info: StructInfo = match ast.try_into() {
        Ok(struct_info) => struct_info,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let new_struct = struct_info.generate_struct();
    let impls = struct_info.generate_impls();

    quote! {
        const _: () = {
            #new_struct
            #impls
        };
    }
    .into()
}

struct StructInfo {
    name: Ident,
    vectorized_name: Ident,
    all_fields: Vec<(Ident, Type)>,
    vector_fields: Vec<(Ident, Ident)>, // (original field, new_embedding_field)
}

impl StructInfo {
    fn generate_struct(&self) -> proc_macro2::TokenStream {
        let name = &self.vectorized_name;
        let fields = self.generate_field_expressions();
        let vector_fields = self.generate_vector_field_expressions();

        quote! {
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct #name {
                #fields
                #vector_fields
            }
        }
    }

    fn generate_impls(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let vectorized_name = &self.vectorized_name;
        let field_exprs = self
            .all_fields
            .iter()
            .map(|(name, _)| quote! { #name: self.#name.clone() });
        let vector_field_exprs = self.vector_fields
            .iter()
            .map(|(name, embed_name)| {
                let msg = format!("created embeddings for {}", name);
                quote! { #embed_name: model.embed([&self.#name], None).map(|vv| vv[0].to_owned()).expect(#msg) }
            });
        let overrides_exprs = self.vector_fields
            .iter()
            .map(|(_, embed_name)| {
                let msg = format!("{embed_name} embedding field created");
                let embed_name = format!("{embed_name}");
                quote! {
                    .overwrite(#embed_name, std::sync::Arc::new(vector_db::fixed_size_list_field(#embed_name, 384)))
                    .expect(#msg)
                }
            });

        quote! {
            impl vector_db::Embeddable for #name {
                type Item = #vectorized_name;

                fn embed(&self, model: &mut fastembed::TextEmbedding) -> Self::Item {
                    #vectorized_name {
                        #(#field_exprs,)*
                        #(#vector_field_exprs,)*
                    }
                }

                fn tracing_options(&self) -> vector_db::serde_arrow::schema::TracingOptions {
                    vector_db::serde_arrow::schema::TracingOptions::default()
                        #(#overrides_exprs)*
                }
            }
        }
    }

    fn generate_field_expressions(&self) -> proc_macro2::TokenStream {
        let exprs: Vec<proc_macro2::TokenStream> = self
            .all_fields
            .iter()
            .map(|(name, ty)| quote! { #name: #ty })
            .collect();

        quote! { #(#exprs,)* }
    }

    fn generate_vector_field_expressions(&self) -> proc_macro2::TokenStream {
        let exprs: Vec<proc_macro2::TokenStream> = self
            .vector_fields
            .iter()
            .map(|(_, name)| quote! { #name: Vec<f32> })
            .collect();

        quote! { #(#exprs,)* }
    }
}

impl TryFrom<&DeriveInput> for StructInfo {
    type Error = syn::Error;

    fn try_from(ast: &DeriveInput) -> Result<Self, Self::Error> {
        let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) = ast.data
        else {
            return Err(syn::Error::new(
                Span::call_site(),
                "VectorDbRecord can only be applied to structs with named fields",
            ));
        };

        let name = ast.ident.clone();
        let vectorized_name = format_ident!("{}WithEmbeddingFields", name);
        let all_fields = named
            .iter()
            .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
            .collect();
        let vector_fields = named
            .iter()
            .filter(|f| {
                f.attrs
                    .iter()
                    .filter(|attr| &attr.meta.path().get_ident().unwrap().to_string() == "vector")
                    .count()
                    > 0
            })
            .map(|f| {
                let name = f.ident.clone().unwrap();
                let embed_name = format_ident!("{}_embedding", name);
                (name, embed_name)
            })
            .collect();

        Ok(Self {
            name,
            vectorized_name,
            all_fields,
            vector_fields,
        })
    }
}
