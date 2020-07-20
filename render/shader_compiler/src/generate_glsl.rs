use {
    crate::{
        ast::*,
        env::VarKind,
        ident::Ident,
        lit::{
            Lit,
            TyLit
        },
        span::Span,
        ty::Ty,
    },
    std::{
        cell::{
            Cell,
            RefCell,
        },
        fmt::Write,
    }
};

struct BlockGenerator<'a> {
    string: &'a mut String,
    indent_level: usize,
}

impl<'a> BlockGenerator<'a> {
    fn generate_block(&mut self, block: &Block) {
        write!(self.string, "{{").unwrap();
        if !block.stmts.is_empty() {
            writeln!(self.string).unwrap();
            self.indent_level += 1;
            for stmt in &block.stmts {
                self.generate_stmt(stmt);
            }
            self.indent_level -= 1;
        }
        write!(self.string, "}}").unwrap()
    }

    fn generate_stmt(&mut self, stmt: &Stmt) {
        self.write_indent();
        match *stmt {
            Stmt::Break {
                span
            } => self.generate_break_stmt(span),
            Stmt::Continue {
                span
            } => self.generate_continue_stmt(span),
            Stmt::For {
                span,
                ident,
                ref from_expr,
                ref to_expr,
                ref step_expr,
                ref block,
            } => self.generate_for_stmt(span, ident, from_expr, to_expr, step_expr, block),
            Stmt::If {
                span,
                ref expr,
                ref block_if_true,
                ref block_if_false,
            } => self.generate_if_stmt(span, expr, block_if_true, block_if_false),
            Stmt::Let {
                span,
                ref ty,
                ident,
                ref ty_expr,
                ref expr,
            } => self.generate_let_stmt(span, ty, ident, ty_expr, expr),
            Stmt::Return {
                span,
                ref expr
            } => self.generate_return_stmt(span, expr),
            Stmt::Block {
                span,
                ref block
            } => self.generate_block_stmt(span, block),
            Stmt::Expr {
                span,
                ref expr
            } => self.generate_expr_stmt(span, expr),
        }
    }

    fn generate_break_stmt(&mut self, _span: Span) {
        writeln!(self.string, "break;").unwrap();
    }

    fn generate_continue_stmt(&mut self, _span: Span) {
        writeln!(self.string, "continue;").unwrap();
    }

    fn generate_for_stmt(
        &mut self,
        _span: Span,
        ident: Ident,
        from_expr: &Expr,
        to_expr: &Expr,
        step_expr: &Option<Expr>,
        block: &Block,
    ) {
        let from = from_expr.val.borrow().as_ref().unwrap().to_int().unwrap();
        let to = to_expr.val.borrow().as_ref().unwrap().to_int().unwrap();
        let step = if let Some(step_expr) = step_expr {
            step_expr.val.borrow().as_ref().unwrap().to_int().unwrap()
        } else if from < to {
            1
        } else {
            -1
        };
        write!(
            self.string,
            "for (int {0} = {1}; {0} {2} {3}; {0} {4} {5}) ",
            ident,
            if from <= to { from } else { from - 1 },
            if from <= to { "<" } else { ">=" },
            to,
            if step > 0 { "+=" } else { "-=" },
            step.abs()
        )
        .unwrap();
        self.generate_block(block);
        writeln!(self.string).unwrap();
    }

    fn generate_if_stmt(
        &mut self,
        _span: Span,
        expr: &Expr,
        block_if_true: &Block,
        block_if_false: &Option<Box<Block>>,
    ) {
        write!(self.string, "if (").unwrap();
        self.generate_expr(expr);
        write!(self.string, " ").unwrap();
        self.generate_block(block_if_true);
        if let Some(block_if_false) = block_if_false {
            self.generate_block(block_if_false);
        }
        writeln!(self.string).unwrap();
    }

    fn generate_let_stmt(
        &mut self,
        _span: Span,
        ty: &RefCell<Option<Ty>>,
        ident: Ident,
        _ty_expr: &Option<TyExpr>,
        expr: &Option<Expr>,
    ) {
        write_ident_and_ty(&mut self.string, ident, ty.borrow().as_ref().unwrap());
        if let Some(expr) = expr {
            write!(self.string, " = ").unwrap();
            self.generate_expr(expr);
        }
        writeln!(self.string, ";").unwrap();
    }

