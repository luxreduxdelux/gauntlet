/*
* Copyright (c) 2025 luxreduxdelux
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions are met:
*
* 1. Redistributions of source code must retain the above copyright notice,
* this list of conditions and the following disclaimer.
*
* 2. Redistributions in binary form must reproduce the above copyright notice,
* this list of conditions and the following disclaimer in the documentation
* and/or other materials provided with the distribution.
*
* Subject to the terms and conditions of this license, each copyright holder
* and contributor hereby grants to those receiving rights under this license
* a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
* (except for failure to satisfy the conditions of this license) patent license
* to make, have made, use, offer to sell, sell, import, and otherwise transfer
* this software, where such license applies only to those patent claims, already
* acquired or hereafter acquired, licensable by such copyright holder or
* contributor that are necessarily infringed by:
*
* (a) their Contribution(s) (the licensed copyrights of copyright holders and
* non-copyrightable additions of contributors, in source or binary form) alone;
* or
*
* (b) combination of their Contribution(s) with the work of authorship to which
* such Contribution(s) was added by such copyright holder or contributor, if,
* at the time the Contribution is added, such addition causes such combination
* to be necessarily infringed. The patent license shall not apply to any other
* combinations which include the Contribution.
*
* Except as expressly stated above, no rights or licenses from any copyright
* holder or contributor is granted under this license, whether expressly, by
* implication, estoppel or otherwise.
*
* DISCLAIMER
*
* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
* AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
* IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE
* FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
* DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
* SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
* CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
* OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
* OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use proc_macro::TokenStream;
use serde::Serialize;
use std::collections::HashMap;
use syn::LitFloat;
use syn::LitInt;
use syn::LitStr;
use syn::{
    Attribute, Lit, Result,
    parse::{Parse, ParseStream},
};
use syn::{DeriveInput, parse_macro_input};

//================================================================

#[derive(Serialize)]
struct Entity {
    info: Option<EntityInfo>,
    data: HashMap<String, EntityField>,
}

impl Entity {
    const INFO_PATH: &str = "engine_macro/info";

    fn write(name: &str, info: Option<EntityInfo>, data: HashMap<String, EntityField>) {
        let entity = Entity { info, data };
        let entity = serde_json::to_string_pretty(&entity).unwrap();

        if !std::fs::exists(Self::INFO_PATH).unwrap() {
            std::fs::create_dir(Self::INFO_PATH).unwrap();
        }

        std::fs::write(format!("{}/{name}.json", Self::INFO_PATH), entity).unwrap();
    }

    fn parse_field(
        entity_data: &mut HashMap<String, EntityField>,
        field_name: &str,
        field_type: &str,
        attribute: &Attribute,
    ) {
        match field_type {
            "Object" => {
                let field: FieldObject = attribute.parse_args().unwrap();

                entity_data.insert(
                    field_name.to_string(),
                    EntityField::Object {
                        name: field.name.value(),
                        info: field.info.value(),
                    },
                );
            }
            "bool" => {
                let field: Field = attribute.parse_args().unwrap();

                if let syn::Lit::Bool(data) = &field.data {
                    entity_data.insert(
                        field_name.to_string(),
                        EntityField::Boolean {
                            name: field.name.value(),
                            info: field.info.value(),
                            data: data.value(),
                        },
                    );
                }
            }
            "i32" | "i64" => {
                let field: Field = attribute.parse_args().unwrap();

                if let syn::Lit::Int(data) = &field.data {
                    entity_data.insert(
                        field_name.to_string(),
                        EntityField::Integer {
                            name: field.name.value(),
                            info: field.info.value(),
                            data: data.base10_digits().parse().unwrap(),
                        },
                    );
                }
            }
            "f32" | "f64" => {
                let field: Field = attribute.parse_args().unwrap();

                if let syn::Lit::Float(data) = &field.data {
                    entity_data.insert(
                        field_name.to_string(),
                        EntityField::Decimal {
                            name: field.name.value(),
                            info: field.info.value(),
                            data: data.base10_digits().parse().unwrap(),
                        },
                    );
                }
            }
            "String" => {
                let field: Field = attribute.parse_args().unwrap();

                if let syn::Lit::Str(data) = &field.data {
                    entity_data.insert(
                        field_name.to_string(),
                        EntityField::String {
                            name: field.name.value(),
                            info: field.info.value(),
                            data: data.value(),
                        },
                    );
                }
            }
            "Color" => {
                let field: FieldColor = attribute.parse_args().unwrap();

                entity_data.insert(
                    field_name.to_string(),
                    EntityField::Color {
                        name: field.name.value(),
                        info: field.info.value(),
                        data: (
                            field.r.base10_digits().parse().unwrap(),
                            field.g.base10_digits().parse().unwrap(),
                            field.b.base10_digits().parse().unwrap(),
                        ),
                    },
                );
            }
            _ => {
                let field: FieldEnumerator = attribute.parse_args().unwrap();

                let pick = field
                    .pick
                    .iter()
                    .map(|x| (x.0.value(), x.1.value()))
                    .collect();

                entity_data.insert(
                    field_name.to_string(),
                    EntityField::Enumerator {
                        name: field.name.value(),
                        info: field.info.value(),
                        data: field.data.value(),
                        pick,
                    },
                );
            }
        };
    }
}

//================================================================

struct Info {
    text: LitStr,
    scale: Option<FieldVector3>,
}

impl Parse for Info {
    fn parse(input: ParseStream) -> Result<Self> {
        let text = input.parse()?;

        let scale = if input.parse::<syn::token::Comma>().is_ok() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { text, scale })
    }
}

//================================================================

struct Field {
    name: LitStr,
    info: LitStr,
    data: Lit,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let info = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let data = input.parse()?;

        Ok(Self { name, info, data })
    }
}

//================================================================

struct FieldVector3 {
    x: LitFloat,
    y: LitFloat,
    z: LitFloat,
}

impl Parse for FieldVector3 {
    fn parse(input: ParseStream) -> Result<Self> {
        let x = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let y = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let z = input.parse()?;

        Ok(Self { x, y, z })
    }
}

//================================================================

struct FieldColor {
    name: LitStr,
    info: LitStr,
    r: LitInt,
    g: LitInt,
    b: LitInt,
}

impl Parse for FieldColor {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let info = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let r = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let g = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let b = input.parse()?;

        Ok(Self {
            name,
            info,
            r,
            g,
            b,
        })
    }
}

//================================================================

struct FieldEnumerator {
    name: LitStr,
    info: LitStr,
    data: LitStr,
    pick: Vec<(LitStr, LitStr)>,
}

impl Parse for FieldEnumerator {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pick = Vec::default();

        let name = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let info = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let data = input.parse()?;

        while input.parse::<syn::token::Comma>().is_ok() {
            let name = input.parse()?;
            input.parse::<syn::token::Comma>()?;
            let info = input.parse()?;

            pick.push((name, info));
        }

        Ok(Self {
            name,
            info,
            data,
            pick,
        })
    }
}

//================================================================

struct FieldObject {
    name: LitStr,
    info: LitStr,
}

impl Parse for FieldObject {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        input.parse::<syn::token::Comma>()?;
        let info = input.parse()?;

        Ok(Self { name, info })
    }
}

//================================================================

#[derive(Serialize)]
#[serde(tag = "type")]
enum EntityField {
    Object {
        name: String,
        info: String,
    },
    Boolean {
        name: String,
        info: String,
        data: bool,
    },
    Integer {
        name: String,
        info: String,
        data: i32,
    },
    Decimal {
        name: String,
        info: String,
        data: f32,
    },
    String {
        name: String,
        info: String,
        data: String,
    },
    Color {
        name: String,
        info: String,
        data: (u8, u8, u8),
    },
    Enumerator {
        name: String,
        info: String,
        data: String,
        pick: Vec<(String, String)>,
    },
}

//================================================================

#[derive(Serialize)]
struct EntityInfo {
    text: String,
    scale: Option<(f32, f32, f32)>,
}

impl From<Info> for EntityInfo {
    fn from(value: Info) -> Self {
        let scale = if let Some(scale) = value.scale {
            Some((
                scale.x.base10_parse().unwrap(),
                scale.y.base10_parse().unwrap(),
                scale.z.base10_parse().unwrap(),
            ))
        } else {
            None
        };

        Self {
            text: value.text.value(),
            scale,
        }
    }
}

//================================================================

#[proc_macro_derive(Meta, attributes(info, field))]
pub fn derive_meta(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    //std::fs::write("debug.txt", format!("{:#?}", input));

    let entity_name = input.ident.to_string();
    let mut entity_info = None;
    let mut entity_data: HashMap<String, EntityField> = HashMap::default();

    if let syn::Data::Struct(data_struct) = input.data
        && let syn::Fields::Named(fields_named) = &data_struct.fields
    {
        for attribute in input.attrs {
            if attribute.path().is_ident("info") {
                let info: Info = attribute.parse_args().unwrap();
                entity_info = Some(info.into());
            }
        }

        // For each field in the structure...
        for field in &fields_named.named {
            // For each attribute in the structure...
            for attribute in &field.attrs {
                // Check if it's the entity field attribute.
                let path = attribute.path();

                if path.is_ident("field") {
                    // Try getting the field name and type.
                    if let Some(ident) = &field.ident {
                        let field_name = ident.to_string();
                        if let syn::Type::Path(type_path) = &field.ty
                            && let Some(field_type) = type_path.path.get_ident()
                        {
                            Entity::parse_field(
                                &mut entity_data,
                                &field_name,
                                &field_type.to_string(),
                                attribute,
                            );
                        }
                    }
                }
            }
        }
    }

    Entity::write(&entity_name, entity_info, entity_data);

    TokenStream::new()
}
