use self::AstNode::*;
use crate::parser::{FireworkParser, Rule};
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Debug, Clone)]
pub enum AstNode {
    Str(String),
    Int(i128),
    Char(char),
    Boolean(bool),
    Identifier(String),
    Type(String),
    FnArgs(Vec<(self::AstNode, self::AstNode)>),
    AnonFn {
        args: Box<self::AstNode>,
        return_type: Box<self::AstNode>,
        value: Box<self::AstNode>,
    },

    Fn {
        name: String,
        return_type: Box<self::AstNode>,
        args: Box<self::AstNode>,
        value: Box<self::AstNode>,
    },
    Eoi,
}

pub fn parse(input: &str) -> Result<Vec<AstNode>, Error<Rule>> {
    let mut ast = vec![];
    let pairs = FireworkParser::parse(Rule::program, input)?;

    for pair in pairs {
        ast.push(build_ast(pair));
    }
    Ok(ast)
}

fn build_ast(pair: Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::name => Identifier(pair.as_str().to_string()),
        Rule::fn_name => Identifier(pair.as_str().to_string()),
        Rule::firework_type => Type(pair.as_str().to_string()),
        Rule::type_signature => build_ast(pair.into_inner().next().unwrap()),
        Rule::return_type => build_ast(pair.into_inner().next().unwrap()),
        Rule::int => Int(pair.as_str().parse().unwrap()),
        Rule::boolean => Boolean(pair.as_str().parse().unwrap()),
        Rule::string => Str(pair.as_str().to_string().replace("\"", "")),
        Rule::char => Char(pair.as_str().chars().nth(1).unwrap()),
        Rule::literal => build_ast(pair.into_inner().next().unwrap()),
        Rule::identifier => Identifier(pair.as_str().to_string()),
        Rule::fn_args => {
            let mut args: Vec<AstNode> = vec![];
            let mut types: Vec<AstNode> = vec![];

            pair.into_inner().for_each(|x| match x.as_rule() {
                Rule::name => args.push(build_ast(x)),
                Rule::type_signature => types.push(build_ast(x)),
                _ => unreachable!(),
            });

            FnArgs(
                args.iter()
                    .cloned()
                    .zip(types.iter())
                    .map(|(a, b)| (a, b.clone()))
                    .collect::<Vec<(_, _)>>(),
            )
        }
        Rule::declaration => {
            let mut _pair = pair.into_inner();
            let name = _pair.next().unwrap();
            let arg_or_type = _pair.next().unwrap();

            if let Rule::fn_args = arg_or_type.as_rule() {
                Fn {
                    name: name.as_str().to_string(),
                    args: Box::new(build_ast(arg_or_type)),
                    return_type: Box::new(build_ast(_pair.next().unwrap())),
                    value: Box::new(build_ast(_pair.next().unwrap())),
                }
            } else {
                Fn {
                    name: name.as_str().to_string(),
                    args: Box::new(AstNode::FnArgs(Vec::new())),
                    return_type: Box::new(build_ast(arg_or_type)),
                    value: Box::new(build_ast(_pair.next().unwrap())),
                }
            }
        }
        // Placeholder because pest can't parse anonymous functions :/
        Rule::anon_fn => AnonFn {
            args: Box::new(FnArgs(Vec::new())),
            return_type: Box::new(Type("".to_string())),
            value: Box::new(Boolean(false)),
        },
        Rule::expr => build_ast(pair.into_inner().next().unwrap()),
        Rule::EOI => Eoi,
        _ => unreachable!(),
    }
}