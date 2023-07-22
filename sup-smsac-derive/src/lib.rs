/// SPDX-FileCopyrightText: 2023 sup39 <sms@sup39.dev>
/// SPDX-License-Identifier: MIT OR Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(DecodeBE)]
pub fn decode_be_derive(input: TokenStream) -> TokenStream {
  let ast: syn::DeriveInput = syn::parse(input).expect("Fail to parse input token stream");
  let syn::Data::Struct(data) = &ast.data else {
    panic!("Only struct can derives DecodeBE");
  };

  let type_name = &ast.ident;
  let type_generics = &ast.generics;
  let type_generics_params = ast.generics.params.iter().map(|e| match e {
    syn::GenericParam::Type(ty) => {let ident = &ty.ident; quote! {#ident}},
    syn::GenericParam::Lifetime(ty) => {let lifetime = &ty.lifetime; quote! {#lifetime}},
    syn::GenericParam::Const(ty) => {let ident = &ty.ident; quote! {#ident}},
  });
  let q_impl = quote! {impl #type_generics DecodeBE for #type_name<#(#type_generics_params),*>};
  let fields = match &data.fields {
    syn::Fields::Named(fields) => &fields.named,
    syn::Fields::Unnamed(fields) => &fields.unnamed,
    syn::Fields::Unit => panic!("Unit type cannot derive DecodeBE"),
  };
  let q_size = fields.iter().map(|e| {
    let ty = &e.ty;
    quote! {<#ty>::PACKED_SIZE}
  });

  // TODO empty struct
  let mut ty0: Option<&syn::Type> = None;
  let q_new_self = match &data.fields {
    syn::Fields::Named(_) => {
      let q_decode_fields = fields.iter().map(|e| {
        let name = &e.ident;
        let ty = &e.ty;
        let ty_last = ty0;
        ty0 = Some(ty);
        if let Some(ty0) = ty_last {
          quote! {#name: {ptr = ptr.add(#ty0::PACKED_SIZE); <#ty>::decode_be(ptr)}}
        } else {
          quote! {#name: <#ty>::decode_be(ptr)}
        }
      });
      quote! {
        Self{#(#q_decode_fields),*}
      }
    },
    syn::Fields::Unnamed(_) => {
      let q_decode_fields = fields.iter().map(|e| {
        let ty = &e.ty;
        let ty_last = ty0;
        ty0 = Some(ty);
        if let Some(ty0) = ty_last {
          quote! {{ptr = ptr.add(#ty0::PACKED_SIZE); <#ty>::decode_be(ptr)}}
        } else {
          quote! {<#ty>::decode_be(ptr)}
        }
      });
      quote! {
        Self(#(#q_decode_fields),*)
      }
    },
    syn::Fields::Unit => panic!("Unit type cannot derive DecodeBE"),
  };
  quote! {
    #q_impl {
      const PACKED_SIZE: usize = #(#q_size)+*;
      unsafe fn decode_be(ptr: *const u8) -> Self {
        let mut ptr: *const u8 = ptr;
        #q_new_self
      }
    }
  }.into()
}

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[proc_macro]
pub fn match_class_from_json(input: TokenStream) -> TokenStream {
  let file_name = input.to_string();
  let file_name = if file_name.starts_with('\"') && file_name.ends_with('\"') {
    &file_name[1..file_name.len()-1]
  } else {
    &file_name
  };
  // let file = File::open(&file_name).unwrap();
  // let cwd = std::env::current_dir().unwrap();
  let file = File::open(&file_name).expect(&file_name);
  let reader = BufReader::new(file);
  let vt: HashMap<String, String> = serde_json::from_reader(reader)
    .expect("The JSON file is not in the form of HashMap<String, String>");
  let entries = vt.iter().map(|e| {
    let (addr, name) = e;
    let addr = u32::from_str_radix(addr, 16).unwrap();
    quote! {#addr => Some(#name)}
  });
  quote! {
    |x| match x {
      #(#entries),*,
      _ => None,
    }
  }.into()
}
