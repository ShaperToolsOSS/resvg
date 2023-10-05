// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::rc::Rc;

use svgtypes::FuzzyZero;

use kurbo::{ParamCurveArclen, ParamCurveExtrema};

use crate::{Rect, Line};
use super::Transform;

/// A path's absolute segment.
///
/// Unlike the SVG spec, can contain only `M`, `L`, `C` and `Z` segments.
/// All other segments will be converted into this one.
#[cfg(not(feature = "accurate-arcs"))]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum PathSegment {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    ClosePath,
}

#[cfg(feature = "accurate-arcs")]
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum PathSegment {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    ArcTo {
        rx: f64, 
        ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64, 
        y: f64, 
    },
    ClosePath,
}

/// An SVG path data container.
///
/// All segments are in absolute coordinates.
#[derive(Clone, Default, Debug)]
pub struct PathData(pub Vec<PathSegment>);

/// A reference-counted `PathData`.
///
/// `PathData` is usually pretty big and it's expensive to clone it,
/// so we are using `Rc`.
pub type SharedPathData = Rc<PathData>;

impl PathData {
    /// Creates a new path.
    #[inline]
    pub fn new() -> Self {
        PathData(Vec::new())
    }

    /// Creates a new path with a specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        PathData(Vec::with_capacity(capacity))
    }

    /// Creates a path from a rect.
    #[inline]
    pub fn from_rect(rect: Rect) -> Self {
        let mut path = PathData::with_capacity(5);
        path.push_rect(rect);
        path
    }

    /// Pushes a MoveTo segment to the path.
    #[inline]
    pub fn push_move_to(&mut self, x: f64, y: f64) {
        self.push(PathSegment::MoveTo { x, y });
    }

    /// Pushes a LineTo segment to the path.
    #[inline]
    pub fn push_line_to(&mut self, x: f64, y: f64) {
        self.push(PathSegment::LineTo { x, y });
    }

    /// Pushes a CurveTo segment to the path.
    #[inline]
    pub fn push_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.push(PathSegment::CurveTo { x1, y1, x2, y2, x, y });
    }

    /// Pushes a QuadTo segment to the path.
    ///
    /// Will be converted into cubic curve.
    #[inline]
    pub fn push_quad_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        let (prev_x, prev_y) = self.last_pos();
        self.push(quad_to_curve(prev_x, prev_y, x1, y1, x, y));
    }

    //Shaper needs accurate arcs, as we cut a lot of circles
    //So pass arc commands through to usvg path, rather than convert to beziers
    #[cfg(feature = "accurate-arcs")]
    #[inline]
    pub fn push_arc_to(
        &mut self,
        rx: f64, ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64, y: f64,
    ) {
        self.push(PathSegment::ArcTo {rx, ry, x_axis_rotation, large_arc, sweep, x, y });
    }

    /// Converts svg path arc command to kurbo::Arc
    ///
    /// Used by:
    ///   #[cfg(not(feature = "accurate-arcs"))]
    ///   pub fn push_arc_to()
    ///
    ///   #[cfg(feature = "accurate-arcs")]
    ///   arc_length
    pub fn convert_svg_arc(
        prev_x: f64, prev_y:f64,
        rx: f64, ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64, y: f64,
    ) -> Option<kurbo::Arc>{

        let svg_arc = kurbo::SvgArc {
            from: kurbo::Point::new(prev_x, prev_y),
            to: kurbo::Point::new(x, y),
            radii: kurbo::Vec2::new(rx, ry),
            x_rotation: x_axis_rotation.to_radians(),
            large_arc,
            sweep,
        };

        kurbo::Arc::from_svg_arc(&svg_arc)
    }

    /// Pushes an ArcTo segment to the path.
    ///
    /// Arc will be converted into cubic curves.
    #[cfg(not(feature = "accurate-arcs"))]
    pub fn push_arc_to(
        &mut self,
        rx: f64, ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64, y: f64,
    ) {
        let (prev_x, prev_y) = self.last_pos();

        match PathData::convert_svg_arc(prev_x, prev_y,rx, ry, x_axis_rotation, large_arc, sweep, x, y) {
            
            Some(arc) => {
                arc.to_cubic_beziers(0.1, |p1, p2, p| {
                    self.push_curve_to(p1.x, p1.y, p2.x, p2.y, p.x, p.y);
                });
            }
            None => {
                self.push_line_to(x, y);
            }
        }
    }

    /// Pushes a ClosePath segment to the path.
    #[inline]
    pub fn push_close_path(&mut self) {
        self.push(PathSegment::ClosePath);
    }

    /// Pushes a rect to the path.
    #[inline]
    pub fn push_rect(&mut self, rect: Rect) {
        self.extend_from_slice(&[
            PathSegment::MoveTo { x: rect.x(),     y: rect.y() },
            PathSegment::LineTo { x: rect.right(), y: rect.y() },
            PathSegment::LineTo { x: rect.right(), y: rect.bottom() },
            PathSegment::LineTo { x: rect.x(),     y: rect.bottom() },
            PathSegment::ClosePath,
        ]);
    }

    #[inline]
    #[cfg(not(feature = "accurate-arcs"))]
    fn last_pos(&self) -> (f64, f64) {
        let seg = self.last().expect("path must not be empty").clone();
        match seg {
              PathSegment::MoveTo { x, y }
            | PathSegment::LineTo { x, y }
            | PathSegment::CurveTo { x, y, .. } => {
               (x, y)
            }
            PathSegment::ClosePath => {
                panic!("the previous segment must be M/L/C")
            }
        }
    }

    #[inline]
    #[cfg(feature = "accurate-arcs")]
    fn last_pos(&self) -> (f64, f64) {
        let seg = self.last().expect("path must not be empty").clone();
        match seg {
              PathSegment::MoveTo { x, y }
            | PathSegment::LineTo { x, y }
            | PathSegment::CurveTo { x, y, .. }
            | PathSegment::ArcTo { x, y, .. } => {
               (x, y)
            }
            PathSegment::ClosePath => {
                panic!("the previous segment must be M/L/C")
            }
        }
    }

    /// Calculates path's bounding box.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn bbox(&self) -> Option<Rect> {
        calc_bbox(self)
    }

    /// Calculates path's bounding box with a specified transform.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn bbox_with_transform(
        &self,
        ts: Transform,
        stroke: Option<&super::Stroke>,
    ) -> Option<Rect> {
        calc_bbox_with_transform(self, ts, stroke)
    }

    /// Checks that path has a bounding box.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn has_bbox(&self) -> bool {
        has_bbox(self)
    }

    /// Calculates path's length.
    ///
    /// Length from the first segment to the first MoveTo, ClosePath or slice end.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn length(&self) -> f64 {
        calc_length(self)
    }

    /// Applies the transform to the path.
    #[inline]
    pub fn transform(&mut self, ts: Transform) {
        transform_path(self, ts);
    }

    /// Applies the transform to the path from the specified offset.
    #[inline]
    pub fn transform_from(&mut self, offset: usize, ts: Transform) {
        transform_path(&mut self[offset..], ts);
    }

    /// Returns an iterator over path subpaths.
    #[inline]
    pub fn subpaths(&self) -> SubPathIter {
        SubPathIter {
            path: self,
            index: 0,
        }
    }
}

