use std::{any::Any, borrow::BorrowMut, cmp::min, collections::HashMap, fs::File, io::Write};

use cranelift::{
    codegen::{
        ir::{self, Function, UserFuncName},
        Context,
    },
    prelude::{
        isa::{self, x64::settings::builder},
        settings::{self, Builder},
        types, AbiParam, Configurable, EntityRef, FunctionBuilder, FunctionBuilderContext,
        InstBuilder, Signature, Variable,
    },
};

use cranelift_module::{Linkage, Module};
use cranelift_object::object::pe;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct PedroPestParser;

fn main_test() {
    // println!("Hello, world!");

    // println!("{:?}", LibCall::all_libcalls());

    // let mut sig = Signature::new(CallConv::SystemV);

    // sig.returns.push(AbiParam::new(ir::types::I32));
    // sig.params.push(AbiParam::new(ir::types::I32));

    // let mut fn_builder_ctx = FunctionBuilderContext::new();

    // let mut main_func = Function::with_name_signature(ir::UserFuncName::user(0, 0), sig);

    // let mut builder = FunctionBuilder::new(&mut main_func, &mut fn_builder_ctx);

    // let main_block = builder.create_block();

    // builder.append_block_params_for_function_params(main_block);

    // builder.switch_to_block(main_block);

    // builder.seal_block(main_block);

    // let r = builder.ins().iconst(ir::types::I32, 1);
    // builder.ins().return_(&[r]);
    // builder.finalize();

    // let flags = settings::Flags::new(settings::builder());
    // let res = verify_function(&main_func, &flags);
    // println!("{}", main_func.display());
    // if let Err(errors) = res {
    //     panic!("{}", errors);
    // }

    // let mut _jit = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

    //let mut module = cranelift_jit::JITModule::new(jit);

    // let mut output_vec = Vec::new();

    // let builder = settings::builder();
    // let flags = settings::Flags::new(builder);

    // let isa = isa::lookup(target_lexicon::triple!("x86_64-linux-elf"))
    //     .unwrap()
    //     .finish(flags)
    //     .unwrap();

    // let mut cp = ControlPlane::default();

    // let mut ctx = Context::for_function(main_func);
    // ctx.set_disasm(true);
    // let code = ctx
    //     .compile_and_emit(&*isa, &mut output_vec, &mut cp)
    //     .unwrap();
    // println!("[DASM]:\n{}", code.vcode.clone().unwrap());

    // match ctx.verify(&*isa) {
    //     Ok(_) => println!("verified"),
    //     Err(errors) => {
    //         println!("{}", errors);
    //     }
    // }

    // fs::write("a.out", output_vec).unwrap();

    // COMPILATION

    let mut shared_builder = settings::builder();
    shared_builder.enable("is_pic").unwrap();
    let shared_flags = settings::Flags::new(shared_builder);

    let isa_builder = isa::lookup(target_lexicon::triple!("x86_64-linux-gnu")).unwrap();
    let isa = isa_builder.finish(shared_flags).unwrap();
    let call_conv = isa.default_call_conv();

    let obj_builder = cranelift_object::ObjectBuilder::new(
        isa,
        "main",
        cranelift_module::default_libcall_names(),
    )
    .unwrap();
    let mut obj_module = cranelift_object::ObjectModule::new(obj_builder);

    let mut sig = Signature::new(call_conv);
    sig.returns.push(AbiParam::new(ir::types::I32));
    let fid = obj_module
        .declare_function("main", Linkage::Export, &sig)
        .unwrap();

    let mut main_func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut fn_builder = FunctionBuilder::new(&mut main_func, &mut func_ctx);

    // Putchar Definition
    let mut putchar_sig = obj_module.make_signature();
    putchar_sig.params.push(AbiParam::new(types::I8));
    putchar_sig.returns.push(AbiParam::new(types::I32));
    let putchar_fn = obj_module
        .declare_function("putchar", Linkage::Import, &putchar_sig)
        .unwrap();
    let local_putchar = obj_module.declare_func_in_func(putchar_fn, fn_builder.func);

    let block = fn_builder.create_block();
    fn_builder.switch_to_block(block);
    fn_builder.seal_block(block);

    // PARSER

    let mut variables = HashMap::new();

    let parser = PedroPestParser::parse(
        Rule::program,
        r"
        var num : i8 ;

        num = 80 ;
        libc_putchar(num) ;

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

                let identifier = pairs.next().unwrap().as_str();
                let pedro_type = pairs.next().unwrap().as_str();

                let pedro_type_parsed;

                match pedro_type {
                    "i8" => {
                        pedro_type_parsed = types::I8;
                    }
                    "i32" | _ => {
                        pedro_type_parsed = types::I32;
                    }
                }

                let index = variables.len();
                let var = Variable::new(index);

                fn_builder.declare_var(var, pedro_type_parsed);

                let pedro_var = PedroVariable {
                    cl_variable: var,
                    cl_type: pedro_type_parsed,
                    cl_index: index,
                    identifier: identifier.to_owned(),
                };

                println!("{:?}", pedro_var);

                variables.insert(identifier, pedro_var);

                //builder.def_var(var, tmp);
            }
            Rule::var_assignment => {
                let mut pairs = pair.into_inner();

                let identifier = pairs.next().unwrap().as_str();
                let value_str = pairs.next().unwrap().into_inner().next().unwrap().as_str();

                println!("{} {}", identifier, value_str);

                let pedro_var = variables.get(identifier).unwrap();

                let parsed_value;

                match pedro_var.cl_type {
                    types::I8 => parsed_value = value_str.parse::<i64>().unwrap(),
                    types::I32 | _ => parsed_value = value_str.parse::<i64>().unwrap(),
                }

                let cl_val = fn_builder.ins().iconst(pedro_var.cl_type, parsed_value);

                fn_builder.def_var(pedro_var.cl_variable, cl_val);
            }
            Rule::fn_call => {
                let mut pairs = pair.into_inner();

                let fn_name = pairs.next().unwrap().as_str();
                //let expr_str = pairs.next().unwrap().into_inner().next().unwrap().as_str();

                println!("{}", fn_name);

                if fn_name.eq("libc_putchar") {
                    for p in pairs.next().unwrap().into_inner() {
                        match p.as_rule() {
                            Rule::number => {}
                            Rule::identifier => {
                                let pedro_var = variables.get(p.as_str()).unwrap();

                                let cl_val = fn_builder.use_var(pedro_var.cl_variable);
                                fn_builder.ins().call(local_putchar, &[cl_val]);
                            }
                            _ => {}
                        }
                    }
                }
            }
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

    // ------------------------------

    //let _ = fn_builder.use_var(offset);
    let val = fn_builder.ins().iconst(ir::types::I32, 0);
    fn_builder.ins().return_(&[val]);
    fn_builder.seal_all_blocks();
    fn_builder.finalize();

    // print ir
    println!("{}", main_func.display());
    let mut ir_file = File::create("a.clif").unwrap();
    ir_file
        .write_all(format!("{}", main_func.display()).as_bytes())
        .unwrap();

    let mut context = Context::for_function(main_func);
    obj_module.define_function(fid, &mut context).unwrap();
    let res = obj_module.finish();

    let mut file = File::create("OUTPUT.o").unwrap();
    res.object.write_stream(&mut file).unwrap();

    // let mut obj = faerie::ArtifactBuilder::new(old_target_lexicon::triple!("x86_64-linux-elf"))
    //     .name("out.o".to_owned())
    //     .finish();

    // let mut f = File::create("out.o").unwrap();

    // obj.declare("main", faerie::SectionDecl::new(faerie::SectionKind::Text))
    //     .unwrap();

    // obj.define("main", output_vec).unwrap();

    // obj.write(f).unwrap();
}

