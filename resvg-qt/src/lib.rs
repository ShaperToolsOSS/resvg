// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
[resvg](https://github.com/RazrFalcon/resvg) backend implementation
using the [Qt](https://www.qt.io/) library.
*/

#![doc(html_root_url = "https://docs.rs/resvg-qt/0.9.1")]

#![warn(missing_docs)]

/// Unwraps `Option` and invokes `return` on `None`.
macro_rules! try_opt {
    ($task:expr) => {
        match $task {
            Some(v) => v,
            None => return,
        }
    };
}

/// Unwraps `Option` and invokes `return $ret` on `None`.
macro_rules! try_opt_or {
    ($task:expr, $ret:expr) => {
        match $task {
            Some(v) => v,
            None => return $ret,
        }
    };
}

/// Unwraps `Option` and invokes `return` on `None` with a warning.
macro_rules! try_opt_warn {
    ($task:expr, $msg:expr) => {
        match $task {
            Some(v) => v,
            None => {
                log::warn!($msg);
                return;
            }
        }
    };
    ($task:expr, $fmt:expr, $($arg:tt)*) => {
        match $task {
            Some(v) => v,
            None => {
                log::warn!($fmt, $($arg)*);
                return;
            }
        }
    };
}


use usvg::{NodeExt, ScreenSize};
use log::warn;

mod clip;
mod filter;
mod image;
mod layers;
mod mask;
mod paint_server;
mod path;
mod qt;
mod render;


/// Rendering options.
pub struct Options {
    /// `usvg` preprocessor options.
    pub usvg: usvg::Options,

    /// Fits the image using specified options.
    ///
    /// Does not affect rendering to canvas.
    pub fit_to: usvg::FitTo,

    /// An image background color.
    ///
    /// Sets an image background color. Does not affect rendering to canvas.
    ///
    /// `None` equals to transparent.
    pub background: Option<usvg::Color>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            usvg: usvg::Options::default(),
            fit_to: usvg::FitTo::Original,
            background: None,
        }
    }
}


/// Renders SVG to image.
pub fn render_to_image(
    tree: &usvg::Tree,
    opt: &Options,
) -> Option<qt::Image> {
    let (mut img, img_size) = render::create_root_image(tree.svg_node().size.to_screen_size(), opt)?;

    let mut painter = qt::Painter::new(&mut img);
    render_to_canvas(tree, opt, img_size, &mut painter);
    painter.end();

    Some(img)
}

/// Renders SVG node to image.
pub fn render_node_to_image(
    node: &usvg::Node,
    opt: &Options,
) -> Option<qt::Image> {
    let node_bbox = if let Some(bbox) = node.calculate_bbox() {
        bbox
    } else {
        warn!("Node '{}' has zero size.", node.id());
        return None;
    };

    let vbox = usvg::ViewBox {
        rect: node_bbox,
        aspect: usvg::AspectRatio::default(),
    };

    let (mut img, img_size) = render::create_root_image(node_bbox.size().to_screen_size(), opt)?;

    let mut painter = qt::Painter::new(&mut img);
    render_node_to_canvas(node, opt, vbox, img_size, &mut painter);
    painter.end();

    Some(img)
}

/// Renders `tree` onto the canvas.
///
/// The caller must guarantee that `img_size` is large enough.
///
/// Canvas must not have a transform.
pub fn render_to_canvas(
    tree: &usvg::Tree,
    opt: &Options,
    img_size: ScreenSize,
    painter: &mut qt::Painter,
) {
    render_node_to_canvas(&tree.root(), opt, tree.svg_node().view_box, img_size, painter);
}

/// Renders `node` onto the canvas.
///
/// The caller must guarantee that `img_size` is large enough.
///
/// Canvas must not have a transform.
pub fn render_node_to_canvas(
    node: &usvg::Node,
    opt: &Options,
    view_box: usvg::ViewBox,
    img_size: ScreenSize,
    painter: &mut qt::Painter,
) {
    render::render_node_to_canvas(node, opt, view_box, img_size, &mut render::RenderState::Ok, painter)
}

/// Converts a raw pointer into a QPainter object.
///
/// Used only by C-API.
pub unsafe fn painter_from_ptr(painter: *mut std::ffi::c_void) -> qt::Painter {
    qt::Painter::from_raw(painter as _)
}
