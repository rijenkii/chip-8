const VERT_SRC: &str = concat!(
    r#"#version 330 core
    out vec2 f_pos;

    const vec2 data[4] = vec2[](
        vec2(-1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2( 1.0, -1.0)
    );

    void main() {
        f_pos = (vec2(data[gl_VertexID].x, -data[gl_VertexID].y) + 1.0) / 2.0;
        gl_Position = vec4(data[gl_VertexID], 0.0, 1.0);
    }"#,
    "\0",
);

const FRAG_SRC: &str = concat!(
    r#"#version 330 core
    in vec2 f_pos;

    out vec4 o_color;

    uniform bool screen[64 * 32];

    void main() {
        if (screen[min(int(f_pos.y * 32.0), 31) * 64 + min(int(f_pos.x * 64.0), 63)]) {
            o_color = vec4(1.0, 1.0, 1.0, 1.0);
        } else {
            o_color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }"#,
    "\0",
);

pub struct GlutinWindow {
    events_loop: glutin::EventsLoop,
    windowed_context: glutin::WindowedContext<glutin::PossiblyCurrent>,

    program: u32,
    program_screen: i32,

    dummy_vao: u32,

    gl: GlFunctions,
}

impl GlutinWindow {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new()
            .with_title("CHIP8")
            .with_dimensions(glutin::dpi::LogicalSize::from_physical(
                (16 * 64, 16 * 32),
                events_loop.get_primary_monitor().get_hidpi_factor(),
            ));
        let windowed_context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(wb, &events_loop)
            .unwrap();

        let windowed_context = unsafe { windowed_context.make_current().unwrap() };

        let gl = GlFunctions::new(|x| windowed_context.get_proc_address(x) as _);

        let program = unsafe {
            use std::ptr::{null, null_mut};

            let vert = (gl.create_shader)(0x8B31); // VERTEX_SHADER
            (gl.shader_source)(vert, 1, &VERT_SRC.as_ptr(), null());
            (gl.compile_shader)(vert);

            {
                let mut status = 0;
                (gl.get_shaderiv)(vert, 0x8B81, &mut status); // COMPILE_STATUS
                if status != 1 {
                    let mut info_len = 0;
                    (gl.get_shaderiv)(vert, 0x8B84, &mut info_len); // INFO_LOG_LENGTH
                    let mut info_log = vec![0u8; info_len as _];
                    (gl.get_shader_info_log)(
                        vert,
                        info_len as _,
                        null_mut(),
                        info_log.as_mut_ptr(),
                    );
                    info_log.truncate(info_len as usize - 1);
                    panic!(
                        "Vertex shader compilation error: \n{}",
                        std::ffi::CString::new(info_log)
                            .unwrap()
                            .into_string()
                            .unwrap(),
                    );
                }
            }

            let frag = (gl.create_shader)(0x8B30); // FRAGMENT_SHADER
            (gl.shader_source)(frag, 1, &FRAG_SRC.as_ptr(), null());
            (gl.compile_shader)(frag);

            {
                let mut status = 0;
                (gl.get_shaderiv)(frag, 0x8B81, &mut status); // COMPILE_STATUS
                if status != 1 {
                    let mut info_len = 0;
                    (gl.get_shaderiv)(frag, 0x8B84, &mut info_len); // INFO_LOG_LENGTH
                    let mut info_log = vec![0u8; info_len as _];
                    (gl.get_shader_info_log)(
                        frag,
                        info_len as _,
                        null_mut(),
                        info_log.as_mut_ptr(),
                    );
                    info_log.truncate(info_len as usize - 1);
                    panic!(
                        "Fragment shader compilation error: \n{}",
                        std::ffi::CString::new(info_log)
                            .unwrap()
                            .into_string()
                            .unwrap(),
                    );
                }
            }

            let prog = (gl.create_program)();
            (gl.attach_shader)(prog, vert);
            (gl.attach_shader)(prog, frag);
            (gl.link_program)(prog);

            {
                let mut status = 0;
                (gl.get_programiv)(prog, 0x8B82, &mut status); // LINK_STATUS
                if status != 1 {
                    let mut info_len = 0;
                    (gl.get_programiv)(prog, 0x8B84, &mut info_len); // INFO_LOG_LENGTH
                    let mut info_log = vec![0u8; info_len as _];
                    (gl.get_program_info_log)(
                        prog,
                        info_len as _,
                        null_mut(),
                        info_log.as_mut_ptr(),
                    );
                    info_log.truncate(info_len as usize - 1);
                    panic!(
                        "Program linking error: \n{}",
                        std::ffi::CString::new(info_log)
                            .unwrap()
                            .into_string()
                            .unwrap(),
                    );
                }
            }

            (gl.detach_shader)(prog, vert);
            (gl.detach_shader)(prog, frag);
            (gl.delete_shader)(vert);
            (gl.delete_shader)(frag);

            prog
        };

        let program_screen = unsafe { (gl.get_uniform_location)(program, "screen\0".as_ptr()) };

        let mut dummy_vao = 0;
        unsafe {
            (gl.gen_vertex_arrays)(1, &mut dummy_vao);
        }

        unsafe {
            (gl.use_program)(program);
            (gl.bind_vertex_array)(dummy_vao);
            (gl.viewport)(0, 0, 16 * 64, 16 * 32);
        }

        Self {
            events_loop,
            windowed_context,

            program,
            program_screen,

            dummy_vao,

            gl,
        }
    }

    pub fn run(&mut self, freq: u8, mut machine: crate::machine::Machine) {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        };

        // 1 2 3 C -> 02 03 04 05 (1 2 3 4)
        // 4 5 6 D -> 16 17 18 19 (Q W E R)
        // 7 8 9 E -> 30 31 32 33 (A S D F)
        // A 0 B F -> 44 45 46 47 (Z X C V)

        let keyboard_settings = [45, 2, 3, 4, 16, 17, 18, 30, 31, 32, 44, 46, 5, 19, 33, 47];

        let keyboard = Arc::new(Mutex::new([false; 16]));
        let running = Arc::new(AtomicBool::new(true));
        let screen = Arc::new(Mutex::new([[0u32; 64]; 32]));
        let needs_redraw = Arc::new(AtomicBool::new(false));

        let machine_thread = {
            let keyboard = keyboard.clone();
            let running = running.clone();
            let screen = screen.clone();
            let needs_redraw = needs_redraw.clone();

            std::thread::spawn(move || {
                let mut loop_helper = spin_sleep::LoopHelper::builder()
                    .build_with_target_rate(60.0 * f64::from(freq));

                while running.load(Ordering::SeqCst) {
                    loop_helper.loop_start();

                    machine.step(*keyboard.lock().unwrap());
                    if machine.screen().needs_redraw() {
                        needs_redraw.store(true, Ordering::SeqCst);

                        let mut screen_lock = screen.lock().unwrap();
                        let machine_screen = machine.screen().buffer();
                        for y in 0..32 {
                            for x in 0..64 {
                                screen_lock[y][x] = machine_screen[y][x] as _;
                            }
                        }

                        machine.screen().redrawn();
                    }

                    loop_helper.loop_sleep();
                }
            })
        };

        let mut loop_helper = spin_sleep::LoopHelper::builder().build_with_target_rate(60.0);

        while running.load(Ordering::SeqCst) {
            loop_helper.loop_start();

            let mut events = Vec::new();
            self.events_loop.poll_events(|e| events.push(e));

            for event in events {
                if let glutin::Event::WindowEvent { event, .. } = event {
                    use glutin::WindowEvent::*;
                    match event {
                        CloseRequested | Destroyed => running.store(false, Ordering::SeqCst),
                        Resized(lsize) => {
                            // Set glviewport
                            let size = lsize
                                .to_physical(self.windowed_context.window().get_hidpi_factor());
                            unsafe {
                                (self.gl.viewport)(0, 0, size.width as _, size.height as _);
                            }
                        },
                        KeyboardInput { input, .. } => {
                            for (i, key) in keyboard_settings.iter().enumerate() {
                                if input.scancode == *key {
                                    (*keyboard.lock().unwrap())[i] =
                                        input.state == glutin::ElementState::Pressed;
                                }
                            }
                        },
                        _ => {},
                    }
                }
            }

            if needs_redraw.load(Ordering::SeqCst) {
                unsafe {
                    (self.gl.uniform_1iv)(
                        self.program_screen,
                        64 * 32,
                        screen.lock().unwrap().as_ptr() as _,
                    );
                }
                needs_redraw.store(false, Ordering::SeqCst);
            }

            unsafe {
                (self.gl.draw_arrays)(0x0005, 0, 4); // TRIANGLE_STRIP
            }

            self.windowed_context.swap_buffers().unwrap();

            loop_helper.loop_sleep();
        }

        machine_thread.join().unwrap();
    }
}

