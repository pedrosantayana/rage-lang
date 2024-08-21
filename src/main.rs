use cranelift::{codegen::ir::{self, Function}, prelude::{isa::CallConv, AbiParam, FunctionBuilder, FunctionBuilderContext, Signature}};
use pest_derive::Parser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

fn main() {
    println!("Hello, world!");


    let mut sig = Signature::new(CallConv::SystemV);

    sig.returns.push(AbiParam::new(ir::types::I32));
    sig.params.push(AbiParam::new(ir::types::I32));


    let mut fn_builder_ctx = FunctionBuilderContext::new();

    let mut func = Function::with_name_signature(ir::UserFuncName::user(0, 0), sig);

    let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

    let parser = MyParser::parse(
        Rule::program, 
        r"
        var num = 10
        num = 12
        print(num)
        "
    ).unwrap();

    for pair in parser {
        println!("Rule:    {:?}", pair.as_rule());
        println!("Span:    {:?}", pair.as_span());
        println!("Text:    {}", pair.as_str());

        match pair.as_rule() {
            Rule::var_declaration => todo!(),
            Rule::fn_call => todo!(),
            Rule::var_assignment => todo!(),
            Rule::EOI => todo!(),
            Rule::program => todo!(),
            Rule::statement => todo!(),
            Rule::identifier => todo!(),
            Rule::number => todo!(),
            Rule::literal => todo!(),
            Rule::expression => todo!(),
            Rule::WHITESPACE => todo!(),
        }
    }
}
