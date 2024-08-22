use std::{collections::HashMap, iter::Map};

use cranelift::{
    codegen::ir::{self, Function},
    prelude::{
        isa::CallConv, AbiParam, EntityRef, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature, Value, Variable
    },
};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

fn main() {
    println!("Hello, world!");

    let mut sig = Signature::new(CallConv::SystemV);

    sig.returns.push(AbiParam::new(ir::types::I32));
    sig.params.push(AbiParam::new(ir::types::I32));

    let mut fn_builder_ctx = FunctionBuilderContext::new();

    let mut main_func = Function::with_name_signature(ir::UserFuncName::user(0, 0), sig);

    let mut builder = FunctionBuilder::new(&mut main_func, &mut fn_builder_ctx);

    let main_block = builder.create_block();

    let mut variables = HashMap::new();

    let parser = MyParser::parse(
        Rule::program,
        r"
        var num;
        num = 12;
        print(num);
        ",
    )
    .unwrap();

    for pair in parser {
        // println!("Rule:    {:?}", pair.as_rule());
        // println!("Span:    {:?}", pair.as_span());
        // println!("Text:    {}", pair.as_str());

        match pair.as_rule() {
            Rule::var_declaration => {
                let mut pairs = pair.into_inner();

                let identifier = pairs.next().unwrap();
                //let value = pairs.next().unwrap().into_inner().next().unwrap();
                //println!("{} {}", identifier.as_str(), value.as_str());

                //let val_u32 = value.as_str().parse::<i64>().unwrap();



                let var = Variable::new(variables.len() + 1);
                builder.declare_var(var, ir::types::I32);

                variables.insert(identifier.as_str(), var);

                //builder.def_var(var, tmp);
            },
            Rule::fn_call => {},
            Rule::var_assignment => {
                let mut pairs = pair.into_inner();

                let identifier = pairs.next().unwrap();
                let value = pairs.next().unwrap().into_inner().next().unwrap();

                println!("{} {}", identifier.as_str(), value.as_str());

                let parsed_value = value.as_str().parse::<i32>().unwrap();

                //let cl_val = builder.ins().iconst(ir::types::I32, parsed_value);

                //builder.def_var(, val)
            },
            Rule::EOI => {},
            Rule::program => {},
            Rule::statement => {},
            Rule::identifier => {},
            Rule::number => {}
            Rule::literal => {}
            Rule::expression => {}
            Rule::WHITESPACE => {}
        }
    }
}
