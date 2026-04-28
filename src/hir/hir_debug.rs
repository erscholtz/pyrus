//! Compact debug printing for HIR
//!
//! This module provides human-readable formatting for HIR structures
//! that's much more compact than the default Debug output.

use std::fmt;

use crate::hir::hir_types::*;

/// Trait for types that can be printed in a compact HIR format
pub trait HirDebug {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl HirDebug for HIRModule {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== HIR Module: {} ===", self.file)?;

        // Globals (only if non-empty)
        if !self.globals.is_empty() {
            writeln!(f)?;
            writeln!(f, "globals:")?;
            for (id, global) in &self.globals {
                writeln!(
                    f,
                    "  {}: {} = {} {}",
                    id.hir_string(),
                    global.name,
                    global.literal.hir_string(),
                    if global.mutable { "(mut)" } else { "" }
                )?;
            }
        }

        // Functions
        if !self.functions.is_empty() {
            writeln!(f)?;
            writeln!(f, "functions:")?;
            for (id, func) in &self.functions {
                write!(f, "  {} ", id.hir_string())?;
                func.hir_fmt(f)?;
            }
        }

        // Element declarations
        if !self.element_decls.is_empty() {
            writeln!(f)?;
            writeln!(f, "element declarations:")?;
            for (id, decl) in &self.element_decls {
                write!(f, "  {} ", id.hir_string())?;
                decl.hir_fmt(f)?;
            }
        }

        // Elements
        if !self.elements.is_empty() {
            writeln!(f)?;
            writeln!(f, "elements:")?;
            for (i, elem) in self.elements.iter().enumerate() {
                write!(f, "  [{:2}] ", i)?;
                elem.hir_fmt(f)?;
                writeln!(f)?;
            }
        }

        // Element metadata (optional, only shown if needed for debugging)
        if !self.element_metadata.is_empty() && f.alternate() {
            writeln!(f)?;
            writeln!(f, "element metadata:")?;
            for (i, meta) in self.element_metadata.iter().enumerate() {
                writeln!(
                    f,
                    "  [{:2}] type={:?} id={:?} classes={:?} parent={:?} attrs=#{}",
                    i, meta.element_type, meta.id, meta.classes, meta.parent, meta.attributes_ref
                )?;
            }
        }

        Ok(())
    }
}

impl HirDebug for FuncDecl {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = self.args.iter().map(|t| format!("{:?}", t)).collect();
        let ret = match &self.return_type {
            Some(t) => format!("{:?}", t),
            None => "void".to_string(),
        };

        writeln!(f, "{}({}) -> {}:", self.name, args.join(", "), ret)?;

        if self.body.ops.is_empty() {
            writeln!(f, "    (empty body)")?;
        } else {
            for (i, op) in self.body.ops.iter().enumerate() {
                write!(f, "    {:4} | ", i)?;
                op.hir_fmt(f)?;
                writeln!(f)?;
            }
        }

        if let Some(ret_ref) = self.body.returned_element_ref {
            writeln!(f, "    (returns element#{})", ret_ref)?;
        }

        Ok(())
    }
}

impl HirDebug for HirElementDecl {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = self.args.iter().map(|t| format!("{:?}", t)).collect();
        writeln!(f, "{}({})", self.name, args.join(", "))?;
        writeln!(f, "    body ops: {}", self.body.ops.len())?;
        for (i, op) in self.body.ops.iter().enumerate() {
            write!(f, "      {:2} | ", i)?;
            op.hir_fmt(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl HirDebug for Op {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Const {
                result,
                name,
                literal,
                ty,
            } => {
                write!(
                    f,
                    "{} = const {} {} {}",
                    result.hir_string(),
                    name,
                    literal.hir_string(),
                    ty.hir_string()
                )
            }
            Op::Var {
                result,
                name,
                literal,
                ty,
            } => {
                write!(
                    f,
                    "{} = var {} {} {}",
                    result.hir_string(),
                    name,
                    literal.hir_string(),
                    ty.hir_string()
                )
            }
            Op::Binary {
                result,
                op,
                lhs,
                rhs,
            } => {
                write!(
                    f,
                    "{} = {} {} {}",
                    result.hir_string(),
                    lhs.hir_string(),
                    op.hir_string(),
                    rhs.hir_string()
                )
            }
            Op::FuncCall { result, func, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.hir_string()).collect();
                match result {
                    Some(r) => write!(
                        f,
                        "{} = call {}({})",
                        r.hir_string(),
                        func.hir_string(),
                        args_str.join(", ")
                    ),
                    None => write!(f, "call {}({})", func.hir_string(), args_str.join(", ")),
                }
            }
            Op::ElementCall {
                result,
                element,
                args,
            } => {
                let args_str: Vec<String> = args.iter().map(|a| a.hir_string()).collect();
                write!(
                    f,
                    "{} = elem_call {}({})",
                    result.hir_string(),
                    element.hir_string(),
                    args_str.join(", ")
                )
            }
            Op::Return { doc_element_ref } => {
                write!(f, "return elem#{}", doc_element_ref)
            }
            Op::HirElementEmit { index } => {
                write!(f, "emit elem#{}", index)
            }
            Op::StringConcat { result, parts } => {
                let parts_str: Vec<String> = parts.iter().map(|p| p.hir_string()).collect();
                write!(
                    f,
                    "{} = concat [{}]",
                    result.hir_string(),
                    parts_str.join(", ")
                )
            }
        }
    }
}

