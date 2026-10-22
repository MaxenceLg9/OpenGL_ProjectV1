use gl::types::{GLint, GLuint};
use shared::common::error::err_type::{CustomErrorKind, GameError};
use shared::common::file::path::Reader;
use shared::common::file::reader;
use shared::custom_error;
use std::ffi::CString;

pub struct Shader {
    pub(crate) program: GLuint,
}

impl Shader {
    pub unsafe fn new<R: Reader>(vertex_path: R, fragment_path: R) -> Result<Shader, GameError> {
        let vertex_code = reader::read_file_to_string(&vertex_path).expect(format!("Cannot read vertex file {}", vertex_path).as_str());
        let fragment_code = reader::read_file_to_string(&fragment_path).expect(format!("Cannot read fragment file {}", fragment_path).as_str());

        // vertex Shader
        let vertex: GLuint = Shader::compile_shader(vertex_code, gl::VERTEX_SHADER, "GL_VERTEX_SHADER").expect(format!("Cannot compile shader {}", vertex_path).as_str());
        let fragment: GLuint = Shader::compile_shader(fragment_code, gl::FRAGMENT_SHADER, "GL_FRAGMENT_SHADER").expect(format!("Cannot compile shader {}", fragment_path).as_str());

        // shader Program
        let program: GLuint = gl::CreateProgram();
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success); //debug
        if success == 0 {
            // 0 means failure in COMPILE_STATUS
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0;

            // 3. Get the actual log
            gl::GetProgramInfoLog(program, 1024, &mut log_len, v.as_mut_ptr() as *mut i8);
            v.set_len(log_len as usize);

            let message = String::from_utf8_lossy(&v);
            return Err(custom_error!(CustomErrorKind::InvalidData, message.into_owned()));
        }
        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
        Ok(Self { program })
    }

    unsafe fn compile_shader(code: String, s_type: u32, type_name: &str) -> Result<u32, GameError> {
        unsafe {
            let shader = gl::CreateShader(s_type);

            // 1. CStrings are required for OpenGL (null-terminated)
            let c_str = std::ffi::CString::new(code.as_bytes())?;

            // 2. ShaderSource expects a pointer to a pointer
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);

            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

            if success == 0 {
                // 0 means failure in COMPILE_STATUS
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0;

                // 3. Get the actual log
                gl::GetShaderInfoLog(shader, 1024, &mut log_len, v.as_mut_ptr() as *mut i8);
                v.set_len(log_len as usize);

                let message = String::from_utf8_lossy(&v);
                return Err(GameError::new(CustomErrorKind::InvalidData, format!("{} Shader Error: {}", type_name, message)));
            }

            Ok(shader)
        }
    }

    // use/activate the shader
    pub unsafe fn use_shader(&self) {
        gl::UseProgram(self.program);
    }

    pub unsafe fn set_int(&self, name: &str, value: i32) {
        gl::Uniform1i(self.get_location(name), value);
    }

    pub unsafe fn set_float(&self, name: &str, value: f32) {
        gl::Uniform1f(self.get_location(name), value);
    }

    pub unsafe fn set_vec2(&self, name: &str, v1: f32, v2: f32) {
        gl::Uniform2f(self.get_location(name), v1, v2);
    }

    pub unsafe fn set_vec3(&self, name: &str, value: glam::Vec3) {
        gl::Uniform3f(self.get_location(name), value.x, value.y, value.z);
    }

    unsafe fn get_location(&self, name: &str) -> GLint {
        name.as_bytes().as_ptr() as *const i8;
        gl::GetUniformLocation(self.program, CString::new(name).expect(format!("Cannot unwrap {}", name).as_str()).as_bytes().as_ptr() as *const i8).clone()
    }

    pub unsafe fn set_vec4(&self, name: &str, vec4: glam::Vec4) {
        gl::Uniform4f(self.get_location(name), vec4.x, vec4.y, vec4.z, vec4.w);
    }

    // pub unsafe fn set_vec4(&self, name: &str, v1: f32, v2: f32, v3: f32, v4: f32) {
    //     gl::Uniform4f(self.get_location(name), v1, v2, v3, v4);
    // }

    pub unsafe fn set_matrix4fv(&self, name: &str, matrix: glam::Mat4) {
        gl::UniformMatrix4fv(self.get_location(name), 1, gl::FALSE, matrix.to_cols_array().as_ptr());
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}
