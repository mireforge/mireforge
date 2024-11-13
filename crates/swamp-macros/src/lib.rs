/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/swamp/swamp
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Resource)]
pub fn resource_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Resource for #name #ty_generics #where_clause {
        }

        // TODO: Make sure type do not support Copy or Clone
        // TODO: Should be a conf to have this as an option
        // impl !Clone for #name {}
        // impl !Copy for #name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Message)]
pub fn message_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Message for #name #ty_generics #where_clause {
        }

        // TODO: Make sure type do not support Copy or Clone
        // TODO: Should be a conf to have this as an option
        // impl !Clone for #name {}
        // impl !Copy for #name {}
        impl #impl_generics #name #ty_generics #where_clause {
            #[doc(hidden)]
            const __PREVENT_COPY_CLONE: () = {
                let _marker: std::marker::PhantomData<*const ()> = std::marker::PhantomData;
            };
        }

    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Asset)]
pub fn asset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Asset for #name #ty_generics #where_clause {
        }

        // TODO: Make sure type do not support Copy or Clone
        // TODO: Should be a conf to have this as an option
        // impl !Clone for #name {}
        // impl !Copy for #name {}
    };

    TokenStream::from(expanded)
}
