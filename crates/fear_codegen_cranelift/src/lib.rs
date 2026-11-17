use std::sync::Arc;

use cranelift::codegen::ir::{BlockArg, Function, LibCall, UserFuncName};
use cranelift::codegen::isa::{CallConv, TargetIsa};
use cranelift_codegen::ir::InstBuilder;
use cranelift_module::{Linkage as CLinkage};
use cranelift_object::{ObjectBuilder, ObjectModule};
use cranelift_jit::{JITBuilder, JITModule};
use target_lexicon::Triple;

use fear::prelude::*;
use std::collections::HashMap;

fn select_int(width: u8) -> cranelift::prelude::types::Type {
    match width {
        8 => cranelift::prelude::types::I8,
        16 => cranelift::prelude::types::I16,
        32 => cranelift::prelude::types::I32,
        64 => cranelift::prelude::types::I64,
        128 => cranelift::prelude::types::I128,
        _ => panic!("unsupported integer"),
    }
}

fn map_type(triple: &Triple, ty: Type) -> cranelift::prelude::types::Type {
    match ty {
        // cranelift has no int1 type
        Type::Int1 => cranelift::prelude::types::I8,
        Type::Int8 => cranelift::prelude::types::I8,
        Type::Int16 => cranelift::prelude::types::I16,
        Type::Int32 => cranelift::prelude::types::I32,
        Type::Int64 => cranelift::prelude::types::I64,
        Type::Float32 => cranelift::prelude::types::F32,
        Type::Float64 => cranelift::prelude::types::F64,
        //
        Type::Pointer => select_int(ty.get_bitwidth(triple) as u8),
        Type::Void => panic!("cranelift has no void type"),
    }
}

fn map_linkage(func: &fear::Function) -> CLinkage {
    match (&func.linkage, func.get_definition().is_some()) {
        (Linkage::External, true) => CLinkage::Export,
        (Linkage::External, false) => CLinkage::Import,

        (Linkage::Internal, true) => CLinkage::Local,
        (Linkage::Internal, false) => {
            log::warn!("undefined internal function");
            CLinkage::Import
        }

        (Linkage::Weak, true) => CLinkage::Preemptible,
        (Linkage::Weak, false) => CLinkage::Import,
    }
}

fn map_int_cond(kind: IntCmp) -> cranelift::prelude::IntCC {
    match kind {
        IntCmp::Eq => cranelift::prelude::IntCC::Equal,
        IntCmp::Ne => cranelift::prelude::IntCC::NotEqual,
        IntCmp::Lt => cranelift::prelude::IntCC::SignedLessThan,
        IntCmp::Le => cranelift::prelude::IntCC::SignedLessThanOrEqual,
        IntCmp::Gt => cranelift::prelude::IntCC::SignedGreaterThan,
        IntCmp::Ge => cranelift::prelude::IntCC::SignedGreaterThanOrEqual,
        IntCmp::ULt => cranelift::prelude::IntCC::UnsignedLessThan,
        IntCmp::ULe => cranelift::prelude::IntCC::UnsignedLessThanOrEqual,
        IntCmp::UGt => cranelift::prelude::IntCC::UnsignedGreaterThan,
        IntCmp::UGe => cranelift::prelude::IntCC::UnsignedGreaterThanOrEqual,
    }
}

fn map_float_cond(kind: FloatCmp) -> cranelift::prelude::FloatCC {
    match kind {
        FloatCmp::OEq => cranelift::prelude::FloatCC::Equal,
        FloatCmp::ONe => cranelift::prelude::FloatCC::NotEqual,
        FloatCmp::OLt => cranelift::prelude::FloatCC::LessThan,
        FloatCmp::OLe => cranelift::prelude::FloatCC::LessThanOrEqual,
        FloatCmp::OGt => cranelift::prelude::FloatCC::GreaterThan,
        FloatCmp::OGe => cranelift::prelude::FloatCC::GreaterThanOrEqual,

        FloatCmp::UEq => cranelift::prelude::FloatCC::UnorderedOrEqual,
        // UNe: any unordered comparison is "not equal", use UnorderedOrLessThan | UnorderedOrGreaterThan
        // Cranelift does not have a single UnorderedOrNotEqual; use Ordered negation via Unordered.
        // The closest correct mapping: if either is NaN, they're "not equal".
        FloatCmp::UNe => cranelift::prelude::FloatCC::UnorderedOrLessThan,
        FloatCmp::ULt => cranelift::prelude::FloatCC::UnorderedOrLessThan,
        FloatCmp::ULe => cranelift::prelude::FloatCC::UnorderedOrLessThanOrEqual,
        FloatCmp::UGt => cranelift::prelude::FloatCC::UnorderedOrGreaterThan,
        FloatCmp::UGe => cranelift::prelude::FloatCC::UnorderedOrGreaterThanOrEqual,

        FloatCmp::Ord => cranelift::prelude::FloatCC::Ordered,
        FloatCmp::Uno => cranelift::prelude::FloatCC::Unordered,
    }
}

