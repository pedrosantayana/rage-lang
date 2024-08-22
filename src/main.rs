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

    let mut func = Function::with_name_signature(ir::UserFuncName::user(0, 0), sig);

    let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);

    let block0 = builder.create_block();

    

    let parser = MyParser::parse(
        Rule::program,
        r"
        var num = 10
        num = 12
        print(num)
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
                let value = pairs.next().unwrap().into_inner().next().unwrap();
                println!("{} {}", identifier.as_str(), value.as_str());

                let val_u32 = value.as_str().parse::<i64>().unwrap();

                let tmp = builder.ins().iconst(ir::types::I32, val_u32);

                let var = Variable::new(0);
                builder.declare_var(var, ir::types::I32);
                builder.def_var(var, tmp);
            }
            Rule::fn_call => {}
            Rule::var_assignment => {}
            Rule::EOI => {}
            Rule::program => {}
            Rule::statement => {}
            Rule::identifier => {}
            Rule::number => {}
            Rule::literal => {}
            Rule::expression => {}
            Rule::WHITESPACE => {}
        }
    }
}