fn main() {
    compiler::compile(r"
        var asd: i8;
        asd = 80;
        libc_putchar(asd);
    ");
}

#[derive(Debug)]
struct PedroVariable {
    pub cl_variable: Variable,
    pub cl_type: types::Type,
    pub cl_index: usize,
    pub identifier: String,
}

struct PedroCompiler<'a> {
    source_code: String,
    box_cl_fn_builder: Box<FunctionBuilder<'a>>,
    cl_utils: CLUtils,
}

//#[derive( Clone, Copy)]
struct CLUtils {
    box_cl_function: Box<Function>,
    box_cl_fn_ctx_builder: Box<FunctionBuilderContext>,
    //cl_fn_builder: FunctionBuilder<'a>
}

// impl PedroCompiler<'_> {
//     pub fn init() -> CLUtils {

//     let mut shared_builder = settings::builder();
//     shared_builder.enable("is_pic").unwrap();
//     let shared_flags = settings::Flags::new(shared_builder);

//     let isa_builder = isa::lookup(target_lexicon::triple!("x86_64-linux-gnu")).unwrap();
//     let isa = isa_builder.finish(shared_flags).unwrap();
//     let call_conv = isa.default_call_conv();

//     let obj_builder = cranelift_object::ObjectBuilder::new(
//         isa,
//         "main",
//         cranelift_module::default_libcall_names(),
//     )
//     .unwrap();

//     let mut obj_module = cranelift_object::ObjectModule::new(obj_builder);

//     let mut sig = Signature::new(call_conv);
//     sig.returns.push(AbiParam::new(ir::types::I32));

//     let fid = obj_module
//         .declare_function("main", Linkage::Export, &sig)
//         .unwrap();

//     // let mut cl_function =   Function::with_name_signature(UserFuncName::user(0, 0), sig);
//     // let mut cl_fn_ctx_builder =   FunctionBuilderContext::new();

//     let box_cl_function = Box::new(Function::with_name_signature(UserFuncName::user(0, 0), sig));
//     let mut box_cl_fn_ctx_builder = Box::new(FunctionBuilderContext::new());

//     CLUtils {
//         box_cl_function,
//         box_cl_fn_ctx_builder,
//     }

//     // Putchar Definition
//     // let mut putchar_sig = obj_module.make_signature();
//     // putchar_sig.params.push(AbiParam::new(types::I8));
//     // putchar_sig.returns.push(AbiParam::new(types::I32));
//     // let putchar_fn = obj_module
//     //     .declare_function("putchar", Linkage::Import, &putchar_sig)
//     //     .unwrap();
//     // let local_putchar = obj_module.declare_func_in_func(putchar_fn, fn_builder.func);

//     // let block = pedro_compiler.cl_fn_builder.create_block();
//     // fn_builder.switch_to_block(block);
//     // fn_builder.seal_block(block);

//     }

//     pub fn test<'a>() -> FunctionBuilder<'a> {
//         let mut shared_builder = settings::builder();
//         shared_builder.enable("is_pic").unwrap();
//         let shared_flags = settings::Flags::new(shared_builder);

//         let isa_builder = isa::lookup(target_lexicon::triple!("x86_64-linux-gnu")).unwrap();
//         let isa = isa_builder.finish(shared_flags).unwrap();
//         let call_conv = isa.default_call_conv();

//         let mut sig = Signature::new(call_conv);
//         sig.returns.push(AbiParam::new(ir::types::I32));

//         let mut cl_function = Function::with_name_signature(UserFuncName::user(0, 0), sig);
//         let mut cl_fn_ctx_builder = FunctionBuilderContext::new();

//         FunctionBuilder::new(&mut cl_function, &mut cl_fn_ctx_builder)
//     }

// }

mod compiler;
mod parser;