use gl::types::*;

use super::window::Window;

struct Renderer {
    vbo: u32,
    ibo: u32,
    vao: u32,
    shader: u32,
}


struct Atlas {
}

static INDEX_DATA: [u32; 6] = [0, 1, 2, 0, 2, 3];

/*
    let ortho = glm::ortho(0f32, w as f32, h as f32, 0.0, -1f32, 1f32);
    unsafe {
        let un_tex = gl::GetUniformLocation(program, CString::new("text").unwrap().as_ptr());
        gl::Uniform1i(un_tex, tex.id as i32);
        let un_per = gl::GetUniformLocation(program, CString::new("per").unwrap().as_ptr());
        gl::UniformMatrix4fv(un_per, 1, gl::FALSE, ortho.as_ptr());

        let un_clr = gl::GetUniformLocation(program, CString::new("background").unwrap().as_ptr());
        gl::Uniform4f(un_clr, 1.0, 1.0, 1.0, 1.0);

        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Viewport(0, 0, w, h);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
        // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::MULTISAMPLE);
        gl::DepthMask(gl::FALSE);
    }
*/
