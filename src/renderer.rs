use crate::simulation::Simulation;
use js_sys::Float32Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext,
};

#[wasm_bindgen(inline_js = r#"
const FLOATS_PER_INSTANCE = 12;
const BYTES_PER_INSTANCE = FLOATS_PER_INSTANCE * 4;

const shaderSource = `
struct Uniforms {
  resolution: vec2<f32>,
  time: f32,
  kind: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOut {
  @builtin(position) position: vec4<f32>,
  @location(0) local: vec2<f32>,
  @location(1) hue: f32,
  @location(2) energy: f32,
  @location(3) state: f32,
  @location(4) traits: vec4<f32>,
  @location(5) kind: f32,
};

fn hslToRgb(hue: f32, saturation: f32, lightness: f32) -> vec3<f32> {
  let c = (1.0 - abs(2.0 * lightness - 1.0)) * saturation;
  let h = hue / 60.0;
  let x = c * (1.0 - abs((h % 2.0) - 1.0));
  var rgb = vec3<f32>(0.0);
  if (h < 1.0) {
    rgb = vec3<f32>(c, x, 0.0);
  } else if (h < 2.0) {
    rgb = vec3<f32>(x, c, 0.0);
  } else if (h < 3.0) {
    rgb = vec3<f32>(0.0, c, x);
  } else if (h < 4.0) {
    rgb = vec3<f32>(0.0, x, c);
  } else if (h < 5.0) {
    rgb = vec3<f32>(x, 0.0, c);
  } else {
    rgb = vec3<f32>(c, 0.0, x);
  }
  let m = lightness - c * 0.5;
  return rgb + vec3<f32>(m);
}

fn stateTint(state: f32) -> vec3<f32> {
  if (state < 0.5) {
    return vec3<f32>(0.50, 0.78, 0.92);
  }
  if (state < 1.5) {
    return vec3<f32>(1.00, 0.26, 0.18);
  }
  if (state < 2.5) {
    return vec3<f32>(0.48, 1.00, 0.52);
  }
  if (state < 3.5) {
    return vec3<f32>(1.00, 0.78, 0.30);
  }
  if (state < 4.5) {
    return vec3<f32>(1.00, 0.42, 0.10);
  }
  return vec3<f32>(0.40, 0.82, 1.00);
}

@vertex
fn vs_main(
  @builtin(vertex_index) vertexIndex: u32,
  @location(0) center: vec2<f32>,
  @location(1) velocity: vec2<f32>,
  @location(2) size: f32,
  @location(3) hue: f32,
  @location(4) energy: f32,
  @location(5) state: f32,
  @location(6) traits: vec4<f32>,
) -> VertexOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
  );

  let local = corners[vertexIndex];
  let isAgent = uniforms.kind > 0.5;
  let speed = length(velocity);
  var direction = vec2<f32>(1.0, 0.0);
  if (speed > 0.0001) {
    direction = normalize(velocity);
  }
  let tangent = vec2<f32>(-direction.y, direction.x);

  let aggression = traits.z;
  let stealth = traits.w;
  let lengthScale = select(1.0, 1.20 + aggression * 0.55 + min(speed * 0.025, 0.40), isAgent);
  let widthScale = select(1.0, 0.72 + energy * 0.26 - stealth * 0.16, isAgent);
  let resourceBreath = 1.0 + 0.08 * sin(uniforms.time * 1.8 + center.x * 0.017 + center.y * 0.011);
  let agentBreath = 1.0 + 0.04 * sin(uniforms.time * 2.4 + hue * 0.03);
  let breath = select(resourceBreath, agentBreath, isAgent);

  let worldOffset =
    direction * local.x * size * lengthScale * breath +
    tangent * local.y * size * widthScale * breath;
  let position = center + worldOffset;
  let clip = (position / uniforms.resolution) * 2.0 - 1.0;

  var out: VertexOut;
  out.position = vec4<f32>(clip.x, -clip.y, 0.0, 1.0);
  out.local = vec2<f32>(local.x / lengthScale, local.y / widthScale);
  out.hue = hue;
  out.energy = energy;
  out.state = state;
  out.traits = traits;
  out.kind = uniforms.kind;
  return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
  let dist = length(in.local);
  if (dist > 1.0) {
    discard;
  }

  let edge = 1.0 - smoothstep(0.78, 1.0, dist);
  let membrane = smoothstep(0.98, 0.72, dist) * smoothstep(0.42, 0.94, dist);
  let core = 1.0 - smoothstep(0.0, 0.32, dist);
  let organic = 0.5 + 0.5 * sin(in.local.x * 12.0 + in.local.y * 9.0 + uniforms.time * 1.8);

  if (in.kind < 0.5) {
    let spawn = in.traits.x;
    let depletion = in.traits.y;
    let base = hslToRgb(in.hue, 0.58, 0.52 + in.energy * 0.08);
    let glow = hslToRgb(118.0, 0.42, 0.72);
    let color = mix(base, glow, 0.22 + organic * 0.08);
    let alpha = (edge * 0.18 + membrane * 0.22 + core * 0.18) * (1.0 - depletion * 0.72) * (0.55 + spawn * 0.45);
    return vec4<f32>(color, alpha);
  }

  let spawn = in.traits.x;
  let death = in.traits.y;
  let aggression = in.traits.z;
  let stealth = in.traits.w;
  let geneColor = hslToRgb(in.hue, 0.72, 0.45 + in.energy * 0.18);
  let behaviorColor = stateTint(in.state);
  let shell = hslToRgb(in.hue + 18.0, 0.52, 0.76);
  var color = mix(geneColor, behaviorColor, 0.34 + aggression * 0.18);
  color = mix(color, shell, membrane * 0.28);
  color += core * vec3<f32>(0.24, 0.22, 0.16);
  color += organic * 0.035;

  let stateGlow = select(0.12, 0.34, in.state > 0.5);
  let alpha = (edge * 0.42 + membrane * 0.30 + core * 0.26 + stateGlow * (1.0 - dist)) *
    (0.86 - stealth * 0.22) *
    (0.45 + in.energy * 0.55) *
    (1.0 - death * 0.78) *
    (0.60 + spawn * 0.40);

  return vec4<f32>(color, min(alpha, 0.92));
}
`;

