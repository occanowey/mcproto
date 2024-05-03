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
                    .push(syn::Ident::new("read", Span::call_site()).into());

                path
            })
        });

        let read_impl = if let Some(read_with) = read_with {
            quote! { #read_with(__reader, __data_length - __length) }
        } else {
            quote! { <#field_type>::read(__reader) }
        };

        (
            value_ident.clone(),
            quote! {
                let (#value_ident, __value_length) = #read_impl?;
                __length += __value_length;
            },
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
            fn read_data<__R: Read>(__reader: &mut __R, __data_length: usize) -> Result<Self> {
                let mut __length = 0;

                #(#field_read_impls)*

                assert_eq!(__length, __data_length);
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
                    .push(syn::Ident::new("write", Span::call_site()).into());

                path
            })
        });

        if let Some(write_with) = write_with {
            quote! { #write_with(__packet, &#ident)?; }
        } else {
            quote! { __packet.write(&#ident)?; }
        }
    });

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl PacketWrite for #ident #ty #r#where {
            fn write_data(&self, __packet: &mut PacketBuilder) -> Result<()> {
                #(#field_write_impls)*

                Ok(())
            }
        }
    })
}
