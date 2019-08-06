use gl::types::*;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::path::PathBuf;
use std::fs;
use std::convert::AsRef;
use nalgebra_glm as glm;

use super::Atlas;
use super::Result;

pub fn load_file<P: AsRef<std::path::Path>>(filename: P) -> String {
    let content = fs::read_to_string(&filename)
            .expect(format!("Failed to load file: '{}'", filename.as_ref().to_str().unwrap()).as_str());
    content
}

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);

        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;

        if gl::GetShaderiv::is_loaded() {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        }
        else {
            println!("GetShaderiv is not loaded");
        }
        println!("{}", status);
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            //if len == 0 {
            //    println!("Status {} but len is 0", status);
            //    return shader;
            //}

            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{:?} {}",
                ty,
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

pub fn link_shader(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);

        gl::LinkProgram(program);

        let mut status = gl::FALSE as GLint;

        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            //if len == 0 {
            //    println!("Status {} but len is 0", status);
            //    return program;
            //}
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("LinkStatus not valid utf8")
            );
        }
        program
    }
}

static TEXT_VS_SOURCE: &'static str = "shaders/vs.glsl";
static TEXT_FS_SOURCE: &'static str = "shaders/fs.glsl";

static RECT_VS_SOURCE: &'static str = "shaders/rect.vs.glsl";
static RECT_FS_SOURCE: &'static str = "shaders/rect.fs.glsl";

pub struct TextShader {
    program: u32,
    // uniform location
    per_loc: i32,
    // uniform atlas
    atlas_loc: i32,
    // size of each cell
    cell_loc: i32,
    // shader used
    active: bool,
}

pub struct RectShader {
    program: u32,
    // uniform location
    per_loc: i32,
    // size of each cell
    cell_loc: i32,
    // shader used
    active: bool,
}

impl TextShader {
    pub fn new() -> Result<Self> {
        let vs_src = shader::load_file(TEXT_VS_SOURCE);
        let fs_src = shader::load_file(TEXT_FS_SOURCE);

        let vs = shader::compile_shader(vs_src.as_str(), gl::VERTEX_SHADER);
        let fs = shader::compile_shader(fs_src.as_str(), gl::FRAGMENT_SHADER);

        let program = shader::link_shader(vs, fs);

        
        let per_loc = unsafe { gl::GetUniformLocation(program, CString::new("projection").unwrap().as_ptr()) };
        let atlas_loc = unsafe { gl::GetUniformLocation(program, CString::new("atlas").unwrap().as_ptr()) };
        let cell_loc = unsafe { gl::GetUniformLocation(program, CString::new("cell_size").unwrap().as_ptr()) };

        Ok(Self {
            program,
            per_loc,
            atlas_loc,
            cell_loc,
            active: false,
        })
    }

    pub fn set_perspective(&self, per: glm::Mat4) {
        unsafe { gl::UniformMatrix4fv(self.per_loc, 1, gl::FALSE, per.as_ptr()) };
    }

    pub fn set_font_atlas(&self, atlas: &Atlas) {
        unsafe { gl::Uniform1i(self.atlas_loc, atlas.texture_id as i32) };
    }

    pub fn set_cell_size(&self, size: (f32, f32)) {
        unsafe { gl::Uniform2f(self.cell_loc, size.0, size.1) };
    }

    pub fn activate(&mut self) {
        unsafe { gl::UseProgram(self.program) };
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        unsafe { gl::UseProgram(0) };
        self.active = false;
    }
}

impl RectShader {
    pub fn new() -> Result<Self> {
        let vs_src = shader::load_file(RECT_VS_SOURCE);
        let fs_src = shader::load_file(RECT_FS_SOURCE);

        let vs = shader::compile_shader(vs_src.as_str(), gl::VERTEX_SHADER);
        let fs = shader::compile_shader(fs_src.as_str(), gl::FRAGMENT_SHADER);

        let program = shader::link_shader(vs, fs);

        
        let per_loc = unsafe { gl::GetUniformLocation(program, CString::new("projection").unwrap().as_ptr()) };
        let cell_loc = unsafe { gl::GetUniformLocation(program, CString::new("cell_size").unwrap().as_ptr()) };

        Ok(Self {
            program,
            per_loc,
            cell_loc,
            active: false,
        })
    }

    pub fn set_perspective(&self, per: glm::Mat4) {
        unsafe { gl::UniformMatrix4fv(self.per_loc, 1, gl::FALSE, per.as_ptr()) };
    }

    pub fn set_cell_size(&self, size: (f32, f32)) {
        unsafe { gl::Uniform2f(self.cell_loc, size.0, size.1) };
    }

    pub fn activate(&mut self) {
        if self.active { return; }
        unsafe { gl::UseProgram(self.program) };
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        if !self.active { return; }
        unsafe { gl::UseProgram(0) };
        self.active = false;
    }
}