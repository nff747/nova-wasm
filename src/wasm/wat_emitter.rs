//! WebAssembly Text Format (WAT) S-Expression Emitter for Nova.

use crate::ast::*;

pub struct WatEmitter;

impl WatEmitter {
    pub fn emit(program: &Program) -> String {
        let mut out = String::new();
        out.push_str("(module\n");

        // Memory
        if let Some(mem) = &program.memory {
            let export_str = if mem.is_export { " (export \"memory\")" } else { "" };
            if let Some(max) = mem.max_pages {
                out.push_str(&format!("  (memory{} {} {})\n", export_str, mem.initial_pages, max));
            } else {
                out.push_str(&format!("  (memory{} {})\n", export_str, mem.initial_pages));
            }
        } else {
            out.push_str("  (memory (export \"memory\") 1)\n");
        }

        // Imports
        for imp in &program.imports {
            out.push_str(&format!("  (import \"{}\" \"{}\" (func ${}", imp.module, imp.field, imp.name));
            for p in &imp.params {
                out.push_str(&format!(" (param ${} {})", p.name, p.ty));
            }
            if imp.return_type != Type::Void {
                out.push_str(&format!(" (result {})", imp.return_type));
            }
            out.push_str("))\n");
        }

        // Functions
        for func in &program.functions {
            out.push_str(&format!("  (func ${}", func.name));
            if func.is_export {
                out.push_str(&format!(" (export \"{}\")", func.name));
            }

            for p in &func.params {
                out.push_str(&format!(" (param ${} {})", p.name, p.ty));
            }

            if func.return_type != Type::Void {
                out.push_str(&format!(" (result {})", func.return_type));
            }
            out.push('\n');

            // Locals
            let mut declared_locals = Vec::new();
            collect_locals(&func.body, &mut declared_locals);
            for (name, ty) in declared_locals {
                out.push_str(&format!("    (local ${} {})\n", name, ty));
            }

            // Statements
            for stmt in &func.body.statements {
                Self::emit_statement(stmt, 2, &mut out);
            }

            out.push_str("  )\n");
        }

        out.push_str(")\n");
        out
    }

    fn emit_statement(stmt: &Statement, indent: usize, out: &mut String) {
        let pad = "  ".repeat(indent);
        match stmt {
            Statement::Let { name, init, .. } => {
                Self::emit_expression(init, indent, out);
                out.push_str(&format!("{}local.set ${}\n", pad, name));
            }
            Statement::Assignment { target, value, .. } => {
                Self::emit_expression(value, indent, out);
                out.push_str(&format!("{}local.set ${}\n", pad, target));
            }
            Statement::MemoryStore { ptr, value, .. } => {
                Self::emit_expression(ptr, indent, out);
                Self::emit_expression(value, indent, out);
                out.push_str(&format!("{}i32.store\n", pad));
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                Self::emit_expression(condition, indent, out);
                out.push_str(&format!("{}if\n", pad));
                for s in &then_branch.statements {
                    Self::emit_statement(s, indent + 1, out);
                }
                if let Some(else_b) = else_branch {
                    out.push_str(&format!("{}else\n", pad));
                    for s in &else_b.statements {
                        Self::emit_statement(s, indent + 1, out);
                    }
                }
                out.push_str(&format!("{}end\n", pad));
            }
            Statement::While { condition, body, .. } => {
                out.push_str(&format!("{}block\n", pad));
                out.push_str(&format!("{}  loop\n", pad));
                Self::emit_expression(condition, indent + 2, out);
                out.push_str(&format!("{}    i32.eqz\n", pad));
                out.push_str(&format!("{}    br_if 1\n", pad));

                for s in &body.statements {
                    Self::emit_statement(s, indent + 2, out);
                }

                out.push_str(&format!("{}    br 0\n", pad));
                out.push_str(&format!("{}  end\n", pad));
                out.push_str(&format!("{}end\n", pad));
            }
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    Self::emit_expression(expr, indent, out);
                }
                out.push_str(&format!("{}return\n", pad));
            }
            Statement::Break(_) => {
                out.push_str(&format!("{}br 1\n", pad));
            }
            Statement::Expr { expr, .. } => {
                Self::emit_expression(expr, indent, out);
                out.push_str(&format!("{}drop\n", pad));
            }
        }
    }

    fn emit_expression(expr: &Expression, indent: usize, out: &mut String) {
        let pad = "  ".repeat(indent);
        match expr {
            Expression::LiteralInt(v, _) => {
                out.push_str(&format!("{}i32.const {}\n", pad, v));
            }
            Expression::LiteralFloat(v, _) => {
                out.push_str(&format!("{}f32.const {}\n", pad, v));
            }
            Expression::LiteralBool(b, _) => {
                out.push_str(&format!("{}i32.const {}\n", pad, if *b { 1 } else { 0 }));
            }
            Expression::Variable(name, _) => {
                out.push_str(&format!("{}local.get ${}\n", pad, name));
            }
            Expression::Binary { op, left, right, .. } => {
                Self::emit_expression(left, indent, out);
                Self::emit_expression(right, indent, out);
                let op_str = match op {
                    BinaryOp::Add => "i32.add",
                    BinaryOp::Sub => "i32.sub",
                    BinaryOp::Mul => "i32.mul",
                    BinaryOp::Div => "i32.div_s",
                    BinaryOp::Rem => "i32.rem_s",
                    BinaryOp::Eq => "i32.eq",
                    BinaryOp::Ne => "i32.ne",
                    BinaryOp::Lt => "i32.lt_s",
                    BinaryOp::Le => "i32.le_s",
                    BinaryOp::Gt => "i32.gt_s",
                    BinaryOp::Ge => "i32.ge_s",
                    BinaryOp::BitAnd => "i32.and",
                    BinaryOp::BitOr => "i32.or",
                    BinaryOp::BitXor => "i32.xor",
                    BinaryOp::Shl => "i32.shl",
                    BinaryOp::Shr => "i32.shr_s",
                };
                out.push_str(&format!("{}{}\n", pad, op_str));
            }
            Expression::Unary { op, expr, .. } => match op {
                UnaryOp::Neg => {
                    out.push_str(&format!("{}i32.const 0\n", pad));
                    Self::emit_expression(expr, indent, out);
                    out.push_str(&format!("{}i32.sub\n", pad));
                }
                UnaryOp::Not => {
                    Self::emit_expression(expr, indent, out);
                    out.push_str(&format!("{}i32.eqz\n", pad));
                }
            },
            Expression::Call { callee, args, .. } => {
                for arg in args {
                    Self::emit_expression(arg, indent, out);
                }
                out.push_str(&format!("{}call ${}\n", pad, callee));
            }
            Expression::MemoryLoad { ptr, .. } => {
                Self::emit_expression(ptr, indent, out);
                out.push_str(&format!("{}i32.load\n", pad));
            }
        }
    }
}

fn collect_locals(block: &Block, out: &mut Vec<(String, Type)>) {
    for stmt in &block.statements {
        match stmt {
            Statement::Let { name, ty, .. } => {
                out.push((name.clone(), ty.clone().unwrap_or(Type::I32)));
            }
            Statement::If { then_branch, else_branch, .. } => {
                collect_locals(then_branch, out);
                if let Some(else_b) = else_branch {
                    collect_locals(else_b, out);
                }
            }
            Statement::While { body, .. } => {
                collect_locals(body, out);
            }
            _ => {}
        }
    }
}
