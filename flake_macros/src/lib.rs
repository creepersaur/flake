use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, ItemFn, LitBool, LitStr, Token, parse_macro_input};

struct FlakeArgs {
    title: Option<LitStr>,
    skip_taskbar: Option<LitBool>,
}

impl Parse for FlakeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = FlakeArgs {
            title: None,
            skip_taskbar: None,
        };

        while !input.is_empty() {
            if input.peek(LitStr) {
                // bare literal = title: #[flake("Hello")]
                args.title = Some(input.parse()?);
            } else {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                match key.to_string().as_str() {
                    "title" => args.title = Some(input.parse()?),
                    "skip_taskbar" => args.skip_taskbar = Some(input.parse()?),
                    _ => return Err(syn::Error::new_spanned(key, "unknown option")),
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(args)
    }
}

/// # Flake Main Function
///
/// A macro designed to make writing flake apps easier.
/// Does some of the heavy lifting and allows the main function to be async
/// (Required so that the window doesn't freeze when running an async task).
///
/// ## Examples
///
/// ### Simple window:
///
/// ```
/// use flake::prelude::*;
///
/// #[flake]
/// async fn main() {
///     loop {
///         next_frame().await
///     }
/// }
/// ```
///
/// ### Window with custom title
///
/// ```
/// #[flake("Hello")]  // sets the title of the window to `Hello`
/// async fn main() {
///     loop {
///         next_frame().await
///     }
/// }
/// ```
///
/// ### Window that skips the taskbar
///
/// ```
/// #[flake(skip_taskbar = true)]
/// async fn main() {
///     loop {
///         next_frame().await
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as FlakeArgs);

    let func = parse_macro_input!(item as ItemFn);
    let body = &func.block;

    if func.sig.asyncness.is_none() {
        let mut err = syn::Error::new_spanned(&func.sig, "`#[flake]` functions must be `async`");
        err.combine(syn::Error::new_spanned(
            &func.sig.fn_token,
            "help: write `async fn main()`",
        ));
        return err.to_compile_error().into();
    }

    let title_stmt = args
        .title
        .map(|t| quote! { ::flake::prelude::set_window_title(#t); });

    let skip_taskbar_stmt = args
        .skip_taskbar
        .map(|t| quote! { ::flake::prelude::set_skip_taskbar(#t); });

    quote! {
        fn main() -> ::flake::prelude::FlakeResult {
            #title_stmt
            #skip_taskbar_stmt
            ::flake::run_async(async #body)
        }
    }
    .into()
}
