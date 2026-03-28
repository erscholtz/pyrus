use crate::backend::render::pdf::pdf_renderer::PdfRenderer;
use crate::hir::HIRModule;
use crate::layout::{ComputedLayout, LayoutEngine};

pub enum Renderer {
    Pdf,
    Epub,
    Wasm,
}

pub struct Backend {
    pub renderer: Renderer,
}

impl Backend {
    pub fn new(renderer: Renderer) -> Self {
        Self { renderer }
    }

    pub fn render(
        &self,
        hlir: HIRModule,
        _layout: &LayoutEngine,
        computed_layouts: &[ComputedLayout],
    ) -> Result<(), std::io::Error> {
        match self.renderer {
            Renderer::Pdf => {
                let renderer = PdfRenderer::new();
                renderer.render(hlir, computed_layouts)
            }
            Renderer::Epub => todo!(),
            Renderer::Wasm => todo!(),
        }
    }
}
