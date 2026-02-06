use glow::HasContext;

use crate::rain::particles::ParticleSystem;

/// Vertex shader for raindrops (instanced line-segment quads)
const RAINDROP_VERT: &str = r#"#version 300 es
precision highp float;

// Per-vertex: unit quad (2 triangles making a line segment)
layout(location = 0) in vec2 a_quad;

// Per-instance attributes
layout(location = 1) in vec2 a_pos;       // drop position (center-bottom)
layout(location = 2) in vec2 a_vel;       // velocity (for direction)
layout(location = 3) in float a_length;   // drop length
layout(location = 4) in float a_width;    // drop width
layout(location = 5) in float a_alpha;    // alpha
layout(location = 6) in float a_fade_in;  // fade-in factor

uniform mat4 u_projection;

out float v_alpha;
out float v_along; // 0 at top, 1 at bottom of drop

void main() {
    // Direction of the drop (normalized)
    vec2 dir = normalize(a_vel);
    vec2 perp = vec2(-dir.y, dir.x);

    // Build the quad: a_quad.x is [-0.5, 0.5] (width), a_quad.y is [0, 1] (length)
    vec2 offset = perp * a_quad.x * a_width + dir * a_quad.y * a_length;
    vec2 world_pos = a_pos - dir * a_length + offset;

    gl_Position = u_projection * vec4(world_pos, 0.0, 1.0);
    v_alpha = a_alpha * a_fade_in;
    v_along = a_quad.y;
}
"#;

/// Fragment shader for raindrops
const RAINDROP_FRAG: &str = r#"#version 300 es
precision highp float;

uniform vec4 u_color;

in float v_alpha;
in float v_along;

out vec4 frag_color;

void main() {
    // Fade at the top of the drop for a natural look
    float fade = smoothstep(0.0, 0.3, v_along);
    frag_color = vec4(u_color.rgb, u_color.a * v_alpha * fade);
}
"#;

/// Vertex shader for splashes (expanding circles)
const SPLASH_VERT: &str = r#"#version 300 es
precision highp float;

// Per-vertex: unit circle point
layout(location = 0) in vec2 a_quad;

// Per-instance
layout(location = 1) in vec2 a_pos;
layout(location = 2) in float a_radius;
layout(location = 3) in float a_alpha;

uniform mat4 u_projection;

out float v_alpha;
out vec2 v_uv;

void main() {
    vec2 world_pos = a_pos + a_quad * a_radius;
    gl_Position = u_projection * vec4(world_pos, 0.0, 1.0);
    v_alpha = a_alpha;
    v_uv = a_quad;
}
"#;

/// Fragment shader for splashes
const SPLASH_FRAG: &str = r#"#version 300 es
precision highp float;

uniform vec4 u_color;

in float v_alpha;
in vec2 v_uv;

out vec4 frag_color;

void main() {
    float dist = length(v_uv);
    if (dist > 1.0) discard;
    // Ring-like splash: fade toward center and edge
    float ring = smoothstep(0.5, 0.8, dist) * smoothstep(1.0, 0.9, dist);
    frag_color = vec4(u_color.rgb, u_color.a * v_alpha * ring);
}
"#;

/// OpenGL rain renderer
pub struct Renderer {
    gl: glow::Context,

    // Raindrop rendering
    drop_program: glow::Program,
    drop_vao: glow::VertexArray,
    drop_quad_vbo: glow::Buffer,
    drop_instance_vbo: glow::Buffer,
    drop_projection_loc: glow::UniformLocation,
    drop_color_loc: glow::UniformLocation,

    // Splash rendering
    splash_program: glow::Program,
    splash_vao: glow::VertexArray,
    splash_quad_vbo: glow::Buffer,
    splash_instance_vbo: glow::Buffer,
    splash_projection_loc: glow::UniformLocation,
    splash_color_loc: glow::UniformLocation,

    width: f32,
    height: f32,
}

impl Renderer {
    pub fn new(gl: glow::Context, width: f32, height: f32) -> Result<Self, String> {
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.disable(glow::DEPTH_TEST);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.viewport(0, 0, width as i32, height as i32);

            // === Raindrop program ===
            let drop_program = compile_program(&gl, RAINDROP_VERT, RAINDROP_FRAG)?;
            let drop_projection_loc = gl
                .get_uniform_location(drop_program, "u_projection")
                .ok_or("Missing u_projection in drop shader")?;
            let drop_color_loc = gl
                .get_uniform_location(drop_program, "u_color")
                .ok_or("Missing u_color in drop shader")?;

            // Unit quad for a raindrop line segment: 6 vertices (2 triangles)
            // x: [-0.5, 0.5], y: [0, 1]
            #[rustfmt::skip]
            let quad_verts: [f32; 12] = [
                -0.5, 0.0,   0.5, 0.0,   0.5, 1.0,
                -0.5, 0.0,   0.5, 1.0,  -0.5, 1.0,
            ];

            let drop_vao = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(drop_vao));

