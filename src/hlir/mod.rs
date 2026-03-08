mod hlir;

pub use hlir::lower;

mod ir_types;
mod util;

pub use ir_types::{
    ElementMetadata, Func, FuncId, HLIRModule, HlirElement, Id, Literal, Op, StyleAttributes, Type,
};
pub use util::assign_func;
pub use util::assign_vars;
pub use util::style_resolver::resolve_styles;
