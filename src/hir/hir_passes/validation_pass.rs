use std::collections::HashMap;

use crate::hlir::ir_types::{Func, FuncBlock, Global, Id, Op};
use crate::hlir::HLIRModule;

impl HLIRModule {
    pub fn validate(&self, hlir: &HLIRModule) -> Result<(), String> {
        self.validate_globals(&hlir.globals);
        self.validate_functions(&hlir.functions);

        Ok(())
    }

    fn validate_globals(&self, globals: &HashMap<Id, Global>) {}

    fn validate_functions(&self, functions: &HashMap<Id, Func>) {}
}
