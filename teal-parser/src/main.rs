#![allow(unused)]

use color_eyre::eyre::{eyre, ContextCompat};
use color_eyre::{Report, Result};
use std::collections::HashMap;
use std::rc::Rc;

macro_rules! include_root_str {
    ($path: expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
    };
}

macro_rules! include_query {
    ($query: literal) => {
        include_root_str!(concat!("queries/", $query, ".scm"))
    };
}

#[derive(Debug, Clone)]
pub struct Enum {
    name: String,
    variants: Vec<String>,
}

pub type TypeArgs = Vec<String>;

#[derive(Debug, Clone)]
pub struct Function {
    type_args: TypeArgs,
    parameter_types: Vec<Type>,
    return_types: Vec<Type>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Any,
    String,
    Boolean,
    Nil,
    Number,
    Integer,
    Tuple(Vec<Type>),
    Map(Rc<Type>, Rc<Type>),
    Function(Function),
    Array(Rc<Type>),
    Union(Vec<Type>),
}

impl TryFrom<Node<'_>> for Type {
    type Error = Report;

    fn try_from(node: Node) -> std::result::Result<Self, Self::Error> {
        match node.kind() {
            "simple_type" => match node.content().as_str() {
                "any" => Ok(Self::Any),
                "integer" => Ok(Self::Integer),
                "number" => Ok(Self::Number),
                "nil" => Ok(Self::Nil),
                "boolean" => Ok(Self::Boolean),
                "string" => Ok(Self::String),
                v => Err(eyre!("{} is not a valid type", v)),
            },
            "type_union" => flatten_type_union(node)
                .into_iter()
                .map(|n| n.try_into())
                .collect::<Result<Vec<Type>>>()
                .map(|v| Self::Tuple(v)),
            "table_type" => todo!("table_type not handled yet..."),
            "function_type" => {
                let type_args = node
                    .child_by_field_name("type_args")
                    .map(|n| {
                        let mut c = n.walk();
                        n.named_children(&mut c).map(|n| n.content()).collect()
                    })
                    .unwrap_or(Vec::new());

                Ok(Self::Function(Function {
                    type_args,
                    parameter_types: vec![],
                    return_types: vec![],
                }))
            }
            n => unreachable!("{} is not a valid kind for a type node", n),
        }
    }
}

fn flatten_type_union(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .flat_map(|n| {
            if n.kind() == "type_union" {
                flatten_type_union(n)
            } else {
                vec![n]
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Record {
    name: String,
    type_args: TypeArgs,
    array_type: Option<Type>,
    record_defs: Vec<Record>,
    enum_defs: Vec<Enum>,
    type_defs: HashMap<String, Type>,
    fields: HashMap<String, Type>,
}

impl TryFrom<Node<'_>> for Record {
    fn try_from(node: Node) -> Result<Self, Self::Error> {
        if node.kind() != "record_declaration" {
            return Err(eyre!("test"));
        }

        let name = node
            .child_by_field_name("name")
            .context("No record name")?
            .content();

        let mut cursor = node.walk();
        let type_args = node
            .child_by_field_name("typeargs")
            .map(|tan| {
                tan.named_children(&mut cursor)
                    .map(|n| n.content())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let body = node
            .child_by_field_name("record_body")
            .context("No record body")?;

        cursor = body.walk();
        let array_type = match body
            .named_children(&mut cursor)
            .find(|n| n.kind() == "record_array_type")
            .map(|n| {
                n.named_child(0)
                    .context("no named child on record_array_type")?
                    .try_into()
            }) {
            Some(v) => Some(v?),
            None => None,
        };

        // TODO: Find nested Records
        // TODO: Find type declarations

        cursor = body.walk();
        let fields = body
            .named_children(&mut cursor)
            .filter_map(|n| {
                if n.kind() != "field" {
                    return None;
                }
                Some((
                    n.child_by_field_name("key")?.content(),
                    n.child_by_field_name("type")?.try_into().ok()?,
                ))
            })
            .collect();

        Ok(Record {
            name: name.into(),
            type_args,
            array_type,
            record_defs: Vec::new(),
            enum_defs: Vec::new(),
            type_defs: HashMap::new(),
            fields: fields,
        })
    }

    type Error = Report;
}

use once_cell::sync::OnceCell;
use tree_sitter::{Node, Parser, Query, QueryCapture, QueryCursor};
fn exec_query(
    query: &str,
    root_node: Node,
    f: impl FnMut(&QueryCapture) -> Result<()>,
) -> Result<()> {
    let query = Query::new(tree_sitter_teal::language(), query)?;
    let mut qc = QueryCursor::new();
    let qc = qc.matches(&query, root_node, &[] as &[u8]);
    qc.flat_map(|qm| qm.captures)
        .map(f)
        .collect::<Result<Vec<()>>>()?;
    Ok(())
}

static CONTENT: OnceCell<String> = OnceCell::new();

trait Content {
    fn content(&self) -> String;
}
impl<'a> Content for Node<'a> {
    fn content(&self) -> String {
        std::str::from_utf8(&CONTENT.get().unwrap().as_bytes()[self.byte_range()])
            .unwrap()
            .to_owned()
    }
}

fn main() {
    CONTENT
        .set(std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap())
        .unwrap();

    let mut parser = Parser::new();
    parser.set_language(tree_sitter_teal::language()).unwrap();
    let tree = parser
        .parse(CONTENT.get().unwrap().to_string(), None)
        .unwrap();
    let root = tree.root_node();
    let mut cursor = root.walk();

    root.named_children(&mut cursor)
        .filter(|n| n.kind() == "record_declaration")
        .map(Record::try_from)
        .for_each(|r| {
            dbg!(r);
        });
}
