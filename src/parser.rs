use std::collections::HashMap;

use cranelift::{
    codegen::ir::{self, FuncRef},
    prelude::{FunctionBuilder, InstBuilder, Value, Variable},
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::compiler::PedroLibC;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct RagePestParser;

#[derive(Debug)]
struct RageVariable {
    pub cl_variable: Variable,
    //pub cl_type: ir::types::Type,
    pub rage_type: RageType
}

pub struct RageParser<'a> {
    cl_fn_builder: FunctionBuilder<'a>,
    rage_variables: HashMap<String, RageVariable>,
    libc_decl: HashMap<PedroLibC, FuncRef>,
    rage_identifier_table: HashMap<String, RageExpr>
}

impl RageParser<'_> {
    pub fn parse<'a>(
        input: &'a str,
        cl_fn_builder: FunctionBuilder<'a>,
        pedro_libc: HashMap<PedroLibC, FuncRef>,
    ) -> FunctionBuilder<'a> {
        let pairs = RagePestParser::parse(Rule::program, input).unwrap();

        let mut pedro_parser = RageParser {
            cl_fn_builder,
            rage_variables: HashMap::new(),
            libc_decl: pedro_libc,
            rage_identifier_table: HashMap::new()
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::var_declaration => pedro_parser.parse_var_decl(pair),
                Rule::var_definition => pedro_parser.parse_var_def(pair),
                Rule::fn_call => pedro_parser.parse_fn_call(pair),
                Rule::fn_def => {},
                Rule::EOI => {}
                _ => {}
            }
        }

        pedro_parser.cl_fn_builder
    }

    fn parse_var_decl(&mut self, pair: Pair<Rule>) {
        let pairs = pair.into_inner();

        let mut identifier = "";
        let mut rage_type = RageType::Null;

        for pair in pairs {
            match pair.as_rule() {
                Rule::identifier => identifier = pair.as_str(),
                Rule::typing => rage_type = self.parse_typing(pair).unwrap(),
                _ => unreachable!()
            }
        }

        let index = self.rage_variables.len();
        let var = Variable::from_u32(index.try_into().unwrap());

        self.cl_fn_builder.declare_var(var, rage_type.to_cl_type());

        let pedro_var = RageVariable {
            cl_variable: var,
            //cl_type: rage_type.to_cl_type(),
            rage_type
        };

        self.rage_variables.insert(identifier.to_string(), pedro_var);
    }

    fn parse_var_def(&mut self, pair: Pair<Rule>) {
        let mut pairs = pair.into_inner();

        let identifier = pairs.next().unwrap().as_str().trim();
        let inner_pair = pairs.next().unwrap().into_inner().next().unwrap();

        let parsed_val = self.parse_literal(inner_pair);

        let rage_var = self.rage_variables.get(identifier).unwrap();

        let cl_val;

        match rage_var.rage_type {
            RageType::I8 
            | RageType::I16
            | RageType::I32
            | RageType::I64
            | RageType::Boolean
            | RageType::Char
             => cl_val = self.cl_fn_builder.ins().iconst(rage_var.rage_type.to_cl_type(), parsed_val),
            RageType::F32 => cl_val = self.cl_fn_builder.ins().f32const(parsed_val as f32),
            RageType::F64 => cl_val = self.cl_fn_builder.ins().f64const(parsed_val as f64),
            RageType::String => todo!(),
            RageType::Pointer => todo!(),
            RageType::Null => todo!(),
        }

        self.cl_fn_builder.def_var(rage_var.cl_variable, cl_val);
    }

    fn parse_fn_call(&mut self, pair: Pair<Rule>) {
        let pairs = pair.into_inner();

        let mut fn_name = "";
        let mut expressions = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::identifier => fn_name = pair.as_str(),
                Rule::expression => expressions.push(self.parse_expr(pair).unwrap()),
                _ => unreachable!()
            }
        }

        match fn_name {
            "libc_putchar" => {
                //let rage_var = self.rage_variables.get(&expressions.pop().unwrap()).unwrap();

                match expressions.pop().unwrap() {
                    RageExpr::Literal(cl_val) => {
                        self.cl_fn_builder.ins().call(
                            *self.libc_decl.get(&PedroLibC::PutChar).unwrap(),
                            &[cl_val],
                        );
                    },
                    RageExpr::Identifier(_id) => {
                        // let cl_var = self.rage_identifier_table.get(&id).unwrap();
                        // let cl_val = self.cl_fn_builder.use_var(cl_var);
                        // self.cl_fn_builder.ins().call(
                        //     *self.libc_decl.get(&PedroLibC::PutChar).unwrap(),
                        //     &[cl_val],
                        // );
                        todo!()
                    },
                }
            }
            "libc_getchar" => {
                todo!()
            },
            _ => unimplemented!()
        }
    }

    fn parse_literal(&mut self, pair: Pair<Rule>) -> i64 {
        let inner_pair = pair.into_inner().next().unwrap();

        let val_str = inner_pair.as_str();
    
        //println!("{}", val_str);
    
        match inner_pair.as_rule() {
            Rule::number => val_str.parse::<i64>().unwrap(),
            Rule::char => val_str.parse::<char>().unwrap() as i64,
            Rule::string => todo!(),
            Rule::boolean => val_str.parse::<bool>().unwrap() as i64,
            Rule::null => todo!(),
            _ => unimplemented!()
        }
    }

    fn parse_literal_new(&mut self, pair: Pair<Rule>) -> Option<Value> {
        let inner_pair = pair.into_inner().next().unwrap();

        let val_str = inner_pair.as_str();
    
        //println!("{}", val_str);

        match inner_pair.as_rule() {
            Rule::number => Some(self.cl_fn_builder.ins().iconst(ir::types::I64, val_str.parse::<i64>().unwrap())),
            Rule::char => Some(self.cl_fn_builder.ins().iconst(ir::types::I8, val_str.parse::<char>().unwrap() as i64)),
            Rule::string => todo!(),
            Rule::boolean => Some(self.cl_fn_builder.ins().iconst(ir::types::I8, val_str.parse::<bool>().unwrap() as i64)),
            Rule::null => todo!(),
            _ => unreachable!()
        }
    }

    fn parse_typing(&mut self, pair: Pair<Rule>) -> Option<RageType> {
        let primitive_type = pair.into_inner().next().unwrap();

        RageType::from_str(primitive_type.as_str())
    }

    fn parse_expr(&mut self, pair: Pair<Rule>) -> Option<RageExpr> {
        let expr = pair.into_inner().next().unwrap();

        match expr.as_rule() {
            Rule::literal => Some(RageExpr::Literal(self.parse_literal_new(expr).unwrap())),
            Rule::identifier => {
                if self.rage_identifier_table.contains_key(expr.as_str()) {
                    Some(self.rage_identifier_table.get(expr.as_str()).unwrap().clone())
                } else {
                    Some(RageExpr::Identifier(expr.as_str().to_string()))
                }
            },
            Rule::fn_call => todo!(),
            _ => unreachable!()
        }
    }

}