    fn generate_return_stmt(&mut self, _span: Span, expr: &Option<Expr>) {
        write!(self.string, "return").unwrap();
        if let Some(expr) = expr {
            write!(self.string, " ").unwrap();
            self.generate_expr(expr);
        }
        writeln!(self.string, ";").unwrap();
    }

    fn generate_block_stmt(&mut self, _span: Span, block: &Block) {
        self.generate_block(block);
        writeln!(self.string).unwrap();
    }

    fn generate_expr_stmt(&mut self, _span: Span, expr: &Expr) {
        self.generate_expr(expr);
        writeln!(self.string, ";").unwrap();
    }

    fn generate_expr(&mut self, expr: &Expr) {
        ExprGenerator {
            string: self.string,
        }
        .generate_expr(expr)
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            write!(self.string, "    ").unwrap();
        }
    }
}

struct ExprGenerator<'a> {
    string: &'a mut String,
}

impl<'a> ExprGenerator<'a> {
    fn generate_expr(&mut self, expr: &Expr) {
        match expr.kind {
            ExprKind::Cond {
                span,
                ref expr,
                ref expr_if_true,
                ref expr_if_false,
            } => self.generate_cond_expr(span, expr, expr_if_true, expr_if_false),
            ExprKind::Bin {
                span,
                op,
                ref left_expr,
                ref right_expr,
            } => self.generate_bin_expr(span, op, left_expr, right_expr),
            ExprKind::Un {
                span,
                op,
                ref expr
            } => self.generate_un_expr(span, op, expr),
            ExprKind::MethodCall {
                span,
                ident,
                ref arg_exprs
            } => self.generate_method_call_expr(span, ident, arg_exprs),
            ExprKind::Field {
                span,
                ref expr,
                field_ident,
            } => self.generate_field_expr(span, expr, field_ident),
            ExprKind::Index {
                span,
                ref expr,
                ref index_expr,
            } => self.generate_index_expr(span, expr, index_expr),
            ExprKind::Call {
                span,
                ident,
                ref arg_exprs,
            } => self.generate_call_expr(span, ident, arg_exprs),
            ExprKind::MacroCall {
                ref analysis,
                span,
                ident,
                ref arg_exprs,
                ..
            } => self.generate_macro_call_expr(analysis, span, ident, arg_exprs),
            ExprKind::ConsCall {
                span,
                ty_lit,
                ref arg_exprs,
            } => self.generate_cons_call_expr(span, ty_lit, arg_exprs),
            ExprKind::Var {
                span,
                ref is_lvalue,
                ref kind,
                ident,
            } => self.generate_var_expr(span, is_lvalue, kind, ident),
            ExprKind::Lit {
                span,
                lit
            } => self.generate_lit_expr(span, lit),
        }
    }

    fn generate_cond_expr(
        &mut self,
        _span: Span,
        expr: &Expr,
        expr_if_true: &Expr,
        expr_if_false: &Expr
    ) {
        write!(self.string, "(").unwrap();
        self.generate_expr(expr);
        write!(self.string, " ? ").unwrap();
        self.generate_expr(expr_if_true);
        write!(self.string, " : ").unwrap();
        self.generate_expr(expr_if_false);
        write!(self.string, ")").unwrap();
    }

    fn generate_bin_expr(
        &mut self,
        _span: Span,
        op: BinOp,
        left_expr: &Expr,
        right_expr: &Expr
    ) {
        write!(self.string, "(").unwrap();
        self.generate_expr(left_expr);
        write!(self.string, " {} ", op).unwrap();
        self.generate_expr(right_expr);
        write!(self.string, ")").unwrap();
    }

    fn generate_un_expr(
        &mut self,
        _span: Span,
        op: UnOp,
        expr: &Expr
    ) {
        write!(self.string, "{}", op).unwrap();
        self.generate_expr(expr);
    }

    fn generate_method_call_expr(
        &mut self,
        span: Span,
        ident: Ident,
        arg_exprs: &[Expr]
    ) {
        match arg_exprs[0].ty.borrow().as_ref().unwrap() {
            Ty::Struct { ident: struct_ident } => {
                self.generate_call_expr(
                    span,
                    Ident::new(format!("{}::{}", struct_ident, ident)),
                    arg_exprs
                );
            },
            _ => panic!()
        }
    }