pub struct CraneliftCodegen<M: cranelift_module::Module> {
    pub triple: Triple,
    pub module: M,

    functions: HashMap<FuncId, cranelift_module::FuncId>,
}

impl CraneliftCodegen<ObjectModule> {
    pub fn new(
        name: &str,
        triple: Triple,
        isa: Arc<dyn TargetIsa>,
        libcall_names: Box<dyn Fn(LibCall) -> String + Send + Sync>,
    ) -> Self {
        let builder = ObjectBuilder::new(isa, name, libcall_names).unwrap();
        let module = ObjectModule::new(builder);
        Self {
            triple,
            module,
            functions: HashMap::new(),
        }
    }

    pub fn emit(self) -> Vec<u8> {
        self.module.finish().emit().unwrap()
    }
}

impl CraneliftCodegen<JITModule> {
    pub fn new(
        triple: Triple,
        isa: Arc<dyn TargetIsa>,
        libcall_names: Box<dyn Fn(LibCall) -> String + Send + Sync>,
    ) -> Self {
        let builder = JITBuilder::with_isa(isa, libcall_names);
        let module = JITModule::new(builder);
        Self {
            triple,
            module,
            functions: HashMap::new(),
        }
    }

    pub fn finalize(&mut self) {
        self.module.finalize_definitions().unwrap();
    }

    pub fn get_function_ptr(&self, fid: FuncId) -> *const u8 {
        let cl_fid = self.func_id(fid).unwrap();
        self.module.get_finalized_function(cl_fid)
    }
}

impl<M: cranelift_module::Module> CraneliftCodegen<M> {
    pub fn get_module(&self) -> &M {
        &self.module
    }

    pub fn get_module_mut(&mut self) -> &mut M {
        &mut self.module
    }

    pub fn into_module(self) -> M {
        self.module
    }

    pub fn func_id(&self, fid: FuncId) -> Option<cranelift_module::FuncId> {
        self.functions.get(&fid).copied()
    }

    fn map_callconv(
        &self,
        target_conf: &cranelift::prelude::isa::TargetFrontendConfig,
        callconv: CallingConvention,
    ) -> CallConv {
        use CallingConvention::*;

        match callconv {
            C =>
            /* C abi is a default abi on your machine */
            {
                target_conf.default_call_conv
            }
            SystemV => CallConv::SystemV,
            MicrosoftAbi => CallConv::WindowsFastcall,
        }
    }

    fn make_signature(&self, callconv: CallingConvention, sig: &FunctionSignature) -> cranelift::prelude::Signature {
        let call_conv = self.map_callconv(&self.module.target_config(), callconv);
        let mut s = cranelift::prelude::Signature::new(call_conv);
        for &p in &sig.params {
            s.params.push(cranelift::prelude::AbiParam::new(map_type(&self.triple, p)));
        }
        if sig.returns != Type::Void {
            s.returns
                .push(cranelift::prelude::AbiParam::new(map_type(&self.triple, sig.returns)));
        }
        s
    }

    pub fn lower_module(&mut self, m: &fear::Module) {
        for (fid, func) in m.iter_functions() {
            log::debug!("declaring function '{}' (fid={})", func.name, fid);
            self.declare_function(m, fid);
        }

        for (fid, func) in m.iter_functions() {
            if func.get_definition().is_some() {
                log::debug!("lowering function '{}' (fid={})", func.name, fid);
                self.lower_function(m, fid);
            }
        }
    }

    pub fn declare_function(&mut self, m: &fear::Module, fid: FuncId) {
        let func = m.get_function(fid).unwrap();
        let sig = self.make_signature(func.calling_convention, &func.signature);
        let cl_fid = self
            .module
            .declare_function(&func.name, map_linkage(func), &sig)
            .unwrap();
        self.functions.insert(fid, cl_fid);
    }