impl std::ops::Deref for PathData {
    type Target = Vec<PathSegment>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PathData {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


/// An iterator over `PathData` subpaths.
#[allow(missing_debug_implementations)]
pub struct SubPathIter<'a> {
    path: &'a [PathSegment],
    index: usize,
}

impl<'a> Iterator for SubPathIter<'a> {
    type Item = SubPathData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.path.len() {
            return None;
        }

        let mut i = self.index;
        while i < self.path.len() {
            match self.path[i] {
                PathSegment::MoveTo { .. } => {
                    if i != self.index {
                        break;
                    }
                }
                PathSegment::ClosePath => {
                    i += 1;
                    break;
                }
                _ => {}
            }

            i += 1;
        }

        let start = self.index;
        self.index = i;

        Some(SubPathData(&self.path[start..i]))
    }
}


/// A reference to a `PathData` subpath.
#[derive(Clone, Copy, Debug)]
pub struct SubPathData<'a>(pub &'a [PathSegment]);

impl<'a> SubPathData<'a> {
    /// Calculates path's bounding box.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn bbox(&self) -> Option<Rect> {
        calc_bbox(self)
    }

    /// Calculates path's bounding box with a specified transform.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn bbox_with_transform(
        &self,
        ts: Transform,
        stroke: Option<&super::Stroke>,
    ) -> Option<Rect> {
        calc_bbox_with_transform(self, ts, stroke)
    }

    /// Checks that path has a bounding box.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn has_bbox(&self) -> bool {
        has_bbox(self)
    }

    /// Calculates path's length.
    ///
    /// This operation is expensive.
    #[inline]
    pub fn length(&self) -> f64 {
        calc_length(self)
    }
}

