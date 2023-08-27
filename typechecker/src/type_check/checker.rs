use ast::{Expression, Statement};
use parser::ast::{self, *};

use crate::{
    ast_visitor::TraversalVisitor, ast_visitor_immut::TraversalVisitorImmut, settings::Settings,
    state::State, symbol_table::Declaration,
};

use super::{
    type_inference::{self, type_check_bin_op, type_equal},
    types::Type,
};

pub struct TypeChecker<'a> {
    pub errors: Vec<String>,
    // TODO: currently only supporting a single file
    pub module: &'a State,
    pub options: &'a Settings,
}

impl<'a> TypeChecker<'a> {
    pub fn new(module: &'a State, options: &'a Settings) -> Self {
        TypeChecker {
            errors: vec![],
            module,
            options,
        }
    }

    pub fn type_check(&mut self) {
        for stmt in &self.module.file.body {
            self.visit_stmt(&stmt);
        }
    }

    fn infer_declaration_type(&mut self, declaration: &Declaration) -> Type {
        match declaration {
            Declaration::Variable(v) => {
                if let Some(type_annotation) = &v.type_annotation {
                    type_inference::get_type_from_annotation(type_annotation.clone())
                } else {
                    Type::Unknown
                }
            }
            _ => Type::Unknown,
        }
    }

    fn infer_expr_type(&mut self, expr: &ast::Expression) -> Type {
        match expr {
            ast::Expression::Constant(c) => match c.value {
                ast::ConstantValue::Int(_) => Type::Int,
                ast::ConstantValue::Float(_) => Type::Float,
                ast::ConstantValue::Str(_) => Type::Str,
                ast::ConstantValue::Bool(_) => Type::Bool,
                ast::ConstantValue::None => Type::None,
                _ => Type::Unknown,
            },
            ast::Expression::Name(n) => match self.module.symbol_table.lookup_in_scope(&n.id) {
                Some(symbol) => match symbol.last_declaration() {
                    Some(declaration) => self.infer_declaration_type(declaration),
                    None => Type::Unknown,
                },
                None => Type::Unknown,
            },
            _ => Type::Unknown,
        }
    }
}

impl<'a> TraversalVisitor for TypeChecker<'a> {
    fn visit_stmt(&mut self, s: &Statement) {
        // map all statements and call visit
        match s {
            Statement::ExpressionStatement(e) => self.visit_expr(e),
            Statement::Import(i) => self.visit_import(i),
            Statement::ImportFrom(i) => self.visit_import_from(i),
            Statement::AssignStatement(a) => self.visit_assign(a),
            Statement::AnnAssignStatement(a) => self.visit_ann_assign(a),
            Statement::AugAssignStatement(a) => self.visit_aug_assign(a),
            Statement::Assert(a) => self.visit_assert(a),
            Statement::Pass(p) => self.visit_pass(p),
            Statement::Delete(d) => self.visit_delete(d),
            Statement::Return(r) => self.visit_return(r),
            Statement::Raise(r) => self.visit_raise(r),
            Statement::Break(b) => self.visit_break(b),
            Statement::Continue(c) => self.visit_continue(c),
            Statement::Global(g) => self.visit_global(g),
            Statement::Nonlocal(n) => self.visit_nonlocal(n),
            Statement::IfStatement(i) => self.visit_if(i),
            Statement::WhileStatement(w) => self.visit_while(w),
            Statement::ForStatement(f) => self.visit_for(f),
            Statement::WithStatement(w) => self.visit_with(w),
            Statement::TryStatement(t) => self.visit_try(t),
            Statement::TryStarStatement(t) => self.visit_try_star(t),
            Statement::FunctionDef(f) => self.visit_function_def(f),
            Statement::ClassDef(c) => self.visit_class_def(c),
            Statement::Match(m) => self.visit_match(m),
        }
    }