impl Drop for GlutinWindow {
    fn drop(&mut self) {
        unsafe {
            (self.gl.delete_program)(self.program);
            (self.gl.delete_vertex_arrays)(1, &self.dummy_vao);
        }
    }
}

struct GlFunctions {
    create_shader: unsafe extern "C" fn(u32) -> u32,
    delete_shader: unsafe extern "C" fn(u32),
    shader_source: unsafe extern "C" fn(u32, isize, *const *const u8, *const i32),
    compile_shader: unsafe extern "C" fn(u32),
    get_shaderiv: unsafe extern "C" fn(u32, u32, *mut i32),
    get_shader_info_log: unsafe extern "C" fn(u32, isize, *mut isize, *mut u8),

    create_program: unsafe extern "C" fn() -> u32,
    delete_program: unsafe extern "C" fn(u32),
    attach_shader: unsafe extern "C" fn(u32, u32),
    detach_shader: unsafe extern "C" fn(u32, u32),
    link_program: unsafe extern "C" fn(u32),
    get_programiv: unsafe extern "C" fn(u32, u32, *mut i32),
    get_program_info_log: unsafe extern "C" fn(u32, isize, *mut isize, *mut u8),

    gen_vertex_arrays: unsafe extern "C" fn(isize, *mut u32),
    bind_vertex_array: unsafe extern "C" fn(u32),
    delete_vertex_arrays: unsafe extern "C" fn(isize, *const u32),
    use_program: unsafe extern "C" fn(u32),
    get_uniform_location: unsafe extern "C" fn(u32, *const u8) -> i32,
    uniform_1iv: unsafe extern "C" fn(i32, isize, *const i32),

