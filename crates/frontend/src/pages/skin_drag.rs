//! Drag-to-rotate for the skin preview: grab the figure and turn it by hand.

use super::common::Cx;
use crate::state::LauncherUI;
use gpui::{
    div, prelude::*, AnyElement, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Window,
};

/// Degrees of yaw per pixel of horizontal drag — a drag across the preview
/// turns the figure most of the way round.
const DRAG_SENSITIVITY: f32 = 0.7;

/// Start a drag when the preview box is pressed. Movement and release are
/// handled by [`drag_overlay`], which covers the page for the rest of the drag.
pub fn on_grab(this: &mut LauncherUI, e: &MouseDownEvent, _w: &mut Window, cx: &mut Cx) {
    this.skin_dragging = true;
    this.skin_drag_x = f32::from(e.position.x);
    cx.notify();
}

/// Transparent layer over the whole page, present only while dragging. Without
/// it the rotation would stall the moment the cursor left the preview box,
/// since GPUI only delivers mouse moves to the element under the cursor.
pub fn drag_overlay(cx: &mut Cx) -> AnyElement {
    div()
        .absolute()
        .inset_0()
        .cursor(CursorStyle::ClosedHand)
        .on_mouse_move(cx.listener(on_move))
        .on_mouse_up(MouseButton::Left, cx.listener(on_release))
        .into_any_element()
}

fn on_move(this: &mut LauncherUI, e: &MouseMoveEvent, _w: &mut Window, cx: &mut Cx) {
    if !this.skin_dragging {
        return;
    }
    // A release outside the window never reaches us; treat a move with no
    // button held as the end of the drag.
    if !e.dragging() {
        this.skin_dragging = false;
        cx.notify();
        return;
    }
    let x = f32::from(e.position.x);
    let dx = x - this.skin_drag_x;
    this.skin_drag_x = x;
    this.rotate_skin(dx * DRAG_SENSITIVITY, cx);
}

fn on_release(this: &mut LauncherUI, _e: &MouseUpEvent, _w: &mut Window, cx: &mut Cx) {
    this.skin_dragging = false;
    cx.notify();
}
