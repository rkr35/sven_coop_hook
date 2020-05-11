use proc_macro::TokenStream as OldTokenStream;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Error, Expr, FnArg, Ident, parenthesized, parse_macro_input, Pat, Token, Type, Visibility};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

enum FunctionKind {
    Regular,
    Virtual,
}

struct Function {
    index: Expr,
    visibility: Visibility,
    name: Ident,
    args: Punctuated<FnArg, Token![,]>,
    return_type: Option<Type>,
    kind: FunctionKind,
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        // 51 pub get_local_player() -> *const Entity

        // 51
        let index = input.parse()?;
        
        // pub
        let visibility = input.parse()?;

        // get_local_player
        let name = input.parse()?;
        
        // ()
        let args;
        parenthesized!(args in input);

        let args = args.parse_terminated(FnArg::parse)?;

        // ->
        let return_type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
    
            // *const Entity
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            index,
            visibility,
            name,
            args,
            return_type,
            kind: FunctionKind::Regular,
        })
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Function { index, visibility, name, args, return_type, kind } = &self;
        
        let arg_names = args
            .iter()
            .map(|arg| if let FnArg::Typed(pattern_type) = arg {
                if let Pat::Ident(p) = &*pattern_type.pat {
                    p.ident.to_token_stream()
                } else {
                    Error::new_spanned(arg, "Unsupported argument form.").to_compile_error()
                }
            } else {
                Error::new_spanned(arg, "Unsupported argument form.").to_compile_error()
            });

        let index = quote! { (#index) as usize };

        let (function_type, address, call) = match kind {
            FunctionKind::Regular => (
                quote! { "C" fn(#args) },
                quote! { self.functions[#index] },
                quote! { function(#(#arg_names),*) }
            ),

            FunctionKind::Virtual => (
                quote! { "fastcall" fn(this: usize, edx: usize, #args) },
                quote! { (*self.vtable)[#index] },
                quote! { function(self as *const _ as usize, 0, #(#arg_names),*) }
            ),
        };

        let return_type = if let Some(return_type) = return_type {
            quote! { -> #return_type }
        } else {
            quote! {}
        };

        *tokens = quote! {
            #visibility fn #name(&self, #args) #return_type {
                type Function = extern #function_type #return_type;
                let function = unsafe { core::mem::transmute::<usize, Function>(#address) };
                #call
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

    generated.into()
}