//! Press/hover tests for Cmd/Ctrl deep selection (Figma parity): the
//! deep-select modifier bypasses the one-level drill, selects the deepest
//! hit, and enters its parent as the sibling scope.
//!
//! Geometry discipline matches `canvas_select_drag_tests.rs`: viewport
//! 1440×900, fixtures at doc x ≥ 400 so presses land right of the chrome.

use super::WidgetHostNative;
use op_editor_core::NodeId;

const VIEWPORT_W: f32 = 1440.0;
const VIEWPORT_H: f32 = 900.0;

/// Four selection depths at the shared probe point: root > l1 > l2 > l3,
/// plus a top-level `other` rect for multi-selection.
const FOUR_LEVEL: &str = r#"{"version":"1.0.0","children":[
  {"type":"frame","id":"root","name":"Root","x":400,"y":60,"width":240,"height":240,
   "children":[
     {"type":"frame","id":"l1","name":"Level 1","x":20,"y":20,"width":200,"height":200,
      "children":[
        {"type":"frame","id":"l2","name":"Level 2","x":20,"y":20,"width":160,"height":160,
         "children":[
           {"type":"rectangle","id":"l3","name":"Level 3","x":20,"y":20,"width":60,"height":60}
         ]}
      ]}
   ]},
  {"type":"rectangle","id":"other","name":"Other","x":700,"y":60,"width":40,"height":40}
]}"#;

fn seed(host: &mut WidgetHostNative) {
    let doc = jian_ops_schema::load_str(FOUR_LEVEL)
        .expect("fixture JSON parses")
        .value;
    *host.editor_state_mut() = op_editor_core::EditorState::from_document(doc);
    host.mark_paint_dirty_for_test();
}

fn screen_at(host: &WidgetHostNative, doc_x: f32, doc_y: f32) -> (f32, f32) {
    let (cx0, cy0, _cw, _ch) = host.canvas_region(VIEWPORT_W, VIEWPORT_H);
    (cx0 + doc_x, cy0 + doc_y)
}

fn click_doc(host: &mut WidgetHostNative, doc_x: f32, doc_y: f32) {
    let (x, y) = screen_at(host, doc_x, doc_y);
    host.apply_press(x, y, VIEWPORT_W, VIEWPORT_H);
    let _ = host.apply_release_with_viewport(VIEWPORT_W, VIEWPORT_H);
}

#[test]
fn cmd_click_selects_deepest_and_enters_its_parent() {
    let mut host = WidgetHostNative::new();
    seed(&mut host);
    host.set_modifier_deep_select(true);

    click_doc(&mut host, 470.0, 130.0);

    assert_eq!(host.editor_state().selection.anchor, NodeId::new("l3"));
    assert_eq!(
        host.editor_state().editor_ui.entered_container,
        Some(NodeId::new("l2")),
        "deep select enters the deepest node's parent as the sibling scope"
    );
}

#[test]
fn cmd_double_click_stays_on_deepest() {
    let mut host = WidgetHostNative::new();
    seed(&mut host);
    host.set_modifier_deep_select(true);

    host.set_now_ms(1_000);
    click_doc(&mut host, 470.0, 130.0);
    host.set_now_ms(1_200);
    click_doc(&mut host, 470.0, 130.0);

    assert_eq!(host.editor_state().selection.anchor, NodeId::new("l3"));
    assert_eq!(
        host.editor_state().editor_ui.entered_container,
        Some(NodeId::new("l2"))
    );
    assert!(
        host.editor_state().editor_ui.image_crop_editing.is_none(),
        "a Cmd double-click is not a drill or edit gesture"
    );
}

#[test]
fn plain_click_after_cmd_click_keeps_the_entered_sibling_scope() {
    let mut host = WidgetHostNative::new();
    seed(&mut host);
    host.set_modifier_deep_select(true);
    click_doc(&mut host, 470.0, 130.0);
    host.set_modifier_deep_select(false);

    host.set_now_ms(5_000);
    click_doc(&mut host, 470.0, 130.0);

    assert_eq!(
        host.editor_state().selection.anchor,
        NodeId::new("l3"),
        "the next plain click resolves from the l2 scope, not the root"
    );
}

#[test]
fn shift_cmd_click_adds_deepest_to_the_set_without_entering_scope() {
    let mut host = WidgetHostNative::new();
    seed(&mut host);
    host.editor_state_mut()
        .set_single_selection(NodeId::new("other"));
    host.set_modifier_deep_select(true);
    host.set_modifier_shift(true);

    click_doc(&mut host, 470.0, 130.0);

    assert!(host.editor_state().is_selected(&NodeId::new("other")));
    assert!(host.editor_state().is_selected(&NodeId::new("l3")));
    assert_eq!(host.editor_state().editor_ui.entered_container, None);
}

#[test]
fn cmd_hover_previews_the_deepest_node() {
    let mut host = WidgetHostNative::new();
    seed(&mut host);
    let _ = host.layout_scene();
    host.last_viewport_w = VIEWPORT_W;
    host.last_viewport_h = VIEWPORT_H;
    let (x, y) = screen_at(&host, 470.0, 130.0);

    assert!(host.apply_cursor_move(x, y));
    assert_eq!(
        host.editor_state().editor_ui.canvas_hover_node,
        Some(NodeId::new("l1"))
    );

    host.set_modifier_deep_select(true);
    let _ = host.apply_cursor_move(x, y);
    assert_eq!(
        host.editor_state().editor_ui.canvas_hover_node,
        Some(NodeId::new("l3")),
        "pressing Cmd re-resolves hover at the deepest level"
    );
}
