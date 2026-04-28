use std::collections::HashMap;

use crate::hir::hir_types::{FuncBlock, FuncDecl, Global, Op};
use crate::hir::{FuncId, GlobalId, HIRModule};

impl HIRModule {
    pub fn validate(&self, hir: &HIRModule) -> Result<(), String> {
        self.validate_globals(&hir.globals);
        self.validate_functions(&hir.functions);

        Ok(())
    }

    fn validate_globals(&self, globals: &HashMap<GlobalId, Global>) {}

    fn validate_functions(&self, functions: &HashMap<FuncId, FuncDecl>) {}
}