    fn generate_field_expr(
        &mut self,
        _span: Span,
        expr: &Expr,
        field_ident: Ident
    ) {
        self.generate_expr(expr);
        write!(self.string, ".{}", field_ident).unwrap();
    }

    fn generate_index_expr(
        &mut self,
        _span: Span,
        expr: &Expr,
        index_expr: &Expr
    ) {
        self.generate_expr(expr);
        write!(self.string, "[").unwrap();
        self.generate_expr(index_expr);
        write!(self.string, "]").unwrap();
    }

    fn generate_call_expr(
        &mut self,
        _span: Span,
        ident: Ident,
        arg_exprs: &[Expr],
    ) {
        write!(self.string, "{}(", ident).unwrap();
        let mut sep = "";
        for arg_expr in arg_exprs {
            write!(self.string, "{}", sep).unwrap();
            self.generate_expr(arg_expr);
            sep = ", ";
        }
        write!(self.string, ")").unwrap();
    }

    fn generate_macro_call_expr(
        &mut self,
        analysis: &Cell<Option<MacroCallAnalysis>>,
        _span: Span,
        _ident: Ident,
        _arg_exprs: &[Expr],
    ) {
        match analysis.get().unwrap() {
            MacroCallAnalysis::Color { r, g, b, a } => {
                write!(self.string, "vec4({}, {}, {}, {})", r, g, b, a).unwrap();
            }
        }
    }

    fn generate_cons_call_expr(
        &mut self,
        _span: Span,
        ty_lit: TyLit,
        arg_exprs: &[Expr]
    ) {
        write!(self.string, "_m_{}", ty_lit).unwrap();
        for arg_expr in arg_exprs {
            write!(self.string, "_{}", arg_expr.ty.borrow().as_ref().unwrap()).unwrap();
        }
        write!(self.string, "(").unwrap();
        let mut sep = "";
        for arg_expr in arg_exprs {
            write!(self.string, "{}", sep).unwrap();
            self.generate_expr(arg_expr);
            sep = ", ";
        }
        write!(self.string, ")").unwrap();
    }

    fn generate_var_expr(
        &mut self,
        _span: Span,
        _is_lvalue: &Cell<Option<bool>>,
        _kind: &Cell<Option<VarKind>>,
        ident: Ident,
    ) {
        write!(self.string, "{}", ident).unwrap()
    }

    fn generate_lit_expr(
        &mut self,
        _span: Span,
        lit: Lit
    ) {
        write!(self.string, "{}", lit).unwrap();
    }
}

fn write_ident_and_ty(string: &mut String, ident: Ident, ty: &Ty) {
    match *ty {
        Ty::Void => write!(string, "void {}", ident).unwrap(),
        Ty::Bool => write!(string, "bool {}", ident).unwrap(),
        Ty::Int => write!(string, "int {}", ident).unwrap(),
        Ty::Float => write!(string, "float {}", ident).unwrap(),
        Ty::Bvec2 => write!(string, "bvec2 {}", ident).unwrap(),
        Ty::Bvec3 => write!(string, "bvec3 {}", ident).unwrap(),
        Ty::Bvec4 => write!(string, "bvec4 {}", ident).unwrap(),
        Ty::Ivec2 => write!(string, "ivec2 {}", ident).unwrap(),
        Ty::Ivec3 => write!(string, "ivec3 {}", ident).unwrap(),
        Ty::Ivec4 => write!(string, "ivec4 {}", ident).unwrap(),
        Ty::Vec2 => write!(string, "vec2 {}", ident).unwrap(),
        Ty::Vec3 => write!(string, "vec3 {}", ident).unwrap(),
        Ty::Vec4 => write!(string, "vec4 {}", ident).unwrap(),
        Ty::Mat2 => write!(string, "mat2 {}", ident).unwrap(),
        Ty::Mat3 => write!(string, "mat3 {}", ident).unwrap(),
        Ty::Mat4 => write!(string, "mat4 {}", ident).unwrap(),
        Ty::Texture2d => panic!(),
        Ty::Array { ref elem_ty, len } => {
            write_ident_and_ty(string, ident, elem_ty);
            write!(string, "[{}]", len).unwrap();
        }
        Ty::Struct {
            ident: struct_ident,
        } => write!(string, "{} {}", struct_ident, ident).unwrap(),
    }
}
