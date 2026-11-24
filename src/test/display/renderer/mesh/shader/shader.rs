use gl::types::{GLint, GLuint};
use std::ffi::CString;
use std::fs;

pub struct Shader {
    pub(crate) program: GLuint,
}

impl Shader {
    pub unsafe fn new(vertex_path: &str, fragment_path: &str) -> Shader {
        unsafe {
            let vertex_code = fs::read_to_string(vertex_path).unwrap_or_else(|_| panic!("Cannot read file {}", vertex_path));
            let fragment_code = fs::read_to_string(fragment_path).unwrap_or_else(|_| panic!("Cannot read file {}", vertex_path));

            // vertex Shader
            let vertex: GLuint = Shader::compile_shader(vertex_code, gl::VERTEX_SHADER, "GL_VERTEX_SHADER").unwrap_or_else(|_| panic!("Cannot compile vertex shader {}", vertex_path));
            let fragment: GLuint = Shader::compile_shader(fragment_code, gl::FRAGMENT_SHADER, "GL_FRAGMENT_SHADER").unwrap_or_else(|_| panic!("Cannot compile fragment shader {}", fragment_path));

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

                let _message = String::from_utf8_lossy(&v);
            }
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            Self { program }
        }
    }

    unsafe fn compile_shader(code: String, s_type: u32, type_name: &str) -> Result<u32, Box<dyn std::error::Error>> {
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
                return Err(format!("{} Shader Error: {}", type_name, message).into());
            }

            Ok(shader)
        }
    }

    // use/activate the shader
    pub unsafe fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub unsafe fn set_int(&self, name: &str, value: i32) {
        unsafe {
            gl::Uniform1i(self.get_location(name), value);
        }
    }

    unsafe fn set_float(&self, name: &str, value: f32) {
        unsafe {
            gl::Uniform1f(self.get_location(name), value);
        }
    }

    unsafe fn set_vec2(&self, name: &str, v1: f32, v2: f32) {
        unsafe {
            gl::Uniform2f(self.get_location(name), v1, v2);
        }
    }

    pub unsafe fn set_vec3(&self, name: &str, value: glam::Vec3) {
        unsafe {
            gl::Uniform3f(self.get_location(name), value.x, value.y, value.z);
        }
    }

    unsafe fn get_location(&self, name: &str) -> GLint {
        unsafe { gl::GetUniformLocation(self.program, CString::new(name).unwrap_or_else(|_| panic!("Cannot unwrap {}", name)).as_bytes().as_ptr() as *const i8) }
    }

    unsafe fn set_vec4(&self, name: &str, v1: f32, v2: f32, v3: f32, v4: f32) {
        unsafe {
            gl::Uniform4f(self.get_location(name), v1, v2, v3, v4);
        }
    }

    pub unsafe fn set_matrix4fv(&self, name: &str, matrix: glam::Mat4) {
        unsafe {
            gl::UniformMatrix4fv(self.get_location(name), 1, gl::FALSE, matrix.to_cols_array().as_ptr());
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}
