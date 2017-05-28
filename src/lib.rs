/// Struct for a dynamic length
#[derive(Debug, Clone)]
pub enum DynLen {
  /// A relative length, with a number as the proportion (as a ratio with other
  /// relatively sized components) of free size this length takes up.
  /// For example, if we have 3 relative components sized 1, 2, 1 and 400px of
  /// free space, they get 100px, 200px, and 100px respectively.
  Relative(f32),

  /// An absolute length, doesn't change according to parent size.
  Absolute(f32),
}

#[derive(Debug, Clone)]
pub enum Layout {
  Horizontal, Vertical
}

#[derive(Debug, Clone)]
pub struct Node {
  id: u32,
  children_layout : Layout,
  children: Vec<Node>,
  size: DynLen,
}

impl Node {
  pub fn new(id: u32, children_layout: Layout, size: DynLen) -> Node {
    Node {
      id: id, 
      children_layout: children_layout, 
      children: Vec::new(), 
      size: size,
    }
  }

  pub fn add_child(&mut self, child: Node) {
    self.children.push(child);
  }
  pub fn add_children(&mut self, mut children: Vec<Node>) {
    self.children.append(&mut children);
  }

  /// Creates a buffer of Rect structs to be used when laying out.
  pub fn alloc_rect_buffer(&self) -> Vec<Rect> {
    let mut buf = Vec::with_capacity(self.children.len() + 1);
    for c in &self.children {
      let mut other_children = c.alloc_rect_buffer();
      buf.append(&mut other_children);
    }
    buf.push(Rect::new(self.id));
    return buf;
  }

  /// Layout this node tree, storing final rectangles in the given buffer of rects. 
  /// # Params
  /// * `rect_buffer` - A buffer of rectangles to avoid repeated allocations on
  ///                   many layouts per frame. Use alloc_rect_buffer() to
  ///                   create a buffer of the correct size.
  /// * `x` - The x position of the final node tree
  /// * `y` - The y position of the final node tree
  /// * `w` - The width given to layout the nodes.
  /// * `h` - The height given to layout the nodes.
  /// * `layer` - The lowest z index layer this node should reside in.
  ///             Children's z indexes will be increased by 1 for each 'layer'
  ///             in the tree.
  /// # Returns
  /// The number of rectangles returned. This is mainly for recursive calls
  /// into node tree children.
  /// # Panics
  /// ## In debug build
  /// * If layout isn't large enough to account for absolutely sized components.
  /// * If provided rect_buffer isn't large enough to accommodate for all final rectangles.
  pub fn layout(&self, rect_buffer: &mut [Rect], x: f32, y: f32, w: f32, h: f32, layer: f32) -> usize {
    let mut curr_index = 0;
    // First, count up free space to split between relative components, and the
    // total sum of relative proportions (to use when calculating the ratio)
    let mut free_space = 
      match self.children_layout {
        Layout::Horizontal => w,
        Layout::Vertical => h,
      };
    let mut ratio_size = 0.0;
    for c in &self.children {
      match c.size {
        DynLen::Absolute(l) => free_space -= l,
        DynLen::Relative(l) => ratio_size += l,
      }
    }

    debug_assert!(free_space > 0.0, "Not enough free space to fit in all the absolute components in layout.");

    // Add children to layed out rectangles
    // Keep track of space used laying out components for x / y positions
    let mut size_used = 0.0;
    for c in &self.children {
      debug_assert!(curr_index < rect_buffer.len(), "Layout rect buffer overflow.");
      // Calculate the size to give this child.
      let (c_x, c_y, c_w, c_h);
      match self.children_layout {
        Layout::Horizontal => {
          match c.size {
            DynLen::Absolute(l) => { c_w = l; c_h = h; }
            DynLen::Relative(l) => { c_w = free_space * l/ratio_size; c_h = h; }
          }
          size_used += c_w;
        }
        Layout::Vertical => {
          match c.size {
            DynLen::Absolute(l) => { c_w = w; c_h = l; }
            DynLen::Relative(l) => { c_w = w; c_h = free_space * l/ratio_size; }
          }
          size_used += c_h;
        }
      }

      // Calculate the position to give this child.
      match self.children_layout {
        Layout::Horizontal => { c_x = x + size_used - c_w; c_y = y; }
        Layout::Vertical => { c_x = x; c_y = y + size_used - c_h; }
      }

      // Add child's rectangles to the list
      let rects_created = c.layout(&mut rect_buffer[curr_index..], c_x, c_y, c_w, c_h, layer + 1.0);
      curr_index += rects_created;
    }
    // Add self to the buffer.
    debug_assert!(curr_index < rect_buffer.len(), "Layout rect buffer overflow.");
    rect_buffer[curr_index].id = self.id;
    rect_buffer[curr_index].pos[0] = x;
    rect_buffer[curr_index].pos[1] = y;
    rect_buffer[curr_index].size[0] = w;
    rect_buffer[curr_index].size[1] = h;
    rect_buffer[curr_index].layer = layer;
    return curr_index + 1;
  }
}

/// A rectangle with a defined size in space. Created from laying out nodes.
/// These can be drawn, and will be correctly layed out.
#[derive(Debug, Clone)]
pub struct Rect {
  pub id: u32,
  pub pos: [f32; 2],
  pub size: [f32; 2],
  /// Z index this rect resides in.
  pub layer: f32,
}

impl Rect {
  fn new(id: u32) -> Rect {
    Rect {
      id: id, pos: [0.0, 0.0], size: [0.0, 0.0], layer: 0.0,
    }
  }
}