impl std::ops::Deref for SubPathData<'_> {
    type Target = [PathSegment];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}


fn calc_bbox(segments: &[PathSegment]) -> Option<Rect> {
    debug_assert!(!segments.is_empty());

    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    let mut minx = 0.0;
    let mut miny = 0.0;
    let mut maxx = 0.0;
    let mut maxy = 0.0;

    if let PathSegment::MoveTo { x, y } = segments[0].clone() {
        prev_x = x;
        prev_y = y;
        minx = x;
        miny = y;
        maxx = x;
        maxy = y;
    }

    for seg in segments.iter().cloned() {
        match seg {
              PathSegment::MoveTo { x, y }
            | PathSegment::LineTo { x, y } => {
                prev_x = x;
                prev_y = y;

                if x > maxx { maxx = x; }
                else if x < minx { minx = x; }

                if y > maxy { maxy = y; }
                else if y < miny { miny = y; }
            }
            PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                prev_x = x;
                prev_y = y;
                
                let curve = kurbo::CubicBez::from_points(prev_x, prev_y, x1, y1, x2, y2, x, y);
                let r = curve.bounding_box();

                if r.x0 < minx { minx = r.x0; }
                if r.x1 > maxx { maxx = r.x1; }
                if r.y0 < miny { miny = r.y0; }
                if r.y1 > maxy { maxy = r.y1; }
            }
            #[cfg(feature="accurate-arcs")]
            PathSegment::ArcTo {rx, ry, x_axis_rotation, large_arc, sweep, x, y} => {

                match PathData::convert_svg_arc(prev_x, prev_y, rx, ry, x_axis_rotation, large_arc, sweep, x, y) {
                    Some(arc) => {
                        prev_x = x;
                        prev_y = y;
                        
                        use kurbo::Shape;
                        let r = arc.bounding_box();
                        if r.x0 < minx { minx = r.x0; }
                        if r.x1 > maxx { maxx = r.x1; }
                        if r.y0 < miny { miny = r.y0; }
                        if r.y1 > maxy { maxy = r.y1; }
                    }
                    None => {
                        //If arc is really a line, update bbox as LineTo
                        
                        prev_x = x;
                        prev_y = y;
                        if x > maxx { maxx = x; }
                        else if x < minx { minx = x; }

                        if y > maxy { maxy = y; }
                        else if y < miny { miny = y; }
                    }
                }
            }
            PathSegment::ClosePath => {}
        }
    }

    let width = maxx - minx;
    let height = maxy - miny;

    Rect::new(minx, miny, width, height)
}



