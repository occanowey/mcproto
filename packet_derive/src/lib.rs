use proc_macro2::Literal;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Packet, attributes(id))]
pub fn packet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let packet_id: Literal = input
        .attrs
        .get(0)
        .and_then(|attr| attr.parse_args().ok())
        .expect("Expected packet id");

    proc_macro::TokenStream::from(quote! {
        impl Packet for #name {
            const PACKET_ID: i32 = #packet_id;
        }
    })
}

#[proc_macro_derive(PacketRead)]
pub fn packet_read(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let read_impl = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let read_calls = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;

                    quote_spanned! {f.span() => #name: #ty::read(reader)?.0,}
                });

                quote! {
                    #name {
                        #(#read_calls)*
                    }
                }
            }

            Fields::Unit => {
                quote! {#name}
            }

            _ => todo!(),
        },

        _ => todo!(),
    };

    proc_macro::TokenStream::from(quote! {
        impl PacketRead for #name {
            fn read_data<R: Read>(reader: &mut R, _: usize) -> Result<#name> {
                Ok(#read_impl)
            }
        }
    })
}

#[proc_macro_derive(PacketWrite)]
pub fn packet_write(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let write_impl = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let write_calls = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span() => packet.write(&self.#name)?; }
                });

                quote! {
                    #(#write_calls)*
                }
            }

            Fields::Unit => {
                quote! {}
            }

            _ => todo!(),
        },

        _ => todo!(),
    };

    proc_macro::TokenStream::from(quote! {
        impl PacketWrite for #name {
            fn write_data(&self, packet: &mut PacketBuilder) -> Result<()> {
                #write_impl

                Ok(())
            }
        }
    })
}
