use darling::{FromDeriveInput, FromField};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
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

    proxy: Option<syn::Type>,
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

    let field_reads = r#struct.fields.into_iter().map(|field| {
        let r#impl = if let Some(proxy_ty) = field.proxy {
            quote_spanned! { proxy_ty.span() => <#proxy_ty>::read(reader)?.0.into() }
        } else {
            let ty = field.ty;
            quote_spanned! { ty.span() => <#ty>::read(reader)?.0 }
        };

        if let Some(ident) = field.ident {
            quote! { #ident: #r#impl, }
        } else {
            quote! { #r#impl, }
        }
    });

    let read_impl = match r#struct.style {
        darling::ast::Style::Struct => quote! {
            Self { #(#field_reads)* }
        },

        darling::ast::Style::Tuple => quote! {
            Self( #(#field_reads)* )
        },

        darling::ast::Style::Unit => quote! { Self },
    };

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl PacketRead for #ident #ty #r#where {
            fn read_data<R: Read>(reader: &mut R, _: usize) -> Result<Self> {
                Ok(#read_impl)
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

    let field_writes = r#struct.fields.into_iter().enumerate().map(|(i, field)| {
        let ident = if let Some(ident) = field.ident {
            quote! { self.#ident }
        } else {
            let i = syn::Index::from(i);
            quote! { self.#i }
        };

        if let Some(proxy_ty) = field.proxy {
            quote_spanned! {proxy_ty.span() => packet.write::<#proxy_ty>(&(&#ident).into())?;}
        } else {
            quote_spanned! {field.ty.span() => packet.write(&#ident)?;}
        }
    });

    let write_impl = match r#struct.style {
        darling::ast::Style::Struct | darling::ast::Style::Tuple => quote! {
            #(#field_writes)*
        },

        darling::ast::Style::Unit => quote! {},
    };

    proc_macro::TokenStream::from(quote! {
        #[automatically_derived]
        impl #r#impl PacketWrite for #ident #ty #r#where {
            fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
                #write_impl

                Ok(())
            }
        }
    })
}
