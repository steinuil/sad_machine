//! state_machine macro

// quote! macro needs a higher recursion limit
#![recursion_limit = "512"]
#![forbid(
    future_incompatible,
    macro_use_extern_crate,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    variant_size_differences
)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    single_use_lifetimes,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    unused
)]

use crate::machine::Machines;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod event;
mod initial_state;
mod machine;
mod state;
mod state_transition;
mod transition;

/// Generate the declaratively described state machine diagram.
#[proc_macro]
pub fn state_machine(input: TokenStream) -> TokenStream {
    let machines: Machines = parse_macro_input!(input as Machines);

    quote!(#machines).into()
}