            let drop_quad_vbo = gl.create_buffer().map_err(|e| e.to_string())?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(drop_quad_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&quad_verts),
                glow::STATIC_DRAW,
            );
            // location 0: a_quad (vec2)
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(0);

            // Instance buffer for drops
            let drop_instance_vbo = gl.create_buffer().map_err(|e| e.to_string())?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(drop_instance_vbo));

            // Per-instance layout: pos(2f) + vel(2f) + length(1f) + width(1f) + alpha(1f) + fade_in(1f) = 8 floats = 32 bytes
            let stride = 32;
            // location 1: a_pos
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_divisor(1, 1);
            // location 2: a_vel
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, 8);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_divisor(2, 1);
            // location 3: a_length
            gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, stride, 16);
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_divisor(3, 1);
            // location 4: a_width
            gl.vertex_attrib_pointer_f32(4, 1, glow::FLOAT, false, stride, 20);
            gl.enable_vertex_attrib_array(4);
            gl.vertex_attrib_divisor(4, 1);
            // location 5: a_alpha
            gl.vertex_attrib_pointer_f32(5, 1, glow::FLOAT, false, stride, 24);
            gl.enable_vertex_attrib_array(5);
            gl.vertex_attrib_divisor(5, 1);
            // location 6: a_fade_in
            gl.vertex_attrib_pointer_f32(6, 1, glow::FLOAT, false, stride, 28);
            gl.enable_vertex_attrib_array(6);
            gl.vertex_attrib_divisor(6, 1);

            gl.bind_vertex_array(None);

            // === Splash program ===
            let splash_program = compile_program(&gl, SPLASH_VERT, SPLASH_FRAG)?;
            let splash_projection_loc = gl
                .get_uniform_location(splash_program, "u_projection")
                .ok_or("Missing u_projection in splash shader")?;
            let splash_color_loc = gl
                .get_uniform_location(splash_program, "u_color")
                .ok_or("Missing u_color in splash shader")?;

            // Unit quad for splashes: circle approximated as quad, discard in frag shader
            #[rustfmt::skip]
            let splash_verts: [f32; 12] = [
                -1.0, -1.0,   1.0, -1.0,   1.0,  1.0,
                -1.0, -1.0,   1.0,  1.0,  -1.0,  1.0,
            ];

            let splash_vao = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(splash_vao));

            let splash_quad_vbo = gl.create_buffer().map_err(|e| e.to_string())?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(splash_quad_vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&splash_verts),
                glow::STATIC_DRAW,
            );
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(0);

            // Splash instance buffer: pos(2f) + radius(1f) + alpha(1f) = 4 floats = 16 bytes
            let splash_instance_vbo = gl.create_buffer().map_err(|e| e.to_string())?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(splash_instance_vbo));
            let s_stride = 16;
            // location 1: a_pos
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, s_stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_divisor(1, 1);
            // location 2: a_radius
            gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, s_stride, 8);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_divisor(2, 1);
            // location 3: a_alpha
            gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, s_stride, 12);
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_divisor(3, 1);

            gl.bind_vertex_array(None);

            let mut renderer = Self {
                gl,
                drop_program,
                drop_vao,
                drop_quad_vbo,
                drop_instance_vbo,
                drop_projection_loc,
                drop_color_loc,
                splash_program,
                splash_vao,
                splash_quad_vbo,
                splash_instance_vbo,
                splash_projection_loc,
                splash_color_loc,
                width,
                height,
            };
            renderer.update_projection();
            Ok(renderer)
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
        self.update_projection();
    }

    fn update_projection(&mut self) {
        // Orthographic projection: (0,0) top-left, (w,h) bottom-right
        let proj = ortho_matrix(0.0, self.width, self.height, 0.0);
        unsafe {
            self.gl.use_program(Some(self.drop_program));
            self.gl
                .uniform_matrix_4_f32_slice(Some(&self.drop_projection_loc), false, &proj);
            self.gl.use_program(Some(self.splash_program));
            self.gl
                .uniform_matrix_4_f32_slice(Some(&self.splash_projection_loc), false, &proj);
        }
    }

    /// Render all particles
    pub fn render(&self, particles: &ParticleSystem) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        let color = particles.color();

        self.render_drops(particles, color);
        self.render_splashes(particles, color);
    }

    fn render_drops(&self, particles: &ParticleSystem, color: [f32; 4]) {
        if particles.drops.is_empty() {
            return;
        }

        // Build instance data: [pos.x, pos.y, vel.x, vel.y, length, width, alpha, fade_in]
        let mut instance_data = Vec::with_capacity(particles.drops.len() * 8);
        for drop in &particles.drops {
            instance_data.push(drop.x);
            instance_data.push(drop.y);
            instance_data.push(drop.vx);
            instance_data.push(drop.vy);
            instance_data.push(drop.length);
            instance_data.push(drop.width);
            instance_data.push(drop.alpha);
            instance_data.push(drop.fade_in);
        }

        unsafe {
            self.gl.use_program(Some(self.drop_program));
            self.gl
                .uniform_4_f32(Some(&self.drop_color_loc), color[0], color[1], color[2], color[3]);

            self.gl.bind_vertex_array(Some(self.drop_vao));
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.drop_instance_vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&instance_data),
                glow::STREAM_DRAW,
            );

            self.gl
                .draw_arrays_instanced(glow::TRIANGLES, 0, 6, particles.drops.len() as i32);
            self.gl.bind_vertex_array(None);
        }
    }

    fn render_splashes(&self, particles: &ParticleSystem, color: [f32; 4]) {
        if particles.splashes.is_empty() {
            return;
        }

        // Build instance data: [pos.x, pos.y, radius, alpha]
        let mut instance_data = Vec::with_capacity(particles.splashes.len() * 4);
        for splash in &particles.splashes {
            instance_data.push(splash.x);
            instance_data.push(splash.y);
            instance_data.push(splash.radius);
            instance_data.push(splash.alpha);
        }

        unsafe {
            self.gl.use_program(Some(self.splash_program));
            self.gl.uniform_4_f32(
                Some(&self.splash_color_loc),
                color[0],
                color[1],
                color[2],
                color[3],
            );

            self.gl.bind_vertex_array(Some(self.splash_vao));
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.splash_instance_vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&instance_data),
                glow::STREAM_DRAW,
            );

            self.gl.draw_arrays_instanced(
                glow::TRIANGLES,
                0,
                6,
                particles.splashes.len() as i32,
            );
            self.gl.bind_vertex_array(None);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.drop_program);
            self.gl.delete_vertex_array(self.drop_vao);
            self.gl.delete_buffer(self.drop_quad_vbo);
            self.gl.delete_buffer(self.drop_instance_vbo);
            self.gl.delete_program(self.splash_program);
            self.gl.delete_vertex_array(self.splash_vao);
            self.gl.delete_buffer(self.splash_quad_vbo);
            self.gl.delete_buffer(self.splash_instance_vbo);
        }
    }
}

