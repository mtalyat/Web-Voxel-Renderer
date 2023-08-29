use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlShader, WebGlProgram};
use gl_matrix::common::*;
use gl_matrix::{vec3, mat4};
use std::cell::RefCell;
use std::rc::Rc;
extern crate js_sys;

// link js functions to rs 
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn get_window() -> web_sys::Window
{
    return web_sys::window().expect("No Window exists.");
}

fn request_animation_frame(f: &Closure<dyn FnMut()>)
{
    get_window().request_animation_frame(f.as_ref().unchecked_ref()).expect("Should register requestAnimationFrame OK.");
}

pub fn init_webgl_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    gl.viewport(
        0,
        0,
        canvas.width().try_into().unwrap(),
        canvas.height().try_into().unwrap(),
    );

    Ok(gl)
}

pub fn create_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()),
        ))
    }
}

pub fn setup_shaders(gl: &WebGlRenderingContext, vertex_shader_source: &str, fragment_shader_source: &str) -> Result<WebGlProgram, JsValue> {
    let vertex_shader = create_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )
    .unwrap();
    let fragment_shader = create_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )
    .unwrap();

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);


    if gl
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // Set the shader program as active.
        gl.use_program(Some(&shader_program));
        Ok(shader_program)
    } else {
        return Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ));
    }
}

pub fn setup_vertices(gl: &WebGlRenderingContext, vertices: &[f32], indices: &[u16], shader_program: &WebGlProgram) {
    let vertices_array = unsafe { js_sys::Float32Array::view(&vertices) };
    let indices_array = unsafe { js_sys::Uint16Array::view(&indices) };

    // set vertices
    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    // set indices
    // must set indices last, so draw_elements will use this bound buffer to draw
    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        &indices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    // setup attributes for the vertices
    let vert_pos_location = gl.get_attrib_location(&shader_program, "vertPosition") as u32;
    gl.vertex_attrib_pointer_with_i32(
        vert_pos_location, // location
        2, // number of elements
        WebGlRenderingContext::FLOAT, // element type
        false, // is data normalized
        (std::mem::size_of::<f32>() as i32) * 2, // size of total vertex in bytes
        0, // offset from beginning in bytes
    );
    gl.enable_vertex_attrib_array(vert_pos_location);
}

struct Shape
{
    position: [f32; 3],
    size: [f32; 3],
    color: [f32; 4],
    shape: i32,
}

pub fn update_uniforms(gl: &WebGlRenderingContext, shader_program: &WebGlProgram, time: f64)
{
    let time_location = gl.get_uniform_location(shader_program, "time").unwrap();
    gl.uniform1f(Some(&time_location), time as f32);
}

pub fn setup_uniforms(gl: &WebGlRenderingContext, shader_program: &WebGlProgram, resolution: &Vec<f32>, bg_color: &Vec<f32>)
{
    let resolution_location = gl.get_uniform_location(shader_program, "resolution").unwrap();
    gl.uniform2fv_with_f32_array(Some(&resolution_location), resolution);
    
    let bg_color_location = gl.get_uniform_location(shader_program, "background_color").unwrap();
    gl.uniform3fv_with_f32_array(Some(&bg_color_location), bg_color);

    let fov_location = gl.get_uniform_location(shader_program, "fov").unwrap();
    gl.uniform1f(Some(&fov_location), 45.0);

    let camera_location = gl.get_uniform_location(shader_program, "eye").unwrap();
    gl.uniform3fv_with_f32_array(Some(&camera_location), &vec![8.0, 5.0, 7.0]);

    let up_location = gl.get_uniform_location(shader_program, "up").unwrap();
    gl.uniform3fv_with_f32_array(Some(&up_location), &vec![0.0, 1.0, 0.0]);

    let near_location = gl.get_uniform_location(shader_program, "near").unwrap();
    gl.uniform1f(Some(&near_location), 0.1);

    let far_location = gl.get_uniform_location(shader_program, "far").unwrap();
    gl.uniform1f(Some(&far_location), 100.0);

    update_uniforms(gl, shader_program, 0.0);
}

