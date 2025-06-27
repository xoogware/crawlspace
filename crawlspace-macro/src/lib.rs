use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit};

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
