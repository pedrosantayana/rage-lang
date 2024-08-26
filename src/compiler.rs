use std::{collections::HashMap, fs::OpenOptions, path::PathBuf};

use cranelift::{
    codegen::{ir::{self, FuncRef, Function, UserFuncName}, Context},
    prelude::{
        isa, settings, AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, Signature
    },
};
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectModule;

use crate::parser;

pub fn compile(input: &str, output: &PathBuf) {
    let mut shared_builder = settings::builder();
    shared_builder.enable("is_pic").unwrap();
    let shared_flags = settings::Flags::new(shared_builder);

    // ISA and Call Convetion
    let isa_builder = isa::lookup(target_lexicon::triple!("x86_64-linux-gnu")).unwrap();
    let isa = isa_builder.finish(shared_flags).unwrap();
    let call_conv = isa.default_call_conv();

    // Signature
    let mut sig = Signature::new(call_conv);
    sig.returns.push(AbiParam::new(ir::types::I32));

    // FuncId declaration
    let obj_builder = cranelift_object::ObjectBuilder::new(
        isa,
        "main",
        cranelift_module::default_libcall_names(),
    )
    .unwrap();
    let mut obj_module = cranelift_object::ObjectModule::new(obj_builder);

    let fid = obj_module
        .declare_function("main", Linkage::Export, &sig)
        .unwrap();

    // FunctionBuilder declaration (triplice aliança do capeta das mut ref, NUNCA refatorar!) TODO: Estudar mais sobre lifetimes
    let mut cl_fn = Function::with_name_signature(UserFuncName::user(0, 0), sig);
    let mut cl_fn_builder_ctx = FunctionBuilderContext::new();
    let cl_fn_builder = FunctionBuilder::new(&mut cl_fn, &mut cl_fn_builder_ctx);

    // Libc definitions
    //let mut libc_decl;
    let (mut obj_module, mut cl_fn_builder, libc_decl) = gen_libc(obj_module, cl_fn_builder);

   


    // Main block declaration
    let block = cl_fn_builder.create_block();
    cl_fn_builder.switch_to_block(block);
    cl_fn_builder.seal_block(block);

    // Parser
    cl_fn_builder = parser::RageParser::parse(input, cl_fn_builder, libc_decl);

    // Return 0 and seal all blocks
    let val = cl_fn_builder.ins().iconst(ir::types::I32, 0);
    cl_fn_builder.ins().return_(&[val]);
    cl_fn_builder.seal_all_blocks();
    cl_fn_builder.finalize();

    // Object generation, using cranelift-object crate
    let mut context = Context::for_function(cl_fn);
    obj_module.define_function(fid, &mut context).unwrap();
    let res = obj_module.finish();

    // Write object bytes to file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output).unwrap();
    res.object.write_stream(&mut file).unwrap();
}

fn gen_libc<'a>(obj_module: ObjectModule, cl_fn_builder: FunctionBuilder<'a>) -> (ObjectModule, FunctionBuilder<'a>, HashMap<PedroLibC, FuncRef>) {
    let mut libc_decl: HashMap<PedroLibC, FuncRef> = HashMap::new();

    let (obj_module, cl_fn_builder,  f) = libc_put_char_decl(obj_module, cl_fn_builder);

    libc_decl.insert(PedroLibC::PutChar, f);

    let (obj_module, cl_fn_builder,  f) = libc_get_char_decl(obj_module, cl_fn_builder);

    libc_decl.insert(PedroLibC::GetChar, f);

    (obj_module, cl_fn_builder, libc_decl)
}

fn libc_put_char_decl<'a>(mut obj_module: ObjectModule, cl_fn_builder: FunctionBuilder<'a>) -> (ObjectModule, FunctionBuilder<'a>, FuncRef) {
    let mut putchar_sig = obj_module.make_signature();
    putchar_sig.params.push(AbiParam::new(ir::types::I8));
    putchar_sig.returns.push(AbiParam::new(ir::types::I32));
    let putchar_fn = obj_module
        .declare_function("putchar", Linkage::Import, &putchar_sig)
        .unwrap();
    let local_putchar = obj_module.declare_func_in_func(putchar_fn, cl_fn_builder.func);

    (obj_module, cl_fn_builder, local_putchar)
}

fn libc_get_char_decl<'a>(mut obj_module: ObjectModule, cl_fn_builder: FunctionBuilder<'a>) -> (ObjectModule, FunctionBuilder<'a>, FuncRef) {
    let mut putchar_sig = obj_module.make_signature();
    putchar_sig.params.push(AbiParam::new(ir::types::I8));
    putchar_sig.returns.push(AbiParam::new(ir::types::I32));
    let putchar_fn = obj_module
        .declare_function("getchar", Linkage::Import, &putchar_sig)
        .unwrap();
    let local_putchar = obj_module.declare_func_in_func(putchar_fn, cl_fn_builder.func);

    (obj_module, cl_fn_builder, local_putchar)
}

#[derive(Eq, Hash, PartialEq)]
pub enum PedroLibC {
    PutChar,
    GetChar
}