function createUniform(device, pipeline) {
  const buffer = device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });
  const bindGroup = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer } }],
  });
  return { buffer, bindGroup };
}

function ensureInstanceBuffer(renderer, slot, count) {
  const required = Math.max(BYTES_PER_INSTANCE, count * BYTES_PER_INSTANCE);
  const existing = renderer[slot];
  if (existing && existing.size >= required) {
    return existing.buffer;
  }
  const size = 2 ** Math.ceil(Math.log2(required));
  const buffer = renderer.device.createBuffer({
    size,
    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
  });
  renderer[slot] = { buffer, size };
  return buffer;
}

export async function createWebGpuRenderer(canvasId) {
  if (!globalThis.navigator?.gpu) {
    throw new Error("WebGPU is not available in this browser");
  }

  const canvas = document.getElementById(canvasId);
  if (!(canvas instanceof HTMLCanvasElement)) {
    throw new Error(`Canvas '${canvasId}' was not found`);
  }

  const adapter = await navigator.gpu.requestAdapter({ powerPreference: "high-performance" });
  if (!adapter) {
    throw new Error("WebGPU adapter request returned no adapter");
  }

  const device = await adapter.requestDevice();
  const context = canvas.getContext("webgpu");
  if (!context) {
    throw new Error("WebGPU canvas context is unavailable");
  }

  const format = navigator.gpu.getPreferredCanvasFormat();
  context.configure({ device, format, alphaMode: "opaque" });

  const module = device.createShaderModule({ label: "battleo organisms", code: shaderSource });
  const instanceLayout = {
    arrayStride: BYTES_PER_INSTANCE,
    stepMode: "instance",
    attributes: [
      { shaderLocation: 0, offset: 0, format: "float32x2" },
      { shaderLocation: 1, offset: 8, format: "float32x2" },
      { shaderLocation: 2, offset: 16, format: "float32" },
      { shaderLocation: 3, offset: 20, format: "float32" },
      { shaderLocation: 4, offset: 24, format: "float32" },
      { shaderLocation: 5, offset: 28, format: "float32" },
      { shaderLocation: 6, offset: 32, format: "float32x4" },
    ],
  };

  const pipeline = device.createRenderPipeline({
    label: "battleo organisms",
    layout: "auto",
    vertex: { module, entryPoint: "vs_main", buffers: [instanceLayout] },
    fragment: {
      module,
      entryPoint: "fs_main",
      targets: [{
        format,
        blend: {
          color: { srcFactor: "src-alpha", dstFactor: "one-minus-src-alpha", operation: "add" },
          alpha: { srcFactor: "one", dstFactor: "one-minus-src-alpha", operation: "add" },
        },
      }],
    },
    primitive: { topology: "triangle-list" },
  });

  const renderer = { canvas, context, device, format, pipeline };
  renderer.resourceUniform = createUniform(device, pipeline);
  renderer.agentUniform = createUniform(device, pipeline);
  return renderer;
}

