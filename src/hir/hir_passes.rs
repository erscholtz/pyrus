pub mod collect_config;
pub mod collect_decls;
pub mod collect_invokes;
pub mod collect_layouts;
// pub mod validation_pass;

use crate::ast::Ast;
use crate::diagnostic::CompilerDiagnostic;
use crate::hir::HIR;

/// Represents a pass to be executed on an HIR module.
pub trait HIRPass {
    fn run(
        &mut self,
        hir: &mut HIR,
        ast: &Ast,
    ) -> Result<(), Vec<CompilerDiagnostic>>;
    fn name(&self) -> &'static str;
}

/// Manages a pipeline of HIR passes to be executed on a module.
pub struct PassManager {
    stop_on_error: bool,
    failed: bool,
    executed_passes: Vec<&'static str>,
    failed_passes: Vec<&'static str>,
    errors: Vec<CompilerDiagnostic>,
}

impl PassManager {
    pub fn new(stop_on_error: bool) -> Self {
        Self {
            stop_on_error,
            failed: false,
            failed_passes: Vec::new(),
            executed_passes: Vec::new(),
            errors: Vec::new(),
        }
    }
    pub fn continue_on_error(&mut self) -> &mut Self {
        self.stop_on_error = false;
        self
    }

    pub fn run<P: HIRPass + Default>(
        &mut self,
        hir: &mut HIR,
        ast: &Ast,
    ) -> &mut Self {
        let mut pass = P::default();
        match pass.run(hir, ast) {
            Ok(()) => {}
            Err(errors) => {
                self.failed = true;
                self.failed_passes.push(pass.name());
                self.errors.extend(errors);
                if self.stop_on_error {
                    return self;
                }
            }
        }
        self.executed_passes.push(pass.name());
        self
    }

    pub fn executed_passes(&self) -> &[&'static str] {
        &self.executed_passes
    }

    pub fn failed_passes(&self) -> &[&'static str] {
        &self.failed_passes
    }

    pub fn finished(&self) -> Result<(), Vec<CompilerDiagnostic>> {
        if self.failed {
            Err(self.errors.clone())
        } else {
            Ok(())
        }
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new(true)
    }
}
