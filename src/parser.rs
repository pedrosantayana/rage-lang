use std::collections::HashMap;

use cranelift::{
    codegen::ir::{self, FuncRef},
    prelude::{FunctionBuilder, InstBuilder, Variable},
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::compiler::PedroLibC;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PedroPestParser;

#[derive(Debug)]
struct PedroVariable {
    pub cl_variable: Variable,
    pub cl_type: ir::types::Type,
}

pub struct PedroParser<'a> {
    cl_fn_builder: FunctionBuilder<'a>,
    variables: HashMap<String, PedroVariable>,
    pedro_libc: HashMap<PedroLibC, FuncRef>,
}

impl PedroParser<'_> {
    pub fn parse<'a>(
        input: &'a str,
        cl_fn_builder: FunctionBuilder<'a>,
        pedro_libc: HashMap<PedroLibC, FuncRef>,
    ) -> FunctionBuilder<'a> {
        let pairs = PedroPestParser::parse(Rule::program, input).unwrap();

        let mut pedro_parser = PedroParser {
            cl_fn_builder,
            variables: HashMap::new(),
            pedro_libc,
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::var_declaration => pedro_parser.parse_var_decl(pair),
                Rule::var_assignment => pedro_parser.parse_var_assign(pair),
                Rule::fn_call => pedro_parser.parse_fn_call(pair),
                Rule::EOI => {}
                Rule::program => {}
                Rule::statement => {}
                Rule::identifier => {}
                Rule::number => {}
                Rule::literal => {}
                Rule::expression => {}
                Rule::WHITESPACE => {}
                Rule::primitive_types => {}
            }
        }

        pedro_parser.cl_fn_builder
    }

    fn parse_var_decl(&mut self, pair: Pair<Rule>) {
        let mut pairs = pair.into_inner();

        let identifier = pairs.next().unwrap().as_str();
        let pedro_type_str = pairs.next().unwrap().as_str();

        let pedro_type = PedroTypes::from_str(pedro_type_str).unwrap();

        let index = self.variables.len();
        let var = Variable::from_u32(index.try_into().unwrap());

        self.cl_fn_builder.declare_var(var, pedro_type.to_cl_type());

        let pedro_var = PedroVariable {
            cl_variable: var,
            cl_type: pedro_type.to_cl_type(),
        };

        self.variables.insert(identifier.to_string(), pedro_var);
    }

    fn parse_var_assign(&mut self, pair: Pair<Rule>) {
        let mut pairs = pair.into_inner();

        let identifier = pairs.next().unwrap().as_str().trim();
        let value_str = pairs.next().unwrap().into_inner().next().unwrap().as_str();

        let pedro_var = self.variables.get(identifier).unwrap();

        let parsed_value;

        match pedro_var.cl_type {
            ir::types::I8 => parsed_value = value_str.parse::<i64>().unwrap(),
            ir::types::I32 | _ => parsed_value = value_str.parse::<i64>().unwrap(),
        }

        let cl_val = self
            .cl_fn_builder
            .ins()
            .iconst(pedro_var.cl_type, parsed_value);

        self.cl_fn_builder.def_var(pedro_var.cl_variable, cl_val);
    }

    fn parse_fn_call(&mut self, pair: Pair<Rule>) {
        let mut pairs = pair.into_inner();

        let fn_name = pairs.next().unwrap().as_str();

        match fn_name {
            "libc_putchar" => {
                for p in pairs.next().unwrap().into_inner() {
                    match p.as_rule() {
                        Rule::number => {}
                        Rule::identifier => {
                            let pedro_var = self.variables.get(p.as_str()).unwrap();

                            let cl_val = self.cl_fn_builder.use_var(pedro_var.cl_variable);
                            self.cl_fn_builder.ins().call(
                                *self.pedro_libc.get(&PedroLibC::PutChar).unwrap(),
                                &[cl_val],
                            );
                        }
                        _ => {}
                    }
                }
            }
            "libc_getchar" => {
                todo!()
            },
            _ => unimplemented!()
        }
    }
}

pub enum PedroTypes {
    I8,
    I16,
    I32,
    I64,
    // U8,
    // U16,
    // U32,
    // U64,
    F32,
    F64,
    // Char,
    Boolean,
}

impl PedroTypes {
    pub fn from_str(input: &str) -> Option<PedroTypes> {
        match input {
            "i8" => Some(PedroTypes::I8),
            "i16" => Some(PedroTypes::I16),
            "i32" => Some(PedroTypes::I32),
            "i64" => Some(PedroTypes::I64),
            "f32" => Some(PedroTypes::F32),
            "f64" => Some(PedroTypes::F64),
            "bool" => Some(PedroTypes::Boolean),
            _ => None,
        }
    }

    pub fn to_cl_type(&self) -> ir::types::Type {
        match self {
            PedroTypes::I8 => ir::types::I8,
            PedroTypes::I16 => ir::types::I16,
            PedroTypes::I32 => ir::types::I32,
            PedroTypes::I64 => ir::types::I64,
            PedroTypes::F32 => ir::types::F32,
            PedroTypes::F64 => ir::types::F64,
            PedroTypes::Boolean => ir::types::I8,
        }
    }
}