    viewport: unsafe extern "C" fn(i32, i32, isize, isize),
    draw_arrays: unsafe extern "C" fn(u32, i32, isize),
}

impl GlFunctions {
    fn new(loader: impl Fn(&str) -> *const ()) -> Self {
        Self {
            create_shader: unsafe { std::mem::transmute(loader("glCreateShader")) },
            delete_shader: unsafe { std::mem::transmute(loader("glDeleteShader")) },
            shader_source: unsafe { std::mem::transmute(loader("glShaderSource")) },
            compile_shader: unsafe { std::mem::transmute(loader("glCompileShader")) },
            get_shaderiv: unsafe { std::mem::transmute(loader("glGetShaderiv")) },
            get_shader_info_log: unsafe { std::mem::transmute(loader("glGetShaderInfoLog")) },

            create_program: unsafe { std::mem::transmute(loader("glCreateProgram")) },
            delete_program: unsafe { std::mem::transmute(loader("glDeleteProgram")) },
            attach_shader: unsafe { std::mem::transmute(loader("glAttachShader")) },
            detach_shader: unsafe { std::mem::transmute(loader("glDetachShader")) },
            link_program: unsafe { std::mem::transmute(loader("glLinkProgram")) },
            get_programiv: unsafe { std::mem::transmute(loader("glGetProgramiv")) },
            get_program_info_log: unsafe { std::mem::transmute(loader("glGetProgramInfoLog")) },

            gen_vertex_arrays: unsafe { std::mem::transmute(loader("glGenVertexArrays")) },
            bind_vertex_array: unsafe { std::mem::transmute(loader("glBindVertexArray")) },
            delete_vertex_arrays: unsafe { std::mem::transmute(loader("glDeleteVertexArrays")) },
            use_program: unsafe { std::mem::transmute(loader("glUseProgram")) },
            get_uniform_location: unsafe { std::mem::transmute(loader("glGetUniformLocation")) },
            uniform_1iv: unsafe { std::mem::transmute(loader("glUniform1iv")) },

            viewport: unsafe { std::mem::transmute(loader("glViewport")) },
            draw_arrays: unsafe { std::mem::transmute(loader("glDrawArrays")) },
        }
    }
}
