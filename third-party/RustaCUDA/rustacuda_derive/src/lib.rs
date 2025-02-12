#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use syn::{
    parse_str, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, Generics,
    TypeParamBound,
};

use proc_macro::TokenStream as BaseTokenStream;

#[proc_macro_derive(DeviceCopy, attributes(rustacuda))]
pub fn derive_device_copy(input: BaseTokenStream) -> BaseTokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_device_copy(&ast);
    BaseTokenStream::from(gen)
}

fn impl_device_copy(input: &DeriveInput) -> TokenStream {
    let input_type = &input.ident;

    let mut rustacuda_core = quote! { ::rustacuda_core };

    for attr in &input.attrs {
        if let Some(ident) = attr.path.get_ident() {
            if *ident == "rustacuda" {
                if let Ok(parens) = syn::parse2::<syn::ExprParen>(attr.tokens.clone()) {
                    if let syn::Expr::Assign(syn::ExprAssign { left, right, .. }) = *parens.expr {
                        if let syn::Expr::Path(syn::ExprPath { path, .. }) = *left {
                            if let Some(ident) = path.get_ident() {
                                if *ident == "core" {
                                    if let syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Str(path),
                                        ..
                                    }) = *right
                                    {
                                        if let Ok(tokens) = path.value().parse() {
                                            rustacuda_core = tokens;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Generate the code to type-check all fields of the derived struct/enum/union.
    // We can't perform type checking at expansion-time, so instead we generate
    // a dummy nested function with a type-bound on DeviceCopy and call it with
    // every type that's in the struct/enum/union. This will fail to compile if
    // any of the nested types doesn't implement DeviceCopy.
    let check_types_code = match input.data {
        Data::Struct(ref data_struct) => type_check_struct(data_struct),
        Data::Enum(ref data_enum) => type_check_enum(data_enum),
        Data::Union(ref data_union) => type_check_union(data_union),
    };

    // We need a function for the type-checking code to live in, so generate a
    // complicated and hopefully-unique name for that
    let type_test_func_name = format!(
        "__verify_{}_can_implement_DeviceCopy",
        input_type.to_string()
    );
    let type_test_func_ident = Ident::new(&type_test_func_name, Span::call_site());

    // If the struct/enum/union is generic, we need to add the DeviceCopy bound to
    // the generics when implementing DeviceCopy.
    let generics = add_bound_to_generics(&input.generics, &rustacuda_core);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // Finally, generate the unsafe impl and the type-checking function.
    let generated_code = quote! {
        unsafe impl#impl_generics #rustacuda_core::DeviceCopy for #input_type#type_generics #where_clause {}

        #[doc(hidden)]
        #[allow(all)]
        fn #type_test_func_ident#impl_generics(value: &#input_type#type_generics) #where_clause {
            fn assert_impl<T: #rustacuda_core::DeviceCopy>() {}
            #check_types_code
        }
    };

    generated_code
}

fn add_bound_to_generics(generics: &Generics, rustacuda_core: &TokenStream) -> Generics {
    let mut new_generics = generics.clone();
    let bound: TypeParamBound =
        parse_str(&quote! {#rustacuda_core::DeviceCopy}.to_string()).unwrap();

    for type_param in &mut new_generics.type_params_mut() {
        type_param.bounds.push(bound.clone())
    }

    new_generics
}

fn type_check_struct(s: &DataStruct) -> TokenStream {
    let checks = match s.fields {
        Fields::Named(ref named_fields) => {
            let fields: Vec<&Field> = named_fields.named.iter().collect();
            check_fields(&fields)
        },
        Fields::Unnamed(ref unnamed_fields) => {
            let fields: Vec<&Field> = unnamed_fields.unnamed.iter().collect();
            check_fields(&fields)
        },
        Fields::Unit => vec![],
    };
    quote!(
        #(#checks)*
    )
}

fn type_check_enum(s: &DataEnum) -> TokenStream {
    let mut checks = vec![];

    for variant in &s.variants {
        match variant.fields {
            Fields::Named(ref named_fields) => {
                let fields: Vec<&Field> = named_fields.named.iter().collect();
                checks.extend(check_fields(&fields));
            },
            Fields::Unnamed(ref unnamed_fields) => {
                let fields: Vec<&Field> = unnamed_fields.unnamed.iter().collect();
                checks.extend(check_fields(&fields));
            },
            Fields::Unit => {},
        }
    }
    quote!(
        #(#checks)*
    )
}

fn type_check_union(s: &DataUnion) -> TokenStream {
    let fields: Vec<&Field> = s.fields.named.iter().collect();
    let checks = check_fields(&fields);
    quote!(
        #(#checks)*
    )
}

fn check_fields(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;
            quote! {assert_impl::<#field_type>();}
        })
        .collect()
}
