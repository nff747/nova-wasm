//! Direct WebAssembly binary module generator for Nova.
//! Emits valid, zero-overhead WebAssembly binary bytecode without LLVM.

use crate::ast::*;
use crate::wasm::leb128::*;
use crate::wasm::opcodes::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    LoopExit,
    LoopHead,
    If,
}

pub struct WasmBinaryEmitter {
    functions: HashMap<String, u32>,
    string_offsets: HashMap<String, u32>,
    data_pool: Vec<u8>,
    data_base_offset: u32,
}

impl WasmBinaryEmitter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            string_offsets: HashMap::new(),
            data_pool: Vec::new(),
            data_base_offset: 1024,
        }
    }

    pub fn emit(&mut self, program: &Program) -> Result<Vec<u8>, String> {
        let mut wasm = Vec::new();

        // 0. Collect string literals into static data pool
        self.collect_string_literals(program);

        // 1. Magic header: \0asm
        wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);
        // 2. Version: 1
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        // Register function indices (imports take indices 0..num_imports, defined funcs follow)
        let num_imports = program.imports.len() as u32;
        for (idx, imp) in program.imports.iter().enumerate() {
            self.functions.insert(imp.name.clone(), idx as u32);
        }
        for (idx, func) in program.functions.iter().enumerate() {
            self.functions.insert(func.name.clone(), num_imports + (idx as u32));
        }

        // 3. Type Section (Section 1)
        self.emit_type_section(program, &mut wasm);

        // 4. Import Section (Section 2)
        if !program.imports.is_empty() {
            self.emit_import_section(program, &mut wasm);
        }

        // 5. Function Section (Section 3)
        self.emit_function_section(program, &mut wasm);

        // 6. Memory Section (Section 5)
        if let Some(mem) = &program.memory {
            self.emit_memory_section(mem, &mut wasm);
        } else {
            // Compute required linear memory pages to comfortably hold data pool
            let required_pages = ((self.data_base_offset + self.data_pool.len() as u32 + 65535) / 65536).max(1);
            let default_mem = MemoryDecl {
                initial_pages: required_pages,
                max_pages: None,
                is_export: true,
                span: crate::token::Span::new(0, 0, 0),
            };
            self.emit_memory_section(&default_mem, &mut wasm);
        }

        // 7. Export Section (Section 7)
        self.emit_export_section(program, &mut wasm);

        // 8. Code Section (Section 10)
        self.emit_code_section(program, &mut wasm)?;

        // 9. Data Section (Section 11)
        if !self.data_pool.is_empty() {
            self.emit_data_section(&mut wasm);
        }

        Ok(wasm)
    }

    fn emit_type_section(&self, program: &Program, wasm: &mut Vec<u8>) {
        let mut section_payload = Vec::new();

        let total_types = program.imports.len() + program.functions.len();
        encode_u32(total_types as u32, &mut section_payload);

        // Import function types first
        for imp in &program.imports {
            section_payload.push(FUNC_TYPE);
            encode_u32(imp.params.len() as u32, &mut section_payload);
            for p in &imp.params {
                section_payload.push(type_to_wasm_val(&p.ty));
            }
            if imp.return_type == Type::Void {
                encode_u32(0, &mut section_payload);
            } else {
                encode_u32(1, &mut section_payload);
                section_payload.push(type_to_wasm_val(&imp.return_type));
            }
        }

        // Defined function types
        for func in &program.functions {
            section_payload.push(FUNC_TYPE);
            encode_u32(func.params.len() as u32, &mut section_payload);
            for p in &func.params {
                section_payload.push(type_to_wasm_val(&p.ty));
            }
            if func.return_type == Type::Void {
                encode_u32(0, &mut section_payload);
            } else {
                encode_u32(1, &mut section_payload);
                section_payload.push(type_to_wasm_val(&func.return_type));
            }
        }

        self.write_section(SECTION_TYPE, &section_payload, wasm);
    }

    fn emit_import_section(&self, program: &Program, wasm: &mut Vec<u8>) {
        let mut section_payload = Vec::new();
        encode_u32(program.imports.len() as u32, &mut section_payload);

        for (idx, imp) in program.imports.iter().enumerate() {
            encode_name(&imp.module, &mut section_payload);
            encode_name(&imp.field, &mut section_payload);
            section_payload.push(0x00); // Kind 0x00 = Function
            encode_u32(idx as u32, &mut section_payload); // Type index
        }

        self.write_section(SECTION_IMPORT, &section_payload, wasm);
    }

    fn emit_function_section(&self, program: &Program, wasm: &mut Vec<u8>) {
        let mut section_payload = Vec::new();
        let num_imports = program.imports.len() as u32;

        encode_u32(program.functions.len() as u32, &mut section_payload);
        for idx in 0..program.functions.len() {
            // Each defined function corresponds to type index following imports
            encode_u32(num_imports + (idx as u32), &mut section_payload);
        }

        self.write_section(SECTION_FUNCTION, &section_payload, wasm);
    }

    fn emit_memory_section(&self, mem: &MemoryDecl, wasm: &mut Vec<u8>) {
        let mut section_payload = Vec::new();

        encode_u32(1, &mut section_payload); // 1 memory limit defined
        if let Some(max) = mem.max_pages {
            section_payload.push(0x01); // flags: has max limit
            encode_u32(mem.initial_pages, &mut section_payload);
            encode_u32(max, &mut section_payload);
        } else {
            section_payload.push(0x00); // flags: initial only
            encode_u32(mem.initial_pages, &mut section_payload);
        }

        self.write_section(SECTION_MEMORY, &section_payload, wasm);
    }

    fn emit_export_section(&self, program: &Program, wasm: &mut Vec<u8>) {
        let mut exports = Vec::new();
        let num_imports = program.imports.len() as u32;

        // Collect function exports with global function index
        for (idx, func) in program.functions.iter().enumerate() {
            if func.is_export {
                exports.push((func.name.clone(), EXPORT_DESC_FUNC, num_imports + (idx as u32)));
            }
        }

        // Memory export
        let export_mem = program.memory.as_ref().map_or(true, |m| m.is_export);
        if export_mem {
            exports.push(("memory".to_string(), EXPORT_DESC_MEM, 0));
        }

        let mut section_payload = Vec::new();
        encode_u32(exports.len() as u32, &mut section_payload);

        for (name, desc, index) in exports {
            encode_name(&name, &mut section_payload);
            section_payload.push(desc);
            encode_u32(index, &mut section_payload);
        }

        self.write_section(SECTION_EXPORT, &section_payload, wasm);
    }

    fn emit_code_section(&self, program: &Program, wasm: &mut Vec<u8>) -> Result<(), String> {
        let mut section_payload = Vec::new();
        encode_u32(program.functions.len() as u32, &mut section_payload);

        for func in &program.functions {
            let func_body = self.compile_function_body(func)?;
            encode_u32(func_body.len() as u32, &mut section_payload);
            section_payload.extend_from_slice(&func_body);
        }

        self.write_section(SECTION_CODE, &section_payload, wasm);
        Ok(())
    }

    fn compile_function_body(&self, func: &Function) -> Result<Vec<u8>, String> {
        let mut body = Vec::new();

        // 1. Collect local variables declared with 'let'
        let mut locals_map = HashMap::new();
        let mut local_idx = 0u32;

        // Parameters occupy locals 0..params.len() - 1
        for p in &func.params {
            locals_map.insert(p.name.clone(), local_idx);
            local_idx += 1;
        }

        let mut declared_locals: Vec<(String, Type)> = Vec::new();
        collect_let_locals(&func.body, &mut declared_locals);

        for (name, _ty) in &declared_locals {
            locals_map.insert(name.clone(), local_idx);
            local_idx += 1;
        }

        // Emit locals declaration in function header
        // Group consecutive locals of identical type
        encode_u32(declared_locals.len() as u32, &mut body);
        for (_, ty) in &declared_locals {
            encode_u32(1, &mut body); // count = 1
            body.push(type_to_wasm_val(ty));
        }

        // 2. Compile statements
        let mut block_stack = Vec::new();
        for stmt in &func.body.statements {
            self.compile_statement(stmt, &locals_map, &mut block_stack, &mut body)?;
        }

        // 3. End function opcode
        body.push(END);
        Ok(body)
    }

    fn compile_statement(
        &self,
        stmt: &Statement,
        locals: &HashMap<String, u32>,
        block_stack: &mut Vec<BlockKind>,
        body: &mut Vec<u8>,
    ) -> Result<(), String> {
        match stmt {
            Statement::Let { name, init, .. } => {
                let idx = *locals.get(name).unwrap();
                self.compile_expression(init, locals, body)?;
                body.push(LOCAL_SET);
                encode_u32(idx, body);
            }
            Statement::Assignment { target, value, .. } => {
                let idx = *locals.get(target).unwrap();
                self.compile_expression(value, locals, body)?;
                body.push(LOCAL_SET);
                encode_u32(idx, body);
            }
            Statement::MemoryStore { ty, ptr, offset, value, .. } => {
                self.compile_expression(ptr, locals, body)?;
                self.compile_expression(value, locals, body)?;
                let opcode = match ty {
                    Type::I32 => I32_STORE,
                    Type::I64 => I64_STORE,
                    Type::F32 => F32_STORE,
                    Type::F64 => F64_STORE,
                    _ => return Err(format!("Unsupported type for memory store: {:?}", ty)),
                };
                body.push(opcode);
                encode_u32(2, body); // alignment = 2 (2^2 = 4 bytes)
                encode_u32(*offset, body);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.compile_expression(condition, locals, body)?;
                body.push(IF);
                body.push(BLOCK_EMPTY);
                block_stack.push(BlockKind::If);

                for s in &then_branch.statements {
                    self.compile_statement(s, locals, block_stack, body)?;
                }

                if let Some(else_b) = else_branch {
                    body.push(ELSE);
                    for s in &else_b.statements {
                        self.compile_statement(s, locals, block_stack, body)?;
                    }
                }

                body.push(END);
                block_stack.pop();
            }
            Statement::While { condition, body: loop_body, .. } => {
                // Loop pattern:
                // block $exit (depth for break)
                //   loop $continue
                //     <cond>
                //     i32.eqz
                //     br_if 1 (exit)
                //     <body>
                //     br 0 (repeat)
                //   end
                // end
                body.push(BLOCK);
                body.push(BLOCK_EMPTY);
                block_stack.push(BlockKind::LoopExit);

                body.push(LOOP);
                body.push(BLOCK_EMPTY);
                block_stack.push(BlockKind::LoopHead);

                self.compile_expression(condition, locals, body)?;
                body.push(I32_EQZ);
                body.push(BR_IF);
                encode_u32(1, body); // break to outer block

                for s in &loop_body.statements {
                    self.compile_statement(s, locals, block_stack, body)?;
                }

                body.push(BR);
                encode_u32(0, body); // branch back to top of loop

                body.push(END); // close loop
                block_stack.pop();

                body.push(END); // close block
                block_stack.pop();
            }
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.compile_expression(expr, locals, body)?;
                }
                body.push(RETURN);
            }
            Statement::Break(_) => {
                // Find relative distance to the enclosing LoopExit block
                let depth = block_stack
                    .iter()
                    .rev()
                    .position(|k| *k == BlockKind::LoopExit)
                    .ok_or_else(|| "break statement outside of loop".to_string())?;
                body.push(BR);
                encode_u32(depth as u32, body);
            }
            Statement::Expr { expr, .. } => {
                self.compile_expression(expr, locals, body)?;
                // If expression produces a value on the stack that is unused, drop it
                // Note: Calls to void functions don't produce values. For others, drop.
                if let Expression::Call { callee, .. } = expr {
                    if let Some(&_idx) = self.functions.get(callee) {
                        // Drop result if not void
                        body.push(DROP);
                    }
                }
            }
        }
        Ok(())
    }

    fn compile_expression(
        &self,
        expr: &Expression,
        locals: &HashMap<String, u32>,
        body: &mut Vec<u8>,
    ) -> Result<(), String> {
        match expr {
            Expression::LiteralInt(v, _) => {
                body.push(I32_CONST);
                encode_i32(*v as i32, body);
            }
            Expression::LiteralFloat(v, _) => {
                body.push(F32_CONST);
                body.extend_from_slice(&((*v as f32).to_bits().to_le_bytes()));
            }
            Expression::LiteralBool(b, _) => {
                body.push(I32_CONST);
                encode_i32(if *b { 1 } else { 0 }, body);
            }
            Expression::LiteralString(s, _) => {
                let offset = self.string_offsets.get(s).copied().unwrap_or(0);
                body.push(I32_CONST);
                encode_i32(offset as i32, body);
            }
            Expression::Variable(name, _) => {
                let idx = *locals
                    .get(name)
                    .ok_or_else(|| format!("Variable '{}' not found in local table", name))?;
                body.push(LOCAL_GET);
                encode_u32(idx, body);
            }
            Expression::Binary { op, left, right, .. } => {
                self.compile_expression(left, locals, body)?;
                self.compile_expression(right, locals, body)?;

                let opcode = match op {
                    BinaryOp::Add => I32_ADD,
                    BinaryOp::Sub => I32_SUB,
                    BinaryOp::Mul => I32_MUL,
                    BinaryOp::Div => I32_DIV_S,
                    BinaryOp::Rem => I32_REM_S,
                    BinaryOp::Eq => I32_EQ,
                    BinaryOp::Ne => I32_NE,
                    BinaryOp::Lt => I32_LT_S,
                    BinaryOp::Le => I32_LE_S,
                    BinaryOp::Gt => I32_GT_S,
                    BinaryOp::Ge => I32_GE_S,
                    BinaryOp::BitAnd => I32_AND,
                    BinaryOp::BitOr => I32_OR,
                    BinaryOp::BitXor => I32_XOR,
                    BinaryOp::Shl => I32_SHL,
                    BinaryOp::Shr => I32_SHR_S,
                };
                body.push(opcode);
            }
            Expression::Unary { op, expr, .. } => {
                match op {
                    UnaryOp::Neg => {
                        body.push(I32_CONST);
                        encode_i32(0, body);
                        self.compile_expression(expr, locals, body)?;
                        body.push(I32_SUB);
                    }
                    UnaryOp::Not => {
                        self.compile_expression(expr, locals, body)?;
                        body.push(I32_EQZ);
                    }
                }
            }
            Expression::Call { callee, args, .. } => {
                for arg in args {
                    self.compile_expression(arg, locals, body)?;
                }
                let func_idx = *self
                    .functions
                    .get(callee)
                    .ok_or_else(|| format!("Function '{}' not found in function index", callee))?;
                body.push(CALL);
                encode_u32(func_idx, body);
            }
            Expression::MemoryLoad { ty, ptr, offset, .. } => {
                self.compile_expression(ptr, locals, body)?;
                let opcode = match ty {
                    Type::I32 => I32_LOAD,
                    Type::I64 => I64_LOAD,
                    Type::F32 => F32_LOAD,
                    Type::F64 => F64_LOAD,
                    _ => return Err(format!("Unsupported type for memory load: {:?}", ty)),
                };
                body.push(opcode);
                encode_u32(2, body); // alignment = 2 (4 bytes)
                encode_u32(*offset, body);
            }
        }
        Ok(())
    }

    fn emit_data_section(&self, wasm: &mut Vec<u8>) {
        if self.data_pool.is_empty() {
            return;
        }
        let mut section_payload = Vec::new();
        // Number of data segments: 1
        encode_u32(1, &mut section_payload);
        // Active data segment on memory 0: flags = 0x00
        section_payload.push(0x00);
        // Initialization expression: i32.const <data_base_offset> end
        section_payload.push(I32_CONST);
        encode_i32(self.data_base_offset as i32, &mut section_payload);
        section_payload.push(END);
        // Data length
        encode_u32(self.data_pool.len() as u32, &mut section_payload);
        // Data bytes
        section_payload.extend_from_slice(&self.data_pool);

        self.write_section(SECTION_DATA, &section_payload, wasm);
    }

    fn collect_string_literals(&mut self, program: &Program) {
        let mut strings = Vec::new();
        for func in &program.functions {
            for stmt in &func.body.statements {
                scan_statement(stmt, &mut strings);
            }
        }
        for s in strings {
            if !self.string_offsets.contains_key(&s) {
                let offset = self.data_base_offset + self.data_pool.len() as u32;
                self.string_offsets.insert(s.clone(), offset);
                self.data_pool.extend_from_slice(s.as_bytes());
                self.data_pool.push(0); // Null terminator
            }
        }
    }

    fn write_section(&self, section_id: u8, payload: &[u8], wasm: &mut Vec<u8>) {
        wasm.push(section_id);
        encode_u32(payload.len() as u32, wasm);
        wasm.extend_from_slice(payload);
    }
}

