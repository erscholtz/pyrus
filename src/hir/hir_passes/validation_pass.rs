use std::collections::HashMap;

use crate::hir::ir_types::{Func, FuncBlock, Global, Id, Op};
use crate::hir::HIRModule;

impl HIRModule {
    pub fn validate(&self, hir: &HIRModule) -> Result<(), String> {
        self.validate_globals(&hir.globals);
        self.validate_functions(&hir.functions);

        Ok(())
    }

    fn validate_globals(&self, globals: &HashMap<Id, Global>) {}

    fn validate_functions(&self, functions: &HashMap<Id, Func>) {}
}
