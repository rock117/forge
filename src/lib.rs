#![feature(bind_by_move_pattern_guards, try_from)]

mod parser;
mod exec;
mod error;
mod output;

// Reexports
pub use exec::{
    ExecError,
    ExecResult,
    Io,
    DefaultIo,
    Value,
    Scope,
    Obj,
    GlobalScope,
};
pub use error::{
    ForgeResult,
    ForgeError,
};

use std::{
    ops::DerefMut,
    rc::Rc,
};

pub struct EngineBuilder {
    io: Box<dyn Io>,
    global_scope: GlobalScope,
}

impl EngineBuilder {
    pub fn with_io<T: Io + 'static>(mut self, io: T) -> Self {
        self.io = Box::new(io);
        self
    }

    pub fn with_global<T: Into<Value>>(mut self, name: &str, val: T) -> Self {
        self.global_scope.declare_var(name.to_string(), val.into());
        self
    }

    pub fn with_global_custom<T: Obj>(mut self, name: &str, val: T) -> Self {
        self.global_scope.declare_var(name.to_string(), Value::Custom(Rc::new(val)));
        self
    }

    pub fn finish(self) -> Engine {
        Engine {
            io: self.io,
            global_scope: self.global_scope,
        }
    }
}

pub struct Engine {
    io: Box<dyn Io>,
    global_scope: GlobalScope,
}

impl Engine {
    pub fn build() -> EngineBuilder {
        EngineBuilder {
            io: Box::new(DefaultIo),
            global_scope: GlobalScope::empty(),
        }
    }

    pub fn eval(&mut self, expr_str: &str) -> ForgeResult<Value> {
        let mut eval_fn = || {
            let expr = parser::Parser::new(expr_str)?.parse_expr()?;

            // TODO: Remove this
            //expr.print_debug(0);

            Ok(
                self.global_scope.eval_expr(&expr, self.io.deref_mut(), &Rc::new(expr_str.to_string()))
                    .map_err(|err| ForgeError::InSrc(expr_str.to_string(), Box::new(err.into())))?
            )
        };
        eval_fn()
    }

    pub fn exec(&mut self, module: &str) -> ForgeResult<()> {
        let mut exec_fn = || {
            let stmts = parser::Parser::new(module)?.parse_stmts()?;

            for stmt in &stmts {
                // stmt.0.print_debug(0); // TODO: Remove this
                self.global_scope.eval_stmt(&stmt.0, self.io.deref_mut(), &Rc::new(module.to_string()))
                    .map_err(|err| ForgeError::InSrc(module.to_string(), Box::new(err.into())))?;
            }

            Ok(())
        };
        exec_fn()
    }

    pub fn prompt(&mut self, input: &str) -> ForgeResult<Option<Value>> {
        match parser::Parser::new(input)?.parse_stmts() {
            Ok(stmts) => {
                for stmt in &stmts {
                    self.global_scope.eval_stmt(&stmt.0, self.io.deref_mut(), &Rc::new(input.to_string()))?;
                }
                Ok(None)
            },
            Err(stmts_err) => Ok(Some(self.global_scope.eval_expr(
                &parser::Parser::new(input)?.parse_expr().map_err(|err|
                    ForgeError::InSrc(input.to_string(), Box::new(err.max(stmts_err).into()))
                )?,
                self.io.deref_mut(),
                &Rc::new(input.to_string()),
            ).map_err(|err| ForgeError::InSrc(input.to_string(), Box::new(err.into())))?)),
        }
    }

    pub fn global_scope(&self) -> &GlobalScope {
        &self.global_scope
    }

    pub fn take(&mut self, name: &str) -> Option<Value> {
        self.global_scope.take_var(name)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::build().finish()
    }
}