fn calc_bbox_with_transform(
    segments: &[PathSegment],
    ts: Transform,
    stroke: Option<&super::Stroke>,
) -> Option<Rect> {
    debug_assert!(!segments.is_empty());

    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    let mut minx = 0.0;
    let mut miny = 0.0;
    let mut maxx = 0.0;
    let mut maxy = 0.0;

    if let Some(PathSegment::MoveTo { x, y }) = TransformedPath::new(segments, ts).next() {
        prev_x = x;
        prev_y = y;
        minx = x;
        miny = y;
        maxx = x;
        maxy = y;
    }

    for seg in TransformedPath::new(segments, ts) {
        match seg {
              PathSegment::MoveTo { x, y }
            | PathSegment::LineTo { x, y } => {
                prev_x = x;
                prev_y = y;

                if x > maxx { maxx = x; }
                else if x < minx { minx = x; }

                if y > maxy { maxy = y; }
                else if y < miny { miny = y; }
            }
            PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                let curve = kurbo::CubicBez::from_points(prev_x, prev_y, x1, y1, x2, y2, x, y);
                let r = curve.bounding_box();

                if r.x0 < minx { minx = r.x0; }
                if r.x1 > maxx { maxx = r.x1; }
                if r.y0 < miny { miny = r.y0; }
                if r.y1 > maxy { maxy = r.y1; }
            }
            #[cfg(feature="accurate-arcs")]
            PathSegment::ArcTo {rx, ry, x_axis_rotation, large_arc, sweep, x, y} => {

                match PathData::convert_svg_arc(prev_x, prev_y, rx, ry, x_axis_rotation, large_arc, sweep, x, y) {
                    Some(arc) => {
                        prev_x = x;
                        prev_y = y;
                        
                        use kurbo::Shape;
                        let r = arc.bounding_box();
                        if r.x0 < minx { minx = r.x0; }
                        if r.x1 > maxx { maxx = r.x1; }
                        if r.y0 < miny { miny = r.y0; }
                        if r.y1 > maxy { maxy = r.y1; }
                    }
                    None => {
                        //If arc is really a line, update bbox as LineTo
                        
                        prev_x = x;
                        prev_y = y;
                        if x > maxx { maxx = x; }
                        else if x < minx { minx = x; }

                        if y > maxy { maxy = y; }
                        else if y < miny { miny = y; }
                    }
                }
            }
            PathSegment::ClosePath => {}
        }
    }

    // TODO: find a better way
    // It's an approximation, but it's better than nothing.
    if let Some(ref stroke) = stroke {
        let w = stroke.width.value() / 2.0;
        minx -= w;
        miny -= w;
        maxx += w;
        maxy += w;
    }

    let width = maxx - minx;
    let height = maxy - miny;

    Rect::new(minx, miny, width, height)
}

fn has_bbox(segments: &[PathSegment]) -> bool {
    debug_assert!(!segments.is_empty());

    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    let mut minx = 0.0;
    let mut miny = 0.0;
    let mut maxx = 0.0;
    let mut maxy = 0.0;

    if let PathSegment::MoveTo { x, y } = segments[0] {
        prev_x = x;
        prev_y = y;
        minx = x;
        miny = y;
        maxx = x;
        maxy = y;
    }

    for seg in segments {
        match *seg {
              PathSegment::MoveTo { x, y }
            | PathSegment::LineTo { x, y } => {
                prev_x = x;
                prev_y = y;

                if x > maxx { maxx = x; }
                else if x < minx { minx = x; }

                if y > maxy { maxy = y; }
                else if y < miny { miny = y; }
            }
            PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                let curve = kurbo::CubicBez::from_points(prev_x, prev_y, x1, y1, x2, y2, x, y);
                let r = curve.bounding_box();

                if r.x0 < minx { minx = r.x0; }
                if r.x1 > maxx { maxx = r.x1; }
                if r.x0 < miny { miny = r.y0; }
                if r.y1 > maxy { maxy = r.y1; }
            }
            #[cfg(feature="accurate-arcs")]
            PathSegment::ArcTo {rx, ry, x_axis_rotation, large_arc, sweep, x, y} => {

                match PathData::convert_svg_arc(prev_x, prev_y, rx, ry, x_axis_rotation, large_arc, sweep, x, y) {
                    Some(arc) => {
                        prev_x = x;
                        prev_y = y;
                        
                        use kurbo::Shape;
                        let r = arc.bounding_box();
                        if r.x0 < minx { minx = r.x0; }
                        if r.x1 > maxx { maxx = r.x1; }
                        if r.y0 < miny { miny = r.y0; }
                        if r.y1 > maxy { maxy = r.y1; }
                    }
                    None => {
                        //If arc is really a line, update bbox as LineTo
                        
                        prev_x = x;
                        prev_y = y;
                        if x > maxx { maxx = x; }
                        else if x < minx { minx = x; }

                        if y > maxy { maxy = y; }
                        else if y < miny { miny = y; }
                    }
                }
            }
            PathSegment::ClosePath => {}
        }

        let width = (maxx - minx) as f64;
        let height = (maxy - miny) as f64;
        if !(width.is_fuzzy_zero() || height.is_fuzzy_zero()) {
            return true;
        }
    }

    false
}

