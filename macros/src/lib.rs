use proc_macro::TokenStream as OldTokenStream;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, LitInt, parenthesized, parse_macro_input, Token, Type};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

struct Function {
    index: LitInt,
    name: Ident,
    return_type: Type,
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        // 51: get_local_player() -> *const Entity

        // 51
        let index = input.parse()?;
        
        // :
        input.parse::<Token![:]>()?;
        
        // get_local_player
        let name = input.parse()?;
        
        // ()
        let _args;
        parenthesized!(_args in input);

        // ->
        input.parse::<Token![->]>()?;

        // *const Entity
        let return_type = input.parse()?;

        Ok(Self {
            index,
            name,
            return_type,
        })
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Function { index, name, return_type } = &self;

        *tokens = quote! {
            pub fn #name(&self) -> #return_type {
                type Function = extern "C" fn() -> #return_type;
                let address = self.functions[#index];
                let function = unsafe { core::mem::transmute::<usize, Function>(address) };
                function()
            }
        };
    }
}

struct Functions {
    functions: Punctuated<Function, Token![,]>,
}

impl Parse for Functions {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            functions: input.parse_terminated(Function::parse)?,
        })
    }
}

#[proc_macro]
pub fn functions(input: OldTokenStream) -> OldTokenStream {
    let Functions { functions } = parse_macro_input!(input as Functions);
    
    let generated: TokenStream = functions
        .into_iter()
        .map(ToTokens::into_token_stream)
        .collect();

    generated.into()
}
