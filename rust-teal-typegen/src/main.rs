// TODO: Array Record generics and function lifetimes
// TODO: Union function lifetimes and generics
// TODO: Enum lifetimes and FromLua
// TODO: Array Record FromLua
// TODO: IntoLua impl
// TODO: Make static HashMap for enums and generate them all at the end

// use std::collections::HashMap;
// use std::str::from_utf8;
// use std::{collections::HashSet, env::args};

// use once_cell::sync::{Lazy, OnceCell};
// use tree_sitter::{Node, Parser, Query, QueryCapture, QueryCursor};

// type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// fn teal_type_to_rust(node: Node) -> String {
//     match node_content(node).as_str() {
//         "any" => "mlua::Value",
//         "nil" => "()",
//         "boolean" => "bool",
//         "integer" => "i64",
//         "number" => "f64",
//         "string" => "String",
//         n => n,
//     }
//     .to_string()
// }

// macro_rules! include_root_str {
//     ($path: expr) => {
//         include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
//     };
// }

// macro_rules! include_query {
//     ($query: literal) => {
//         include_root_str!(concat!("queries/", $query, ".scm"))
//     };
// }

// static CONTENT: OnceCell<String> = OnceCell::new();
// static ENUMS: Lazy<HashSet<Vec<String>>> = Lazy::new(|| HashSet::new());

// struct Record {
//     name: String,
//     generics: Vec<String>,
//     lifetime: bool,
//     fields: HashMap<String, String>,
// }
// struct ArrayRecord {
//     name: String,
//     generics: Vec<String>,
//     lifetime: bool,
//     fields: HashMap<String, String>,
// }

// fn exec_query(query: &str, root_node: Node, f: impl FnMut(&QueryCapture) -> Result<()>) -> Result {
//     let query = Query::new(tree_sitter_teal::language(), query)?;
//     let mut qc = QueryCursor::new();
//     let qc = qc.matches(&query, root_node, &[] as &[u8]);
//     qc.flat_map(|qm| qm.captures)
//         .map(f)
//         .collect::<Result<Vec<()>>>()?;
//     Ok(())
// }

// fn get_types_in_union(node: Node) -> Vec<Node> {
//     let mut cursor = node.walk();
//     node.named_children(&mut cursor)
//         .flat_map(|n| {
//             if n.kind() == "type_union" {
//                 get_types_in_union(n)
//             } else {
//                 vec![n]
//             }
//         })
//         .collect::<Vec<Node>>()
// }

// fn enum_types_name(types: &[String]) -> String {
//     enum_type_name(&types.join("_"))
// }

// fn enum_type_name(typ: &str) -> String {
//     typ.chars()
//         .filter(|c| c.is_alphanumeric())
//         .collect::<String>()
// }

// fn resolve_function(node: Node) -> Result<(String, bool)> {
//     let return_type = match node
//         .child_by_field_name("return_type")
//         .map(|n| n.named_child(0))
//         .flatten()
//         .map(resolve_node_type)
//     {
//         Some(Ok((t, _))) => Some(t),
//         Some(Err(_)) => None,
//         None => None,
//     };
//     let mut cursor = node.walk();
//     let mut arguments = node
//         .child_by_field_name("arguments")
//         .ok_or("no arguments node")?
//         .named_children(&mut cursor)
//         .map(|n| {
//             resolve_node_type(
//                 n.child_by_field_name("type")
//                     .ok_or("no argument type node")?,
//             )
//         })
//         .collect::<Result<Vec<_>>>()?;
//     let args = arguments
//         .iter_mut()
//         .map(|(t, _)| t.as_str())
//         .collect::<Vec<_>>();

//     Ok((
//         format!(
//             "Box<dyn Fn({}) -> {} +'lua>",
//             args.join(", "),
//             return_type.unwrap_or("()".into())
//         ),
//         true,
//     ))
// }