fn calc_length(segments: &[PathSegment]) -> f64 {
    debug_assert!(!segments.is_empty());

    let (mut prev_x, mut prev_y) = {
        if let PathSegment::MoveTo { x, y } = segments[0] {
            (x, y)
        } else {
            panic!("first segment must be MoveTo");
        }
    };

    let start_x = prev_x;
    let start_y = prev_y;

    let mut is_first_seg = true;
    let mut length = 0.0f64;
    for seg in segments {
        match *seg {
            PathSegment::MoveTo { .. } => {
                if !is_first_seg {
                    break;
                }
            }
            PathSegment::LineTo { x, y } => {
                length += Line::new(prev_x, prev_y, x, y).length();

                prev_x = x;
                prev_y = y;
            }
            PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                let curve = kurbo::CubicBez::from_points(prev_x, prev_y, x1, y1, x2, y2, x, y);
                length += curve.arclen(1.0);

                prev_x = x;
                prev_y = y;
            }
            
            #[cfg(feature="accurate-arcs")]
            PathSegment::ArcTo {rx, ry, x_axis_rotation, large_arc, sweep, x, y} => {

                match PathData::convert_svg_arc(prev_x, prev_y, rx, ry, x_axis_rotation, large_arc, sweep, x, y) {
                    Some(arc) => {
                        arc.to_cubic_beziers(0.1,|p1, p2, p| {
                            //to_cubic_beziers() calls this closure function for each path segment

                            let curve = kurbo::CubicBez::from_points(prev_x, prev_y, p1.x, p1.y, p2.x, p2.y, p.x, p.y);

                            length += curve.arclen(1.0);

                            //Advance prev_pt to curve end point
                            prev_x = p.x;
                            prev_y = p.y;
                        }); 
                    }
                    None => {
                        length += Line::new(prev_x, prev_y, x, y).length();

                        prev_x = x;
                        prev_y = y;
                    }
                } 
            }
            PathSegment::ClosePath => {
                length += Line::new(prev_x, prev_y, start_x, start_y).length();
                break;
            }
        }

        is_first_seg = false;
    }

    length
}


#[cfg(feature = "accurate-arcs")]
pub mod arc_util {
    use crate::{PathData};
    use crate::Transform;
    use kurbo::{Point, Vec2, Arc, SvgArc, CubicBez, QuadBez};
    use std::f64::consts::PI;
    use std::ops::{Add,Sub};

    /// Determines if transform flips handedness of arc parameters
    pub fn does_transform_flip_handedness(ts : Transform) -> bool {
        //Extract basis vectors from columns of the transform
        let x_axis = Vec2::new(ts.a, ts.b);
        let y_axis = Vec2::new(ts.c, ts.d);

        let typical_x_axis = Vec2::new(y_axis.y, -y_axis.x);

        typical_x_axis.dot(x_axis) < 0.
    }

    /// Transforms arc in centerpoint format
    pub fn transform_centerpoint_arc(arc: &mut Arc, ts: Transform) {
        
        //Transformed center
        let center_t_tuple = ts.apply(arc.center.x, arc.center.y);
        //No easy way to spread a tuple into function args and need to keep original center value, so need intermediate tuple.

        let center_t = Vec2::new(center_t_tuple.0, center_t_tuple.1);

        //Compute vectors for rx, ry

        let xr = arc.x_rotation % (2.0 * PI);
        let x_rotation_cos = xr.cos();
        let x_rotation_sin = xr.sin();

        //Rotate rx and ry each by x_rotation ,then add to center
        let rx_rot = Vec2::new(arc.radii.x * x_rotation_cos, arc.radii.x * x_rotation_sin).add(arc.center.to_vec2());

        let ry_rot = Vec2::new(-arc.radii.y * x_rotation_sin, arc.radii.y * x_rotation_cos).add(arc.center.to_vec2());
        // Transform rotated radii vectors to new coordinate space

        let temp_rx_t = ts.apply(rx_rot.x, rx_rot.y);
        let rx_t = Vec2::new(temp_rx_t.0, temp_rx_t.1);
        
        let temp_ry_t = ts.apply(ry_rot.x, ry_rot.y);
        let ry_t = Vec2::new(temp_ry_t.0, temp_ry_t.1);

        //Subtract rx_t, ry_t from center_t and get lengths for radii_t
        let radii_t = Vec2::new(
            rx_t.sub(center_t).hypot(),
            ry_t.sub(center_t).hypot());

        //New x_rotation
        let x_rotation_t = rx_t.sub(center_t).atan2();

        //start_angle and sweep angle are unchanged by transform, unless there is a flip
        
        //Test flip handedness
        let flip = if does_transform_flip_handedness(ts){
            -1.0_f64
        } else {
            1.0_f64
        };
        
        //Now modify the arc
        *arc = Arc {
            center: center_t.to_point(),
            radii: radii_t,
            start_angle: flip * arc.start_angle,
            sweep_angle: flip * arc.sweep_angle,
            x_rotation: x_rotation_t,
            //In case arc is ever extended with other fields, make sure to keep them.
            ..*arc
        }; 
    }

