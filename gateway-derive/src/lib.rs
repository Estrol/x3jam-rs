use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let route_id = parse_macro_input!(attr as syn::Expr);
    let function = parse_macro_input!(item as ItemFn);

    let fn_name = &function.sig.ident;

    let expanded = quote! {
        #function

        inventory::submit! {
            crate::gateway::routes::Route {
                id: crate::gateway::commands::#route_id,
                handler: |client, packet| {
                    Box::pin(#fn_name(client, packet))
                },
            }
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let event_id = parse_macro_input!(attr as syn::Expr);
    let function = parse_macro_input!(item as ItemFn);

    let fn_name = &function.sig.ident;

    let expanded = quote! {
        #function

        inventory::submit! {
            crate::gateway::events::Event {
                id: crate::gateway::events::#event_id,
                handler: |client, event| {
                    Box::pin(#fn_name(client, event))
                },
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl crate::gateway::events::IEventData for #name {}
    };

    expanded.into()
}
