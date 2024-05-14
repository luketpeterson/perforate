

use quote::{quote, quote_spanned};
use proc_macro2::{TokenTree, Span};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned,
    spanned::Spanned,
    Attribute, Data, DataStruct, DeriveInput, Error, Field, Fields, Ident, Meta, Result,
};
use heck::AsUpperCamelCase;

#[proc_macro_derive(Perforate, attributes(perforate))]
pub fn perforate(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let result = perforate_impl(item);
    result.unwrap_or_else(|err| err.to_compile_error().into())
}

fn perforate_impl(item: DeriveInput) -> Result<proc_macro::TokenStream> {
    let item_ident = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    //Inject #[repr(C)] if it's not there already
    ensure_repr_c(&item)?;

    let mut perforated_structs = vec![];
    let mut perforate_impl_members = vec![];
    let mut perforated_struct_impls = vec![];
    if let Data::Struct(the_struct) = &item.data {
        let mut temp_struct = the_struct.clone();

        //Go over all the fields, and record which ones have the inner attrib set
        let mut variant_idents = vec![];
        visit_struct_fields(&mut temp_struct, |field| {
            if field_has_attrib(field, "perforate") {
                variant_idents.push(field.ident.clone().unwrap());
            }
        })?;

        //For each variant, generate a perforated struct, a function to get there, and an impl to get back
        for variant_ident in variant_idents {

            //Compose the new structs
            let new_struct_name = Ident::new(&format!("{}Perf{}", item_ident.to_string(), AsUpperCamelCase(variant_ident.to_string())), variant_ident.span());
            let mut new_perforated_struct = the_struct.clone();
            let mut taken_type = None;
            visit_struct_fields(&mut new_perforated_struct, |field| {
                if &variant_ident == field.ident.as_ref().unwrap() {
                    let field_ty = &field.ty;
                    taken_type = Some(field_ty.clone());
                    *field = parse_quote_spanned!(variant_ident.span()=> __perforation: core::mem::MaybeUninit<[u8; core::mem::size_of::<#field_ty>()]>);
                } else {
                    //QUESTION: Since we strip off the outer attribs (like derives), we need to strip
                    // off the field attribs too.  But is there a more correct behavior here?
                    field.attrs = vec![];
                }
            })?;
            perforated_structs.push(derive_input_from_struct(new_struct_name.clone(), new_perforated_struct, &item));

            //Compose the methods to take the field
            let taken_type = taken_type.unwrap();
            let perf_func_ident = Ident::new(&format!("perforate_{}", variant_ident.to_string()), variant_ident.span());
            perforate_impl_members.push(quote_spanned!{variant_ident.span()=>
                pub fn #perf_func_ident(self) -> (#new_struct_name #type_generics, #taken_type) {
                    let perf_struct: #new_struct_name = unsafe { core::mem::transmute(self) };
                    let taken_val: #taken_type = unsafe { core::mem::transmute(perf_struct.__perforation) };
                    (perf_struct, taken_val)
                }
            });

            //Compose the impls to put the field back
            perforated_struct_impls.push(quote_spanned!{variant_ident.span()=>
                impl #impl_generics #new_struct_name #type_generics #where_clause {
                    pub fn replace_perf(mut self, taken_val: #taken_type) -> #item_ident {
                        unsafe{ core::ptr::copy_nonoverlapping::<u8>( core::ptr::from_ref(&taken_val).cast(), self.__perforation.as_mut_ptr().cast(), core::mem::size_of::<#taken_type>() ); }
                        core::mem::forget(taken_val);
                        unsafe{ core::mem::transmute(self) }
                    }
                }
            });
        }

    } else {
        return Err(Error::new( Span::call_site(), format!("perforate macro currently implemented for only `struct`")))
    }

    Ok(quote! {
        #(#perforated_structs)*

        impl #impl_generics #item_ident #type_generics #where_clause {
            #(#perforate_impl_members)*
        }

        #(#perforated_struct_impls)*
    }.into())
}

/// Adds `#[repr(C)]` to the attributs list if it's not there.  Error if an incompatible repr exists already
fn ensure_repr_c(item: &DeriveInput) -> Result<()> {
    let err_str = || format!("perforate macro requires type be `#[repr(C)]`");
    let mut found = false;
    for attrib in item.attrs.iter() {
        match attr_is_repr(attrib) {
            Some(ident) => {
                if ident.to_string() == "C" {
                    found = true;
                } else {
                    return Err(Error::new( ident.span(), err_str()))
                }
            }
            None => {}
        }
    }
    match found {
        true => Ok(()),
        false => Err(Error::new( Span::call_site(), err_str()))
    }
}

//NOTE, The below dead-code is a parallel implementation based on an outer attribute.  It's theoretically
// more flexible than the derive macro, but involves more non-standard syntax
//  whould be nice to do this with inner attributes (field atributes but... https://github.com/rust-lang/rust/issues/54726)

// #[proc_macro_attribute]
// pub fn perforate(
//     attr: proc_macro::TokenStream,
//     item: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let args = parse_macro_input!(attr as PerforateArgs);
//     let item = parse_macro_input!(item as DeriveInput);
//     let result = perforate_impl(args, item);
//     result.unwrap_or_else(|err| err.to_compile_error().into())
// }

// struct PerforateArgs {
//     fields: Punctuated::<Ident, Token![,]>,
// }

// impl Parse for PerforateArgs {
//     fn parse(input: ParseStream) -> Result<Self> {
//         Ok(Self {
//             fields: Punctuated::<Ident, Token![,]>::parse_terminated(input)?
//         })
//     }
// }

// fn perforate_impl(args: PerforateArgs, mut item: DeriveInput) -> Result<proc_macro::TokenStream> {

//     //Inject #[repr(C)] if it's not there already
//     ensure_repr_c(&mut item)?;

//     // let mut perforated_structs = vec![];
//     if let Data::Struct(the_struct) = &item.data {

//         for variant in args.fields.iter() {

//         }
//     } else {
//         return Err(Error::new( Span::call_site(), format!("perforate macro currently implemented for only `struct`")))
//     }

//     Ok(quote! {
//         #item
//     }.into())
// }

// /// Adds `#[repr(C)]` to the attributs list if it's not there.  Error if an incompatible repr exists already
// fn ensure_repr_c(item: &mut DeriveInput) -> Result<()> {
//     let mut repr_c: Option<Attribute> = Some(parse_quote!(#[repr(C)]));
//     for attrib in item.attrs.iter() {
//         match attr_is_repr(attrib) {
//             Some(ident) => {
//                 if ident.to_string() == "C" {
//                     repr_c = None;
//                 } else {
//                     return Err(Error::new( ident.span(), format!("perforate macro requires type be `#[repr(C)]`")))
//                 }
//             }
//             None => {}
//         }
//     }
//     if let Some(attr) = repr_c {
//         item.attrs.push(attr);
//     }
//     Ok(())
// }

/// Extracts the "XX" from a `#[repr(XX)]` attribute, or None
fn attr_is_repr(attr: &Attribute) -> Option<Ident> {
    match &attr.meta {
        Meta::List(list) => {
            let attr_ident = list.path.get_ident()?;
            if attr_ident.to_string() == "repr" {
                list.tokens.clone().into_iter().next().and_then(|tok| {
                    match tok {
                        TokenTree::Ident(ident) => Some(ident),
                        _ => None
                    }
                })
            } else {
                None
            }
        },
        _ => None
    }
}

/// Extracts the field name identifiers from the struct
fn visit_struct_fields<F: FnMut(&mut Field)>(the_struct: &mut DataStruct, f: F) -> Result<()> {
    match &mut the_struct.fields {
        Fields::Named(fields) => {
            fields.named.iter_mut().for_each(f);
            Ok(())
        },
        _ => return Err(Error::new( the_struct.fields.span(), "struct fields must be named"))
    }
}

/// Tests whether a field has a specific simple (Meta::Path based) attrib
fn field_has_attrib(field: &Field, test_attr: &str) -> bool {
    for attr in &field.attrs {
        match &attr.meta {
            Meta::Path(path) => {
                if let Some(attr_ident) = path.get_ident() {
                    if attr_ident.to_string() == test_attr {
                        return true;
                    }
                }
            },
            _ => {}
        }
    }
    false
}

fn derive_input_from_struct(name: Ident, the_struct: DataStruct, source: &DeriveInput) -> DeriveInput {
    DeriveInput {
        attrs: vec![parse_quote!{#[allow(dead_code)]}, parse_quote!{#[repr(C)]}],
        vis: source.vis.clone(),
        ident: name,
        generics: source.generics.clone(),
        data: Data::Struct(the_struct)
    }
}