    /// Transforms SVG format arc
    ///
    /// Converts SVG endpoint parameterized arc to centerpoint arc, applies transform, and then converts back to SVG format. 
    pub fn transform_svg_arc(
        prev_x: f64, 
        prev_y: f64,
        rx: f64, 
        ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64, 
        y: f64, 
        ts: Transform,
    ) -> Option<SvgArc> {
   
        //Convert svg arc to centerpoint arc
        let arc_result = PathData::convert_svg_arc(
            prev_x, 
            prev_y, 
            rx, 
            ry, 
            x_axis_rotation, 
            large_arc, 
            sweep, 
            x, 
            y,
        );
        match arc_result{
            Some(mut arc) => {
                //Transform centerpoint arc
                transform_centerpoint_arc(&mut arc, ts);
                
                Some(SvgArc::from_arc(&arc))
            }
            None => None
        }
    }


    /// Return arc tangent vector at vertex
    ///
    /// vertex_distance is normalized distance from beginning of arc 0.0 = start vertex, 1.0 = end vertex
    ///
    /// Finding tangent vector of arc points using derivative of centerpoint parameterization
    /// See https://www.w3.org/TR/SVG/implnote.html#ArcImplementationNotes
    pub fn centerpoint_arc_tangent(arc: Arc, vertex_distance: f64) -> Vec2 {

        let Arc {radii: Vec2 {x: rx, y: ry}, start_angle, sweep_angle, x_rotation, ..} = arc;

        let vertex_angle = start_angle + vertex_distance * sweep_angle;


        let tx = -rx * x_rotation.cos() * vertex_angle.sin() - ry * x_rotation.sin() * vertex_angle.cos();

        let ty = -rx * x_rotation.sin() * vertex_angle.sin() + ry * x_rotation.cos() * vertex_angle.cos();

        Vec2::new(tx, ty)
    }
}

fn transform_path(segments: &mut [PathSegment], ts: Transform) {
    
    if !segments.is_empty(){

        let (mut _prev_x, mut _prev_y) = {
            if let PathSegment::MoveTo { x, y } = segments[0] {
                (x, y)
            } else {
                panic!("first segment must be MoveTo");
            }
        };
        
        for seg in segments {
            match seg {
                PathSegment::MoveTo { x, y } => {
                    ts.apply_to(x, y);
                    _prev_x = *x;
                    _prev_y = *y;
                }
                PathSegment::LineTo { x, y } => {
                    ts.apply_to(x, y);
                    _prev_x = *x;
                    _prev_y = *y;
                }
                PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                    ts.apply_to(x1, y1);
                    ts.apply_to(x2, y2);
                    ts.apply_to(x, y);
                    _prev_x = *x;
                    _prev_y = *y;
                }

                #[cfg(feature="accurate-arcs")]
                PathSegment::ArcTo{
                    rx, 
                    ry, 
                    x_axis_rotation,
                    large_arc,
                    sweep,
                    x, 
                    y,} => {
                    
                    match arc_util::transform_svg_arc( _prev_x, _prev_y, *rx, *ry, *x_axis_rotation, *large_arc, *sweep, *x, *y, ts) {
                        Some(svg_arc_t) => {
                            *rx = svg_arc_t.radii.x;
                            *ry = svg_arc_t.radii.y;
                            *x = svg_arc_t.to.x;
                            *y = svg_arc_t.to.y;
                            *x_axis_rotation = svg_arc_t.x_rotation;
                            *large_arc = svg_arc_t.large_arc;
                            *sweep = svg_arc_t.sweep;
                            _prev_x = *x;
                            _prev_y = *y;
                        }
                        None => {
                            //If arc segment is really a line, transform the endpoint like a line.
                            ts.apply_to(x, y);
                            _prev_x = *x;
                            _prev_y = *y;
                        }
                    }
                }
                PathSegment::ClosePath => {}
            }
        }
    }
}