// fn resolve_union(node: Node) -> Result<(String, bool)> {
//     let inner_types = get_types_in_union(node);
//     if inner_types
//         .iter()
//         .any(|n| n.child_by_field_name("name").map(|n| node_content(n)) == Some("nil".into()))
//     {
//         let non_nil_types = inner_types
//             .into_iter()
//             .filter(|n| {
//                 n.child_by_field_name("name").map(|n| node_content(n)) != Some("nil".into())
//             })
//             .collect::<Vec<_>>();
//         if non_nil_types.len() == 1 {
//             let (type_name, lifetime) = resolve_node_type(non_nil_types[0])?;
//             return Ok((format!("Option<{}>", type_name), lifetime));
//         }
//         let mut this_enum = vec![];
//         let mut lifetime = false;
//         for typ in non_nil_types {
//             let (type_name, inner_lifetime) = resolve_node_type(typ)?;
//             lifetime |= inner_lifetime;
//             this_enum.push(type_name);
//         }
//         return Ok((format!("Option<{}>", enum_types_name(&this_enum)), lifetime));
//     }

//     let mut this_enum = vec![];
//     let mut lifetime = false;
//     for typ in inner_types {
//         let (type_name, inner_lifetime) = resolve_node_type(typ)?;
//         lifetime |= inner_lifetime;
//         this_enum.push(type_name);
//     }
//     Ok((format!("{}", enum_types_name(&this_enum)), lifetime))
// }

// fn resolve_node_type(node: Node) -> Result<(String, bool)> {
//     match node.kind() {
//         "simple_type" => Ok((
//             teal_type_to_rust(
//                 node.child_by_field_name("name")
//                     .ok_or("no name on simple type node")?,
//             ),
//             false,
//         )),
//         "type_union" => resolve_union(node),
//         "table_type" if node.named_child_count() == 1 => match node.named_child(0) {
//             Some(mut n) => {
//                 if n.kind() == "table_type" {
//                     n = n.named_child(0).unwrap();
//                 }
//                 let (type_name, lifetime) = resolve_node_type(n)?;
//                 Ok((format!("Vec<{}>", type_name,), lifetime))
//             }
//             None => Ok((String::new(), false)),
//         },
//         "table_type" if node.child_by_field_name("tuple_type").is_some() => {
//             let mut type_name = String::new();
//             type_name.push_str("(");
//             let mut cursor = node.walk();
//             let node_types = node
//                 .named_children(&mut cursor)
//                 .map(resolve_node_type)
//                 .collect::<Result<Vec<(_, bool)>>>()?;
//             let inner_tuple = node_types
//                 .iter()
//                 .map(|(n, _)| n.as_ref())
//                 .collect::<Vec<_>>()
//                 .join(", ");

//             type_name.push_str(&inner_tuple);
//             type_name.push_str(")");
//             let lifetime = node_types.iter().any(|(_, b)| *b);

//             Ok((type_name, lifetime))
//         }
//         "table_type" => {
//             let (key_type, key_lifetime) =
//                 resolve_node_type(node.child_by_field_name("key_type").ok_or("no key type")?)?;
//             let (value_type, value_lifetime) = resolve_node_type(
//                 node.child_by_field_name("value_type")
//                     .ok_or("no value type")?,
//             )?;
//             Ok((
//                 format!("HashMap<{}, {}>", key_type, value_type),
//                 key_lifetime || value_lifetime,
//             ))
//         }
//         "function_type" => resolve_function(node),
//         k => unreachable!("Node kind {} not valid", k),
//     }
// }

// fn generate_array_reccord_def(node: Node) -> Result<String> {
//     //     let mut string = String::new();
//     //     string.push_str("struct ");
//     //     let body_node = node.child_by_field_name("record_body").unwrap();
//     //     let (array_type, mut enums, lifetime) = resolve_node_type(
//     //         node.child_by_field_name("record_body")
//     //             .ok_or("no record_body node")?
//     //             .named_child(0)
//     //             .ok_or("record_body has no children")?
//     //             .child(0)
//     //             .ok_or("no inner child for record_body")?,
//     //     )?;
//     //     string.push_str(&node_content(node.child_by_field_name("name").unwrap()));
//     //     string.push_str(" {\n");
//     //     string.push_str(&format!("\t__arr: Vec<{}>,\n", array_type));
//     //     exec_query(include_query!("record_fields"), body_node, |qc| {
//     //         string.push('\t');
//     //         string.push_str(&node_content(
//     //             qc.node
//     //                 .child_by_field_name("key")
//     //                 .ok_or("no key field on record field")?,
//     //         ));
//     //         string.push_str(": ");
//     //         let (type_name, mut field_enums, _) = resolve_node_type(
//     //             qc.node
//     //                 .child_by_field_name("type")
//     //                 .ok_or("no type field on record field")?,
//     //         )?;
//     //         enums.append(&mut field_enums);
//     //         string.push_str(&type_name);
//     //         string.push_str(",\n");
//     //         Ok(())
//     //     })?;
//     //     string.push_str("}\n");