    fn visit_expr(&mut self, e: &Expression) {
        match e {
            Expression::Constant(c) => self.visit_constant(c),
            Expression::List(l) => self.visit_list(l),
            Expression::Tuple(t) => self.visit_tuple(t),
            Expression::Dict(d) => self.visit_dict(d),
            Expression::Set(s) => self.visit_set(s),
            Expression::Name(n) => self.visit_name(n),
            Expression::BoolOp(b) => self.visit_bool_op(b),
            Expression::UnaryOp(u) => self.visit_unary_op(u),
            Expression::BinOp(b) => self.visit_bin_op(b),
            Expression::NamedExpr(n) => self.visit_named_expr(n),
            Expression::Yield(y) => self.visit_yield(y),
            Expression::YieldFrom(y) => self.visit_yield_from(y),
            Expression::Starred(s) => self.visit_starred(s),
            Expression::Generator(g) => self.visit_generator(g),
            Expression::ListComp(l) => self.visit_list_comp(l),
            Expression::SetComp(s) => self.visit_set_comp(s),
            Expression::DictComp(d) => self.visit_dict_comp(d),
            Expression::Attribute(a) => self.visit_attribute(a),
            Expression::Subscript(s) => self.visit_subscript(s),
            Expression::Slice(s) => self.visit_slice(s),
            Expression::Call(c) => self.visit_call(c),
            Expression::Await(a) => self.visit_await(a),
            Expression::Compare(c) => self.visit_compare(c),
            Expression::Lambda(l) => self.visit_lambda(l),
            Expression::IfExp(i) => self.visit_if_exp(i),
            Expression::JoinedStr(j) => self.visit_joined_str(j),
            Expression::FormattedValue(f) => self.visit_formatted_value(f),
        }
    }

    fn visit_import(&mut self, i: &Import) {
        todo!();
    }

    fn visit_import_from(&mut self, i: &ImportFrom) {
        todo!();
    }

    fn visit_if(&mut self, i: &parser::ast::If) {
        // self.visit_stmt(i.test);
        for stmt in &i.body {
            self.visit_stmt(&stmt);
        }
        for stmt in &i.orelse {
            self.visit_stmt(&stmt);
        }
    }

    fn visit_while(&mut self, w: &parser::ast::While) {
        for stmt in &w.body {
            self.visit_stmt(&stmt)
        }
    }

    fn visit_for(&mut self, f: &parser::ast::For) {
        for stmt in &f.body {
            self.visit_stmt(&stmt);
        }
    }

    fn visit_with(&mut self, w: &parser::ast::With) {
        for stmt in &w.body {
            self.visit_stmt(&stmt);
        }
        for with_items in &w.items {
            self.visit_expr(&*&with_items.context_expr);
            match &with_items.optional_vars {
                Some(items) => self.visit_expr(&items),
                None => (),
            }
        }
    }

    fn visit_try(&mut self, t: &parser::ast::Try) {
        for stmt in &t.body {
            self.visit_stmt(&stmt);
        }
        for stmt in &t.orelse {
            self.visit_stmt(&stmt);
        }
        for stmt in &t.finalbody {
            self.visit_stmt(&stmt);
        }
        // TODO: need to visit exception handler name and type but let's keep it simple for now
        for handler in &t.handlers {
            for stmt in &handler.body {
                self.visit_stmt(&stmt);
            }
        }
    }

    fn visit_try_star(&mut self, t: &parser::ast::TryStar) {
        for stmt in &t.body {
            self.visit_stmt(&stmt);
        }
        for stmt in &t.orelse {
            self.visit_stmt(&stmt);
        }
        for stmt in &t.finalbody {
            self.visit_stmt(&stmt);
        }
        // TODO: need to visit exception handler name and type but let's keep it simple for now
        for handler in &t.handlers {
            for stmt in &handler.body {
                self.visit_stmt(&stmt);
            }
        }
    }

    fn visit_function_def(&mut self, f: &parser::ast::FunctionDef) {
        for stmt in &f.body {
            self.visit_stmt(&stmt);
        }
    }