impl HirDebug for HirElementOp {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HirElementOp::Section {
                children,
                attributes,
            } => {
                write!(f, "section children={:?} attrs=#{}", children, attributes)
            }
            HirElementOp::List {
                children,
                attributes,
            } => {
                write!(f, "list children={:?} attrs=#{}", children, attributes)
            }
            HirElementOp::Text {
                content,
                attributes,
            } => {
                let truncated = truncate_str(content, 40);
                write!(f, "text {:?} attrs=#{}", truncated, attributes)
            }
            HirElementOp::Image { src, attributes } => {
                write!(f, "image src={:?} attrs=#{}", src, attributes)
            }
            HirElementOp::Table { table, attributes } => {
                write!(
                    f,
                    "table rows={} cols={} attrs=#{}",
                    table.len(),
                    table.first().map(|r| r.len()).unwrap_or(0),
                    attributes
                )
            }
        }
    }
}

impl HirDebug for FuncId {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hir_string())
    }
}

impl HirDebug for GlobalId {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hir_string())
    }
}

impl HirDebug for ValueId {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hir_string())
    }
}

impl HirDebug for ElementId {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hir_string())
    }
}

impl HirDebug for Literal {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::String(s) => write!(f, "{:?}", truncate_str(s, 30)),
            Literal::Color(c) => write!(f, "color({})", c),
        }
    }
}

impl HirDebug for Type {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hir_string())
    }
}

impl HirDebug for BinOp {
    fn hir_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hir_string())
    }
}

// Helper trait to get compact string representations
trait HirString {
    fn hir_string(&self) -> String;
}

impl HirString for FuncId {
    fn hir_string(&self) -> String {
        format!("func#{}", self.0)
    }
}

impl HirString for GlobalId {
    fn hir_string(&self) -> String {
        format!("global#{}", self.0)
    }
}

impl HirString for ValueId {
    fn hir_string(&self) -> String {
        format!("val#{}", self.0)
    }
}

impl HirString for ElementId {
    fn hir_string(&self) -> String {
        format!("elem#{}", self.0)
    }
}

impl HirString for Type {
    fn hir_string(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "str".to_string(),
            Type::Color => "color".to_string(),
            Type::DocElement => "doc".to_string(),
        }
    }
}

impl HirString for BinOp {
    fn hir_string(&self) -> String {
        match self {
            BinOp::Add => "+".to_string(),
            BinOp::Sub => "-".to_string(),
            BinOp::Mul => "*".to_string(),
            BinOp::Div => "/".to_string(),
            BinOp::Eq => "==".to_string(),
        }
    }
}

impl HirString for Literal {
    fn hir_string(&self) -> String {
        match self {
            Literal::Int(i) => i.to_string(),
            Literal::Float(fl) => fl.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::String(s) => format!("{:?}", truncate_str(s, 30)),
            Literal::Color(c) => format!("color({})", c),
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// Wrapper types for use with format!()

pub struct HirDisplay<'a, T: HirDebug + ?Sized>(pub &'a T);

impl<'a, T: HirDebug + ?Sized> fmt::Display for HirDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hir_fmt(f)
    }
}

impl<'a, T: HirDebug + ?Sized> fmt::Debug for HirDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hir_fmt(f)
    }
}

/// Extension trait to easily get HirDisplay wrapper
pub trait HirDisplayExt {
    fn hir_display(&self) -> HirDisplay<'_, Self>
    where
        Self: HirDebug;
}

impl<T: HirDebug> HirDisplayExt for T {
    fn hir_display(&self) -> HirDisplay<'_, Self> {
        HirDisplay(self)
    }
}
