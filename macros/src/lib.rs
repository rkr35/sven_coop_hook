use proc_macro::TokenStream;

use syn::parse::{Parse, ParseStream, Result};

struct Functions {
    
}

impl Parse for Functions {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!()
    }
}

#[proc_macro]
pub fn functions(input: TokenStream) -> TokenStream {
    todo!()
}
