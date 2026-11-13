//! Equipped armor icons and remaining durability.

macro_rules! document {
    () => {
        concat!(
            r##"{"armor_edit":{"type":"panel","size":[84,24],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":10,"bindings":[{"binding_name":"#armor_edit_visible","binding_name_override":"#visible"},{"binding_name":"#armor_edit_offset","binding_name_override":"#offset"},{"binding_name":"#armor_edit_w","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_edit_h","binding_name_override":"#size_binding_y_absolute"}],"controls":["##,
            $crate::counter::overlay_doc!("armoredit"),
            r##"]}},"##,
            r##"{"armor_strip":{"type":"panel","size":[82,22],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":0,"bindings":[{"binding_name":"#armor_strip_pos","binding_name_override":"#offset"},{"binding_name":"#armor_any","binding_name_override":"#visible"},{"binding_name":"#armor_dim_82","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_22","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"frame":{"type":"panel","size":["100%","100%"],"bindings":[{"binding_name":"#armor_join","binding_name_override":"#visible"}],"controls":[{"top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%",1],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,0.55],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"}]}},{"bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%",1],"anchor_from":"bottom_left","anchor_to":"bottom_left","radius":0,"color":[0,0,0,0.55],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"}]}},{"left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,"100%"],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,0.55],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"}]}},{"right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,"100%"],"anchor_from":"top_right","anchor_to":"top_right","radius":0,"color":[0,0,0,0.55],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"}]}}]}}]}},"##,
            r##"{"armor_slot_0":{"type":"panel","size":[20,20],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":1,"bindings":[{"binding_name":"#armor_slot_pos_0","binding_name_override":"#offset"},{"binding_name":"#armor_any","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"slot_outer":{"type":"panel","size":[20,20],"bindings":[{"binding_name":"#armor_background","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"ring_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"ring_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,18],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_18","binding_name_override":"#offset"}]}},{"ring_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[0,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_2","binding_name_override":"#offset"}]}},{"ring_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[18,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_18_2","binding_name_override":"#offset"}]}},{"edge_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,19],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_19","binding_name_override":"#offset"}]}},{"edge_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[19,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_19_0","binding_name_override":"#offset"}]}},{"slot_inset":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,16],"offset":[2,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,0.5],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_2_2","binding_name_override":"#offset"}]}}]}}]}},"##,
            r##"{"armor_0":{"type":"custom","renderer":"inventory_item_renderer","size":[16,16],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":2,"bindings":[{"binding_name":"#armor_pos_0","binding_name_override":"#offset"},{"binding_name":"#armor_item_0","binding_name_override":"#item_renderer_data"},{"binding_name":"#armor_visible_0","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"armor_bar_0":{"type":"panel","size":[16,3],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":5,"bindings":[{"binding_name":"#armor_bar_pos_0","binding_name_override":"#offset"},{"binding_name":"#armor_bar_visible_0","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"bar_back":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,3],"radius":0,"color":[0,0,0,1],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}]}},{"bar_fill":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[14,1],"offset":[1,1],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,1,0.1,1],"bindings":[{"binding_name":"#armor_bar_width_0","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_bar_color_0","binding_name_override":"#color"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"}]}}]}},"##,
            r##"{"armor_text_panel_0":{"type":"panel","size":[20,10],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":3,"bindings":[{"binding_name":"#armor_text_pos_0","binding_name_override":"#offset"},{"binding_name":"#armor_visible_0","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"text_shadow":{"type":"label","text":"#armor_text_0","size":[20,10],"offset":[1,1],"color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":1,"bindings":[{"binding_name":"#armor_text_0"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}},{"text":{"type":"label","text":"#armor_text_0","size":[20,10],"layer":2,"color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"bindings":[{"binding_name":"#armor_text_0"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}}]}},"##,
            r##"{"armor_slot_1":{"type":"panel","size":[20,20],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":1,"bindings":[{"binding_name":"#armor_slot_pos_1","binding_name_override":"#offset"},{"binding_name":"#armor_any","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"slot_outer":{"type":"panel","size":[20,20],"bindings":[{"binding_name":"#armor_background","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"ring_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"ring_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,18],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_18","binding_name_override":"#offset"}]}},{"ring_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[0,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_2","binding_name_override":"#offset"}]}},{"ring_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[18,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_18_2","binding_name_override":"#offset"}]}},{"edge_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,19],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_19","binding_name_override":"#offset"}]}},{"edge_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[19,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_19_0","binding_name_override":"#offset"}]}},{"slot_inset":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,16],"offset":[2,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,0.5],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_2_2","binding_name_override":"#offset"}]}}]}}]}},"##,
            r##"{"armor_1":{"type":"custom","renderer":"inventory_item_renderer","size":[16,16],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":2,"bindings":[{"binding_name":"#armor_pos_1","binding_name_override":"#offset"},{"binding_name":"#armor_item_1","binding_name_override":"#item_renderer_data"},{"binding_name":"#armor_visible_1","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"armor_bar_1":{"type":"panel","size":[16,3],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":5,"bindings":[{"binding_name":"#armor_bar_pos_1","binding_name_override":"#offset"},{"binding_name":"#armor_bar_visible_1","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"bar_back":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,3],"radius":0,"color":[0,0,0,1],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}]}},{"bar_fill":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[14,1],"offset":[1,1],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,1,0.1,1],"bindings":[{"binding_name":"#armor_bar_width_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_bar_color_1","binding_name_override":"#color"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"}]}}]}},"##,
            r##"{"armor_text_panel_1":{"type":"panel","size":[20,10],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":3,"bindings":[{"binding_name":"#armor_text_pos_1","binding_name_override":"#offset"},{"binding_name":"#armor_visible_1","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"text_shadow":{"type":"label","text":"#armor_text_1","size":[20,10],"offset":[1,1],"color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":1,"bindings":[{"binding_name":"#armor_text_1"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}},{"text":{"type":"label","text":"#armor_text_1","size":[20,10],"layer":2,"color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"bindings":[{"binding_name":"#armor_text_1"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}}]}},"##,
            r##"{"armor_slot_2":{"type":"panel","size":[20,20],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":1,"bindings":[{"binding_name":"#armor_slot_pos_2","binding_name_override":"#offset"},{"binding_name":"#armor_any","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"slot_outer":{"type":"panel","size":[20,20],"bindings":[{"binding_name":"#armor_background","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"ring_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"ring_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,18],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_18","binding_name_override":"#offset"}]}},{"ring_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[0,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_2","binding_name_override":"#offset"}]}},{"ring_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[18,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_18_2","binding_name_override":"#offset"}]}},{"edge_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,19],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_19","binding_name_override":"#offset"}]}},{"edge_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[19,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_19_0","binding_name_override":"#offset"}]}},{"slot_inset":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,16],"offset":[2,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,0.5],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_2_2","binding_name_override":"#offset"}]}}]}}]}},"##,
            r##"{"armor_2":{"type":"custom","renderer":"inventory_item_renderer","size":[16,16],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":2,"bindings":[{"binding_name":"#armor_pos_2","binding_name_override":"#offset"},{"binding_name":"#armor_item_2","binding_name_override":"#item_renderer_data"},{"binding_name":"#armor_visible_2","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"armor_bar_2":{"type":"panel","size":[16,3],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":5,"bindings":[{"binding_name":"#armor_bar_pos_2","binding_name_override":"#offset"},{"binding_name":"#armor_bar_visible_2","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"bar_back":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,3],"radius":0,"color":[0,0,0,1],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}]}},{"bar_fill":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[14,1],"offset":[1,1],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,1,0.1,1],"bindings":[{"binding_name":"#armor_bar_width_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_bar_color_2","binding_name_override":"#color"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"}]}}]}},"##,
            r##"{"armor_text_panel_2":{"type":"panel","size":[20,10],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":3,"bindings":[{"binding_name":"#armor_text_pos_2","binding_name_override":"#offset"},{"binding_name":"#armor_visible_2","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"text_shadow":{"type":"label","text":"#armor_text_2","size":[20,10],"offset":[1,1],"color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":1,"bindings":[{"binding_name":"#armor_text_2"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}},{"text":{"type":"label","text":"#armor_text_2","size":[20,10],"layer":2,"color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"bindings":[{"binding_name":"#armor_text_2"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}}]}},"##,
            r##"{"armor_slot_3":{"type":"panel","size":[20,20],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":1,"bindings":[{"binding_name":"#armor_slot_pos_3","binding_name_override":"#offset"},{"binding_name":"#armor_any","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"slot_outer":{"type":"panel","size":[20,20],"bindings":[{"binding_name":"#armor_background","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"ring_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"ring_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,2],"offset":[0,18],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_18","binding_name_override":"#offset"}]}},{"ring_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[0,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_2","binding_name_override":"#offset"}]}},{"ring_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[2,16],"offset":[18,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.365,0.365,0.365,0.94],"bindings":[{"binding_name":"#armor_dim_2","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_18_2","binding_name_override":"#offset"}]}},{"edge_top":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_left":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.58,0.58,0.58,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_0","binding_name_override":"#offset"}]}},{"edge_bottom":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[20,1],"offset":[0,19],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_0_19","binding_name_override":"#offset"}]}},{"edge_right":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,20],"offset":[19,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,0.2,0.2,1],"bindings":[{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_19_0","binding_name_override":"#offset"}]}},{"slot_inset":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,16],"offset":[2,2],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,0.5],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_2_2","binding_name_override":"#offset"}]}}]}}]}},"##,
            r##"{"armor_3":{"type":"custom","renderer":"inventory_item_renderer","size":[16,16],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":2,"bindings":[{"binding_name":"#armor_pos_3","binding_name_override":"#offset"},{"binding_name":"#armor_item_3","binding_name_override":"#item_renderer_data"},{"binding_name":"#armor_visible_3","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"armor_bar_3":{"type":"panel","size":[16,3],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":5,"bindings":[{"binding_name":"#armor_bar_pos_3","binding_name_override":"#offset"},{"binding_name":"#armor_bar_visible_3","binding_name_override":"#visible"},{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"bar_back":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[16,3],"radius":0,"color":[0,0,0,1],"bindings":[{"binding_name":"#armor_dim_16","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_3","binding_name_override":"#size_binding_y_absolute"}]}},{"bar_fill":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[14,1],"offset":[1,1],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.2,1,0.1,1],"bindings":[{"binding_name":"#armor_bar_width_3","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_bar_color_3","binding_name_override":"#color"},{"binding_name":"#armor_dim_1","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"}]}}]}},"##,
            r##"{"armor_text_panel_3":{"type":"panel","size":[20,10],"anchor_from":"bottom_left","anchor_to":"bottom_left","layer":3,"bindings":[{"binding_name":"#armor_text_pos_3","binding_name_override":"#offset"},{"binding_name":"#armor_visible_3","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"}],"controls":[{"text_shadow":{"type":"label","text":"#armor_text_3","size":[20,10],"offset":[1,1],"color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"layer":1,"bindings":[{"binding_name":"#armor_text_3"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_vec_1_1","binding_name_override":"#offset"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}},{"text":{"type":"label","text":"#armor_text_3","size":[20,10],"layer":2,"color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.55,"bindings":[{"binding_name":"#armor_text_3"},{"binding_name":"#armor_percent","binding_name_override":"#visible"},{"binding_name":"#armor_dim_20","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#armor_dim_10","binding_name_override":"#size_binding_y_absolute"},{"binding_name":"#armor_font","binding_name_override":"#font_scale_factor"}]}}]}}"##,
        )
    };
}
pub(crate) use document;

