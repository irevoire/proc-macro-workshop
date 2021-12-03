use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let builder = generate_builder(&name, &input.data);
    let impl_block = quote! {
        #builder
    };
    println!("{}", impl_block);
    proc_macro::TokenStream::from(impl_block)
    // proc_macro::TokenStream::from(quote!())
}

// Generate a builder with every field from the original structure wrapped in an `Option`.
fn generate_builder(name: &Ident, data: &Data) -> TokenStream {
    let builder_name = Ident::new(&format!("{}Builder", name), name.span());
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    // field_name: Option<type>,
                    let inner_struct = fields.named.iter().map(|f| {
                        let (ident, ty) = (&f.ident, &f.ty);
                        quote_spanned! {f.span()=>
                            #ident: Option<#ty>,
                        }
                    });
                    // Expands to an expression like
                    //
                    // field_name: None,
                    let inner_init = fields.named.iter().map(|f| {
                        let ident = &f.ident;
                        quote_spanned! {f.span()=>
                            #ident: None,
                        }
                    });
                    // Expands to an expression like
                    //
                    // fn field_name(&mut self, field_name: field_type) -> &mut self {
                    //     self.field_name = Some(field_name);
                    //     self
                    // }
                    let builder_methods = fields.named.iter().map(|f| {
                        let (ident, ty) = (&f.ident, &f.ty);
                        quote_spanned! {f.span()=>
                            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                                self.#ident = Some(#ident);
                                self
                            }
                        }
                    });
                    // Expands to an expression like
                    //
                    // field_name: self.struct_name.ok_or("field_name is not set")?,
                    let inner_build = fields.named.iter().map(|f| {
                        let ident = &f.ident;
                        quote_spanned! {f.span()=>
                            #ident: self.#ident.clone().ok_or(format!("{} is not set", stringify!(#ident)))?,
                        }
                    });
                    quote! {
                        pub struct #builder_name {
                            #(#inner_struct)*
                        }

                        impl #name {
                            pub fn builder() -> #builder_name {
                                #builder_name {
                                    #(#inner_init)*
                                }
                            }
                        }

                        impl #builder_name {
                            #(#builder_methods)*

                            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                                Ok(#name {
                                    #(#inner_build)*
                                })
                            }
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}
