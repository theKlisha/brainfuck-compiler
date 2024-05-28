use crate::ast;

#[derive(Debug)]
pub enum Error {}

pub struct QbeGenerator {
    label_counter: usize,
    tmp_counter: usize,
    tape_len: usize,
}

impl QbeGenerator {
    pub fn new() -> Self {
        QbeGenerator {
            tmp_counter: 0,
            label_counter: 0,
            // FIXME: holes in the tape
            tape_len: 16 * 30_000,
        }
    }

    pub fn gen(&mut self, prog: &ast::Prog) -> Result<String, ast::Error> {
        let mut main = qbe::Function::new(
            qbe::Linkage::public(),
            "main".to_string(),
            Vec::new(),
            Some(qbe::Type::Word),
        );
        main.add_block("runtime".to_string());
        self.generate_runtime(&mut main);
        main.add_block("start".to_string());
        self.generate_block(&mut main, prog);
        main.add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(0))));

        let mut module = qbe::Module::new();
        module.add_function(main);
        Ok(format!("{}\n", module))
    }

    fn generate_block(&mut self, func: &mut qbe::Function, block: &ast::NodeBlock) {
        for stat in &block.stats {
            self.generate_statement(func, &stat)
        }
    }

    fn generate_statement(&mut self, func: &mut qbe::Function, stat: &ast::NodeStatement) {
        match &stat.stat {
            ast::Statement::MoveL(n) => {
                func.assign_instr(
                    self.generate_ptr(),
                    qbe::Type::Long,
                    qbe::Instr::Sub(self.generate_ptr(), qbe::Value::Const(*n as u64 * 8)),
                    // TODO: fix pointer addition and offset                         ^^^
                );

                self.generate_bounds_check(func);
            }
            ast::Statement::MoveR(n) => {
                func.assign_instr(
                    self.generate_ptr(),
                    qbe::Type::Long,
                    qbe::Instr::Add(self.generate_ptr(), qbe::Value::Const(*n as u64 * 8)),
                    // TODO: fix pointer addition and offset                         ^^^
                );

                self.generate_bounds_check(func);
            }
            ast::Statement::Add(n) => {
                let tmp = self.generate_tmp();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Load(qbe::Type::Word, self.generate_ptr()),
                );
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Add(tmp.clone(), qbe::Value::Const(*n as u64)),
                );
                func.add_instr(qbe::Instr::Store(
                    qbe::Type::Word,
                    self.generate_ptr(),
                    tmp.clone(),
                ))
            }
            ast::Statement::Sub(n) => {
                let tmp = self.generate_tmp();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Load(qbe::Type::Word, self.generate_ptr()),
                );
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Sub(tmp.clone(), qbe::Value::Const(*n as u64)),
                );
                func.add_instr(qbe::Instr::Store(
                    qbe::Type::Word,
                    self.generate_ptr(),
                    tmp.clone(),
                ))
            }
            ast::Statement::Read => {
                // ssize_t read(int fd, void buf[.count], size_t count);
                func.add_instr(qbe::Instr::Call(
                    "read".to_string(),
                    vec![
                        (qbe::Type::Word, qbe::Value::Const(0)), // 0 for stdin
                        (qbe::Type::Long, self.generate_ptr()),
                        (qbe::Type::Long, qbe::Value::Const(1)), // one byte only
                    ],
                ));
            }
            ast::Statement::Write => {
                // ssize_t write(int fd, const void buf[.count], size_t count);
                func.add_instr(qbe::Instr::Call(
                    "write".to_string(),
                    vec![
                        (qbe::Type::Word, qbe::Value::Const(1)), // 0 for stdout
                        (qbe::Type::Long, self.generate_ptr()),
                        (qbe::Type::Long, qbe::Value::Const(1)), // one byte only
                    ],
                ));
            }
            ast::Statement::Loop(b) => {
                let c = self.label_counter;
                let begin = format!("loop{}", c);
                let end = format!("end{}", c);
                self.label_counter += 1;

                let tmp = self.generate_tmp();
                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Load(qbe::Type::Word, self.generate_ptr()),
                );
                func.add_instr(qbe::Instr::Jnz(tmp.clone(), begin.clone(), end.clone()));
                func.add_block(begin.clone());

                self.generate_block(func, b);

                func.assign_instr(
                    tmp.clone(),
                    qbe::Type::Word,
                    qbe::Instr::Load(qbe::Type::Word, self.generate_ptr()),
                );
                func.add_instr(qbe::Instr::Jnz(tmp.clone(), begin.clone(), end.clone()));
                func.add_block(end.clone());
            }
        }
    }

    fn generate_runtime(&mut self, func: &mut qbe::Function) {
        let tape_val = qbe::Value::Temporary("tape".to_string());

        func.assign_instr(
            tape_val.clone(),
            qbe::Type::Long,
            qbe::Instr::Alloc8(self.tape_len as u64),
        );

        func.assign_instr(
            self.generate_ptr(),
            qbe::Type::Long,
            qbe::Instr::Copy(tape_val.clone()),
        );
    }

    fn generate_bounds_check(&mut self, func: &mut qbe::Function) {
        let tape_val = qbe::Value::Temporary("tape".to_string());

        let cont = self.generate_label("cont");
        let halt = self.generate_label("halt");
        let pred = self.generate_tmp();

        let offset = self.generate_tmp();
        func.assign_instr(
            offset.clone(),
            qbe::Type::Long,
            qbe::Instr::Sub(self.generate_ptr(), tape_val),
        );

        let in_bounds = self.generate_tmp();
        func.assign_instr(
            in_bounds.clone(),
            qbe::Type::Long,
            qbe::Instr::Cmp(
                qbe::Type::Long,
                qbe::Cmp::Sgt,
                qbe::Value::Const(self.tape_len as u64 / 2),
                offset.clone(),
            ),
        );

        func.add_instr(qbe::Instr::Jnz(
            in_bounds.clone(),
            cont.clone(),
            halt.clone(),
        ));

        func.add_block(halt.clone());
        func.add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(1))));
        func.add_block(cont.clone());
    }

    fn generate_ptr(&mut self) -> qbe::Value {
        qbe::Value::Temporary("ptr".to_string())
    }

    fn generate_tmp(&mut self) -> qbe::Value {
        let c = self.tmp_counter;
        self.tmp_counter += 1;
        qbe::Value::Temporary(format!("v{}", c))
    }

    fn generate_label(&mut self, prefix: &str) -> String {
        let c = self.label_counter;
        self.label_counter += 1;
        format!("{}{}", prefix, c)
    }
}
