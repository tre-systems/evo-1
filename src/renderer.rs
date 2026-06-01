use crate::simulation::Simulation;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext,
};

// ============================================================================
// WEB RENDERER
// ============================================================================

pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx_2d: Option<CanvasRenderingContext2d>,
    gl: Option<WebGlRenderingContext>,
    use_webgl: bool,
    // WebGL shaders and buffers
    program: Option<WebGlProgram>,
    vertex_buffer: Option<WebGlBuffer>,
    canvas_width: f32,
    canvas_height: f32,
    // Animation time
    start_time: f64,
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

        // Log which rendering method is being used
        if use_webgl {
            web_sys::console::log_1(&"🎮 Using WebGL rendering".into());
        } else {
            web_sys::console::log_1(&"🎨 Using Canvas 2D rendering (WebGL not available)".into());
        }

        let canvas_width = canvas.width() as f32;
        let canvas_height = canvas.height() as f32;

        let mut renderer = WebRenderer {
            canvas,
            ctx_2d,
            gl: None,
            use_webgl,
            program: None,
            vertex_buffer: None,
            canvas_width,
            canvas_height,
            start_time: 0.0, // Will be set properly in render
        };

        // Initialize WebGL shaders if WebGL is available
        if use_webgl {
            if let Some(gl) = &gl {
                // Enable blending for glow effects
                gl.enable(WebGlRenderingContext::BLEND);
                gl.blend_func(
                    WebGlRenderingContext::SRC_ALPHA,
                    WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
                );

                renderer.init_webgl_shaders(gl)?;
                renderer.gl = Some(gl.clone());
            }
        }

        Ok(renderer)
    }

    pub fn render(&mut self, simulation: &Simulation) {
        if self.use_webgl {
            self.render_webgl(simulation);
        } else {
            self.render_canvas2d(simulation);
        }
    }

    fn init_webgl_shaders(&mut self, gl: &WebGlRenderingContext) -> Result<(), JsValue> {
        // Vertex shader - simple pass-through
        let vertex_shader = gl
            .create_shader(WebGlRenderingContext::VERTEX_SHADER)
            .ok_or("Failed to create vertex shader")?;
        gl.shader_source(
            &vertex_shader,
            r#"
            attribute vec2 a_position;
            varying vec2 v_position;
            varying vec2 v_uv;
            uniform vec2 u_resolution;
            uniform vec2 u_position;
            uniform float u_size;
            uniform float u_time;

            void main() {
                vec2 position = a_position * u_size + u_position;
                vec2 clipSpace = (position / u_resolution) * 2.0 - 1.0;
                gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
                v_position = a_position;
                v_uv = a_position * 0.5 + 0.5; // Convert to 0-1 range
            }
        "#,
        );
        gl.compile_shader(&vertex_shader);

        // Fragment shader
        let fragment_shader = gl
            .create_shader(WebGlRenderingContext::FRAGMENT_SHADER)
            .ok_or("Failed to create fragment shader")?;
        gl.shader_source(
            &fragment_shader,
            r#"
            precision mediump float;
            varying vec2 v_position;
            varying vec2 v_uv;
            uniform vec3 u_color;
            uniform float u_time;
            uniform float u_size;

            void main() {
                // Calculate distance from center
                float dist = length(v_position);

                // Create cell membrane (outer boundary)
                float membrane = 1.0 - smoothstep(0.85, 0.95, dist);
                membrane = pow(membrane, 1.5);

                // Create cell wall (thicker boundary)
                float cell_wall = 1.0 - smoothstep(0.75, 0.9, dist);
                cell_wall = pow(cell_wall, 2.0) * 0.7;

                // Create nucleus (inner core)
                float nucleus = 1.0 - smoothstep(0.0, 0.25, dist);
                nucleus = pow(nucleus, 3.0);

                // Create cytoplasm (middle layer)
                float cytoplasm = 1.0 - smoothstep(0.2, 0.6, dist);
                cytoplasm = pow(cytoplasm, 1.8) * 0.4;

                // Add organic pulsing (breathing effect)
                float breath = 0.9 + 0.1 * sin(u_time * 2.0);

                // Create membrane ripple effect
                float ripple = sin(dist * 15.0 - u_time * 4.0) * 0.1;
                ripple = max(0.0, ripple);

                // Add some organic noise for texture
                float noise = sin(v_uv.x * 8.0 + u_time) * sin(v_uv.y * 8.0 + u_time * 0.7);
                noise = noise * 0.05;

                // Combine cell layers with translucency
                vec3 cell_color = u_color * breath;
                vec3 membrane_color = mix(cell_color, vec3(1.0), 0.3); // Slightly lighter membrane
                vec3 nucleus_color = mix(cell_color, vec3(1.0), 0.2); // Brighter nucleus

                // Build the cell from inside out
                vec3 final_color = nucleus_color * nucleus * 0.8;
                final_color += cell_color * cytoplasm * 0.6;
                final_color += membrane_color * cell_wall * 0.7;
                final_color += membrane_color * membrane * 0.9;

                // Add organic effects
                final_color += ripple + noise;

                // Create translucent alpha with cell-like transparency
                float alpha = membrane * 0.8 + cell_wall * 0.6 + cytoplasm * 0.4 + nucleus * 0.9;
                alpha = min(alpha, 0.85); // Keep some transparency

                gl_FragColor = vec4(final_color, alpha);
            }
        "#,
        );
        gl.compile_shader(&fragment_shader);

        // Create program
        let program = gl.create_program().ok_or("Failed to create program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        // Create vertex buffer for circle
        let vertex_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

        // Generate circle vertices (position only, colors will be uniforms)
        let mut vertices = Vec::new();
        let segments = 16;
        for i in 0..segments {
            let angle1 = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let angle2 = ((i + 1) as f32) * 2.0 * std::f32::consts::PI / (segments as f32);

            // Center vertex
            vertices.extend_from_slice(&[0.0, 0.0]);
            // Edge vertices
            vertices.extend_from_slice(&[angle1.cos(), angle1.sin()]);
            vertices.extend_from_slice(&[angle2.cos(), angle2.sin()]);
        }

        unsafe {
            let vertex_array = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vertex_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        self.program = Some(program);
        self.vertex_buffer = Some(vertex_buffer);

        Ok(())
    }

    fn render_webgl(&mut self, simulation: &Simulation) {
        if let Some(gl) = &self.gl {
            // Update time (simplified for now)
            let current_time = js_sys::Date::now() / 1000.0;

            if self.start_time == 0.0 {
                self.start_time = current_time;
            }

            let time = current_time - self.start_time;

            // Clear the canvas with a more organic, cell-friendly background
            gl.clear_color(0.02, 0.03, 0.08, 1.0);
            gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

            // Get simulation data
            let agents = simulation.get_agents();
            let resources = simulation.get_resources();

            // Render resources first (background)
            for resource in &resources {
                self.render_resource_webgl(gl, resource, time);
            }

            // Render agents on top (foreground)
            for agent in &agents {
                self.render_agent_webgl(gl, agent, time);
            }
        }
    }

    fn render_canvas2d(&mut self, simulation: &Simulation) {
        if let Some(ctx) = &self.ctx_2d {
            // Clear the canvas
            ctx.set_fill_style_str("#1a1a2e");
            ctx.fill_rect(
                0.0,
                0.0,
                self.canvas.width() as f64,
                self.canvas.height() as f64,
            );

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

    fn render_agent_webgl(
        &self,
        gl: &WebGlRenderingContext,
        agent: &crate::agent::Agent,
        time: f64,
    ) {
        if let (Some(program), Some(vertex_buffer)) = (&self.program, &self.vertex_buffer) {
            let x = agent.x as f32;
            let y = agent.y as f32;
            // Size should be based on both genes and current energy level
            let energy_ratio = (agent.energy / agent.max_energy) as f32;
            let size = agent.genes.size as f32 * 8.0 * energy_ratio.max(0.1); // Minimum 10% size to stay visible

            // Convert agent color to RGB - adjust brightness based on energy
            let hue = agent.genes.color_hue as f32;
            let lightness = 60.0 * energy_ratio.max(0.3); // Dimmer when low energy
            let (r, g, b) = hsl_to_rgb(hue, 70.0, lightness);

            // Use shader program
            gl.use_program(Some(program));

            // Set uniforms
            let resolution_location = gl.get_uniform_location(program, "u_resolution");
            if let Some(loc) = resolution_location {
                gl.uniform2f(Some(&loc), self.canvas_width, self.canvas_height);
            }

            let position_location = gl.get_uniform_location(program, "u_position");
            if let Some(loc) = position_location {
                gl.uniform2f(Some(&loc), x, y);
            }

            let size_location = gl.get_uniform_location(program, "u_size");
            if let Some(loc) = size_location {
                gl.uniform1f(Some(&loc), size);
            }

            let time_location = gl.get_uniform_location(program, "u_time");
            if let Some(loc) = time_location {
                gl.uniform1f(Some(&loc), time as f32);
            }

            // Bind vertex buffer
            gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(vertex_buffer));

            // Set up attributes (only position now, colors are uniforms)
            let position_location = gl.get_attrib_location(program, "a_position") as u32;
            gl.enable_vertex_attrib_array(position_location);
            gl.vertex_attrib_pointer_with_i32(
                position_location,
                2,
                WebGlRenderingContext::FLOAT,
                false,
                8,
                0,
            );

            // Set color uniform
            let color_location = gl.get_uniform_location(program, "u_color");
            if let Some(loc) = color_location {
                gl.uniform3f(Some(&loc), r, g, b);
            }

            // Draw the circle
            gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 48); // 16 segments * 3 vertices per triangle
        }
    }

    fn render_resource_webgl(
        &self,
        gl: &WebGlRenderingContext,
        resource: &crate::resource::Resource,
        time: f64,
    ) {
        if let (Some(program), Some(vertex_buffer)) = (&self.program, &self.vertex_buffer) {
            let x = resource.x as f32;
            let y = resource.y as f32;

            // Size should be based on both energy and depletion fade
            let energy_ratio = (resource.energy / resource.max_energy) as f32;
            let depletion_factor = (1.0 - resource.deplete_fade) as f32;
            let size = resource.size as f32 * 6.0 * depletion_factor.max(0.1); // Shrink as depleting

            // Convert resource energy to color
            let (r, g, b) = hsl_to_rgb(energy_ratio * 120.0, 70.0, 60.0);

            // Use shader program
            gl.use_program(Some(program));

            // Set uniforms
            let resolution_location = gl.get_uniform_location(program, "u_resolution");
            if let Some(loc) = resolution_location {
                gl.uniform2f(Some(&loc), self.canvas_width, self.canvas_height);
            }

            let position_location = gl.get_uniform_location(program, "u_position");
            if let Some(loc) = position_location {
                gl.uniform2f(Some(&loc), x, y);
            }

            let size_location = gl.get_uniform_location(program, "u_size");
            if let Some(loc) = size_location {
                gl.uniform1f(Some(&loc), size);
            }

            let time_location = gl.get_uniform_location(program, "u_time");
            if let Some(loc) = time_location {
                gl.uniform1f(Some(&loc), time as f32);
            }

            // Bind vertex buffer
            gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(vertex_buffer));

            // Set up attributes (only position now, colors are uniforms)
            let position_location = gl.get_attrib_location(program, "a_position") as u32;
            gl.enable_vertex_attrib_array(position_location);
            gl.vertex_attrib_pointer_with_i32(
                position_location,
                2,
                WebGlRenderingContext::FLOAT,
                false,
                8,
                0,
            );

            // Set color uniform
            let color_location = gl.get_uniform_location(program, "u_color");
            if let Some(loc) = color_location {
                gl.uniform3f(Some(&loc), r, g, b);
            }

            // Draw the circle
            gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 48); // 16 segments * 3 vertices per triangle
        }
    }

    fn render_agent_canvas2d(&self, ctx: &CanvasRenderingContext2d, agent: &crate::agent::Agent) {
        let x = agent.x;
        let y = agent.y;
        // Size should be based on both genes and current energy level
        let energy_ratio = agent.energy / agent.max_energy;
        let size = agent.genes.size * 8.0 * energy_ratio.max(0.1); // Minimum 10% size to stay visible

        // Convert agent color to HSL - adjust brightness based on energy
        let hue = agent.genes.color_hue;
        let saturation = 70.0;
        let lightness = 60.0 * energy_ratio.max(0.3); // Dimmer when low energy

        // Draw cell membrane (outer ring)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.3)",
            hue,
            saturation,
            lightness + 10.0
        ));
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Draw cell wall (middle ring)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.5)",
            hue,
            saturation,
            lightness + 5.0
        ));
        ctx.begin_path();
        ctx.arc(x, y, size * 0.85, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Draw cytoplasm (inner area)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.4)",
            hue, saturation, lightness
        ));
        ctx.begin_path();
        ctx.arc(x, y, size * 0.6, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Draw nucleus (core)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.8)",
            hue,
            saturation,
            lightness - 10.0
        ));
        ctx.begin_path();
        ctx.arc(x, y, size * 0.25, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Draw membrane border
        ctx.set_stroke_style_str(&format!(
            "hsla({}, {}%, {}%, 0.6)",
            hue,
            saturation,
            lightness + 15.0
        ));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }

    fn render_resource_canvas2d(
        &self,
        ctx: &CanvasRenderingContext2d,
        resource: &crate::resource::Resource,
    ) {
        let x = resource.x;
        let y = resource.y;

        // Size should be based on both energy and depletion fade
        let energy_ratio = resource.energy / resource.max_energy;
        let depletion_factor = 1.0 - resource.deplete_fade;
        let size = resource.size * 6.0 * depletion_factor.max(0.1); // Shrink as depleting

        // Convert resource energy to color
        let hue = energy_ratio * 120.0;

        // Draw resource as a cell-like structure
        // Outer membrane
        let opacity = 0.4 * depletion_factor.max(0.1);
        ctx.set_fill_style_str(&format!("hsla({}, 70%, 60%, {})", hue, opacity));
        ctx.begin_path();
        ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Inner content
        let inner_opacity = 0.6 * depletion_factor.max(0.1);
        ctx.set_fill_style_str(&format!("hsla({}, 70%, 50%, {})", hue, inner_opacity));
        ctx.begin_path();
        ctx.arc(x, y, size * 0.7, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Core
        let core_opacity = 0.8 * depletion_factor.max(0.1);
        ctx.set_fill_style_str(&format!("hsla({}, 70%, 40%, {})", hue, core_opacity));
        ctx.begin_path();
        ctx.arc(x, y, size * 0.4, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap();
        ctx.fill();

        // Membrane border
        let border_opacity = 0.5 * depletion_factor.max(0.1);
        ctx.set_stroke_style_str(&format!("hsla({}, 70%, 70%, {})", hue, border_opacity));
        ctx.set_line_width(2.0);
        ctx.stroke();
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

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}
