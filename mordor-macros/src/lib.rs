use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, ExprPath, Ident, LitStr, Token};

struct MacroInput {
    path: ExprPath,
    _comma: Token![,],
    prefix: LitStr,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            _comma: input.parse()?,
            prefix: input.parse()?,
        })
    }
}

fn get_ident(name: String) -> Ident {
    Ident::new(&name, Span::call_site())
}

#[proc_macro]
pub fn export_seaorm(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as MacroInput);

    let path = &input.path;
    let prefix = &input.prefix.value();

    let active_model = get_ident(format!("{prefix}ActiveModel"));
    let column = get_ident(format!("{prefix}Column"));
    let entity = get_ident(format!("{prefix}Entity"));
    let model = get_ident(format!("{prefix}Model"));

    quote! {
        pub use #path::{
            self, ActiveModel as #active_model, Column as #column, Entity as #entity,
            Model as #model,
        };
    }
    .into()
}
