use crate::gl;
use crate::parameters::Parameters;

use glutin::platform::unix::HeadlessContextExt;
use gl::types::*;


const DEBUG_CTX: bool = true;


pub fn verify_error() {
    match unsafe { gl::GetError() } {
        gl::NO_ERROR => (),
        gl::INVALID_ENUM => panic!("GL_INVALID_ENUM"),
        gl::INVALID_VALUE => panic!("GL_INVALID_VALUE"),
        gl::INVALID_OPERATION => panic!("GL_INVALID_OPERATION"),
        gl::INVALID_FRAMEBUFFER_OPERATION => panic!("GL_INVALID_FRAMEBUFFER_OPERATION"),
        gl::OUT_OF_MEMORY => panic!("GL_OUT_OF_MEMORY"),
        gl::STACK_UNDERFLOW => panic!("GL_STACK_UNDERFLOW"),
        gl::STACK_OVERFLOW => panic!("GL_STACK_OVERFLOW"),
        _ => panic!("Unknown error")
    }
}


extern "system"
fn debug_callback(source: GLenum, gltype: GLenum, _id: GLuint, severity: GLenum, _length: GLsizei, message: *const GLchar, _user_param: *mut std::ffi::c_void) {
    let source = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "WINDOW_SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER_COMPILER",
        gl::DEBUG_SOURCE_THIRD_PARTY => "THIRD_PARTY",
        gl::DEBUG_SOURCE_APPLICATION => "APPLICATION",
        gl::DEBUG_SOURCE_OTHER => "OTHER",
        _ => panic!("Unknown source")
    };

    let msg_type = match gltype {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecation",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined behaviour",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_MARKER => "Marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "Push group",
        gl::DEBUG_TYPE_POP_GROUP => "Pop group",
        gl::DEBUG_TYPE_OTHER => "Other",
        _ => panic!("Unknown type")
    };

    let msg = unsafe { std::ffi::CStr::from_ptr(message).to_str().unwrap() };
    let msg = format!("[{source}] {msg_type}: {msg}");

    match severity {
        gl::DEBUG_SEVERITY_HIGH => tracing::error!("{msg}"),
        gl::DEBUG_SEVERITY_MEDIUM => tracing::warn!("{msg}"),
        gl::DEBUG_SEVERITY_LOW => tracing::info!("{msg}"),
        gl::DEBUG_SEVERITY_NOTIFICATION => tracing::debug!("{msg}"),
        _ => panic!("Unknown severity")
    };
}


unsafe fn setup_debug_callback() {
    gl::Enable(gl::DEBUG_OUTPUT);
    verify_error();

    gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
    verify_error();

    gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());
    verify_error();

    gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, std::ptr::null(), gl::TRUE);
    verify_error();
}


unsafe fn verify_opengl_version() {
    let v = gl::GetString(gl::VERSION);
    verify_error();

    let v = std::ffi::CStr::from_ptr(v as *const i8).to_str().unwrap();
    tracing::info!("OpenGL {v}");
}


#[allow(dead_code)]
unsafe fn verify_spirv_support() {
    let mut binary_formats_count: GLint = 0;
    gl::GetIntegerv(gl::NUM_SHADER_BINARY_FORMATS, &mut binary_formats_count as *mut GLint);
    verify_error();

    let mut binary_formats: Vec<GLint> = vec![0; binary_formats_count as usize];
    gl::GetIntegerv(gl::SHADER_BINARY_FORMATS, binary_formats.as_mut_ptr() as *mut GLint);
    verify_error();

    tracing::debug!("Supported binary formats: {binary_formats:?}");
}


unsafe fn verify_spirv_extension_support() {
    let mut extensions_count: GLint = 0;
    gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut extensions_count as *mut GLint);
    verify_error();

    (0..extensions_count).into_iter().any(|i|{
        let e = gl::GetStringi(gl::EXTENSIONS, i as GLuint);
        verify_error();
        std::ffi::CStr::from_ptr(e as *const i8).to_str().unwrap() == "GL_ARB_gl_spirv"
    }).then_some(()).expect("SPIR-V extension is not supported");
}


