use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input, spanned::Spanned as _};

fn is_c_style_enum(data: &syn::DataEnum) -> bool {
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return false;
        }
    }
    true
}

fn get_repr(input: &DeriveInput) -> Vec<String> {
    let mut reprs = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
                let ident = input.parse::<syn::Ident>()?;
                reprs.push(ident.to_string());
                Ok(())
            });
        }
    }

    reprs
}

#[proc_macro_derive(StructSerializer, attributes(field))]
pub fn derive_iencode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let repr = get_repr(&input);
    let repr_type = if repr.is_empty() {
        quote! { u32 }
    } else {
        match repr[0].as_str() {
            "u8" => quote! { u8 },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "i8" => quote! { i8 },
            "i16" => quote! { i16 },
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            _ => {
                // I assumming it's repr(C) or something else we don't support, so we default to u32
                quote! { u32 }
            }
        }
    };
    let is_unaligned = repr.iter().any(|r| r == "packed");
    let span_clone = input.span().clone();
    let name = input.ident;

    let expanded = match input.data {
        Data::Struct(data) => {
            let mut field_encodings = Vec::new();
            let mut next_index = 0;

            for (i, field) in data.fields.iter().enumerate() {
                let ty = &field.ty;

                let accessor = match &field.ident {
                    Some(ident) => quote!(#ident),
                    None => {
                        let index = syn::Index::from(i);
                        quote!(#index)
                    }
                };

                let mut field_index: Option<u64> = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("field") {
                        if let Ok(lit) = attr.parse_args::<syn::LitInt>() {
                            field_index = lit.base10_parse().ok();
                        }
                    }

                    if attr.path().is_ident("ignore") {
                        continue;
                    }
                }

                let field_index = match field_index {
                    Some(index) => {
                        next_index = index + 1;
                        index
                    }
                    None => {
                        let index = next_index;
                        next_index += 1;
                        index
                    }
                };

                let ty_string = quote!(#ty).to_string().replace(" ", "");

                let write_stmt = match ty_string.as_str() {
                    "bool32" => quote! {
                        writer.write_u32::<LittleEndian>(self.#accessor as u32)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    },
                    _ => {
                        if is_unaligned {
                            quote! {
                                let r#ref = unsafe {
                                    std::ptr::addr_of!(self.#accessor).cast::<#ty>().read_unaligned()
                                };

                                r#ref.impl_encode(writer)?;
                            }
                        } else {
                            quote! {
                                self.#accessor.impl_encode(writer)?;
                            }
                        }
                    }
                };

                field_encodings.push((field_index, write_stmt));
            }

            field_encodings.sort_by_key(|k| k.0);
            let stmts: Vec<_> = field_encodings.into_iter().map(|(_, stmt)| stmt).collect();

            // Check if crate is self-referential and adjust the path accordingly
            if std::env::var("CARGO_PKG_NAME").unwrap() == "encoder" {
                quote! {
                    impl crate::gateway::StructEncodeImpl for #name {
                        #[inline(always)]
                        fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
                            pub use crate::gateway::byteorder_lite::{WriteBytesExt, LittleEndian};

                            #(#stmts)*

                            Ok(())
                        }
                    }
                }
            } else {
                quote! {
                    impl encoder::StructEncodeImpl for #name {
                        #[inline(always)]
                        fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
                            pub use encoder::byteorder_lite::{WriteBytesExt, LittleEndian};
                            pub use encoder::StructEncodeImpl;

                            #(#stmts)*

                            Ok(())
                        }
                    }
                }
            }
        }
        Data::Enum(data) => {
            if !is_c_style_enum(&data) {
                return syn::Error::new(
                    span_clone,
                    "StructSerializer can only be derived for C-style enums",
                )
                .to_compile_error()
                .into();
            }

            if std::env::var("CARGO_PKG_NAME").unwrap() == "encoder" {
                quote! {
                    impl crate::gateway::StructEncodeImpl for #name {
                        #[inline(always)]
                        fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
                            pub use crate::gateway::byteorder_lite::{WriteBytesExt, LittleEndian};

                            let value: #repr_type = (*self as #repr_type);
                            value.impl_encode(writer)
                        }
                    }

                    impl From<#name> for #repr_type {
                        fn from(value: #name) -> Self {
                            value as #repr_type
                        }
                    }
                }
            } else {
                quote! {
                    impl encoder::StructEncodeImpl for #name {
                        #[inline(always)]
                        fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
                            pub use encoder::byteorder_lite::{WriteBytesExt, LittleEndian};
                            pub use encoder::StructEncodeImpl;

                            let value: #repr_type = (*self as #repr_type);
                            value.impl_encode(writer)
                        }
                    }

                    impl From<#name> for #repr_type {
                        fn from(value: #name) -> Self {
                            value as #repr_type
                        }
                    }
                }
            }
        }
        _ => {
            return syn::Error::new(
                span_clone,
                "StructSerializer can only be derived for structs and enums",
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(StructDeserializer, attributes(field))]
pub fn derive_idecode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let repr = get_repr(&input);
    let repr_type = if repr.is_empty() {
        quote! { u32 }
    } else {
        match repr[0].as_str() {
            "u8" => quote! { u8 },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "i8" => quote! { i8 },
            "i16" => quote! { i16 },
            "i32" => quote! { i32 },
            "i64" => quote! { i64 },
            _ => {
                quote! { u32 }
            }
        }
    };
    let is_unaligned = repr.iter().any(|r| r == "packed");
    if is_unaligned {
        return syn::Error::new(
            input.span(),
            "Structs with unaligned fields cannot be deserialized using StructDeserializer",
        )
        .to_compile_error()
        .into();
    }

    let span_clone = input.span().clone();
    let name = input.ident;

    let expanded = match input.data {
        Data::Enum(data) => {
            if !is_c_style_enum(&data) {
                return syn::Error::new(
                    span_clone,
                    "StructDeserializer can only be derived for C-style enums",
                )
                .to_compile_error()
                .into();
            }

            let variant_decodings = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let discriminant = variant
                    .discriminant
                    .as_ref()
                    .expect("C-style enum variants must have discriminants")
                    .1
                    .clone();
                quote! {
                    #discriminant => Ok(Self::#variant_name),
                }
            });

            if std::env::var("CARGO_PKG_NAME").unwrap() == "encoder" {
                quote! {
                    impl crate::gateway::StructDecodeImpl for #name {
                        #[inline(always)]
                        fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
                            pub use crate::gateway::byteorder_lite::{ReadBytesExt, LittleEndian};
                            pub use crate::gateway::StructDecodeImpl;

                            let value: #repr_type = StructDecodeImpl::impl_decode(reader)?;
                            match value {
                                #(#variant_decodings)*
                                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid enum discriminant: {}", value))),
                            }
                        }
                    }
                }
            } else {
                quote! {
                    impl encoder::StructDecodeImpl for #name {
                        #[inline(always)]
                        fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
                            pub use encoder::byteorder_lite::{ReadBytesExt, LittleEndian};
                            pub use encoder::StructDecodeImpl;

                            let value: #repr_type = StructDecodeImpl::impl_decode(reader)?;
                            match value {
                                #(#variant_decodings)*
                                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid enum discriminant: {}", value))),
                            }
                        }
                    }
                }
            }
        }
        Data::Struct(data) => {
            let mut field_decodings = Vec::new();
            let mut next_index = 0;

            for (i, field) in data.fields.iter().enumerate() {
                // Use the ident for named fields, or generate a name like `field_0` for tuple structs
                let var_name = field
                    .ident
                    .clone()
                    .unwrap_or_else(|| quote::format_ident!("field_{}", i));
                let ty = &field.ty;

                let mut field_index: Option<u64> = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("field") {
                        if let Ok(lit) = attr.parse_args::<syn::LitInt>() {
                            field_index = lit.base10_parse().ok();
                        }
                    }
                }

                let field_index = match field_index {
                    Some(index) => {
                        next_index = index + 1;
                        index
                    }
                    None => {
                        let index = next_index;
                        next_index += 1;
                        index
                    }
                };

                let ty_string = quote!(#ty).to_string().replace(" ", "");

                // Updated matching logic for decoding
                let read_stmt = match ty_string.as_str() {
                    "bool32" => quote! {
                        let #var_name = reader.read_u32::<LittleEndian>()
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))? != 0;
                    },
                    _ => quote! {
                        let #var_name = <#ty>::impl_decode(reader)?;
                    },
                };

                // Store original index `i` so we can reconstruct the struct correctly later
                field_decodings.push((field_index, i, var_name, read_stmt));
            }

            // Sort by the `field_index` so we read from the buffer in the correct sequence
            field_decodings.sort_by_key(|k| k.0);
            let stmts: Vec<_> = field_decodings.iter().map(|(_, _, _, stmt)| stmt).collect();

            // To instantiate the struct correctly, we MUST put the variables back in their original order
            let mut init_fields = field_decodings.clone();
            init_fields.sort_by_key(|k| k.1); // Sort by original index
            let field_names: Vec<_> = init_fields.iter().map(|(_, _, name, _)| name).collect();

            // Generate the correct instantiation syntax based on the struct type
            let struct_init = match &data.fields {
                syn::Fields::Named(_) => quote! { Self { #(#field_names),* } },
                syn::Fields::Unnamed(_) => quote! { Self(#(#field_names),*) },
                syn::Fields::Unit => quote! { Self },
            };

            // Check if crate is self-referential and adjust the path accordingly
            if std::env::var("CARGO_PKG_NAME").unwrap_or_default() == "encoder" {
                quote! {
                    impl crate::gateway::StructDecodeImpl for #name {
                        #[inline(always)]
                        fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
                            pub use crate::gateway::byteorder_lite::{ReadBytesExt, LittleEndian};

                            #(#stmts)*

                            // Instantiate the struct
                            Ok(#struct_init)
                        }
                    }
                }
            } else {
                quote! {
                    impl encoder::StructDecodeImpl for #name {
                        #[inline(always)]
                        fn impl_decode(reader: &mut impl std::io::Read) -> std::io::Result<Self> {
                            pub use encoder::byteorder_lite::{ReadBytesExt, LittleEndian};
                            pub use encoder::StructDecodeImpl;

                            #(#stmts)*

                            // Instantiate the struct
                            Ok(#struct_init)
                        }
                    }
                }
            }
        }
        _ => {
            return syn::Error::new(
                span_clone,
                "StructDeserializer can only be derived for structs and enums",
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(expanded)
}