/// An iterator over transformed path segments.
#[allow(missing_debug_implementations)]
pub struct TransformedPath<'a> {
    segments: &'a [PathSegment],
    ts: Transform,
    idx: usize,
    //Attributes on expressions are experimental, so can't use  #[cfg(feature="accurate-arcs")] on usage of self.prev_pt below.

    prev_pt: kurbo::Point,

}

impl<'a> TransformedPath<'a> {
    /// Creates a new `TransformedPath` iterator.
    #[inline]
    pub fn new(segments: &'a [PathSegment], ts: Transform) -> Self {
        TransformedPath { segments, ts, idx: 0, prev_pt: kurbo::Point::new(0.,0.) }
    }
}

impl<'a> Iterator for TransformedPath<'a> {
    type Item = PathSegment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.segments.len() {
            return None;
        }

        let seg = match self.segments[self.idx] {
            PathSegment::MoveTo { x, y } => {
                self.prev_pt = kurbo::Point::new(x,y);
                
                let (x, y) = self.ts.apply(x, y);
                PathSegment::MoveTo { x, y }
            }
            PathSegment::LineTo { x, y } => {
                self.prev_pt = kurbo::Point::new(x,y);
                
                let (x, y) = self.ts.apply(x, y);
                PathSegment::LineTo { x, y }
            }
            #[cfg(feature="accurate-arcs")]
            PathSegment::ArcTo {rx, ry, x_axis_rotation, large_arc, sweep, x, y} => {


                match arc_util::transform_svg_arc(self.prev_pt.x, self.prev_pt.y, rx, ry, x_axis_rotation, large_arc, sweep, x, y, self.ts) {

                    Some(svg_arc_t) => {
                        self.prev_pt = kurbo::Point::new(svg_arc_t.to.x, svg_arc_t.to.y);
                     
                        //Return new pathSegment
                        PathSegment::ArcTo {
                            rx: svg_arc_t.radii.x,
                            ry: svg_arc_t.radii.y,
                            x: svg_arc_t.to.x,
                            y: svg_arc_t.to.y,
                            x_axis_rotation: svg_arc_t.x_rotation,
                            large_arc: svg_arc_t.large_arc,
                            sweep: svg_arc_t.sweep,
                        }
                    }
                    None => {
                        //If arc segment is really a line, transform the endpoint like a line.
                       self.prev_pt = kurbo::Point::new(x,y);
                
                        let (x, y) = self.ts.apply(x, y);
                        PathSegment::LineTo { x, y }
                    }

                }
            }

            PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                self.prev_pt = kurbo::Point::new(x,y);
                
                let (x1, y1) = self.ts.apply(x1, y1);
                let (x2, y2) = self.ts.apply(x2, y2);
                let (x,  y)  = self.ts.apply(x, y);
                PathSegment::CurveTo { x1, y1, x2, y2, x, y }
            }
            PathSegment::ClosePath => PathSegment::ClosePath,
        };

        self.idx += 1;

        Some(seg)
    }
}


#[inline]
fn quad_to_curve(px: f64, py: f64, x1: f64, y1: f64, x: f64, y: f64) -> PathSegment {
    #[inline]
    fn calc(n1: f64, n2: f64) -> f64 {
        (n1 + n2 * 2.0) / 3.0
    }

    PathSegment::CurveTo {
        x1: calc(px, x1), y1: calc(py, y1),
        x2:  calc(x, x1), y2:  calc(y, y1),
        x, y,
    }
}


pub(crate) trait CubicBezExt {
    fn from_points(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Self;
}

impl CubicBezExt for kurbo::CubicBez {
    fn from_points(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Self {
        kurbo::CubicBez {
            p0: kurbo::Point::new(px, py),
            p1: kurbo::Point::new(x1, y1),
            p2: kurbo::Point::new(x2, y2),
            p3: kurbo::Point::new(x, y),
        }
    }
}