    pub fn lower_function(&mut self, m: &fear::Module, fid: FuncId) {
        let ir_func = m.get_function(fid).unwrap();
        let def = ir_func.get_definition().unwrap();
        let cl_fid = self.functions[&fid];

        let sig = self.make_signature(ir_func.calling_convention, &ir_func.signature);
        let mut cl_func =
            Function::with_name_signature(UserFuncName::user(0, cl_fid.as_u32()), sig);

        let mut func_refs: HashMap<cranelift_module::FuncId, cranelift::codegen::ir::FuncRef> =
            HashMap::new();
        for &cl_callee_fid in self.functions.values() {
            let fref = self
                .module
                .declare_func_in_func(cl_callee_fid, &mut cl_func);
            func_refs.insert(cl_callee_fid, fref);
        }

        let mut fctx = cranelift::prelude::FunctionBuilderContext::new();
        let mut fx = cranelift::prelude::FunctionBuilder::new(&mut cl_func, &mut fctx);

        let mut values: HashMap<ValueId, cranelift::prelude::Value> = HashMap::new();
        let mut blocks: HashMap<BlockId, cranelift::prelude::Block> = HashMap::new();

        let entry = def.get_entry();
        for bid in def.get_block_ids() {
            let cl_block = fx.create_block();
            blocks.insert(bid, cl_block);
        }

        {
            let entry_block = blocks[&entry];
            fx.append_block_params_for_function_params(entry_block);
            fx.switch_to_block(entry_block);
            fx.seal_block(entry_block);

            let entry_ir = def.get_block(entry).unwrap();
            for (i, &param_val) in entry_ir.get_params().iter().enumerate() {
                let cl_val = fx.block_params(entry_block)[i];
                values.insert(param_val, cl_val);
            }
        }

        for (&bid, block) in def.get_blocks() {
            if bid == entry {
                continue;
            }
            let cl_block = blocks[&bid];
            for &param in block.get_params() {
                let ty = def.get_type_of(param);
                let cl_val = fx.append_block_param(cl_block, map_type(&self.triple, ty));
                values.insert(param, cl_val);
            }
        }

        let order: Vec<BlockId> = def.compute_rpo();

        for &bid in &order {
            if bid != entry {
                fx.switch_to_block(blocks[&bid]);
            }

            let block = def.get_block(bid).unwrap();
            for &inst_id in block.get_insts() {
                let inst = def.get_inst(inst_id).unwrap();
                compile_inst(
                    def,
                    inst,
                    &mut fx,
                    &mut values,
                    &blocks,
                    &self.functions,
                    &func_refs,
                    &self.triple,
                );
            }
        }

        for bid in def.get_block_ids() {
            if bid != entry {
                fx.seal_block(blocks[&bid]);
            }
        }

        // fx.finalize();
        // self.finan

        let mut ctx = cranelift::codegen::Context::new();
        ctx.func = cl_func;
        self.module.define_function(cl_fid, &mut ctx).unwrap();
    }


}

fn get_or_const(
    def: &fear::FunctionDef,
    v: ValueId,
    values: &HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut cranelift::prelude::FunctionBuilder,
    triple: &Triple,
) -> cranelift::prelude::Value {
    if let Some(c) = def.get_int_const(v) {
        let ty = map_type(triple, def.get_type_of(v));
        fx.ins().iconst(ty, c)
    } else {
        *values
            .get(&v)
            .unwrap_or_else(|| panic!("missing value: {}", v))
    }
}

fn get_float_or_const(
    def: &FunctionDef,
    v: ValueId,
    values: &HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut cranelift::prelude::FunctionBuilder,
) -> cranelift::prelude::Value {
    if let Some(f) = def.get_float_const(v) {
        match def.get_type_of(v) {
            Type::Float32 => fx.ins().f32const(f as f32),
            Type::Float64 => fx.ins().f64const(f),
            _ => panic!("fconst on non-float type"),
        }
    } else {
        *values
            .get(&v)
            .unwrap_or_else(|| panic!("missing value: {}", v))
    }
}

fn get(
    v: ValueId,
    values: &HashMap<ValueId, cranelift::prelude::Value>,
) -> cranelift::prelude::Value {
    *values
        .get(&v)
        .unwrap_or_else(|| panic!("missing value: {}", v))
}

