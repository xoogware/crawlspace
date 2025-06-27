use proc_macro::{Span, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, DeriveInput, Fields, Ident, Index, Lit};

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let syn::Data::Struct(_) = input.data else {
        panic!("Packet must be defined as a struct");
    };

    let mut id = None;
    let mut state = None;
    let mut direction = None;

    for attr in input.attrs {
        if !attr.path().is_ident("packet") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("id") {
                let lit = meta.value()?.parse()?;
                match lit {
                    Lit::Str(i) => {
                        id = Some(i);
                    }
                    _ => panic!("attribute value `id` must be a string"),
                }
            } else if meta.path.is_ident("state") {
                let lit = meta
                    .value()
                    .expect("no value for state")
                    .parse()
                    .expect("couldn't parse value for state");
                let Lit::Str(v) = lit else {
                    panic!("unable to parse state as string");
                };
                state = Some(
                    v.parse_with(syn::Path::parse_mod_style)
                        .expect("couldn't parse state as path"),
                );
            } else if meta.path.is_ident("serverbound") {
                match direction {
                    None => direction = Some("Serverbound"),
                    Some(_) => {
                        panic!("cannot have two directives of type `serverbound` or `clientbound`")
                    }
                }
            } else if meta.path.is_ident("clientbound") {
                match direction {
                    None => direction = Some("Clientbound"),
                    Some(_) => {
                        panic!("cannot have two directives of type `serverbound` or `clientbound`")
                    }
                }
            } else {
                let Some(id) = meta.path.get_ident() else {
                    panic!("unable to get ident for unrecognized directive");
                };
                panic!("unrecognized directive {}", id);
            }

            Ok(())
        })
        .unwrap();
    }

    let id = id.expect("id must be provided for packet");
    let state = state.expect("state must be provided for packet");
    let direction = Ident::new(
        direction.expect("direction must be provided for packet"),
        Span::call_site().into(),
    );

    let name = input.ident;
    let where_clause = input.generics.where_clause.clone();
    let generics = input.generics;

    quote! {
    impl #generics Packet for #name #generics #where_clause {
        fn id() -> &'static str {
            #id
        }

        fn state() -> PacketState {
            #state
        }

        fn direction() -> PacketDirection {
            PacketDirection::#direction
        }
    }
    }
    .into()
}

/// Automatically implements "straight-across" encoding for the given struct, i.e. fields are
/// serialized in order as is. Supports #[varint] and #[varlong] attributes on integer types to
/// serialize as those formats instead.
#[proc_macro_derive(Encode, attributes(varint, varlong))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let syn::Data::Struct(data) = input.data else {
        panic!("Can only derive Encode on a struct");
    };

    let name = input.ident;
    let where_clause = input.generics.where_clause.clone();
    let generics = input.generics;

    let mut fields_encoded = proc_macro2::TokenStream::new();

    match data.fields {
        Fields::Named(fields) => {
            for field in fields.named {
                let field_name = field.ident.unwrap();

                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.meta.path().is_ident("varint"))
                {
                    fields_encoded.extend(quote! {
                        VarInt(self.#field_name as i32).encode(&mut w)?;
                    });
                } else if field
                    .attrs
                    .iter()
                    .any(|attr| attr.meta.path().is_ident("varlong"))
                {
                    fields_encoded.extend(quote! {
                        VarLong(self.#field_name as i64).encode(&mut w)?;
                    });
                } else {
                    fields_encoded.extend(quote! {
                        self.#field_name.encode(&mut w)?;
                    });
                }
            }
        }
        Fields::Unnamed(fields) => {
            for (i, field) in fields.unnamed.iter().enumerate() {
                let i = Index::from(i);

                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.meta.path().is_ident("varint"))
                {
                    fields_encoded.extend(quote! {
                        VarInt(self.#i as i32).encode(&mut w)?;
                    });
                } else if field
                    .attrs
                    .iter()
                    .any(|attr| attr.meta.path().is_ident("varlong"))
                {
                    fields_encoded.extend(quote! {
                        VarLong(self.#i as i64).encode(&mut w)?;
                    });
                } else {
                    fields_encoded.extend(quote! {
                        self.#i.encode(&mut w)?;
                    });
                }
            }
        }
        Fields::Unit => (),
    }

    quote! {
        impl #generics Encode for #name #generics #where_clause {
            fn encode(&self, mut w: impl std::io::Write) -> color_eyre::Result<()> {
                #fields_encoded

                Ok(())
            }
        }
    }
    .into()
}
