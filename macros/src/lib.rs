use proc_macro::TokenStream as OldTokenStream;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, LitInt, parenthesized, parse_macro_input, Token, Type, Visibility};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

enum FunctionKind {
    Regular,
    Virtual,
}

struct Function {
    index: LitInt,
    visibility: Visibility,
    name: Ident,
    return_type: Type,
    kind: FunctionKind,
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        // 51: pub get_local_player() -> *const Entity

        // 51
        let index = input.parse()?;
        
        // :
        input.parse::<Token![:]>()?;
        
        // pub
        let visibility = input.parse()?;

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
            visibility,
            name,
            return_type,
            kind: FunctionKind::Regular,
        })
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Function { index, visibility, name, return_type, kind } = &self;
        
        let address = match kind {
            FunctionKind::Regular => quote! { self.functions[#index] },
            FunctionKind::Virtual => quote! { (*self.vtable)[#index] },
        };

        *tokens = quote! {
            #visibility fn #name(&self) -> #return_type {
                type Function = extern "C" fn() -> #return_type;
                let function = unsafe { core::mem::transmute::<usize, Function>(#address) };
                function()
            }
        };
    }
}

struct Functions {
    entries: Punctuated<Function, Token![,]>,
}

impl Parse for Functions {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            entries: input.parse_terminated(Function::parse)?,
        })
    }
}

struct Vtable {
    entries: Punctuated<Function, Token![,]>,
}

impl Parse for Vtable {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            entries: input.parse_terminated(|p| {
                let mut entry = Function::parse(p)?;
                entry.kind = FunctionKind::Virtual;
                Ok(entry)
            })?,
        })
    }
}

#[proc_macro]
pub fn functions(input: OldTokenStream) -> OldTokenStream {
    let Functions { entries } = parse_macro_input!(input as Functions);
    
    let generated: TokenStream = entries
        .into_iter()
        .map(ToTokens::into_token_stream)
        .collect();

    generated.into()
}

#[proc_macro]
pub fn vtable(input: OldTokenStream) -> OldTokenStream {
    let Vtable { entries } = parse_macro_input!(input as Vtable);
    
    let generated: TokenStream = entries
        .into_iter()
        .map(ToTokens::into_token_stream)
        .collect();

    println!("{}", generated);

    generated.into()
}