#[allow(clippy::too_many_arguments)]
fn compile_inst(
    def: &FunctionDef,
    inst: &Inst,
    fx: &mut cranelift::prelude::FunctionBuilder,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    blocks: &HashMap<BlockId, cranelift::prelude::Block>,
    functions: &HashMap<FuncId, cranelift_module::FuncId>,
    func_refs: &HashMap<cranelift_module::FuncId, cranelift::codegen::ir::FuncRef>,
    triple: &Triple,
) {
    log::trace!(
        "lowering inst: {:?}, operands: {:?}",
        inst.kind,
        inst.operands
    );

    match &inst.kind {
        InstKind::IConst(x) => {
            let ty = map_type(triple, def.get_type_of(inst.result.unwrap()));
            let val = fx.ins().iconst(ty, *x);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::FConst(bits) => {
            let ty = def.get_type_of(inst.result.unwrap());
            let val = match ty {
                Type::Float32 => fx.ins().f32const(f32::from_bits(*bits as u32)),
                Type::Float64 => fx.ins().f64const(f64::from_bits(*bits)),
                _ => panic!("FConst with non-float type"),
            };
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::Add => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().iadd(a, b)
        }),
        InstKind::Sub => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().isub(a, b)
        }),
        InstKind::Mul => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().imul(a, b)
        }),
        InstKind::Div { signed: true } => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().sdiv(a, b)
        }),
        InstKind::Div { signed: false } => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().udiv(a, b)
        }),
        InstKind::Rem { signed: true } => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().srem(a, b)
        }),
        InstKind::Rem { signed: false } => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().urem(a, b)
        }),
        InstKind::Neg => un_int(def, inst, values, fx, triple, |fx, a| fx.ins().ineg(a)),

        InstKind::FAdd => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fadd(a, b)),
        InstKind::FSub => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fsub(a, b)),
        InstKind::FMul => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fmul(a, b)),
        InstKind::FDiv => bin_float(def, inst, values, fx, |fx, a, b| fx.ins().fdiv(a, b)),
        InstKind::FRem => {
            panic!("frem is not a native Cranelift opcode; use an fmodf/fmod libcall")
        }
        InstKind::FNeg => un_float(def, inst, values, fx, |fx, a| fx.ins().fneg(a)),

        InstKind::Not => un_int(def, inst, values, fx, triple, |fx, v| fx.ins().bnot(v)),
        InstKind::And => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().band(a, b)
        }),
        InstKind::Or => bin_int(def, inst, values, fx, triple, |fx, a, b| fx.ins().bor(a, b)),
        InstKind::Xor => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().bxor(a, b)
        }),
        InstKind::LShl => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().ishl(a, b)
        }),
        InstKind::LShr => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().ushr(a, b)
        }),
        InstKind::AShr => bin_int(def, inst, values, fx, triple, |fx, a, b| {
            fx.ins().sshr(a, b)
        }),

        InstKind::Cmp(kind) => {
            let lhs = get_or_const(def, inst.operands[0], values, fx, triple);
            let rhs = get_or_const(def, inst.operands[1], values, fx, triple);
            let cond = map_int_cond(*kind);
            let val = fx.ins().icmp(cond, lhs, rhs);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::FCmp(kind) => {
            let lhs = get_float_or_const(def, inst.operands[0], values, fx);
            let rhs = get_float_or_const(def, inst.operands[1], values, fx);
            let cond = map_float_cond(*kind);
            let val = fx.ins().fcmp(cond, lhs, rhs);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::Alloca(ty) => {
            let ty = map_type(triple, *ty);
            let size = ty.bytes();
            let slot = fx.create_sized_stack_slot(cranelift::prelude::StackSlotData::new(
                cranelift::prelude::StackSlotKind::ExplicitSlot,
                size,
                0,
            ));
            let ptr = fx.ins().stack_addr(cranelift::prelude::types::I64, slot, 0);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::NAlloca(ty, count) => {
            let ty = map_type(triple, *ty);
            let elem_bytes = ty.bytes();
            let size = elem_bytes * (*count as u32);
            let slot_data = cranelift::prelude::StackSlotData::new(cranelift::prelude::StackSlotKind::ExplicitSlot, size, 0);
            let slot = fx.create_sized_stack_slot(slot_data);
            let ptr = fx.ins().stack_addr(cranelift::prelude::types::I64, slot, 0);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::Load { volatile } => {
            let ptr = get(inst.operands[0], values);
            let result_ty = map_type(triple, def.get_type_of(inst.result.unwrap()));
            let mut flags = cranelift::prelude::MemFlags::new();
            if !volatile {
                flags.set_notrap();
            }
            let val = fx.ins().load(result_ty, flags, ptr, 0);
            values.insert(inst.result.unwrap(), val);
        }

        InstKind::Store { volatile } => {
            let ptr = get(inst.operands[0], values);
            let value = get(inst.operands[1], values);
            let mut flags = cranelift::prelude::MemFlags::new();
            if !volatile {
                flags.set_notrap();
            }
            fx.ins().store(flags, value, ptr, 0);
        }

        InstKind::PtrOffset => {
            let base = get(inst.operands[0], values);
            let offset = get_or_const(def, inst.operands[1], values, fx, triple);
            let ptr = fx.ins().iadd(base, offset);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::ElementPtr(ty) => {
            let base = get(inst.operands[0], values);
            let base_ty = fx.func.dfg.value_type(base);

            let raw_offset = get_or_const(def, inst.operands[1], values, fx, triple);
            let offset_ty = fx.func.dfg.value_type(raw_offset);

            let offset_expanded = if offset_ty.bits() < 64 {
                fx.ins().sextend(cranelift::prelude::types::I64, raw_offset)
            } else {
                raw_offset
            };

            let type_size = fx.ins().iconst(cranelift::prelude::types::I64, ty.get_size(triple) as i64);
            let final_offset = fx.ins().imul(offset_expanded, type_size);

            let final_offset_adjusted = if base_ty != cranelift::prelude::types::I64 {
                fx.ins().ireduce(base_ty, final_offset)
            } else {
                final_offset
            };

            let ptr = fx.ins().iadd(base, final_offset_adjusted);
            values.insert(inst.result.unwrap(), ptr);
        }

        InstKind::Call(fid) => {
            let cl_fid = functions
                .get(fid)
                .unwrap_or_else(|| panic!("unknown FuncId {:?} in Call", fid));
            let func_ref = func_refs
                .get(cl_fid)
                .unwrap_or_else(|| panic!("no FuncRef for cl_fid {:?}", cl_fid));

            let args: Vec<cranelift::prelude::Value> = inst
                .operands
                .iter()
                .map(|&v| get_or_const(def, v, values, fx, triple))
                .collect();

            let call_inst = fx.ins().call(*func_ref, &args);

            if let Some(result) = inst.result {
                let results = fx.inst_results(call_inst);
                assert!(
                    !results.is_empty(),
                    "Call has result ValueId but callee returns nothing"
                );
                values.insert(result, results[0]);
            }
        }

        InstKind::Cast(kind) => {
            let src = get(inst.operands[0], values);
            let src_ty = def.get_type_of(inst.operands[0]);
            let dst_ty = def.get_type_of(inst.result.unwrap());
            let result = lower_cast(*kind, src, src_ty, dst_ty, fx, triple);
            values.insert(inst.result.unwrap(), result);
        }

        InstKind::Jump(target) => {
            let target_bb = blocks[target];

            let args: Vec<BlockArg> = inst
                .operands
                .iter()
                .map(|&op| BlockArg::Value(get_or_const(def, op, values, fx, triple)))
                .collect();

            fx.ins().jump(target_bb, &args);
        }

        InstKind::JumpIf {
            then_block,
            else_block,
        } => {
            let cond = get(inst.operands[0], values);

            let (_, then_params, else_params) = def.get_jumpif_params(inst).unwrap();

            let then_bb = blocks[then_block];
            let else_bb = blocks[else_block];

            let then_args: Vec<BlockArg> = then_params
                .iter()
                .map(|&op| BlockArg::Value(get_or_const(def, op, values, fx, triple)))
                .collect();
            let else_args: Vec<BlockArg> = else_params
                .iter()
                .map(|&op| BlockArg::Value(get_or_const(def, op, values, fx, triple)))
                .collect();

            fx.ins()
                .brif(cond, then_bb, &then_args, else_bb, &else_args);
        }

        InstKind::Ret => {
            if inst.operands.is_empty() {
                fx.ins().return_(&[]);
            } else {
                let v = get(inst.operands[0], values);
                fx.ins().return_(&[v]);
            }
        }

        InstKind::Undef => {
            let result_id = inst.result.unwrap();
            let dst_ty = def.get_type_of(result_id);
            let cl_ty = map_type(triple, dst_ty);
            let size = cl_ty.bytes();
            let slot = fx.create_sized_stack_slot(cranelift::prelude::StackSlotData::new(
                cranelift::prelude::StackSlotKind::ExplicitSlot,
                size,
                0,
            ));
            let val = fx.ins().stack_load(cl_ty, slot, 0);
            values.insert(result_id, val);
        }

        InstKind::Select => {
            let cond = get_or_const(def, inst.operands[0], values, fx, triple);
            let tvalue = get_or_const(def, inst.operands[1], values, fx, triple);
            let evalue = get_or_const(def, inst.operands[2], values, fx, triple);
            let result = fx.ins().select(cond, tvalue, evalue);
            values.insert(inst.result.unwrap(), result);
        }
    }
}

fn lower_cast(
    kind: CastKind,
    src: cranelift::prelude::Value,
    src_ty: Type,
    dst_ty: Type,
    fx: &mut cranelift::prelude::FunctionBuilder,
    triple: &Triple,
) -> cranelift::prelude::Value {
    let cl_dst = map_type(triple, dst_ty);

    match kind {
        CastKind::Zext => fx.ins().uextend(cl_dst, src),
        CastKind::Sext => fx.ins().sextend(cl_dst, src),
        CastKind::Trunc => fx.ins().ireduce(cl_dst, src),
        CastKind::Bitcast => match (src_ty, dst_ty) {
            (Type::Int32, Type::Float32)
            | (Type::Int64, Type::Float64)
            | (Type::Float32, Type::Int32)
            | (Type::Float64, Type::Int64) => fx.ins().bitcast(cl_dst, cranelift::prelude::MemFlags::new(), src),
            _ => src,
        },
        CastKind::SIToFP => fx.ins().fcvt_from_sint(cl_dst, src),
        CastKind::UIToFP => fx.ins().fcvt_from_uint(cl_dst, src),
        CastKind::FPToSI => fx.ins().fcvt_to_sint(cl_dst, src),
        CastKind::FPToUI => fx.ins().fcvt_to_uint(cl_dst, src),
        CastKind::FPromote => fx.ins().fpromote(cl_dst, src),
        CastKind::FTrunc => fx.ins().fdemote(cl_dst, src),
    }
}

fn bin_int<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut cranelift::prelude::FunctionBuilder,
    triple: &Triple,
    f: F,
) where
    F: Fn(
        &mut cranelift::prelude::FunctionBuilder,
        cranelift::prelude::Value,
        cranelift::prelude::Value,
    ) -> cranelift::prelude::Value,
{
    let a = get_or_const(def, inst.operands[0], values, fx, triple);
    let b = get_or_const(def, inst.operands[1], values, fx, triple);
    let res = f(fx, a, b);
    values.insert(inst.result.unwrap(), res);
}

fn un_int<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut cranelift::prelude::FunctionBuilder,
    triple: &Triple,

    f: F,
) where
    F: Fn(&mut cranelift::prelude::FunctionBuilder, cranelift::prelude::Value) -> cranelift::prelude::Value,
{
    let a = get_or_const(def, inst.operands[0], values, fx, triple);
    let res = f(fx, a);
    values.insert(inst.result.unwrap(), res);
}

fn bin_float<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut cranelift::prelude::FunctionBuilder,
    f: F,
) where
    F: Fn(
        &mut cranelift::prelude::FunctionBuilder,
        cranelift::prelude::Value,
        cranelift::prelude::Value,
    ) -> cranelift::prelude::Value,
{
    let a = get_float_or_const(def, inst.operands[0], values, fx);
    let b = get_float_or_const(def, inst.operands[1], values, fx);
    let res = f(fx, a, b);
    values.insert(inst.result.unwrap(), res);
}

fn un_float<F>(
    def: &FunctionDef,
    inst: &Inst,
    values: &mut HashMap<ValueId, cranelift::prelude::Value>,
    fx: &mut cranelift::prelude::FunctionBuilder,
    f: F,
) where
    F: Fn(&mut cranelift::prelude::FunctionBuilder, cranelift::prelude::Value) -> cranelift::prelude::Value,
{
    let a = get_float_or_const(def, inst.operands[0], values, fx);
    let res = f(fx, a);
    values.insert(inst.result.unwrap(), res);
}