/// Layout and display preferences for the equipped armor HUD. It is on by default and always shows
/// its four slots, empty or not; `on` is the switch to hide it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Armor {
    pub pos: crate::counter::Pos,
    pub scale: f32,
    pub on: bool,
    pub horizontal: bool,
    pub percent: bool,
    pub background: bool,
}
impl Default for Armor {
    fn default() -> Self {
        Self {
            pos: DEFAULT_POS,
            scale: 1.0,
            on: false,
            horizontal: true,
            percent: false,
            background: true,
        }
    }
}

/// Centre of the default horizontal frame, measured from the left and bottom edges.
pub const DEFAULT_POS: crate::counter::Pos = crate::counter::Pos {
    anchor: [crate::counter::Anchor::Start, crate::counter::Anchor::End],
    off: [52.0, -38.0],
};

/// The frame's unscaled size in each orientation.
pub fn base(armor: Armor) -> [f32; 2] {
    if armor.horizontal {
        [84.0, 24.0]
    } else {
        [24.0, 90.0]
    }
}

/// Offset of the frame's top-left from its saved position, at scale 1.
pub fn corner_offset(armor: Armor) -> [f32; 2] {
    if armor.horizontal {
        [-42.0, -12.0]
    } else {
        [-46.0, -78.0]
    }
}

