//! Semantic analyzer and static type checker for Nova.

use crate::ast::*;
use crate::token::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub ty: Type,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub params: Vec<Type>,
    pub return_type: Type,
}

pub struct TypeChecker {
    functions: HashMap<String, FuncSignature>,
    scopes: Vec<HashMap<String, Symbol>>,
    current_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            scopes: Vec::new(),
            current_return_type: None,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        // First pass: collect all function signatures
        for func in &program.functions {
            if self.functions.contains_key(&func.name) {
                return Err(format!("Duplicate function '{}' defined at {}", func.name, func.span));
            }

            let params = func.params.iter().map(|p| p.ty.clone()).collect();
            self.functions.insert(
                func.name.clone(),
                FuncSignature {
                    params,
                    return_type: func.return_type.clone(),
                },
            );
        }

        // Second pass: check function bodies
        for func in &program.functions {
            self.check_function(func)?;
        }

        Ok(())
    }

    fn check_function(&mut self, func: &Function) -> Result<(), String> {
        self.current_return_type = Some(func.return_type.clone());
        self.enter_scope();

        // Register parameters in function scope
        for param in &func.params {
            self.define_variable(&param.name, param.ty.clone(), false, param.span)?;
        }

        // Check body statements
        for stmt in &func.body.statements {
            self.check_statement(stmt)?;
        }

        self.exit_scope();
        self.current_return_type = None;
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let {
                name,
                is_mut,
                ty,
                init,
                span,
            } => {
                let init_ty = self.check_expression(init)?;
                let var_ty = if let Some(explicit_ty) = ty {
                    if *explicit_ty != init_ty {
                        return Err(format!(
                            "Type mismatch in let binding '{}' at {}: expected '{}', found '{}'",
                            name, span, explicit_ty, init_ty
                        ));
                    }
                    explicit_ty.clone()
                } else {
                    init_ty
                };

                self.define_variable(name, var_ty, *is_mut, *span)?;
            }
            Statement::Assignment {
                target,
                value,
                span,
            } => {
                let symbol = self
                    .lookup_variable(target)
                    .ok_or_else(|| format!("Cannot assign to undefined variable '{}' at {}", target, span))?;

                if !symbol.is_mut {
                    return Err(format!(
                        "Cannot assign to immutable variable '{}' at {}. Consider declaring as 'let mut {}'",
                        target, span, target
                    ));
                }

                let val_ty = self.check_expression(value)?;
                if symbol.ty != val_ty {
                    return Err(format!(
                        "Type mismatch in assignment to '{}' at {}: expected '{}', found '{}'",
                        target, span, symbol.ty, val_ty
                    ));
                }
            }
            Statement::MemoryStore {
                ty,
                ptr,
                value,
                span,
                ..
            } => {
                let ptr_ty = self.check_expression(ptr)?;
                if ptr_ty != Type::I32 {
                    return Err(format!(
                        "Memory store pointer must be of type 'i32' at {}, found '{}'",
                        span, ptr_ty
                    ));
                }

                let val_ty = self.check_expression(value)?;
                if *ty != val_ty {
                    return Err(format!(
                        "Memory store value type mismatch at {}: expected '{}', found '{}'",
                        span, ty, val_ty
                    ));
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                let cond_ty = self.check_expression(condition)?;
                if cond_ty != Type::Bool && cond_ty != Type::I32 {
                    return Err(format!(
                        "If condition must evaluate to 'bool' or 'i32' at {}, found '{}'",
                        span, cond_ty
                    ));
                }

                self.enter_scope();
                for s in &then_branch.statements {
                    self.check_statement(s)?;
                }
                self.exit_scope();

                if let Some(else_b) = else_branch {
                    self.enter_scope();
                    for s in &else_b.statements {
                        self.check_statement(s)?;
                    }
                    self.exit_scope();
                }
            }
            Statement::While {
                condition,
                body,
                span,
            } => {
                let cond_ty = self.check_expression(condition)?;
                if cond_ty != Type::Bool && cond_ty != Type::I32 {
                    return Err(format!(
                        "While condition must evaluate to 'bool' or 'i32' at {}, found '{}'",
                        span, cond_ty
                    ));
                }

                self.enter_scope();
                for s in &body.statements {
                    self.check_statement(s)?;
                }
                self.exit_scope();
            }
            Statement::Return { value, span } => {
                let expected_ty = self.current_return_type.as_ref().unwrap();
                let actual_ty = if let Some(expr) = value {
                    self.check_expression(expr)?
                } else {
                    Type::Void
                };

                if *expected_ty != actual_ty {
                    return Err(format!(
                        "Return type mismatch at {}: expected '{}', returned '{}'",
                        span, expected_ty, actual_ty
                    ));
                }
            }
            Statement::Break(_) => {}
            Statement::Expr { expr, .. } => {
                self.check_expression(expr)?;
            }
        }
        Ok(())
    }

    pub fn check_expression(&self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::LiteralInt(_, _) => Ok(Type::I32),
            Expression::LiteralFloat(_, _) => Ok(Type::F32),
            Expression::LiteralBool(_, _) => Ok(Type::Bool),
            Expression::Variable(name, span) => {
                if let Some(sym) = self.lookup_variable(name) {
                    Ok(sym.ty)
                } else {
                    Err(format!("Undefined variable '{}' referenced at {}", name, span))
                }
            }
            Expression::Binary {
                op,
                left,
                right,
                span,
            } => {
                let left_ty = self.check_expression(left)?;
                let right_ty = self.check_expression(right)?;

                if left_ty != right_ty {
                    return Err(format!(
                        "Binary operator '{:?}' operand type mismatch at {}: '{}' vs '{}'",
                        op, span, left_ty, right_ty
                    ));
                }

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
                        Ok(left_ty)
                    }
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                        if left_ty != Type::I32 && left_ty != Type::I64 {
                            return Err(format!(
                                "Bitwise operator '{:?}' requires integer operands at {}, found '{}'",
                                op, span, left_ty
                            ));
                        }
                        Ok(left_ty)
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Unary { op, expr, span } => {
                let inner_ty = self.check_expression(expr)?;
                match op {
                    UnaryOp::Neg => {
                        if inner_ty != Type::I32 && inner_ty != Type::I64 && inner_ty != Type::F32 && inner_ty != Type::F64 {
                            return Err(format!("Negation operator requires numeric operand at {}, found '{}'", span, inner_ty));
                        }
                        Ok(inner_ty)
                    }
                    UnaryOp::Not => {
                        if inner_ty != Type::Bool && inner_ty != Type::I32 {
                            return Err(format!("Logical not operator requires bool or i32 operand at {}, found '{}'", span, inner_ty));
                        }
                        Ok(inner_ty)
                    }
                }
            }
            Expression::Call { callee, args, span } => {
                let sig = self
                    .functions
                    .get(callee)
                    .ok_or_else(|| format!("Call to undefined function '{}' at {}", callee, span))?
                    .clone();

                if sig.params.len() != args.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments, got {} at {}",
                        callee,
                        sig.params.len(),
                        args.len(),
                        span
                    ));
                }

                for (i, (param_ty, arg_expr)) in sig.params.iter().zip(args.iter()).enumerate() {
                    let arg_ty = self.check_expression(arg_expr)?;
                    if *param_ty != arg_ty {
                        return Err(format!(
                            "Argument {} in call to '{}' type mismatch at {}: expected '{}', found '{}'",
                            i + 1, callee, span, param_ty, arg_ty
                        ));
                    }
                }

                Ok(sig.return_type)
            }
            Expression::MemoryLoad { ty, ptr, span, .. } => {
                let ptr_ty = self.check_expression(ptr)?;
                if ptr_ty != Type::I32 {
                    return Err(format!(
                        "Memory load pointer must be 'i32' at {}, found '{}'",
                        span, ptr_ty
                    ));
                }
                Ok(ty.clone())
            }
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_variable(&mut self, name: &str, ty: Type, is_mut: bool, span: Span) -> Result<(), String> {
        let current_scope = self.scopes.last_mut().expect("No active scope");
        if current_scope.contains_key(name) {
            return Err(format!("Variable '{}' already declared in this scope at {}", name, span));
        }
        current_scope.insert(name.to_string(), Symbol { ty, is_mut });
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym.clone());
            }
        }
        None
    }
}