#[derive(Debug)]
pub enum RageType {
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Boolean,
    Char,
    String,
    Pointer,
    Null,
}

impl RageType {
    pub fn from_str(input: &str) -> Option<RageType> {
        match input {
            "i8" => Some(RageType::I8),
            "i16" => Some(RageType::I16),
            "i32" => Some(RageType::I32),
            "i64" => Some(RageType::I64),
            "f32" => Some(RageType::F32),
            "f64" => Some(RageType::F64),
            "bool" => Some(RageType::Boolean),
            "char" => Some(RageType::Char),
            "str" => Some(RageType::String),
            "ptr" => Some(RageType::Pointer),
            "null" => Some(RageType::Null),
            _ => None,
        }
    }

    pub fn to_cl_type(&self) -> ir::types::Type {
        match self {
            RageType::I8 => ir::types::I8,
            RageType::I16 => ir::types::I16,
            RageType::I32 => ir::types::I32,
            RageType::I64 => ir::types::I64,
            RageType::F32 => ir::types::F32,
            RageType::F64 => ir::types::F64,
            RageType::Boolean => ir::types::I8,
            RageType::Char => ir::types::I8,
            RageType::String => todo!(),
            RageType::Pointer => ir::types::I64,
            RageType::Null => ir::types::I64,
        }
    }
}

#[derive(Debug, Clone)]
enum RageExpr {
    Literal(Value),
    Identifier(String)
}

#[test]
fn test_parse_literal() {

    let input = "null";

    let literal_pair = RagePestParser::parse(Rule::literal, input)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let pair = literal_pair.into_inner().next().unwrap();

    let val_str = pair.as_str();

    println!("{}", val_str);

    match pair.as_rule() {
        Rule::number => {
            let val_str = pair.as_str();

            println!("{}", val_str);
        },
        Rule::char => todo!(),
        Rule::string => todo!(),
        Rule::boolean => todo!(),
        Rule::null => todo!(),
        _ => unimplemented!()
    }
}