//     //     string.push_str(
//     //         &format!(r#"impl<I> std::ops::Index<I> for ArrayRecord where I: std::slice::SliceIndex<[{0}]> {{
//     //     type Output = I::Output;
//     //     fn index(&self, index: I) -> &Self::Output {{ std::ops::Index::index(&self.__arr, index) }}
//     // }}
//     // impl<I> std::ops::IndexMut<I> for ArrayRecord where I: std::slice::SliceIndex<[{0}]> {{
//     //     fn index_mut(&mut self, index: I) -> &mut Self::Output {{ std::ops::IndexMut::index_mut(&mut self.__arr, index) }}
//     // }}"#, array_type));
//     //     string.push('\n');
//     //     for e in enums {
//     //         string.push_str(&format!("pub enum {} {{\n", enum_types_name(&e)));
//     //         for f in e {
//     //             string.push_str(&format!("\t{}({}),\n", enum_type_name(&f), &f));
//     //         }
//     //         string.push_str("}\n");
//     //     }
//     //     Ok(string)
//     todo!()
// }

// fn generate_struct_def(node: Node) -> Result<String> {
//     // let mut string = String::new();
//     // let mut conversion_impl = String::new();
//     // let struct_name = node_content(node.child_by_field_name("name").unwrap());
//     // let mut enums = vec![];
//     // let mut lifetime = false;
//     // let mut generics = vec![];
//     // let mut cursor = node.walk();
//     // string.push_str("pub struct ");
//     // string.push_str(&struct_name);

//     // let mut struct_body = String::new();

//     // let body_node = node
//     //     .child_by_field_name("record_body")
//     //     .ok_or("no record_body node")?;
//     // if let Some(typeargs) = node.child_by_field_name("typeargs") {
//     //     typeargs
//     //         .named_children(&mut cursor)
//     //         .for_each(|n| generics.push(node_content(n)));
//     // }

//     // if generics.len() != 0 {
//     //     conversion_impl.push_str("\nwhere\n");
//     //     for generic in generics.iter() {
//     //         conversion_impl.push('\t');
//     //         conversion_impl.push_str(generic);
//     //         conversion_impl.push_str(": mlua::FromLua<'lua>,\n")
//     //     }
//     // }
//     // conversion_impl.push_str(
//     //     "{\n\tfn from_lua(lua_value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self",
//     // );
//     // if generics.len() != 0 {
//     //     conversion_impl.push('<');
//     //     conversion_impl.push_str(&generics.join(", "));
//     //     conversion_impl.push('>');
//     // }
//     // conversion_impl.push_str("> {\n");
//     // conversion_impl.push_str("\t\tif let mlua::Value::Table(t) = lua_value {\n");
//     // conversion_impl.push_str("\t\t\tOk(Self {\n");

//     // exec_query(include_query!("record_fields"), body_node, |qc| {
//     //     struct_body.push('\t');
//     //     let key_name = node_content(
//     //         qc.node
//     //             .child_by_field_name("key")
//     //             .ok_or("no key field on record_field")?,
//     //     );
//     //     struct_body.push_str(&key_name);
//     //     struct_body.push_str(": ");
//     //     let (type_name, mut field_enums, field_lifetime) = resolve_node_type(
//     //         qc.node
//     //             .child_by_field_name("type")
//     //             .ok_or("no type field on record_field")?,
//     //     )?;
//     //     lifetime |= field_lifetime;
//     //     enums.append(&mut field_enums);
//     //     struct_body.push_str(&type_name);
//     //     struct_body.push_str(",\n");