export function renderWebGpu(renderer, agents, agentCount, resources, resourceCount, time) {
  const { canvas, context, device, pipeline } = renderer;
  const width = canvas.width;
  const height = canvas.height;

  const resourceUniforms = new Float32Array([width, height, time, 0]);
  const agentUniforms = new Float32Array([width, height, time, 1]);
  device.queue.writeBuffer(renderer.resourceUniform.buffer, 0, resourceUniforms);
  device.queue.writeBuffer(renderer.agentUniform.buffer, 0, agentUniforms);

  const resourceBuffer = ensureInstanceBuffer(renderer, "resourceBuffer", resourceCount);
  const agentBuffer = ensureInstanceBuffer(renderer, "agentBuffer", agentCount);
  if (resourceCount > 0) {
    device.queue.writeBuffer(resourceBuffer, 0, resources);
  }
  if (agentCount > 0) {
    device.queue.writeBuffer(agentBuffer, 0, agents);
  }

  const encoder = device.createCommandEncoder({ label: "battleo frame" });
  const view = context.getCurrentTexture().createView();
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      view,
      clearValue: { r: 0.012, g: 0.018, b: 0.034, a: 1 },
      loadOp: "clear",
      storeOp: "store",
    }],
  });

  pass.setPipeline(pipeline);
  if (resourceCount > 0) {
    pass.setBindGroup(0, renderer.resourceUniform.bindGroup);
    pass.setVertexBuffer(0, resourceBuffer);
    pass.draw(6, resourceCount);
  }
  if (agentCount > 0) {
    pass.setBindGroup(0, renderer.agentUniform.bindGroup);
    pass.setVertexBuffer(0, agentBuffer);
    pass.draw(6, agentCount);
  }
  pass.end();
  device.queue.submit([encoder.finish()]);
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = createWebGpuRenderer, catch)]
    fn create_webgpu_renderer(canvas_id: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(js_name = renderWebGpu, catch)]
    fn render_webgpu(
        renderer: &JsValue,
        agents: &Float32Array,
        agent_count: u32,
        resources: &Float32Array,
        resource_count: u32,
        time: f32,
    ) -> Result<(), JsValue>;
}

const INSTANCE_FLOATS: usize = 12;

// ============================================================================
// WEB RENDERER
// ============================================================================

pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    webgpu_renderer: Option<JsValue>,
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
    pub async fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let canvas = Self::canvas_by_id(canvas_id)?;

        match create_webgpu_renderer(canvas_id) {
            Ok(promise) => match JsFuture::from(promise).await {
                Ok(webgpu_renderer) => {
                    web_sys::console::log_1(&"Using WebGPU rendering".into());
                    let canvas_width = canvas.width() as f32;
                    let canvas_height = canvas.height() as f32;

                    Ok(WebRenderer {
                        canvas,
                        webgpu_renderer: Some(webgpu_renderer),
                        ctx_2d: None,
                        gl: None,
                        use_webgl: false,
                        program: None,
                        vertex_buffer: None,
                        canvas_width,
                        canvas_height,
                        start_time: 0.0,
                    })
                }
                Err(error) => {
                    web_sys::console::warn_1(
                        &format!(
                            "WebGPU setup failed, trying WebGL/Canvas2D fallback: {:?}",
                            error
                        )
                        .into(),
                    );
                    Self::new_fallback_from_canvas(canvas)
                }
            },
            Err(error) => {
                web_sys::console::warn_1(
                    &format!(
                        "WebGPU setup failed, trying WebGL/Canvas2D fallback: {:?}",
                        error
                    )
                    .into(),
                );
                Self::new_fallback_from_canvas(canvas)
            }
        }
    }

    pub fn new_fallback(canvas_id: &str) -> Result<Self, JsValue> {
        Self::new_fallback_from_canvas(Self::canvas_by_id(canvas_id)?)
    }

    fn canvas_by_id(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        document
            .get_element_by_id(canvas_id)
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or("Canvas not found".into())
    }

    fn new_fallback_from_canvas(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
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

        let canvas_width = canvas.width() as f32;
        let canvas_height = canvas.height() as f32;

        let mut renderer = WebRenderer {
            canvas,
            webgpu_renderer: None,
            ctx_2d,
            gl: None,
            use_webgl: gl.is_some(),
            program: None,
            vertex_buffer: None,
            canvas_width,
            canvas_height,
            start_time: 0.0,
        };

        if let Some(gl) = &gl {
            gl.enable(WebGlRenderingContext::BLEND);
            gl.blend_func(
                WebGlRenderingContext::SRC_ALPHA,
                WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
            );

            if let Err(error) = renderer.init_webgl_shaders(gl) {
                web_sys::console::warn_1(
                    &format!("WebGL setup failed, trying Canvas2D fallback: {:?}", error).into(),
                );
                renderer.ctx_2d = Some(
                    renderer
                        .canvas
                        .get_context("2d")
                        .map_err(|_| "Failed to get 2D context after WebGL setup failure")?
                        .and_then(|context| context.dyn_into::<CanvasRenderingContext2d>().ok())
                        .ok_or("Canvas2D fallback unavailable after WebGL setup failure")?,
                );
                renderer.use_webgl = false;
            } else {
                renderer.gl = Some(gl.clone());
            }
        }

        if renderer.use_webgl {
            web_sys::console::log_1(&"Using WebGL rendering".into());
        } else {
            web_sys::console::log_1(&"Using Canvas2D rendering".into());
        }

        Ok(renderer)
    }

    pub fn render(&mut self, simulation: &Simulation) {
        if self.webgpu_renderer.is_some() {
            self.render_webgpu(simulation);
        } else if self.use_webgl {
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
        Self::check_shader(gl, &vertex_shader, "vertex")?;

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
        Self::check_shader(gl, &fragment_shader, "fragment")?;

        // Create program
        let program = gl.create_program().ok_or("Failed to create program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);
        Self::check_program(gl, &program)?;

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

    fn check_shader(
        gl: &WebGlRenderingContext,
        shader: &web_sys::WebGlShader,
        label: &str,
    ) -> Result<(), JsValue> {
        if gl
            .get_shader_parameter(shader, WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(format!(
                "{label} shader compile failed: {}",
                gl.get_shader_info_log(shader)
                    .unwrap_or_else(|| "unknown error".to_string())
            )
            .into())
        }
    }

    fn check_program(gl: &WebGlRenderingContext, program: &WebGlProgram) -> Result<(), JsValue> {
        if gl
            .get_program_parameter(program, WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(format!(
                "WebGL program link failed: {}",
                gl.get_program_info_log(program)
                    .unwrap_or_else(|| "unknown error".to_string())
            )
            .into())
        }
    }

    fn render_webgpu(&mut self, simulation: &Simulation) {
        let Some(webgpu_renderer) = &self.webgpu_renderer else {
            return;
        };

        let current_time = js_sys::Date::now() / 1000.0;
        if self.start_time == 0.0 {
            self.start_time = current_time;
        }
        let time = (current_time - self.start_time) as f32;

        let agents = simulation.get_agents();
        let resources = simulation.get_resources();

        let mut agent_data = Vec::with_capacity(agents.len() * INSTANCE_FLOATS);
        for agent in &agents {
            let energy_ratio = (agent.energy / agent.max_energy).clamp(0.0, 1.0) as f32;
            let size = (agent.genes.size * 8.0 * (agent.energy / agent.max_energy).max(0.1)) as f32;
            let death_fade = if agent.is_dying {
                agent.death_fade.clamp(0.0, 1.0) as f32
            } else {
                0.0
            };

            agent_data.extend_from_slice(&[
                agent.x as f32,
                agent.y as f32,
                agent.dx as f32,
                agent.dy as f32,
                size,
                agent.genes.color_hue as f32,
                energy_ratio,
                agent_state_code(&agent.state),
                agent.spawn_fade.clamp(0.0, 1.0) as f32,
                death_fade,
                agent.genes.aggression.clamp(0.0, 1.0) as f32,
                agent.genes.stealth.clamp(0.0, 1.0) as f32,
            ]);
        }

        let mut resource_data = Vec::with_capacity(resources.len() * INSTANCE_FLOATS);
        for resource in &resources {
            let energy_ratio = (resource.energy / resource.max_energy).clamp(0.0, 1.0) as f32;
            let depletion_factor = 1.0 - resource.deplete_fade;
            let size = (resource.size * 3.8 * depletion_factor.max(0.12)) as f32;
            let hue = 92.0 + energy_ratio * 46.0;

            resource_data.extend_from_slice(&[
                resource.x as f32,
                resource.y as f32,
                0.0,
                0.0,
                size,
                hue,
                energy_ratio,
                0.0,
                resource.spawn_fade.clamp(0.0, 1.0) as f32,
                resource.deplete_fade.clamp(0.0, 1.0) as f32,
                0.0,
                0.0,
            ]);
        }

        let agents_array = unsafe { Float32Array::view(&agent_data) };
        let resources_array = unsafe { Float32Array::view(&resource_data) };

        if let Err(error) = render_webgpu(
            webgpu_renderer,
            &agents_array,
            agents.len() as u32,
            &resources_array,
            resources.len() as u32,
            time,
        ) {
            web_sys::console::error_1(&format!("WebGPU render failed: {:?}", error).into());
        }
    }

    fn render_webgl(&mut self, simulation: &Simulation) {
        if let Some(gl) = &self.gl {
            // Update time (simplified for now)
            let current_time = js_sys::Date::now() / 1000.0;

            if self.start_time == 0.0 {
                self.start_time = current_time;
            }

            let time = current_time - self.start_time;

            self.canvas_width = self.canvas.width() as f32;
            self.canvas_height = self.canvas.height() as f32;
            gl.viewport(
                0,
                0,
                self.canvas.width() as i32,
                self.canvas.height() as i32,
            );
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

            for resource in &resources {
                self.render_resource_canvas2d(ctx, resource);
            }

            for agent in &agents {
                self.render_agent_canvas2d(ctx, agent);
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
        let _ = ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill();

        // Draw cell wall (middle ring)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.5)",
            hue,
            saturation,
            lightness + 5.0
        ));
        ctx.begin_path();
        let _ = ctx.arc(x, y, size * 0.85, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill();

        // Draw cytoplasm (inner area)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.4)",
            hue, saturation, lightness
        ));
        ctx.begin_path();
        let _ = ctx.arc(x, y, size * 0.6, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill();

        // Draw nucleus (core)
        ctx.set_fill_style_str(&format!(
            "hsla({}, {}%, {}%, 0.8)",
            hue,
            saturation,
            lightness - 10.0
        ));
        ctx.begin_path();
        let _ = ctx.arc(x, y, size * 0.25, 0.0, 2.0 * std::f64::consts::PI);
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
        let _ = ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill();

        // Inner content
        let inner_opacity = 0.6 * depletion_factor.max(0.1);
        ctx.set_fill_style_str(&format!("hsla({}, 70%, 50%, {})", hue, inner_opacity));
        ctx.begin_path();
        let _ = ctx.arc(x, y, size * 0.7, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill();

        // Core
        let core_opacity = 0.8 * depletion_factor.max(0.1);
        ctx.set_fill_style_str(&format!("hsla({}, 70%, 40%, {})", hue, core_opacity));
        ctx.begin_path();
        let _ = ctx.arc(x, y, size * 0.4, 0.0, 2.0 * std::f64::consts::PI);
        ctx.fill();

        // Membrane border
        let border_opacity = 0.5 * depletion_factor.max(0.1);
        ctx.set_stroke_style_str(&format!("hsla({}, 70%, 70%, {})", hue, border_opacity));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }

    pub fn get_rendering_mode(&self) -> String {
        if self.webgpu_renderer.is_some() {
            "WebGPU".to_string()
        } else if self.use_webgl {
            "WebGL".to_string()
        } else {
            "Canvas2D".to_string()
        }
    }
}

fn agent_state_code(state: &crate::agent::AgentState) -> f32 {
    match state {
        crate::agent::AgentState::Seeking => 0.0,
        crate::agent::AgentState::Hunting => 1.0,
        crate::agent::AgentState::Feeding => 2.0,
        crate::agent::AgentState::Reproducing => 3.0,
        crate::agent::AgentState::Fighting => 4.0,
        crate::agent::AgentState::Fleeing => 5.0,
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
