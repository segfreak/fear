
use crate::*;

impl FunctionDef {

    pub fn make_undef(&mut self, block: BlockId, ty: Type) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Undef,
            },
        )
    }

    pub fn make_select(
        &mut self,
        block: BlockId,
        ty: Type,
        cond: &Expr,
        then_value: &Expr,
        else_value: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Select(
                    Box::new(cond.clone()),
                    Box::new(then_value.clone()),
                    Box::new(else_value.clone()),
                ),
            },
        )
    }

    pub fn make_call(&mut self, block: BlockId, ty: Type, func: FuncId, params: Vec<Expr>) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Call(func, params),
            },
        )
    }

    pub fn make_iconst(&mut self, block: BlockId, ty: Type, value: i64) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Const(value),
            },
        )
    }

    pub fn make_fconst(&mut self, block: BlockId, ty: Type, bits: u64) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FConst(bits),
            },
        )
    }

    pub fn make_cast(&mut self, block: BlockId, ty: Type, kind: CastKind, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Cast(kind, Box::new(value.clone())),
            },
        )
    }

    pub fn make_add(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Add(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_sub(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Sub(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_mul(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Mul(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_square(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Square(Box::new(value.clone())),
            },
        )
    }

    pub fn make_div(
        &mut self,
        block: BlockId,
        ty: Type,
        signed: bool,
        left: &Expr,
        right: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Div(signed, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_rem(
        &mut self,
        block: BlockId,
        ty: Type,
        signed: bool,
        left: &Expr,
        right: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Rem(signed, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_neg(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Neg(Box::new(value.clone())),
            },
        )
    }

    pub fn make_fadd(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FAdd(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fsub(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FSub(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fmul(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FMul(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fdiv(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FDiv(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_frem(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FRem(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fsquare(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FSquare(Box::new(value.clone())),
            },
        )
    }

    pub fn make_fneg(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::FNeg(Box::new(value.clone())),
            },
        )
    }

    pub fn make_shl(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitShl(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_shr(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitShr(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_ashr(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::ArithShr(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_bitneg(&mut self, block: BlockId, ty: Type, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitNeg(Box::new(value.clone())),
            },
        )
    }

    pub fn make_bitand(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitAnd(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_bitor(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitOr(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_bitxor(&mut self, block: BlockId, ty: Type, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::BitXor(Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_icmp(&mut self, block: BlockId, kind: IntCmp, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Int1,
                kind: ExprKind::Cmp(kind, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_fcmp(&mut self, block: BlockId, kind: FloatCmp, left: &Expr, right: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Int1,
                kind: ExprKind::FCmp(kind, Box::new(left.clone()), Box::new(right.clone())),
            },
        )
    }

    pub fn make_alloca(&mut self, block: BlockId, ty: Type) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::Alloca(ty),
            },
        )
    }

    pub fn make_nalloca(&mut self, block: BlockId, ty: Type, count: usize) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::NAlloca(ty, count),
            },
        )
    }

    pub fn make_load(&mut self, block: BlockId, ty: Type, volatile: bool, ptr: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty,
                kind: ExprKind::Load(volatile, Box::new(ptr.clone())),
            },
        )
    }

    pub fn make_store(&mut self, block: BlockId, volatile: bool, ptr: &Expr, value: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Void,
                kind: ExprKind::Store(volatile, Box::new(ptr.clone()), Box::new(value.clone())),
            },
        )
    }

    pub fn make_ptr_offset(&mut self, block: BlockId, base: &Expr, offset: &Expr) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::PtrOffset(Box::new(base.clone()), Box::new(offset.clone())),
            },
        )
    }

    pub fn make_element_ptr(
        &mut self,
        block: BlockId,
        ty: Type,
        base: &Expr,
        offset: &Expr,
    ) -> Expr {
        self.append_expr(
            block,
            Expr {
                ty: Type::Pointer,
                kind: ExprKind::ElementPtr(ty, Box::new(base.clone()), Box::new(offset.clone())),
            },
        )
    }
    pub fn make_ret(&mut self, block: BlockId, value: &Expr) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::Ret(value.clone()));
    }

    pub fn make_retvoid(&mut self, block: BlockId) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::RetVoid);
    }

    pub fn make_br(&mut self, block: BlockId, target: BlockId, params: Vec<Expr>) {
        self.blocks.get_mut(&block).unwrap().terminator =
            Some(Terminator::Br { bb: target, params });
    }

    pub fn make_brif(
        &mut self,
        block: BlockId,
        cond: &Expr,
        then_bb: BlockId,
        then_params: Vec<Expr>,
        else_bb: BlockId,
        else_params: Vec<Expr>,
    ) {
        self.blocks.get_mut(&block).unwrap().terminator = Some(Terminator::BrIf {
            cond: cond.clone(),
            then_bb,
            then_params,
            else_bb,
            else_params,
        });
    }
}