unsafe fn compile_program() -> GLuint {
    let shader = gl::CreateShader(gl::COMPUTE_SHADER);
    verify_error();

    let shader_binary = include_bytes!("../shaders/mandelbrot.rs.spv");
    gl::ShaderBinary(1, &shader as *const GLuint, gl::SHADER_BINARY_FORMAT_SPIR_V_ARB, shader_binary.as_ptr() as *const GLvoid, shader_binary.len() as GLsizei);
    verify_error();

    let entry_point_name = std::ffi::CString::new("main").unwrap();
    gl::SpecializeShader(shader, entry_point_name.as_ptr(), 0, std::ptr::null(), std::ptr::null());
    verify_error();

    let mut compiled_status: GLint = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled_status as *mut GLint);
    if compiled_status != gl::TRUE as GLint {
        panic!("Shader specialization/compilation failed");
    }

    let program = gl::CreateProgram();
    verify_error();

    gl::AttachShader(program, shader);
    verify_error();

    gl::LinkProgram(program);
    verify_error();

    return program;
}


pub unsafe fn run_opengl(params: &Parameters) -> Vec<u8> {
    let el = glutin::event_loop::EventLoop::new();
    let ctx = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(
            glutin::Api::OpenGl, (4, 6)
        ))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_hardware_acceleration(Some(true))
        .with_gl_debug_flag(DEBUG_CTX)
        .build_surfaceless(&el).unwrap();
    let ctx = ctx.make_current().unwrap();

    gl::load_with(|s| ctx.get_proc_address(s) as *const _);

    if DEBUG_CTX {
        setup_debug_callback();
    }

    verify_opengl_version();
    verify_spirv_extension_support();
    // verify_spirv_support(); // FIXME: for some reason SHADER_BINARY_FORMATS is empty even tho SPIR_V is accepted...

    let program = compile_program();

    let (wgsize, uniform_data) = crate::shaders::provider::get_spirv_configuration(&params);


    let mut uniform_buffer: GLuint = 0;
    gl::CreateBuffers(1, &mut uniform_buffer as *mut GLuint);
    verify_error();

    gl::NamedBufferStorage(
        uniform_buffer,
        uniform_data.len() as GLsizeiptr,
        uniform_data.as_ptr() as *const GLvoid,
        0
    );
    verify_error();

    let mut storage_buffer: GLuint = 0;
    gl::CreateBuffers(1, &mut storage_buffer as *mut GLuint);
    verify_error();

    let storage_buffer_size_bytes = std::mem::size_of::<u8>() * params.img_size_px as usize * params.img_size_px as usize;
    gl::NamedBufferStorage(
        storage_buffer,
        storage_buffer_size_bytes as GLsizeiptr,
        std::ptr::null(),
        gl::MAP_READ_BIT
    );
    verify_error();

    gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, uniform_buffer);
    verify_error();
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, storage_buffer);
    verify_error();

    gl::UseProgram(program);
    verify_error();

    gl::DispatchCompute(wgsize.0, wgsize.1, wgsize.2);
    verify_error();

    gl::MemoryBarrier(gl::CLIENT_MAPPED_BUFFER_BARRIER_BIT);
    verify_error();

    let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
    let fence_wait = gl::ClientWaitSync(fence, gl::SYNC_FLUSH_COMMANDS_BIT, GLuint64::MAX);
    verify_error();
    match fence_wait {
        gl::ALREADY_SIGNALED => (),
        gl::TIMEOUT_EXPIRED => panic!("Timeout expired"),
        gl::CONDITION_SATISFIED => (),
        gl::WAIT_FAILED => panic!("Wait failed"),
        _ => panic!("Unknown failure")
    };

    let storage_ptr = gl::MapNamedBuffer(storage_buffer, gl::READ_ONLY);
    verify_error();

    let data: Vec<u8> = std::slice::from_raw_parts(storage_ptr as *const u8, storage_buffer_size_bytes).into();

    gl::DeleteSync(fence);
    if gl::UnmapNamedBuffer(storage_buffer) != gl::TRUE {
        tracing::warn!("Buffer unmapping failed");
    }
    gl::DeleteBuffers(1, &storage_buffer as *const GLuint);
    gl::DeleteBuffers(1, &uniform_buffer as *const GLuint);
    gl::DeleteProgram(program);

    return data;
}
