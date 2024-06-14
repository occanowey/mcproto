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
#[darling(attributes(packet))]
struct PacketReadWriteFieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    with: Option<syn::Path>,

    read_with: Option<syn::Path>,
    write_with: Option<syn::Path>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
struct PacketReadWriteReciever {
    ident: syn::Ident,
    generics: syn::Generics,

    data: darling::ast::Data<(), PacketReadWriteFieldReceiver>,
}

#[proc_macro_derive(PacketRead)]
pub fn packet_read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let PacketReadWriteReciever {
        ident,
        generics,
        data,
    } = PacketReadWriteReciever::from_derive_input(&input).unwrap();

    let (r#impl, ty, r#where) = generics.split_for_impl();
    let r#struct = data.take_struct().unwrap();

    let fields = r#struct.fields.into_iter().enumerate().map(|(i, field)| {
        let field_type = field.ty;
        let value_ident = field.ident.unwrap_or_else(|| format_ident!("value{}", i));

        let read_with = field.read_with.or_else(|| {
            field.with.map(|mut path| {
                path.segments
                    .push(syn::Ident::new("buf_read", Span::call_site()).into());

                path
            })
        });

        let read_impl = if let Some(read_with) = read_with {
            quote! { #read_with(__buf) }
        } else {
            quote! { <#field_type as crate::types::BufType>::buf_read(__buf) }
        };

        (
            value_ident.clone(),
            quote! { let #value_ident = #read_impl?; },
        )
    });

    let (field_idents, field_read_impls): (Vec<_>, Vec<_>) = fields.unzip();
    let struct_create_impl = match r#struct.style {
        darling::ast::Style::Struct => quote! { Self { #(#field_idents),* } },
        darling::ast::Style::Tuple => quote! { Self( #(#field_idents),* ) },
        darling::ast::Style::Unit => quote! { Self },
    };

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl PacketRead for #ident #ty #r#where {
            fn read_data<__B: ::bytes::Buf>(__buf: &mut __B) -> std::result::Result<Self, crate::types::ReadError> {
                #(#field_read_impls)*
                Ok(#struct_create_impl)
            }
        }
    })
}

#[proc_macro_derive(PacketWrite)]
pub fn packet_write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let PacketReadWriteReciever {
        ident,
        generics,
        data,
    } = PacketReadWriteReciever::from_derive_input(&input).unwrap();

    let (r#impl, ty, r#where) = generics.split_for_impl();
    let r#struct = data.take_struct().unwrap();

    let field_write_impls = r#struct.fields.into_iter().enumerate().map(|(i, field)| {
        let ident = if let Some(ident) = field.ident {
            quote! { self.#ident }
        } else {
            let i = syn::Index::from(i);
            quote! { self.#i }
        };

        let write_with = field.write_with.or_else(|| {
            field.with.map(|mut path| {
                path.segments
                    .push(syn::Ident::new("buf_write", Span::call_site()).into());

                path
            })
        });

        if let Some(write_with) = write_with {
            quote! { #write_with(&#ident, __buf); }
        } else {
            quote! { crate::types::BufType::buf_write(&#ident, __buf); }
        }
    });

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl PacketWrite for #ident #ty #r#where {
            fn write_data<__B: bytes::BufMut>(&self, __buf: &mut __B) {
                #(#field_write_impls)*
            }
        }
    })
}
