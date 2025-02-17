use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    meta::ParseNestedMeta, parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::{self, Dot, Token}, Error, Expr, ExprCall, Ident, ItemFn, MetaNameValue, Token
};

#[proc_macro_attribute]
pub fn system(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated<MetaNameValue, Token![,]>::parse_terminated);
    let item = parse_macro_input!(item as ItemFn);

    if args.len() == 0 {
        return Error::new(args.span(), "expected 'schedule'")
            .to_compile_error()
            .into();
    }

    let schedule = &args[0];

    if !schedule.path.is_ident("schedule") {
        return Error::new(schedule.path.span(), "expected 'schedule'")
            .to_compile_error()
            .into();
    }

    let schedule = &schedule.value;

    let (dot, conditions) = if args.len() == 2 {
        let arg = &args[1];

        if !arg.path.is_ident("conditions") {
            return Error::new(arg.path.span(), "expected 'conditions'")
                .to_compile_error()
                .into();
        }

        (Some(Token![.](Span::call_site())), Some(&arg.value))
    } else {
        (None, None)
    };

    let system = &item.sig.ident;
    let mut config = system.to_string().to_case(Case::Pascal);
    config.push_str("SystemConfig");
    let config = Ident::new(config.as_str(), Span::call_site());

    quote! {
        #item

        pub struct #config;

        impl ::bevy_bootstrap::SystemConfig for #config {
            fn add_system(app: &mut ::bevy_app::App) {
                app.add_systems(#schedule, #system #dot #conditions);
            }
        }
    }
    .into()
}