/// Compile a vertex + fragment shader into a program
unsafe fn compile_program(
    gl: &glow::Context,
    vert_src: &str,
    frag_src: &str,
) -> Result<glow::Program, String> {
    let vert = gl
        .create_shader(glow::VERTEX_SHADER)
        .map_err(|e| e.to_string())?;
    gl.shader_source(vert, vert_src);
    gl.compile_shader(vert);
    if !gl.get_shader_compile_status(vert) {
        let log = gl.get_shader_info_log(vert);
        gl.delete_shader(vert);
        return Err(format!("Vertex shader error: {}", log));
    }

    let frag = gl
        .create_shader(glow::FRAGMENT_SHADER)
        .map_err(|e| e.to_string())?;
    gl.shader_source(frag, frag_src);
    gl.compile_shader(frag);
    if !gl.get_shader_compile_status(frag) {
        let log = gl.get_shader_info_log(frag);
        gl.delete_shader(vert);
        gl.delete_shader(frag);
        return Err(format!("Fragment shader error: {}", log));
    }

    let program = gl.create_program().map_err(|e| e.to_string())?;
    gl.attach_shader(program, vert);
    gl.attach_shader(program, frag);
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        let log = gl.get_program_info_log(program);
        gl.delete_program(program);
        gl.delete_shader(vert);
        gl.delete_shader(frag);
        return Err(format!("Program link error: {}", log));
    }

    gl.detach_shader(program, vert);
    gl.detach_shader(program, frag);
    gl.delete_shader(vert);
    gl.delete_shader(frag);

    Ok(program)
}

/// Create an orthographic projection matrix (column-major for OpenGL)
fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [f32; 16] {
    let near = -1.0_f32;
    let far = 1.0_f32;
    [
        2.0 / (right - left),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (top - bottom),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (far - near),
        0.0,
        -(right + left) / (right - left),
        -(top + bottom) / (top - bottom),
        -(far + near) / (far - near),
        1.0,
    ]
}
