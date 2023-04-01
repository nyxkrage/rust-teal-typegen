use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, TypeParam};

#[proc_macro_derive(LuaConvert)]
pub fn derive_lua_convert(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = parse_macro_input!(item);

    let from_tokens = impl_from_lua(ast.clone()).into_token_stream();
    let into_tokens = impl_into_lua(ast).into_token_stream();

    quote!(
        #from_tokens
        #into_tokens
    )
    .into()
}

fn impl_from_lua(ast: syn::DeriveInput) -> TokenStream {
    let name = ast.clone().ident;
    let generics = &mut ast.generics.type_params().collect::<Vec<_>>();
    let lifetimes = &mut ast.generics.lifetimes().collect::<Vec<_>>();
    let mut type_generics = TokenStream::new();
    if generics.len() > 0 || lifetimes.len() > 0 {
        type_generics = {
            let lifetime = if lifetimes.len() > 0 {
                quote!('lua, )
            } else {
                TokenStream::new()
            };
            quote!(<#lifetime #(#generics),*>)
        };
    }
    let where_clause = quote!(
        where
            #(#generics: ::mlua::FromLua<'lua>),*
    );
    match ast.data {
        syn::Data::Struct(s) => record_from_lua(name, generics, type_generics, where_clause, s),
        syn::Data::Enum(e) => {
            let variants = e
                .variants
                .clone()
                .into_iter()
                .map(|v| v.ident)
                .collect::<Vec<_>>();
            enum_from_lua(name, generics, type_generics, where_clause, variants, e)
        }
        syn::Data::Union(_) => panic!("Unions are not supported"),
    }
}

fn impl_into_lua(ast: syn::DeriveInput) -> TokenStream {
    let name = ast.clone().ident;
    let generics = &mut ast.generics.type_params().collect::<Vec<_>>();
    let lifetimes = &mut ast.generics.lifetimes().collect::<Vec<_>>();
    let mut type_generics = TokenStream::new();
    if generics.len() > 0 || lifetimes.len() > 0 {
        type_generics = {
            let lifetime = if lifetimes.len() > 0 {
                quote!('lua, )
            } else {
                TokenStream::new()
            };
            quote!(<#lifetime #(#generics),*>)
        };
    }
    let where_clause = quote!(
        where
            #(#generics: ::mlua::FromLua<'lua>),*
    );
    match ast.data {
        syn::Data::Struct(s) => record_into_lua(name, generics, type_generics, where_clause, s),
        syn::Data::Enum(e) => {
            let variants = e
                .variants
                .clone()
                .into_iter()
                .map(|v| v.ident)
                .collect::<Vec<_>>();
            enum_into_lua(name, generics, type_generics, where_clause, variants, e)
        }
        syn::Data::Union(_) => panic!("Unions are not supported"),
    }
}

fn record_from_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    s: syn::DataStruct,
) -> TokenStream {
    let fields = s
        .fields
        .clone()
        .into_iter()
        .map(|f| f.ident)
        .collect::<Option<Vec<_>>>();

    match fields {
        Some(fields) => {
            if fields.iter().any(|i| i.to_string().as_str() == "__arr") {
                return array_record_from_lua(
                    name,
                    generics,
                    type_generics,
                    where_clause,
                    fields
                        .into_iter()
                        .filter(|i| i.to_string().as_str() == "__arr"),
                );
            }
            quote!(
                impl<'lua #(, #generics)*> ::mlua::FromLua<'lua> for #name #type_generics #where_clause  {
                    fn from_lua(lua_value: ::mlua::Value<'lua>, _: &'lua ::mlua::Lua) -> ::mlua::Result<Self> {
                        if let ::mlua::Value::Table(t) = lua_value {
                            Ok(Self {
                                #(
                                    #fields: t.get(stringify!(#fields))?
                                ),*
                            })
                        } else {
                            Err(::mlua::Error::FromLuaConversionError {
                                from: lua_value.type_name(),
                                to: stringify!(#name),
                                message: Some(format!("{} is not a table", lua_value.type_name())),
                            })
                        }
                    }
                }
            )
        }
        None => todo!("Only named structs are implemented"),
    }
}

fn record_into_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    s: syn::DataStruct,
) -> TokenStream {
    let fields = s
        .fields
        .clone()
        .into_iter()
        .map(|f| f.ident)
        .collect::<Option<Vec<_>>>();

    match fields {
        Some(fields) => {
            if fields.iter().any(|i| i.to_string().as_str() == "__arr") {
                return array_record_into_lua(
                    name,
                    generics,
                    type_generics,
                    where_clause,
                    fields
                        .into_iter()
                        .filter(|i| i.to_string().as_str() == "__arr"),
                );
            }
            quote!(
                impl<'lua #(, #generics)*> ::mlua::IntoLua<'lua> for #name #type_generics #where_clause  {
                    fn into_lua(self, lua: &'lua ::mlua::Lua) -> ::mlua::Result<::mlua::Value<'lua>> {
                        let t = lua.create_table()?;
                        #(
                        t.set(stringify!(#fields), self.#fields)?;
                        );*
                        Ok(::mlua::Value::Table(t))
                    }
                }
            )
        }
        None => todo!("Only named structs are implemented"),
    }
}

fn array_record_from_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    fields: impl Iterator<Item = Ident>,
) -> TokenStream {
    quote!(
        impl<'lua #(, #generics)*> ::mlua::FromLua<'lua> for #name #type_generics #where_clause  {
            fn from_lua(lua_value: ::mlua::Value<'lua>, _: &'lua ::mlua::Lua) -> ::mlua::Result<Self> {
                if let ::mlua::Value::Table(t) = lua_value {
                    let mut s = Self {
                        __arr: Vec::new(),
                        #(
                            #fields: t.get(stringify!(#fields))?
                        ),*
                    };
                    s.__arr = t.sequence_values().collect::<Result<Vec<_>>>()?;
                    Ok(s)
                } else {
                    Err(::mlua::Error::FromLuaConversionError {
                        from: lua_value.type_name(),
                        to: stringify!(#name),
                        message: Some(format!("{} is not a table", lua_value.type_name())),
                    })
                }
            }
        }
    )
}