    fn visit_class_def(&mut self, c: &parser::ast::ClassDef) {
        for stmt in &c.body {
            self.visit_stmt(&stmt);
        }
    }

    fn visit_match(&mut self, m: &parser::ast::Match) {
        for case in &m.cases {
            for stmt in &case.body {
                self.visit_stmt(&stmt);
            }
        }
    }

    fn visit_constant(&mut self, c: &Constant) {}

    fn visit_list(&mut self, l: &List) {
        todo!()
    }

    fn visit_tuple(&mut self, t: &Tuple) {
        todo!()
    }

    fn visit_dict(&mut self, d: &Dict) {
        todo!()
    }

    fn visit_set(&mut self, s: &Set) {
        todo!()
    }

    fn visit_name(&mut self, n: &Name) {}

    fn visit_bool_op(&mut self, b: &BoolOperation) {
        todo!()
    }

    fn visit_unary_op(&mut self, u: &UnaryOperation) {
        todo!()
    }

    fn visit_bin_op(&mut self, b: &BinOp) {
        self.visit_expr(&b.left);
        self.visit_expr(&b.right);
        let l_type = self.infer_expr_type(&b.left);
        let r_type = self.infer_expr_type(&b.right);

        if !type_check_bin_op(&l_type, &r_type, &b.op) {
            self.errors.push(format!(
                "Operator '{}' not supported for types '{}' and '{}'",
                b.op, l_type, r_type
            ));
        }
    }

    fn visit_named_expr(&mut self, n: &NamedExpression) {
        todo!()
    }

    fn visit_yield(&mut self, y: &Yield) {
        todo!()
    }

    fn visit_yield_from(&mut self, y: &YieldFrom) {
        todo!()
    }

    fn visit_starred(&mut self, s: &Starred) {
        todo!()
    }

    fn visit_generator(&mut self, g: &Generator) {
        todo!()
    }

    fn visit_list_comp(&mut self, l: &ListComp) {
        todo!()
    }

    fn visit_set_comp(&mut self, s: &SetComp) {
        todo!()
    }

    fn visit_dict_comp(&mut self, d: &DictComp) {
        todo!()
    }

    fn visit_attribute(&mut self, a: &Attribute) {
        todo!()
    }

    fn visit_subscript(&mut self, s: &Subscript) {
        todo!()
    }

    fn visit_slice(&mut self, s: &Slice) {
        todo!()
    }

    fn visit_call(&mut self, c: &Call) {
        todo!()
    }

    fn visit_await(&mut self, a: &Await) {
        todo!()
    }

    fn visit_compare(&mut self, c: &Compare) {
        todo!()
    }

    fn visit_lambda(&mut self, l: &Lambda) {
        todo!()
    }

    fn visit_if_exp(&mut self, i: &IfExp) {
        todo!()
    }

    fn visit_joined_str(&mut self, j: &JoinedStr) {
        todo!()
    }

    fn visit_formatted_value(&mut self, f: &FormattedValue) {
        todo!()
    }

    fn visit_alias(&mut self, a: &Alias) {
        todo!()
    }

    fn visit_assign(&mut self, a: &Assign) {}

    fn visit_ann_assign(&mut self, a: &AnnAssign) {}

    fn visit_aug_assign(&mut self, a: &AugAssign) {
        todo!()
    }

    fn visit_assert(&mut self, a: &Assert) {
        todo!()
    }

    fn visit_pass(&mut self, p: &Pass) {
        todo!()
    }

    fn visit_delete(&mut self, d: &Delete) {
        todo!()
    }

    fn visit_return(&mut self, r: &Return) {
        todo!()
    }

    fn visit_raise(&mut self, r: &Raise) {
        todo!()
    }

    fn visit_break(&mut self, b: &Break) {
        todo!()
    }

    fn visit_continue(&mut self, c: &Continue) {
        todo!()
    }

    fn visit_global(&mut self, g: &Global) {
        todo!()
    }

    fn visit_nonlocal(&mut self, n: &Nonlocal) {
        todo!()
    }
}