pub fn setup_shapes(gl: &WebGlRenderingContext, shader_program: &WebGlProgram)
{
    // create shapes
    const SHAPE_COUNT: usize = 2;
    let shapes: [Shape; SHAPE_COUNT] =
    [
        Shape { // cube
            position: [-1.5, 0.0, 0.0],
            size: [1.0, 1.0, 1.0],
            color: [1.0, 0.65, 0.0, 1.0],
            shape: 0
        },
        Shape { // sphere
            position: [1.5, 0.0, 0.0],
            size: [1.0, 1.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
            shape: 1
        }
    ];

    // send all data to gpu
    for i in 0..SHAPE_COUNT 
    {
        gl.uniform3fv_with_f32_array(Some(&gl.get_uniform_location(shader_program, &format!("objects[{i}].position")).unwrap()), &shapes[i].position);
        gl.uniform3fv_with_f32_array(Some(&gl.get_uniform_location(shader_program, &format!("objects[{i}].size")).unwrap()), &shapes[i].size);
        gl.uniform4fv_with_f32_array(Some(&gl.get_uniform_location(shader_program, &format!("objects[{i}].color")).unwrap()), &shapes[i].color);
        gl.uniform1i(Some(&gl.get_uniform_location(shader_program, &format!("objects[{i}].shape")).unwrap()), shapes[i].shape);
    }
}

pub fn setup_transforms(gl: &WebGlRenderingContext, shader_program: &WebGlProgram, resolution: &Vec<f32>)
{
    let world_location = gl.get_uniform_location(shader_program, "mWorld").unwrap();
    let view_location = gl.get_uniform_location(shader_program, "mView").unwrap();
    let proj_location = gl.get_uniform_location(shader_program, "mProj").unwrap();

    let mut world_matrix: Mat4 = [0.; 16];
    let mut view_matrix: Mat4 = [0.; 16];
    let mut proj_matrix: Mat4 = [0.; 16];

    let eye = vec3::from_values(0.0, 0.0, -8.0);
    let center = vec3::from_values(0.0, 0.0, 0.0);
    let up = vec3::from_values(0.0, 1.0, 0.0);

    mat4::identity(&mut world_matrix);
    mat4::look_at(&mut view_matrix, &eye, &center, &up);
    mat4::perspective(&mut proj_matrix, to_radian(45.0), resolution[0] / resolution[1], 0.1, Some(1000.0));

    gl.uniform_matrix4fv_with_f32_array(Some(&world_location), false, &world_matrix);
    gl.uniform_matrix4fv_with_f32_array(Some(&view_location), false, &view_matrix);
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_location), false, &proj_matrix);
}

#[wasm_bindgen]
pub fn run_program(
    canvas_id: &str,
    canvas_size: Vec<f32>,
    vertex_shader: &str,
    fragment_shader: &str
) -> Result<(), JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id).unwrap();
    let shader_program: WebGlProgram = setup_shaders(&gl, vertex_shader, fragment_shader).unwrap();

    let half_width = canvas_size[0] / 2.0;
    let half_height = canvas_size[1] / 2.0;
    // let half_width = 1.0;
    // let half_height = 1.0;

    let vertices: [f32; 8] = 
    [
        // X, Y (normalized)
        -half_width, -half_height,
        half_width, -half_height,
        half_width, half_height,
        -half_width, half_height
    ];

    let indices: [u16; 6] =
    [
        0, 1, 2,
        0, 2, 3
    ];

    let bg_color = vec![0.8, 0.8, 0.8];

    setup_vertices(&gl, &vertices, &indices, &shader_program);
    setup_uniforms(&gl, &shader_program, &canvas_size, &bg_color);
    setup_shapes(&gl, &shader_program);
    setup_transforms(&gl, &shader_program, &canvas_size);

    // main loop
    let window = web_sys::window().expect("Should have a window in this context.");
    let performance = window.performance().expect("Performance is not available.");
    let start = performance.now();
    // let frame_time = start; // FPS??

    // initialize game loop
    // https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // game loop:

        // get time
        let current_time = performance.now();
        let elapsed_time = (current_time - start) / 1000.0;
        update_uniforms(&gl, &shader_program, elapsed_time);

        // FPS??

        // clear screen
        gl.clear_color(bg_color[0], bg_color[1], bg_color[2], 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

        // draw elements
        gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            indices.len() as i32,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0, // offset
        );

        // restart loop
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    // start game loop
    request_animation_frame(g.borrow().as_ref().unwrap());

    log("Program completed.");
    
    Ok(())
}