fn scan_expression(expr: &Expression, out: &mut Vec<String>) {
    match expr {
        Expression::LiteralString(s, _) => out.push(s.clone()),
        Expression::Binary { left, right, .. } => {
            scan_expression(left, out);
            scan_expression(right, out);
        }
        Expression::Unary { expr, .. } => scan_expression(expr, out),
        Expression::Call { args, .. } => {
            for a in args {
                scan_expression(a, out);
            }
        }
        Expression::MemoryLoad { ptr, .. } => scan_expression(ptr, out),
        _ => {}
    }
}

fn scan_statement(stmt: &Statement, out: &mut Vec<String>) {
    match stmt {
        Statement::Let { init, .. } => {
            scan_expression(init, out);
        }
        Statement::Assignment { value, .. } => scan_expression(value, out),
        Statement::MemoryStore { ptr, value, .. } => {
            scan_expression(ptr, out);
            scan_expression(value, out);
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            scan_expression(condition, out);
            for s in &then_branch.statements {
                scan_statement(s, out);
            }
            if let Some(eb) = else_branch {
                for s in &eb.statements {
                    scan_statement(s, out);
                }
            }
        }
        Statement::While { condition, body, .. } => {
            scan_expression(condition, out);
            for s in &body.statements {
                scan_statement(s, out);
            }
        }
        Statement::Return { value, .. } => {
            if let Some(v) = value {
                scan_expression(v, out);
            }
        }
        Statement::Expr { expr, .. } => scan_expression(expr, out),
        Statement::Break(_) => {}
    }
}

fn type_to_wasm_val(ty: &Type) -> u8 {
    match ty {
        Type::I32 | Type::Bool => VAL_TYPE_I32,
        Type::I64 => VAL_TYPE_I64,
        Type::F32 => VAL_TYPE_F32,
        Type::F64 => VAL_TYPE_F64,
        Type::Void => panic!("Void cannot be converted to a Wasm value type"),
    }
}

fn collect_let_locals(block: &Block, out: &mut Vec<(String, Type)>) {
    for stmt in &block.statements {
        match stmt {
            Statement::Let { name, ty, .. } => {
                let inferred_ty = ty.clone().unwrap_or(Type::I32);
                out.push((name.clone(), inferred_ty));
            }
            Statement::If { then_branch, else_branch, .. } => {
                collect_let_locals(then_branch, out);
                if let Some(else_b) = else_branch {
                    collect_let_locals(else_b, out);
                }
            }
            Statement::While { body, .. } => {
                collect_let_locals(body, out);
            }
            _ => {}
        }
    }
}
