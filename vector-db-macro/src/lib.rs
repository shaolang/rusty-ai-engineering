use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
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
    let new_struct_name = Ident::new_raw(
        &format!("{}WithEmbeddingFields", struct_info.name),
        struct_info.name.span(),
    );
    let new_fields = struct_info.generate_field_expressions();
    let new_vector_fields = struct_info.generate_vector_field_expressions();

    quote! {
        struct #new_struct_name {
            #new_fields
            #new_vector_fields
        }
    }
    .into()
}

struct StructInfo {
    name: Ident,
    all_fields: Vec<(Ident, Type)>,
    vector_fields: Vec<Ident>,
}

impl StructInfo {
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
            .map(|name| {
                let name = Ident::new_raw(&format!("{}_embedding", name), name.span());
                quote! { #name: Vec<f32> }
            })
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
            .map(|f| f.ident.clone().unwrap())
            .collect();

        Ok(Self {
            name,
            all_fields,
            vector_fields,
        })
    }
}