pub fn frame(armor: Armor, viewport: [f32; 2]) -> ([f32; 2], [f32; 2]) {
    let point = armor.pos.at(viewport);
    let offset = corner_offset(armor);
    (
        [
            point[0] + offset[0] * armor.scale,
            point[1] + offset[1] * armor.scale,
        ],
        base(armor).map(|n| n * armor.scale),
    )
}

/// Bind positions, sizes and the shared editor overlay for the current viewport.
pub fn layout_bindings(
    armor: Armor,
    viewport: [f32; 2],
    edit: Option<crate::counter::Edit>,
    percents: [Option<u32>; 4],
) -> String {
    use std::fmt::Write;
    let scale = armor.scale;
    let mut out = format!(
        r##""#armor_percent":{},"#armor_background":{},"#armor_join":{}"##,
        armor.percent,
        armor.background,
        armor.horizontal && armor.background
    );
    let point = armor.pos.at(viewport);
    let (corner, size) = frame(armor, viewport);
    let _ = write!(
        out,
        r##","#armor_edit_visible":{},"#armor_edit_offset":[{},{}],"#armor_edit_w":{},"#armor_edit_h":{},"#armor_font":{},"#armor_strip_pos":[{},{}],"#armor_dim_82":{},"#armor_dim_22":{}"##,
        edit.is_some(),
        corner[0],
        corner[1] - viewport[1] + size[1],
        size[0],
        size[1],
        0.55 * scale,
        point[0] - 41.0 * scale,
        point[1] - viewport[1] + 11.0 * scale,
        82.0 * scale,
        22.0 * scale
    );
    out.push(',');
    out.push_str(&crate::counter::overlay_bindings(
        "armoredit",
        size,
        scale,
        edit,
    ));
    for (i, percent) in percents.iter().enumerate() {
        let (base_x, base_y) = if armor.horizontal {
            (14.0 + i as f32 * 20.0, -30.0)
        } else {
            (8.0, -96.0 + i as f32 * 22.0)
        };
        let x = point[0] + (base_x - 52.0) * scale;
        let y = point[1] - viewport[1] + (base_y + 38.0) * scale;
        let (tx, ty) = if armor.horizontal {
            (x - 2.0 * scale, y + 10.0 * scale)
        } else {
            (x + 18.0 * scale, y + 3.0 * scale)
        };
        let bar_width = percent.unwrap_or(0).min(100) as f32 * 0.14 * scale;
        let _ = write!(
            out,
            r##","#armor_pos_{i}":[{x},{y}],"#armor_slot_pos_{i}":[{},{}],"#armor_text_pos_{i}":[{tx},{ty}],"#armor_bar_pos_{i}":[{x},{}],"#armor_bar_width_{i}":{bar_width}"##,
            x - 2.0 * scale,
            y + 2.0 * scale,
            y - 1.0 * scale
        );
    }
    let _ = write!(out, r##","#armor_dim_1":{}"##, 1f32 * scale);
    let _ = write!(out, r##","#armor_dim_3":{}"##, 3f32 * scale);
    let _ = write!(out, r##","#armor_dim_10":{}"##, 10f32 * scale);
    let _ = write!(out, r##","#armor_dim_14":{}"##, 14f32 * scale);
    let _ = write!(out, r##","#armor_dim_16":{}"##, 16f32 * scale);
    let _ = write!(out, r##","#armor_dim_20":{}"##, 20f32 * scale);
    let _ = write!(out, r##","#armor_dim_24":{}"##, 24f32 * scale);
    let _ = write!(out, r##","#armor_dim_84":{}"##, 84f32 * scale);
    let _ = write!(
        out,
        r##","#armor_vec_0_0":[{},{}]"##,
        0f32 * scale,
        0f32 * scale
    );
    let _ = write!(
        out,
        r##","#armor_vec_0_19":[{},{}]"##,
        0f32 * scale,
        19f32 * scale
    );
    let _ = write!(
        out,
        r##","#armor_vec_1_1":[{},{}]"##,
        1f32 * scale,
        1f32 * scale
    );
    let _ = write!(
        out,
        r##","#armor_vec_2_2":[{},{}]"##,
        2f32 * scale,
        2f32 * scale
    );
    let _ = write!(
        out,
        r##","#armor_dim_2":{},"#armor_vec_0_18":[{},{}],"#armor_vec_0_2":[{},{}],"#armor_vec_18_2":[{},{}]"##,
        2f32 * scale,
        0f32 * scale,
        18f32 * scale,
        0f32 * scale,
        2f32 * scale,
        18f32 * scale,
        2f32 * scale
    );
    let _ = write!(
        out,
        r##","#armor_vec_19_0":[{},{}]"##,
        19f32 * scale,
        0f32 * scale
    );
    out
}

/// A valid identifier is safe to place in a JSON string and admitted by the host.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(c, b':' | b'_' | b'-' | b'.' | b'/')
        })
}

/// Remaining durability rounded down; unknown values are left blank.
pub fn durability(damage: Option<u32>, maximum: Option<u32>) -> Option<u32> {
    let (damage, max) = (damage?, maximum?);
    (max > 0).then(|| ((u64::from(max.saturating_sub(damage)) * 100) / u64::from(max)) as u32)
}

/// A cheap summary of what the four armor cells would show: for each cell the item's identifier,
/// metadata and durability percent, or nothing. Two equal summaries mean equal bindings, so the
/// guest compares these instead of building the bindings text to compare it. Fuel-wise a few
/// hundred integer operations, against an allocation and some forty formatted fields.
#[derive(Clone, Copy)]
pub struct Print(u64);

impl Print {
    const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;

    pub const fn new() -> Self {
        Self(Self::BASIS)
    }

    fn mix(&mut self, byte: u8) {
        self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(Self::PRIME);
    }

    fn mix_u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.mix(byte);
        }
    }

    /// Takes in the next cell: `(identifier, metadata, durability percent)` of what it shows, or
    /// `None` when it shows nothing. Only a valid identifier is shown, so pass `None` for others.
    pub fn cell(&mut self, shown: Option<(&str, u32, Option<u32>)>) {
        let Some((id, metadata, percent)) = shown else {
            self.mix(0);
            return;
        };
        self.mix(1);
        for byte in id.bytes() {
            self.mix(byte);
        }
        // Not a valid identifier byte, so "ab" + "c" never equals "a" + "bc".
        self.mix(0xff);
        self.mix_u32(metadata);
        match percent {
            Some(percent) => {
                self.mix(1);
                self.mix_u32(percent);
            }
            None => self.mix(0),
        }
    }

    /// The summary, also telling a list with fewer cells from one with empty ones.
    pub fn finish(mut self, cells: usize) -> u64 {
        self.mix_u32(u32::try_from(cells).unwrap_or(u32::MAX));
        self.0
    }
}

impl Default for Print {
    fn default() -> Self {
        Self::new()
    }
}

pub fn initial_bindings() -> String {
    format!(
        "{},{}",
        layout_bindings(Armor::default(), [320.0, 180.0], None, [None; 4]),
        empty_bindings()
    )
}

pub fn empty_bindings() -> String {
    format!(r##""#armor_any":false,{}"##, (0..4).map(|i| format!(r##""#armor_visible_{i}":false,"#armor_bar_visible_{i}":false,"#armor_text_{i}":"","#armor_item_{i}":{{"item":{{"identifier":"minecraft:arrow"}}}}"##)).collect::<Vec<_>>().join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_changes_positions_and_display_bindings() {
        let vertical = layout_bindings(
            Armor {
                horizontal: false,
                percent: true,
                background: false,
                ..Armor::default()
            },
            [320.0, 180.0],
            None,
            [None; 4],
        );
        assert!(vertical.contains(r##""#armor_pos_1":[8,-74]"##));
        assert!(vertical.contains(r##""#armor_percent":true"##));
        let horizontal = layout_bindings(
            Armor {
                on: true,
                horizontal: true,
                percent: false,
                background: true,
                pos: DEFAULT_POS,
                scale: 1.0,
            },
            [320.0, 180.0],
            None,
            [None; 4],
        );
        assert!(horizontal.contains(r##""#armor_pos_1":[34,-30]"##));
        assert!(horizontal.contains(r##""#armor_slot_pos_0":[12,-28]"##));
        assert!(horizontal.contains(r##""#armor_bar_pos_0":[14,-31]"##));
        let with_percent = layout_bindings(
            Armor {
                percent: true,
                horizontal: true,
                ..Armor::default()
            },
            [320.0, 180.0],
            None,
            [None; 4],
        );
        assert!(with_percent.contains(r##""#armor_pos_1":[34,-30]"##));
        assert!(with_percent.contains(r##""#armor_text_pos_0":[12,-20]"##));
        assert!(horizontal.contains(r##""#armor_percent":false"##));
        assert!(horizontal.contains(r##""#armor_background":true"##));
    }
    #[test]
    fn moved_frame_follows_its_saved_anchor_on_resize() {
        let mut armor = Armor::default();
        let (original, _) = frame(armor, [320.0, 180.0]);
        assert_eq!(original, [10.0, 130.0]);
        armor.pos = crate::counter::Pos {
            anchor: [crate::counter::Anchor::Centre, crate::counter::Anchor::End],
            off: [0.0, -38.0],
        };
        let (at, _) = frame(armor, [400.0, 200.0]);
        assert_eq!(at, [158.0, 150.0]);
        let (resized, _) = frame(armor, [600.0, 300.0]);
        assert_eq!(resized, [258.0, 250.0]);
    }
    #[test]
    fn resizing_scales_the_frame_slots_and_durability() {
        let armor = Armor {
            scale: 2.0,
            ..Armor::default()
        };
        let (_, size) = frame(armor, [320.0, 180.0]);
        assert_eq!(size, [168.0, 48.0]);
        let bindings = layout_bindings(
            armor,
            [320.0, 180.0],
            Some(crate::counter::Edit::default()),
            [Some(50), None, None, None],
        );
        assert!(bindings.contains(r##""#armor_dim_20":40"##));
        assert!(bindings.contains(r##""#armor_bar_width_0":14"##));
        assert!(bindings.contains(r##""#armoredit_frame_visible":true"##));
    }
    fn print_of(cells: &[Option<(&str, u32, Option<u32>)>]) -> u64 {
        let mut print = Print::new();
        for cell in cells {
            print.cell(*cell);
        }
        print.finish(cells.len())
    }

    #[test]
    fn the_print_tells_every_shown_difference_and_nothing_else() {
        let helmet = Some(("minecraft:iron_helmet", 0, Some(80)));
        let base = print_of(&[helmet, None, None, None]);
        assert_eq!(base, print_of(&[helmet, None, None, None]), "same cells");
        // Durability, metadata, identifier and which cell holds it each change the print.
        let worn = Some(("minecraft:iron_helmet", 0, Some(79)));
        assert_ne!(base, print_of(&[worn, None, None, None]));
        let meta = Some(("minecraft:iron_helmet", 1, Some(80)));
        assert_ne!(base, print_of(&[meta, None, None, None]));
        let other = Some(("minecraft:gold_helmet", 0, Some(80)));
        assert_ne!(base, print_of(&[other, None, None, None]));
        assert_ne!(base, print_of(&[None, helmet, None, None]));
        // An item with no known durability differs from one at 0%, and from nothing.
        let unknown = Some(("minecraft:iron_helmet", 0, None));
        let broken = Some(("minecraft:iron_helmet", 0, Some(0)));
        assert_ne!(print_of(&[unknown]), print_of(&[broken]));
        assert_ne!(print_of(&[unknown]), print_of(&[None]));
        // Identifiers cannot run into their neighbours, and a short list is not an empty one.
        let split_a = print_of(&[Some(("ab", 0, None)), Some(("c", 0, None))]);
        let split_b = print_of(&[Some(("a", 0, None)), Some(("bc", 0, None))]);
        assert_ne!(split_a, split_b);
        assert_ne!(print_of(&[None, None]), print_of(&[None, None, None, None]));
    }

    #[test]
    fn durability_and_identifier_are_bounded() {
        assert_eq!(durability(Some(25), Some(100)), Some(75));
        assert_eq!(durability(Some(101), Some(100)), Some(0));
        assert_eq!(durability(None, Some(100)), None);
        assert_eq!(durability(Some(1), Some(0)), None);
        assert!(valid_id("minecraft:diamond_helmet"));
        assert!(!valid_id("bad\"identifier"));
    }
}
