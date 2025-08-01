use crate::simulation::Simulation;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebGlRenderingContext};

// ============================================================================
// WEB RENDERER
// ============================================================================

pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx_2d: Option<CanvasRenderingContext2d>,
    gl: Option<WebGlRenderingContext>,
    use_webgl: bool,
}

impl WebRenderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or("Canvas not found")?;

        // Try WebGL first, fallback to Canvas2D
        let gl = canvas
            .get_context("webgl")
            .map_err(|_| "Failed to get WebGL context")?
            .and_then(|context| context.dyn_into::<WebGlRenderingContext>().ok());

        let ctx_2d = if gl.is_none() {
            canvas
                .get_context("2d")
                .map_err(|_| "Failed to get 2D context")?
                .and_then(|context| context.dyn_into::<CanvasRenderingContext2d>().ok())
        } else {
            None
        };

        let use_webgl = gl.is_some();

        Ok(WebRenderer {
            canvas,
            ctx_2d,
            gl,
            use_webgl,
        })
    }

    pub fn render(&mut self, simulation: &Simulation) {
        if self.use_webgl {
            self.render_webgl(simulation);
        } else {
            self.render_canvas2d(simulation);
        }
    }

    fn render_webgl(&mut self, simulation: &Simulation) {
        if let Some(gl) = &self.gl {
            // Clear the canvas
            gl.clear_color(0.1, 0.1, 0.18, 1.0);
            gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

            // Get simulation data
            let agents = simulation.get_agents();
            let resources = simulation.get_resources();

            // Render agents
            for agent in &agents {
                self.render_agent_webgl(gl, agent);
            }

            // Render resources
            for resource in &resources {
                self.render_resource_webgl(gl, resource);
            }
        }
    }

    fn render_canvas2d(&mut self, simulation: &Simulation) {
        if let Some(ctx) = &self.ctx_2d {
            // Clear the canvas
            ctx.set_fill_style(&"#1a1a2e".into());
            ctx.fill_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64);

            // Get simulation data
            let agents = simulation.get_agents();
            let resources = simulation.get_resources();

            // Render agents
            for agent in &agents {
                self.render_agent_canvas2d(ctx, agent);
            }

            // Render resources
            for resource in &resources {
                self.render_resource_canvas2d(ctx, resource);
            }
        }
    }

    fn render_agent_webgl(&self, gl: &WebGlRenderingContext, agent: &crate::agent::Agent) {
        // Simple WebGL rendering - just draw colored rectangles for now
        // In a full implementation, you'd use proper shaders and vertex buffers
        
        let x = agent.x as f32;
        let y = agent.y as f32;
        let size = agent.genes.size as f32 * 3.0;
        
        // Convert agent color to RGB
        let hue = agent.genes.color_hue as f32;
        let (r, g, b) = hsl_to_rgb(hue, 70.0, 60.0);
        
        // Draw agent as a colored rectangle
        gl.enable(WebGlRenderingContext::SCISSOR_TEST);
        gl.scissor(
            (x - size/2.0) as i32,
            (y - size/2.0) as i32,
            size as i32,
            size as i32,
        );
        gl.clear_color(r, g, b, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        gl.disable(WebGlRenderingContext::SCISSOR_TEST);
    }

    fn render_resource_webgl(&self, gl: &WebGlRenderingContext, resource: &crate::resource::Resource) {
        let x = resource.x as f32;
        let y = resource.y as f32;
        let size = resource.size as f32;
        
        // Convert resource energy to color
        let energy_ratio = (resource.energy / resource.max_energy) as f32;
        let (r, g, b) = hsl_to_rgb(energy_ratio * 120.0, 70.0, 60.0);
        
        // Draw resource as a colored rectangle
        gl.enable(WebGlRenderingContext::SCISSOR_TEST);
        gl.scissor(
            (x - size/2.0) as i32,
            (y - size/2.0) as i32,
            size as i32,
            size as i32,
        );
        gl.clear_color(r, g, b, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        gl.disable(WebGlRenderingContext::SCISSOR_TEST);
    }

    fn render_agent_canvas2d(&self, ctx: &CanvasRenderingContext2d, agent: &crate::agent::Agent) {
        let x = agent.x;
        let y = agent.y;
        let size = agent.genes.size * 3.0;
        
        // Convert agent color to HSL
        let hue = agent.genes.color_hue;
        let saturation = 70.0;
        let lightness = 60.0;
        
        // Draw agent
        ctx.set_fill_style(&format!("hsl({}, {}%, {}%)", hue, saturation, lightness).into());
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI).unwrap();
        ctx.fill();
        
        // Draw border
        ctx.set_stroke_style(&"#ffffff".into());
        ctx.set_line_width(1.0);
        ctx.stroke();
    }

    fn render_resource_canvas2d(&self, ctx: &CanvasRenderingContext2d, resource: &crate::resource::Resource) {
        let x = resource.x;
        let y = resource.y;
        let size = resource.size;
        
        // Convert resource energy to color
        let energy_ratio = resource.energy / resource.max_energy;
        let hue = energy_ratio * 120.0;
        
        // Draw resource
        ctx.set_fill_style(&format!("hsl({}, 70%, 60%)", hue).into());
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI).unwrap();
        ctx.fill();
    }

    pub fn get_rendering_mode(&self) -> String {
        if self.use_webgl {
            "WebGL".to_string()
        } else {
            "Canvas2D".to_string()
        }
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0/6.0 {
        (c, x, 0.0)
    } else if h < 2.0/6.0 {
        (x, c, 0.0)
    } else if h < 3.0/6.0 {
        (0.0, c, x)
    } else if h < 4.0/6.0 {
        (0.0, x, c)
    } else if h < 5.0/6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
} 