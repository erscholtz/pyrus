use crate::hir::hir_util::style_resolver::resolve_styles;
use crate::hir::HIRPass;

impl HIRPass {
    /// Run the CSS style resolution pass on the HLIR module
    pub fn style_pass(&mut self, hlir: &mut crate::hir::ir_types::HIRModule) {
        resolve_styles(hlir);
    }
}
