extern crate sdl2;
extern crate glow;

use std::fs;

use glow::HasContext;
use sdl2::event::Event;
use sdl2::sys::exit;
use sdl2::video::GLProfile;

use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32;3],
    color: [f32;3],
}

const VERTICES: [Vertex;24] = [
    // Front face (red)
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
    
    // Back face (green)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
    
    // Left face (blue)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
    
    // Right face (yellow)
    Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [0.5,  0.5, -0.5], color: [1.0, 1.0, 0.0] },
    
    // Top face (magenta)
    Vertex { position: [-0.5, 0.5, -0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, 0.5, -0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, 0.5,  0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [-0.5, 0.5,  0.5], color: [1.0, 0.0, 1.0] },
    
    // Bottom face (cyan)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.0, 1.0, 1.0] },
];


const INDICES: [u32; 36] = [
    // Front face
    0, 1, 2,
    2, 3, 0,
    
    // Back face
    4, 5, 6,
    6, 7, 4,
    
    // Left face
    8, 9,10,
    10,11, 8,
    
    // Right face
    12,13,14,
    14,15,12,
    
    // Top face
    16,17,18,
    18,19,16,
    
    // Bottom face
    20,21,22,
    22,23,20,
];



fn main(){
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    sdl.mouse().set_relative_mouse_mode(true);

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_minor_version(3);
    gl_attr.set_context_major_version(3);

    let window = video.window("OpenGL+SDL2", 800, 500)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    let gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
    };

    let projection = perspective(Deg(45.0), 800.0/500.0, 0.1, 100.0);

    let mut camera_pos: Point3<f32> = Point3::new(0.0, 0.0, 3.0);
    let camera_target: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
    let camera_up: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    let view = Matrix4::look_at_rh(camera_pos, camera_target, camera_up);

    let mut camera_front: Vector3<f32> = (camera_target - camera_pos).normalize();

    let model: Matrix4<f32> = Matrix4::identity();
    let speed = 0.005;

    let vao;
    let view_loc;

    unsafe {
        gl.enable(glow::DEPTH_TEST);

        let fragment = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(fragment, fs::read_to_string("src/fragment.glsl").unwrap().as_str());
        gl.compile_shader(fragment);

        let vertex = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vertex, fs::read_to_string("src/vertex.glsl").unwrap().as_str());
        gl.compile_shader(vertex);

        let program = gl.create_program().unwrap();
        gl.attach_shader(program, fragment);
        gl.attach_shader(program, vertex);
        gl.link_program(program);

        gl.use_program(Some(program));

        vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&VERTICES), glow::STATIC_DRAW);

        let ebo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&INDICES), glow::STATIC_DRAW);

        gl.enable_vertex_attrib_array(0);

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, ::std::mem::size_of::<Vertex>() as i32, 0);

        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, ::std::mem::size_of::<Vertex>() as i32, 12);

        let model_array: [f32; 16] = [
        model.x.x, model.x.y, model.x.z, model.x.w,
        model.y.x, model.y.y, model.y.z, model.y.w,
        model.z.x, model.z.y, model.z.z, model.z.w,
        model.w.x, model.w.y, model.w.z, model.w.w,
    ];

    let view_array: [f32; 16] = [
        view.x.x, view.x.y, view.x.z, view.x.w,
        view.y.x, view.y.y, view.y.z, view.y.w,
        view.z.x, view.z.y, view.z.z, view.z.w,
        view.w.x, view.w.y, view.w.z, view.w.w,
    ];

    let projection_array: [f32; 16] = [
        projection.x.x, projection.x.y, projection.x.z, projection.x.w,
        projection.y.x, projection.y.y, projection.y.z, projection.y.w,
        projection.z.x, projection.z.y, projection.z.z, projection.z.w,
        projection.w.x, projection.w.y, projection.w.z, projection.w.w,
    ];

        let model_loc = gl.get_uniform_location(program, "model");
        gl.uniform_matrix_4_f32_slice(model_loc.as_ref(), false, &model_array);

        view_loc = gl.get_uniform_location(program, "view");
        gl.uniform_matrix_4_f32_slice(view_loc.as_ref(), false, &view_array);

        let projection_loc = gl.get_uniform_location(program, "projection");
        gl.uniform_matrix_4_f32_slice(projection_loc.as_ref(), false, &projection_array);


    }

    let mut event_pump = sdl.event_pump().unwrap();
    let mut yaw: f32 = -90.0; 
    let mut pitch: f32 = 0.0;

    loop {
        let events: Vec<Event> = event_pump.poll_iter().collect();
        for event in events {
            println!("{:?}", event);

            match event {
                Event::Quit { timestamp } => { 
                    println!("Quit event at timestamp: {}", timestamp);
                    println!("Exiting...");
                    unsafe { exit(1) };
                    return;
                },

                Event::MouseMotion { xrel, yrel, .. } => {
                    let sensitivity = 0.1; 
                    let xoffset = xrel as f32 * sensitivity;
                    let yoffset = yrel as f32 * sensitivity;
    
                    yaw += xoffset;
                    pitch -= yoffset; 
    
                    if pitch > 89.0 {
                        pitch = 89.0;
                    }
                    if pitch < -89.0 {
                        pitch = -89.0;
                    }
    
                    camera_front = Vector3 {
                        x: yaw.to_radians().cos() * pitch.to_radians().cos(),
                        y: pitch.to_radians().sin(),
                        z: yaw.to_radians().sin() * pitch.to_radians().cos(),
                    }
                    .normalize();
                },

                _ => {}
            }
        }

        let keys = event_pump.keyboard_state();
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
                camera_pos += camera_front * speed;
            }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
                camera_pos -= camera_front * speed;
            }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
                camera_pos -= camera_front.cross(camera_up).normalize() * speed;
            }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
                camera_pos += camera_front.cross(camera_up).normalize() * speed;
            }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
                camera_pos += camera_up * speed;
            }
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::LShift) {
                camera_pos -= camera_up * speed;
            }
        
        let view = Matrix4::look_at_rh(camera_pos, camera_pos + camera_front, camera_up);

        let view_array: [f32; 16] = [
            view.x.x, view.x.y, view.x.z, view.x.w,
            view.y.x, view.y.y, view.y.z, view.y.w,
            view.z.x, view.z.y, view.z.z, view.z.w,
            view.w.x, view.w.y, view.w.z, view.w.w,
            ];
        
        unsafe {
            gl.uniform_matrix_4_f32_slice(view_loc.as_ref(), false, &view_array);
        }

        unsafe {
            gl.clear_color(0.5, 0.5, 0.5, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.bind_vertex_array(Some(vao));
            gl.draw_elements(glow::TRIANGLES, 36, glow::UNSIGNED_INT, 0);
        }

        window.gl_swap_window();
    }
}