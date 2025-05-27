use darling::{FromDeriveInput, FromField};
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(packet), supports(struct_any))]
struct PacketReciever {
    ident: syn::Ident,
    generics: syn::Generics,

    id: i32,
}

#[proc_macro_derive(Packet, attributes(packet))]
pub fn packet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let PacketReciever {
        ident,
        generics,
        id,
    } = PacketReciever::from_derive_input(&input).unwrap();

    let (r#impl, ty, r#where) = generics.split_for_impl();

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl Packet for #ident #ty #r#where {
            const PACKET_ID: i32 = #id;
        }
    })
}

#[derive(Debug, FromField)]
#[darling(attributes(buftype))]
struct BufTypeFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    with: Option<syn::Path>,

    read_with: Option<syn::Path>,
    write_with: Option<syn::Path>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
struct BufTypeReceiver {
    ident: syn::Ident,
    generics: syn::Generics,

    data: darling::ast::Data<(), BufTypeFieldReceiver>,
}

#[proc_macro_derive(BufType, attributes(buftype))]
pub fn buf_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let BufTypeReceiver {
        ident,
        generics,
        data,
    } = BufTypeReceiver::from_derive_input(&input).unwrap();

    let (r#impl, ty, r#where) = generics.split_for_impl();
    let r#struct = data.take_struct().unwrap();

    let fields = r#struct.fields.into_iter().enumerate().map(|(i, field)| {
        let field_type = field.ty;

        let value_ident = field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("value{}", i));

        let self_field_ident = field
            .ident
            .map(|ident| quote! { self.#ident })
            .unwrap_or_else(|| {
                let i = syn::Index::from(i);
                quote! { self.#i }
            });

        let read_with = field.read_with.or_else(|| {
            field.with.clone().map(|mut path| {
                path.segments
                    .push(syn::Ident::new("buf_read_len", Span::call_site()).into());

                path
            })
        });

        let write_with = field.write_with.or_else(|| {
            field.with.map(|mut path| {
                path.segments
                    .push(syn::Ident::new("buf_write", Span::call_site()).into());

                path
            })
        });

        let read_impl = if let Some(read_with) = read_with {
            quote! { #read_with(__buf) }
        } else {
            quote! { <#field_type as mcproto::types::BufType>::buf_read_len(__buf) }
        };

        let write_impl = if let Some(write_with) = write_with {
            quote! { #write_with(&#self_field_ident, __buf); }
        } else {
            quote! { mcproto::types::BufType::buf_write(&#self_field_ident, __buf); }
        };

        (
            value_ident.clone(),
            (
                quote! {
                    let (#value_ident, __value_length) = #read_impl?;
                    __length += __value_length;
                },
                write_impl,
            ),
        )
    });

    let (field_idents, field_impls): (Vec<_>, (Vec<_>, Vec<_>)) = fields.unzip();
    let field_read_impls = field_impls.0;
    let field_write_impls = field_impls.1;

    let struct_create_impl = match r#struct.style {
        darling::ast::Style::Struct => quote! { Self { #(#field_idents),* } },
        darling::ast::Style::Tuple => quote! { Self( #(#field_idents),* ) },
        darling::ast::Style::Unit => quote! { Self },
    };

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl BufType for #ident #ty #r#where {
            fn buf_read_len<B: ::bytes::Buf>(__buf: &mut B) -> Result<(Self, usize), mcproto::types::ReadError> {
                let mut __length = 0;
                #(#field_read_impls)*
                Ok((#struct_create_impl, __length))
            }

            fn buf_write<B: ::bytes::BufMut>(&self, __buf: &mut B) {
                #(#field_write_impls)*
            }
        }
    })
}
