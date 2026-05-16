use zyn::{
    Diagnostic,
    ext::AttrExt,
    mark::error,
    syn::{FieldsNamed, Ident},
};

#[zyn::derive("Embed", attributes(embed))]
fn vector(
    #[zyn(input)] ident: Ident,
    #[zyn(input)] fields: zyn::Fields<FieldsNamed>,
) -> TokenStream {
    let field_infos: Vec<Result<FieldInfo, Diagnostic>> =
        fields.inner().named.iter().map(|f| f.try_into()).collect();
    let errors: Diagnostic = field_infos
        .iter()
        .filter(|f| f.is_err())
        .map(|e| e.clone().unwrap_err())
        .fold(zyn::mark::new(), |acc, e| acc.add(e))
        .build();

    if !errors.is_empty() {
        errors.emit()
    } else {
        let field_infos: Vec<FieldInfo> = field_infos.iter().map(|f| f.clone().unwrap()).collect();
        zyn::zyn!(
            const _: () = {
                @generate_embed_impl(ident = &ident, fis = &field_infos)
                @generate_from_impl(ident = &ident, fis = &field_infos)
            };
        )
        .into()
    }
}

#[zyn::element]
fn generate_embed_impl<'a>(ident: &'a Ident, fis: &'a [FieldInfo]) -> TokenStream {
    let Some(embed_field) = fis.iter().find(|f| f.embedding) else {
        return error(format!("Must have one embed field for {ident}"))
            .build()
            .into();
    };

    let entity_name = ident.to_string().to_ascii_lowercase();
    let embed_field_name = format!("{}", embed_field.name);

    let create_table_stmt = create_table_stmt(&entity_name, fis);
    let create_virtual_table_stmt = create_virtual_table_stmt(&entity_name, &embed_field_name);
    let search_stmt = search_stmt(&entity_name, &embed_field_name, fis);

    zyn::zyn!(
        impl helpers::vectordb::Embed for {{ ident }} {
            fn create_sqlite_table_stmt() -> String {
                {{ create_table_stmt }}.to_string()
            }

            fn create_vector_index_stmt() -> String {
                {{ create_virtual_table_stmt }}.to_string()
            }

            fn search_stmt() -> String {
                {{ search_stmt }}.to_string()
            }

            fn insert(self, tx: &tokio_rusqlite::Transaction, embedder: &mut fastembed::TextEmbedding) -> helpers::Result<()> {
                use zerocopy::IntoBytes;

                let rowid: i64 = @insert_row_stmt(entity_name = &entity_name, fis = fis);
                let embedding = embedder.embed(&[&self.{{ embed_field.name }}], None)?;
                let embedding = embedding.first().unwrap();
                let embedding_bytes = embedding.as_bytes();
                @insert_embedding_stmt(entity_name = &entity_name, embed_field_name = &embed_field_name)?;
                Ok(())
            }
        }
    )
}

#[zyn::element]
fn generate_from_impl<'a>(ident: &'a Ident, fis: &'a [FieldInfo]) -> TokenStream {
    let fields = fis.iter()
        .enumerate()
        .map(|(i, f)| zyn::zyn!( {{ f.name }}: row.get::<_, String>({{ i }}).unwrap() ));
    zyn::zyn!(
        impl From<&tokio_rusqlite::rusqlite::Row<'_>> for {{ ident }} {
            fn from(row: &tokio_rusqlite::rusqlite::Row) -> Self {
                Self {
                    @for (f in fields) { {{ f }}, }
                }
            }
        }
    )
}

#[derive(Clone, Debug, zyn::Attribute)]
#[zyn("embed")]
struct FieldInfo {
    name: Ident,
    sql_type: String,
    embedding: bool,
}

impl TryFrom<&zyn::syn::Field> for FieldInfo {
    type Error = Diagnostic;

    fn try_from(field: &zyn::syn::Field) -> Result<Self, Self::Error> {
        use zyn::ToTokens;

        let name = field.ident.clone().unwrap();
        let zyn::syn::Type::Path(tp) = field.ty.clone() else {
            return Err(error(format!("Non-primitive type is not supported: {}", name)).build());
        };
        let type_str = match tp.into_token_stream().to_string().as_str() {
            "String" => "TEXT",
            unknown => {
                return Err(error(format!(
                    "Unsupported type '{}' for field: {}",
                    unknown, name
                ))
                .build());
            }
        }
        .to_string();

        Ok(Self {
            name,
            sql_type: type_str,
            embedding: field.attrs.iter().find(|a| a.is("embed")).is_some(),
        })
    }
}

fn create_table_stmt(entity_name: &str, fis: &[FieldInfo]) -> String {
    let column_clauses = fis
        .iter()
        .map(|fi| format!("{} {}", fi.name, fi.sql_type))
        .collect::<Vec<_>>()
        .join(",");
    format!("CREATE TABLE {entity_name} ({column_clauses})")
}

fn create_virtual_table_stmt(entity_name: &str, embed_field_name: &str) -> String {
    format!(
        "CREATE VIRTUAL TABLE {entity_name}_vec USING vec0({embed_field_name}_embedding FLOAT[384])"
    )
}

#[zyn::element]
fn insert_row_stmt<'a>(entity_name: &'a str, fis: &'a [FieldInfo]) -> TokenStream {
    let placeholders = (1..=fis.len())
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let columns = columns(fis);
    let stmt =
        format!("INSERT INTO {entity_name} ({columns}) VALUES ({placeholders}) RETURNING ROWID");
    let field_names = zyn::zyn!(@for (f in fis.iter()) { self.{{ f.name }}, });

    zyn::zyn!(tx.query_row(
        { { stmt } },
        tokio_rusqlite::rusqlite::params![{ { field_names } }],
        |r| r.get(0)
    )?)
}

#[zyn::element]
fn insert_embedding_stmt<'a>(entity_name: &'a str, embed_field_name: &'a str) -> TokenStream {
    let stmt = format!(
        "INSERT INTO {entity_name}_vec (rowid, {embed_field_name}_embedding) VALUES (?1, ?2)"
    );
    zyn::zyn!(tx.execute(
        { { stmt } },
        tokio_rusqlite::rusqlite::params![rowid, embedding_bytes]
    ))
}

fn search_stmt(entity_name: &str, embed_field_name: &str, fis: &[FieldInfo]) -> String {
    let columns = columns(fis);
    format!(
        "SELECT {columns} \
           FROM {entity_name} t JOIN {entity_name}_vec v ON t.rowid = v.rowid \
          WHERE v.{embed_field_name} MATCH ?1 and k = ?2"
    )
}

fn columns(fis: &[FieldInfo]) -> String {
    fis.iter()
        .map(|fi| format!("t.{}", fi.name))
        .collect::<Vec<_>>()
        .join(",")
}