fn array_record_into_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    fields: impl Iterator<Item = Ident>,
) -> TokenStream {
    quote!(
        impl<'lua #(, #generics)*> ::mlua::intolua<'lua> for #name #type_generics #where_clause  {
            fn into_lua(self, lua: &'lua ::mlua::Lua) -> ::mlua::result<::mlua::value<'lua>> {
                let t = lua.create_table()?;
                for v in self.__arr {
                    t.push(v);
                }
                #(
                t.set(stringify!(#fields), self.#fields)?;
                );*
                ok(::mlua::value::table(t))
            }
        }
    )
}

fn enum_from_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    variants: Vec<Ident>,
    e: syn::DataEnum,
) -> TokenStream {
    if e.variants.iter().all(|v| v.fields.len() == 1) {
        union_from_lua(name, generics, type_generics, where_clause, variants, e)
    } else if e.variants.iter().all(|v| v.fields.len() == 1) {
        let variants = e.variants.into_iter().map(|v| v.ident);
        quote!(
            impl<'lua #(, #generics)*> ::mlua::FromLua<'lua> for #name #type_generics #where_clause  {
                fn from_lua(lua_value: ::mlua::Value<'lua>, _: &'lua ::mlua::Lua) -> ::mlua::Result<Self> {
                    match lua_value {
                        #(::mlua::Value::String(s) if s.to_str()? == stringify!(#variants) => Ok(Self::#variants)),*
                        _ => Err(::mlua::Error::FromLuaConversionError {
                            from: lua_value.type_name(),
                            to: stringify!(#name),
                            message: Some(format!("{} is not a string", lua_value.type_name())),
                        })
                    }
                }
            }
        )
    } else {
        panic!("Arbitrary Rust enums cannot be automatically contructed from a Lua union or enum")
    }
}

fn enum_into_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    variants: Vec<Ident>,
    e: syn::DataEnum,
) -> TokenStream {
    if e.variants.iter().all(|v| v.fields.len() == 1) {
        union_into_lua(name, generics, type_generics, where_clause, variants)
    } else if e.variants.iter().all(|v| v.fields.len() == 1) {
        quote!(
            impl<'lua #(, #generics)*> ::mlua::IntoLua<'lua> for #name #type_generics #where_clause  {
                fn into_lua(self, lua: &'lua ::mlua::Lua) -> ::mlua::Result<::mlua::Value<'lua>> {
                    match self {
                        #(Self::#variants => stringify!(#variants).to_string()),*
                    }.into_lua(lua)
                }
            }
        )
    } else {
        panic!("Arbitrary Rust enums cannot be automatically converted to a Lua union or enum")
    }
}

fn union_from_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    variants: Vec<Ident>,
    e: syn::DataEnum,
) -> TokenStream {
    let types = e
        .variants
        .into_iter()
        .map(|v| v.fields.into_iter().nth(0).unwrap().ty);
    quote!(
        impl<'lua #(, #generics)*> ::mlua::FromLua<'lua> for #name #type_generics #where_clause  {
            fn from_lua(lua_value: ::mlua::Value<'lua>, lua: &'lua ::mlua::Lua) -> ::mlua::Result<Self> {
                #(
                    match <#types as ::mlua::FromLua>::from_lua(lua_value.clone(), lua) {
                        Ok(v) => {
                            return Ok(Self::#variants (v))
                        }
                        Err(_) => (),
                    }
                );*
                Err(::mlua::Error::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: stringify!(#name),
                    message: Some(format!("{} is not a of a valid type", lua_value.type_name())),
                })
            }
        }
    )
}

fn union_into_lua(
    name: Ident,
    generics: &[&TypeParam],
    type_generics: TokenStream,
    where_clause: TokenStream,
    variants: Vec<Ident>,
) -> TokenStream {
    quote!(
        impl<'lua #(, #generics)*> ::mlua::IntoLua<'lua> for #name #type_generics #where_clause  {
            fn into_lua(self, lua: &'lua ::mlua::Lua) -> ::mlua::Result<::mlua::Value<'lua>> {
                match self {
                    #(Self::#variants(v) => ::mlua::IntoLua::into_lua(v, lua)),*
                }
            }
        }
    )
}
