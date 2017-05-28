#[macro_use]
extern crate glium;

use glium::backend::glutin_backend::GlutinFacade;

mod lib;

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    col: [f32; 4],
}
implement_vertex!(Vertex, pos, col);

static VERT_SRC : &'static str = r#"
    #version 140

    uniform highp mat4 proj_mat;

    in vec3 pos;
    in vec4 col;

    out vec4 v_col;

    void main() {
      v_col = col;
      gl_Position = vec4(pos, 1.0)* proj_mat;
    }
"#;

static FRAG_SRC : &'static str = r#"
    #version 140

    in vec4 v_col;

    out vec4 color;

    void main() {
      color = v_col;
    }
"#;


fn setup_display() -> GlutinFacade {
  use glium::DisplayBuild;
  glium::glutin::WindowBuilder::new()
    .with_decorations(false)
    .build_glium().unwrap()
}

fn setup_shader(display: &GlutinFacade) -> glium::Program {
  glium::Program::from_source(display, VERT_SRC, FRAG_SRC, None).unwrap()
}

/// Create 2D orthographic projection matrix (near = -1, far = 1)
/// # Params
/// l: Left of the viewport in space
/// t: Top of the viewport in space
/// r: Right of the viewport in space
/// b: Bottom of the viewport in space
#[allow(non_upper_case_globals)] // For n and f planes
fn gen_ortho_proj_mat(l: f32, t: f32, r: f32, b: f32) -> [[f32; 4]; 4] {
  const n: f32 = -10000.0;
  const f: f32 =  10000.0;
  [
    [2.0/(r-l), 0.0,        0.0,       -(r+l)/(r-l)],
    [0.0,       2.0/(t-b),  0.0,       -(t+b)/(t-b)],
    [0.0,       0.0,       -2.0/(f-n),  0.0],
    [0.0,       0.0,        0.0,        1.0]]
}

/// Setup test node tree
fn setup_nodes() -> lib::Node {
  let mut curr_id = 0;
  let mut get_id = || -> u32 { curr_id += 1; return curr_id; };

  // Create sidebar, 200px wide with 4 items 40px high
  let mut sidebar = lib::Node::new(get_id(), lib::Layout::Vertical, lib::DynLen::Absolute(200.0));
  let item_1 = lib::Node::new(get_id(), lib::Layout::Vertical, lib::DynLen::Absolute(40.0));
  let item_2 = lib::Node::new(get_id(), lib::Layout::Vertical, lib::DynLen::Absolute(40.0));
  let item_3 = lib::Node::new(get_id(), lib::Layout::Vertical, lib::DynLen::Absolute(40.0));
  let item_4 = lib::Node::new(get_id(), lib::Layout::Vertical, lib::DynLen::Absolute(40.0));

  sidebar.add_children(vec![item_1, item_2, item_3, item_4]);

  // Create 'body', have it fill the remaining width
  let body = lib::Node::new(get_id(), lib::Layout::Vertical, lib::DynLen::Relative(1.0));

  // Create wrapper
  let mut wrapper = lib::Node::new(get_id(), lib::Layout::Horizontal, lib::DynLen::Relative(1.0));

  wrapper.add_child(sidebar);
  wrapper.add_child(body);

  return wrapper;
}

fn main() {
  let display = setup_display();
  let shader = setup_shader(&display);

  // Setup node tree and allocate space for rects
  let node_tree = setup_nodes();
  let mut rects = node_tree.alloc_rect_buffer();

  let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

  // Get window size
  let (display_w, display_h) = display.get_window().unwrap().get_inner_size().unwrap();

  // Layout node tree
  node_tree.layout(&mut rects[..], 0.0, 0.0, display_w as f32, display_h as f32, 0.0);

  // Buffer node tree rects as vbo
  let mut vbo_data = Vec::with_capacity(rects.len() * 6);
  for (ii, r) in rects.iter().enumerate() {
    let col = ii as f32 / rects.len() as f32;
    vbo_data.push(Vertex{ pos: [r.pos[0]            , r.pos[1]            , r.layer], col: [col, col, col, 1.0] });
    vbo_data.push(Vertex{ pos: [r.pos[0] + r.size[0], r.pos[1]            , r.layer], col: [col, col, col, 1.0] });
    vbo_data.push(Vertex{ pos: [r.pos[0] + r.size[0], r.pos[1] + r.size[1], r.layer], col: [col, col, col, 1.0] });
    vbo_data.push(Vertex{ pos: [r.pos[0]            , r.pos[1]            , r.layer], col: [col, col, col, 1.0] });
    vbo_data.push(Vertex{ pos: [r.pos[0]            , r.pos[1] + r.size[1], r.layer], col: [col, col, col, 1.0] });
    vbo_data.push(Vertex{ pos: [r.pos[0] + r.size[0], r.pos[1] + r.size[1], r.layer], col: [col, col, col, 1.0] });
  }

  let vbo = glium::VertexBuffer::dynamic(&display, &vbo_data).unwrap();

  loop {
    // listing the events produced by the window and waiting to be received
    for ev in display.poll_events() {
      match ev {
        glium::glutin::Event::Closed => { return }   // the window has been closed by the user
        glium::glutin::Event::Resized(w, h) => {
          // Layout node tree
          node_tree.layout(&mut rects[..], 0.0, 0.0, w as f32, h as f32, 0.0);

          // Buffer node tree rects as vbo
          let mut vbo_data = Vec::with_capacity(rects.len() * 6);
          for (ii, r) in rects.iter().enumerate() {
            let col = ii as f32 / rects.len() as f32;
            vbo_data.push(Vertex{ pos: [r.pos[0]            , r.pos[1]            , r.layer], col: [col, col, col, 1.0] });
            vbo_data.push(Vertex{ pos: [r.pos[0] + r.size[0], r.pos[1]            , r.layer], col: [col, col, col, 1.0] });
            vbo_data.push(Vertex{ pos: [r.pos[0] + r.size[0], r.pos[1] + r.size[1], r.layer], col: [col, col, col, 1.0] });
            vbo_data.push(Vertex{ pos: [r.pos[0]            , r.pos[1]            , r.layer], col: [col, col, col, 1.0] });
            vbo_data.push(Vertex{ pos: [r.pos[0]            , r.pos[1] + r.size[1], r.layer], col: [col, col, col, 1.0] });
            vbo_data.push(Vertex{ pos: [r.pos[0] + r.size[0], r.pos[1] + r.size[1], r.layer], col: [col, col, col, 1.0] });
          }
          vbo.write(&vbo_data[..]);
        }
        _ => ()
      }
    }

    // Get window size
    let (display_w, display_h) = display.get_window().unwrap().get_inner_size().unwrap();

    // Create ortho projection matrix
    let proj_mat = gen_ortho_proj_mat(0.0, 0.0, display_w as f32, display_h as f32);

    // Load texture into uniforms
    let uniforms = uniform! { 
      proj_mat: proj_mat,
    };

    let draw_parameters = glium::DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        .. Default::default()
      },
      .. Default::default()
    };

    // Draw
    use glium::Surface;
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    target.clear_depth(10000.0);
    target.draw(&vbo, &indices, &shader, &uniforms, &draw_parameters).unwrap();
    target.finish().unwrap();
  }
}

