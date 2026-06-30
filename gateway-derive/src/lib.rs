use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let route_id = parse_macro_input!(attr as syn::Expr);
    let function = parse_macro_input!(item as ItemFn);

    let fn_name = &function.sig.ident;

    // 1. Extract the specific payload type from the function signature
    let inputs = &function.sig.inputs;

    let payload_arg_type = if inputs.len() == 2 {
        if let FnArg::Typed(pat_type) = &inputs[1] {
            // We expect a reference, e.g., `&GameEventPingRequest`
            if let Type::Reference(type_ref) = &*pat_type.ty {
                &type_ref.elem // This extracts the inner type (GameEventPingRequest)
            } else {
                panic!("The second argument must be a reference (e.g., &MyRequestData)");
            }
        } else {
            panic!("Invalid argument pattern");
        }
    } else {
        panic!("Route handler must take exactly 2 arguments: client and request data");
    };

    // 2. Generate the expanded code with automatic parsing
    let expanded = quote! {
        #function

        inventory::submit! {
            crate::gateway::routes::Route {
                id: crate::gateway::commands::#route_id,
                // The closure now wraps an async block to handle the parsing
                handler: |client, packet| {
                    Box::pin(async move {
                        // Attempt to parse the packet body into the extracted struct type
                        match super::parse_request::<#payload_arg_type>(&packet.body) {
                            Ok(parsed_data) => {
                                // Call your original function with the parsed data
                                #fn_name(client, &parsed_data).await
                            }
                            Err(e) => {
                                // You can replace this with your preferred logging crate (e.g., tracing or log)
                                eprintln!("Failed to parse payload for route: {:?}", e);
                            }
                        }
                    })
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

    // 1. Extract the specific event type from the function signature
    let inputs = &function.sig.inputs;

    let event_arg_type = if inputs.len() == 2 {
        if let FnArg::Typed(pat_type) = &inputs[1] {
            // We expect a reference, e.g., `&ListRoomChatEventArgs`
            if let Type::Reference(type_ref) = &*pat_type.ty {
                &type_ref.elem // This extracts `ListRoomChatEventArgs`
            } else {
                panic!("The second argument must be a reference (e.g., &MyEvent)");
            }
        } else {
            panic!("Invalid argument pattern");
        }
    } else {
        panic!("Event handler must take exactly 2 arguments: client and event data");
    };

    // 2. Generate the expanded code with the downcast included
    let expanded = quote! {
        #function

        inventory::submit! {
            crate::gateway::events::Event {
                id: crate::gateway::events::#event_id,
                handler: |client, event| {
                    // Downcast `event` (&dyn IEventData) to our extracted type
                    let any = event as &dyn std::any::Any;
                    let typed_event = any
                        .downcast_ref::<#event_arg_type>()
                        .expect("CRITICAL: Dispatched event data did not match the expected handler type");

                    // Call the original async function with the correct types
                    Box::pin(#fn_name(client, typed_event))
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
