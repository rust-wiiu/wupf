//! WUPF - Macros
//!
//! Macros to make WUPF even simpler.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Default implementation for `wupf::StaticHandler`.
#[proc_macro_derive(PluginHandler)]
pub fn derive_static_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let name = &input.ident;

    quote! {
        impl ::wupf::StaticHandler for #name {
            fn handler() -> &'static ::wupf::Handler<Self> {
                static HANDLER: ::wupf::Handler<#name> = ::wupf::Handler::new();
                &HANDLER
            }
        }
    }
    .into()
}