//     //     conversion_impl.push_str("\t\t\t\t");
//     //     conversion_impl.push_str(&key_name);
//     //     conversion_impl.push_str(": t.get(\"");
//     //     conversion_impl.push_str(&key_name);
//     //     conversion_impl.push_str("\")?,\n");
//     //     Ok(())
//     // })?;
//     // if lifetime {
//     //     generics.push("'lua".into());
//     // }
//     // if generics.len() != 0 {
//     //     string.push('<');
//     //     string.push_str(&generics.join(", "));
//     //     string.push('>');
//     // }
//     // string.push_str(" {\n");
//     // string.push_str(&struct_body);
//     // string.push_str("}");

//     // conversion_impl.push_str("\t\t\t})\n");
//     // conversion_impl.push_str("\t\t} else {\n");
//     // conversion_impl.push_str(
//     //     "\t\t\tErr(mlua::Error::FromLuaConversionError { from: lua_value.type_name(), to: \"",
//     // );
//     // conversion_impl.push_str(&struct_name);
//     // conversion_impl.push_str("\", message: Some(\"Not a table\".into()) })\n\t\t}\n\t}\n}\n");

//     // string.push('\n');

//     // string.push_str("impl<");
//     // string.push_str(&[generics.as_slice(), &["'lua".into()]].concat().join(", "));

//     // string.push_str("> mlua::FromLua<'lua> for ");
//     // string.push_str(&struct_name);
//     // if generics.len() != 0 {
//     //     string.push('<');
//     //     string.push_str(&generics.join(", "));
//     //     string.push('>');
//     // }
//     // string.push_str(" \n");
//     // string.push_str(&conversion_impl);
//     // for e in enums {
//     //     string.push_str(&format!("pub enum {} {{\n", enum_types_name(&e)));
//     //     for f in e {
//     //         string.push_str(&format!("\t{}({}),\n", enum_type_name(&f), &f));
//     //     }
//     //     string.push_str("}\n");
//     // }
//     // Ok(string)
//     todo!()
// }

// fn generate_enum_def(node: Node) -> Result<String> {
//     let mut string = String::new();
//     string.push_str("pub enum ");
//     string.push_str(&node_content(
//         node.child_by_field_name("name")
//             .ok_or("no name field on enum")?,
//     ));
//     string.push_str(" {\n");
//     exec_query(
//         include_query!("enum_fields"),
//         node.child_by_field_name("enum_body").unwrap(),
//         |cq| {
//             string.push('\t');
//             string.push_str(&node_content(cq.node));
//             string.push_str(",\n");
//             Ok(())
//         },
//     )?;

//     string.push('}');

//     Ok(string)
// }

// fn node_content<'a>(node: Node) -> String {
//     from_utf8(&CONTENT.get().unwrap().as_str().as_bytes()[node.byte_range()])
//         .unwrap()
//         .to_string()
// }

// fn main() -> Result {
//     CONTENT.set(std::fs::read_to_string(
//         args().nth(1).unwrap_or("default".into()),
//     )?)?;
//     let mut parser = Parser::new();
//     parser.set_language(tree_sitter_teal::language()).unwrap();
//     let tree = parser
//         .parse(CONTENT.get().unwrap().to_string(), None)
//         .unwrap();

//     println!("#![allow(non_camel_case_types)]");
//     println!("use std::collections::HashMap;");
//     exec_query(include_query!("record_type"), tree.root_node(), |qc| {
//         match qc.node.prev_named_sibling() {
//             Some(pn) if pn.kind() == "comment" => {
//                 println!("{}", node_content(pn).replace("--", "///"));
//             }
//             Some(_) => {}
//             None => {}
//         }

//         if qc
//             .node
//             .child_by_field_name("record_body")
//             .unwrap()
//             .named_child(0)
//             .map(|n| n.kind() == "record_array_type")
//             .unwrap_or(false)
//         {
//             println!("{}", generate_array_reccord_def(qc.node)?);
//         } else {
//             println!("{}", generate_struct_def(qc.node)?);
//         }
//         Ok(())
//     })?;
//     exec_query(include_query!("enum_type"), tree.root_node(), |qc| {
//         match qc.node.prev_named_sibling() {
//             Some(pn) if pn.kind() == "comment" => {
//                 println!("{}", node_content(pn).replace("--", "///"));
//             }
//             Some(_) => {}
//             None => {}
//         }
//         println!("{}", generate_enum_def(qc.node)?);
//         Ok(())
//     })?;

//     Ok(())
// }

fn main() {}
