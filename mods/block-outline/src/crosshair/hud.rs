//! The HUD documents and patches that show the crosshair and the menu's drawing grid.

use super::*;

/// What the crosshair shows: its look and the HUD's size (width, height and GUI scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub look: Crosshair,
    pub viewport: [f32; 3],
}

/// The crosshair's instance in the HUD document's root, as a string literal: its definition,
/// `xhair`, is replaced whenever its shape changes.
macro_rules! document {
    () => {
        r##"{"crosshair@boldy_hud.xhair":{}}"##
    };
}

pub(crate) use document;

/// The definitions the crosshair and the menu's drawing grid use, as string literal entries of
/// the HUD document: an empty crosshair, hidden, and its empty parts; their fill (`xf`) and
/// outline (`xe`) pieces; the grid (`drawgrid`, 224 units a side) with its cells' parts
/// (`gcells0`... of `gp` pieces), a highlight on the pixel under the pointer and its mirrored
/// pixels (`gc`), and its lines (`gridlines`, one of `lines16`, `lines32` and `lines64`: faint
/// `gl` lines, stronger `gj` lines every 4 or 8 pixels, and `gm` lines through the middle).
macro_rules! definitions {
    () => {
        concat!(
            r##""xhair":{"type":"panel","size":[1,1],"offset":[0,0],"anchor_from":"center","anchor_to":"center","layer":2,"bindings":[{"binding_name":"#crosshair_visible","binding_name_override":"#visible"}]},"##,
            r##""xpart0":{"type":"panel","size":["100%","100%"]},"xpart1":{"type":"panel","size":["100%","100%"]},"xpart2":{"type":"panel","size":["100%","100%"]},"##,
            r##""xpart3":{"type":"panel","size":["100%","100%"]},"xpart4":{"type":"panel","size":["100%","100%"]},"xpart5":{"type":"panel","size":["100%","100%"]},"##,
            r##""xpart6":{"type":"panel","size":["100%","100%"]},"xpart7":{"type":"panel","size":["100%","100%"]},"xpart8":{"type":"panel","size":["100%","100%"]},"##,
            r##""xf":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,1],"layer":2,"bindings":[{"binding_name":"#crosshair_color","binding_name_override":"#color"}]},"##,
            r##""xe":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0,0,0,1],"layer":1,"bindings":[{"binding_name":"#crosshair_edge_color","binding_name_override":"#color"}]},"##,
            r##""drawgrid":{"type":"panel","size":[224,224],"controls":["##,
            r##"{"back":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":["100%","100%"],"radius":0,"color":[0.04,0.045,0.06,1],"layer":1}},"##,
            r##"{"c0@boldy_hud.gcells0":{}},{"c1@boldy_hud.gcells1":{}},{"c2@boldy_hud.gcells2":{}},{"c3@boldy_hud.gcells3":{}},{"c4@boldy_hud.gcells4":{}},"##,
            r##"{"c5@boldy_hud.gcells5":{}},{"c6@boldy_hud.gcells6":{}},{"c7@boldy_hud.gcells7":{}},{"c8@boldy_hud.gcells8":{}},"##,
            r##"{"cursor0@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor0_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor0_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cursor1@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor1_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor1_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cursor2@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor2_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor2_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"cursor3@boldy_hud.gc":{"bindings":[{"binding_name":"#drawgrid_cursor3_visible","binding_name_override":"#visible"},{"binding_name":"#drawgrid_cursor3_offset","binding_name_override":"#offset"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_x_absolute"},{"binding_name":"#drawgrid_cursor_size","binding_name_override":"#size_binding_y_absolute"}]}},"##,
            r##"{"lines@boldy_hud.gridlines":{}}]},"##,
            r##""gcells0":{"type":"panel","size":["100%","100%"],"layer":3},"gcells1":{"type":"panel","size":["100%","100%"],"layer":3},"gcells2":{"type":"panel","size":["100%","100%"],"layer":3},"##,
            r##""gcells3":{"type":"panel","size":["100%","100%"],"layer":3},"gcells4":{"type":"panel","size":["100%","100%"],"layer":3},"gcells5":{"type":"panel","size":["100%","100%"],"layer":3},"##,
            r##""gcells6":{"type":"panel","size":["100%","100%"],"layer":3},"gcells7":{"type":"panel","size":["100%","100%"],"layer":3},"gcells8":{"type":"panel","size":["100%","100%"],"layer":3},"##,
            r##""gp":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,1],"layer":0,"bindings":[{"binding_name":"#drawgrid_color","binding_name_override":"#color"}]},"##,
            r##""gl":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,0.09],"layer":0},"##,
            r##""gj":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,0.26],"layer":0},"##,
            r##""gc":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[14,14],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[1,1,1,0.32],"layer":4},"##,
            r##""gm":{"type":"custom","renderer":"cinnabar_rounded_rectangle","size":[1,1],"offset":[0,0],"anchor_from":"top_left","anchor_to":"top_left","radius":0,"color":[0.66,0.5,0.97,0.55],"layer":1},"##,
            r##""gridlines":{"type":"panel","size":["100%","100%"],"layer":5,"controls":[{"l@boldy_hud.lines16":{}}]},"##,
            r##""lines16":{"type":"panel","size":["100%","100%"],"controls":[{"v0@boldy_hud.gj":{"offset":[-0.25,0],"size":[0.5,224]}},{"h0@boldy_hud.gj":{"offset":[0,-0.25],"size":[224,0.5]}},{"v1@boldy_hud.gl":{"offset":[13.75,0],"size":[0.5,224]}},{"h1@boldy_hud.gl":{"offset":[0,13.75],"size":[224,0.5]}},{"v2@boldy_hud.gl":{"offset":[27.75,0],"size":[0.5,224]}},{"h2@boldy_hud.gl":{"offset":[0,27.75],"size":[224,0.5]}},{"v3@boldy_hud.gl":{"offset":[41.75,0],"size":[0.5,224]}},{"h3@boldy_hud.gl":{"offset":[0,41.75],"size":[224,0.5]}},{"v4@boldy_hud.gj":{"offset":[55.75,0],"size":[0.5,224]}},{"h4@boldy_hud.gj":{"offset":[0,55.75],"size":[224,0.5]}},{"v5@boldy_hud.gl":{"offset":[69.75,0],"size":[0.5,224]}},{"h5@boldy_hud.gl":{"offset":[0,69.75],"size":[224,0.5]}},{"v6@boldy_hud.gl":{"offset":[83.75,0],"size":[0.5,224]}},{"h6@boldy_hud.gl":{"offset":[0,83.75],"size":[224,0.5]}},{"v7@boldy_hud.gl":{"offset":[97.75,0],"size":[0.5,224]}},{"h7@boldy_hud.gl":{"offset":[0,97.75],"size":[224,0.5]}},{"v9@boldy_hud.gl":{"offset":[125.75,0],"size":[0.5,224]}},{"h9@boldy_hud.gl":{"offset":[0,125.75],"size":[224,0.5]}},{"v10@boldy_hud.gl":{"offset":[139.75,0],"size":[0.5,224]}},{"h10@boldy_hud.gl":{"offset":[0,139.75],"size":[224,0.5]}},{"v11@boldy_hud.gl":{"offset":[153.75,0],"size":[0.5,224]}},{"h11@boldy_hud.gl":{"offset":[0,153.75],"size":[224,0.5]}},{"v12@boldy_hud.gj":{"offset":[167.75,0],"size":[0.5,224]}},{"h12@boldy_hud.gj":{"offset":[0,167.75],"size":[224,0.5]}},{"v13@boldy_hud.gl":{"offset":[181.75,0],"size":[0.5,224]}},{"h13@boldy_hud.gl":{"offset":[0,181.75],"size":[224,0.5]}},{"v14@boldy_hud.gl":{"offset":[195.75,0],"size":[0.5,224]}},{"h14@boldy_hud.gl":{"offset":[0,195.75],"size":[224,0.5]}},{"v15@boldy_hud.gl":{"offset":[209.75,0],"size":[0.5,224]}},{"h15@boldy_hud.gl":{"offset":[0,209.75],"size":[224,0.5]}},{"v16@boldy_hud.gj":{"offset":[223.75,0],"size":[0.5,224]}},{"h16@boldy_hud.gj":{"offset":[0,223.75],"size":[224,0.5]}},{"mv@boldy_hud.gm":{"offset":[111.5,0],"size":[1,224]}},{"mh@boldy_hud.gm":{"offset":[0,111.5],"size":[224,1]}}]},"##,
            r##""lines32":{"type":"panel","size":["100%","100%"],"controls":[{"v0@boldy_hud.gj":{"offset":[-0.25,0],"size":[0.5,224]}},{"h0@boldy_hud.gj":{"offset":[0,-0.25],"size":[224,0.5]}},{"v1@boldy_hud.gl":{"offset":[6.75,0],"size":[0.5,224]}},{"h1@boldy_hud.gl":{"offset":[0,6.75],"size":[224,0.5]}},{"v2@boldy_hud.gl":{"offset":[13.75,0],"size":[0.5,224]}},{"h2@boldy_hud.gl":{"offset":[0,13.75],"size":[224,0.5]}},{"v3@boldy_hud.gl":{"offset":[20.75,0],"size":[0.5,224]}},{"h3@boldy_hud.gl":{"offset":[0,20.75],"size":[224,0.5]}},{"v4@boldy_hud.gj":{"offset":[27.75,0],"size":[0.5,224]}},{"h4@boldy_hud.gj":{"offset":[0,27.75],"size":[224,0.5]}},{"v5@boldy_hud.gl":{"offset":[34.75,0],"size":[0.5,224]}},{"h5@boldy_hud.gl":{"offset":[0,34.75],"size":[224,0.5]}},{"v6@boldy_hud.gl":{"offset":[41.75,0],"size":[0.5,224]}},{"h6@boldy_hud.gl":{"offset":[0,41.75],"size":[224,0.5]}},{"v7@boldy_hud.gl":{"offset":[48.75,0],"size":[0.5,224]}},{"h7@boldy_hud.gl":{"offset":[0,48.75],"size":[224,0.5]}},{"v8@boldy_hud.gj":{"offset":[55.75,0],"size":[0.5,224]}},{"h8@boldy_hud.gj":{"offset":[0,55.75],"size":[224,0.5]}},{"v9@boldy_hud.gl":{"offset":[62.75,0],"size":[0.5,224]}},{"h9@boldy_hud.gl":{"offset":[0,62.75],"size":[224,0.5]}},{"v10@boldy_hud.gl":{"offset":[69.75,0],"size":[0.5,224]}},{"h10@boldy_hud.gl":{"offset":[0,69.75],"size":[224,0.5]}},{"v11@boldy_hud.gl":{"offset":[76.75,0],"size":[0.5,224]}},{"h11@boldy_hud.gl":{"offset":[0,76.75],"size":[224,0.5]}},{"v12@boldy_hud.gj":{"offset":[83.75,0],"size":[0.5,224]}},{"h12@boldy_hud.gj":{"offset":[0,83.75],"size":[224,0.5]}},{"v13@boldy_hud.gl":{"offset":[90.75,0],"size":[0.5,224]}},{"h13@boldy_hud.gl":{"offset":[0,90.75],"size":[224,0.5]}},{"v14@boldy_hud.gl":{"offset":[97.75,0],"size":[0.5,224]}},{"h14@boldy_hud.gl":{"offset":[0,97.75],"size":[224,0.5]}},{"v15@boldy_hud.gl":{"offset":[104.75,0],"size":[0.5,224]}},{"h15@boldy_hud.gl":{"offset":[0,104.75],"size":[224,0.5]}},{"v17@boldy_hud.gl":{"offset":[118.75,0],"size":[0.5,224]}},{"h17@boldy_hud.gl":{"offset":[0,118.75],"size":[224,0.5]}},{"v18@boldy_hud.gl":{"offset":[125.75,0],"size":[0.5,224]}},{"h18@boldy_hud.gl":{"offset":[0,125.75],"size":[224,0.5]}},{"v19@boldy_hud.gl":{"offset":[132.75,0],"size":[0.5,224]}},{"h19@boldy_hud.gl":{"offset":[0,132.75],"size":[224,0.5]}},{"v20@boldy_hud.gj":{"offset":[139.75,0],"size":[0.5,224]}},{"h20@boldy_hud.gj":{"offset":[0,139.75],"size":[224,0.5]}},{"v21@boldy_hud.gl":{"offset":[146.75,0],"size":[0.5,224]}},{"h21@boldy_hud.gl":{"offset":[0,146.75],"size":[224,0.5]}},{"v22@boldy_hud.gl":{"offset":[153.75,0],"size":[0.5,224]}},{"h22@boldy_hud.gl":{"offset":[0,153.75],"size":[224,0.5]}},{"v23@boldy_hud.gl":{"offset":[160.75,0],"size":[0.5,224]}},{"h23@boldy_hud.gl":{"offset":[0,160.75],"size":[224,0.5]}},{"v24@boldy_hud.gj":{"offset":[167.75,0],"size":[0.5,224]}},{"h24@boldy_hud.gj":{"offset":[0,167.75],"size":[224,0.5]}},{"v25@boldy_hud.gl":{"offset":[174.75,0],"size":[0.5,224]}},{"h25@boldy_hud.gl":{"offset":[0,174.75],"size":[224,0.5]}},{"v26@boldy_hud.gl":{"offset":[181.75,0],"size":[0.5,224]}},{"h26@boldy_hud.gl":{"offset":[0,181.75],"size":[224,0.5]}},{"v27@boldy_hud.gl":{"offset":[188.75,0],"size":[0.5,224]}},{"h27@boldy_hud.gl":{"offset":[0,188.75],"size":[224,0.5]}},{"v28@boldy_hud.gj":{"offset":[195.75,0],"size":[0.5,224]}},{"h28@boldy_hud.gj":{"offset":[0,195.75],"size":[224,0.5]}},{"v29@boldy_hud.gl":{"offset":[202.75,0],"size":[0.5,224]}},{"h29@boldy_hud.gl":{"offset":[0,202.75],"size":[224,0.5]}},{"v30@boldy_hud.gl":{"offset":[209.75,0],"size":[0.5,224]}},{"h30@boldy_hud.gl":{"offset":[0,209.75],"size":[224,0.5]}},{"v31@boldy_hud.gl":{"offset":[216.75,0],"size":[0.5,224]}},{"h31@boldy_hud.gl":{"offset":[0,216.75],"size":[224,0.5]}},{"v32@boldy_hud.gj":{"offset":[223.75,0],"size":[0.5,224]}},{"h32@boldy_hud.gj":{"offset":[0,223.75],"size":[224,0.5]}},{"mv@boldy_hud.gm":{"offset":[111.5,0],"size":[1,224]}},{"mh@boldy_hud.gm":{"offset":[0,111.5],"size":[224,1]}}]},"##,
            r##""lines64":{"type":"panel","size":["100%","100%"],"controls":[{"v0@boldy_hud.gj":{"offset":[-0.25,0],"size":[0.5,224]}},{"h0@boldy_hud.gj":{"offset":[0,-0.25],"size":[224,0.5]}},{"v2@boldy_hud.gl":{"offset":[6.75,0],"size":[0.5,224]}},{"h2@boldy_hud.gl":{"offset":[0,6.75],"size":[224,0.5]}},{"v4@boldy_hud.gl":{"offset":[13.75,0],"size":[0.5,224]}},{"h4@boldy_hud.gl":{"offset":[0,13.75],"size":[224,0.5]}},{"v6@boldy_hud.gl":{"offset":[20.75,0],"size":[0.5,224]}},{"h6@boldy_hud.gl":{"offset":[0,20.75],"size":[224,0.5]}},{"v8@boldy_hud.gj":{"offset":[27.75,0],"size":[0.5,224]}},{"h8@boldy_hud.gj":{"offset":[0,27.75],"size":[224,0.5]}},{"v10@boldy_hud.gl":{"offset":[34.75,0],"size":[0.5,224]}},{"h10@boldy_hud.gl":{"offset":[0,34.75],"size":[224,0.5]}},{"v12@boldy_hud.gl":{"offset":[41.75,0],"size":[0.5,224]}},{"h12@boldy_hud.gl":{"offset":[0,41.75],"size":[224,0.5]}},{"v14@boldy_hud.gl":{"offset":[48.75,0],"size":[0.5,224]}},{"h14@boldy_hud.gl":{"offset":[0,48.75],"size":[224,0.5]}},{"v16@boldy_hud.gj":{"offset":[55.75,0],"size":[0.5,224]}},{"h16@boldy_hud.gj":{"offset":[0,55.75],"size":[224,0.5]}},{"v18@boldy_hud.gl":{"offset":[62.75,0],"size":[0.5,224]}},{"h18@boldy_hud.gl":{"offset":[0,62.75],"size":[224,0.5]}},{"v20@boldy_hud.gl":{"offset":[69.75,0],"size":[0.5,224]}},{"h20@boldy_hud.gl":{"offset":[0,69.75],"size":[224,0.5]}},{"v22@boldy_hud.gl":{"offset":[76.75,0],"size":[0.5,224]}},{"h22@boldy_hud.gl":{"offset":[0,76.75],"size":[224,0.5]}},{"v24@boldy_hud.gj":{"offset":[83.75,0],"size":[0.5,224]}},{"h24@boldy_hud.gj":{"offset":[0,83.75],"size":[224,0.5]}},{"v26@boldy_hud.gl":{"offset":[90.75,0],"size":[0.5,224]}},{"h26@boldy_hud.gl":{"offset":[0,90.75],"size":[224,0.5]}},{"v28@boldy_hud.gl":{"offset":[97.75,0],"size":[0.5,224]}},{"h28@boldy_hud.gl":{"offset":[0,97.75],"size":[224,0.5]}},{"v30@boldy_hud.gl":{"offset":[104.75,0],"size":[0.5,224]}},{"h30@boldy_hud.gl":{"offset":[0,104.75],"size":[224,0.5]}},{"v34@boldy_hud.gl":{"offset":[118.75,0],"size":[0.5,224]}},{"h34@boldy_hud.gl":{"offset":[0,118.75],"size":[224,0.5]}},{"v36@boldy_hud.gl":{"offset":[125.75,0],"size":[0.5,224]}},{"h36@boldy_hud.gl":{"offset":[0,125.75],"size":[224,0.5]}},{"v38@boldy_hud.gl":{"offset":[132.75,0],"size":[0.5,224]}},{"h38@boldy_hud.gl":{"offset":[0,132.75],"size":[224,0.5]}},{"v40@boldy_hud.gj":{"offset":[139.75,0],"size":[0.5,224]}},{"h40@boldy_hud.gj":{"offset":[0,139.75],"size":[224,0.5]}},{"v42@boldy_hud.gl":{"offset":[146.75,0],"size":[0.5,224]}},{"h42@boldy_hud.gl":{"offset":[0,146.75],"size":[224,0.5]}},{"v44@boldy_hud.gl":{"offset":[153.75,0],"size":[0.5,224]}},{"h44@boldy_hud.gl":{"offset":[0,153.75],"size":[224,0.5]}},{"v46@boldy_hud.gl":{"offset":[160.75,0],"size":[0.5,224]}},{"h46@boldy_hud.gl":{"offset":[0,160.75],"size":[224,0.5]}},{"v48@boldy_hud.gj":{"offset":[167.75,0],"size":[0.5,224]}},{"h48@boldy_hud.gj":{"offset":[0,167.75],"size":[224,0.5]}},{"v50@boldy_hud.gl":{"offset":[174.75,0],"size":[0.5,224]}},{"h50@boldy_hud.gl":{"offset":[0,174.75],"size":[224,0.5]}},{"v52@boldy_hud.gl":{"offset":[181.75,0],"size":[0.5,224]}},{"h52@boldy_hud.gl":{"offset":[0,181.75],"size":[224,0.5]}},{"v54@boldy_hud.gl":{"offset":[188.75,0],"size":[0.5,224]}},{"h54@boldy_hud.gl":{"offset":[0,188.75],"size":[224,0.5]}},{"v56@boldy_hud.gj":{"offset":[195.75,0],"size":[0.5,224]}},{"h56@boldy_hud.gj":{"offset":[0,195.75],"size":[224,0.5]}},{"v58@boldy_hud.gl":{"offset":[202.75,0],"size":[0.5,224]}},{"h58@boldy_hud.gl":{"offset":[0,202.75],"size":[224,0.5]}},{"v60@boldy_hud.gl":{"offset":[209.75,0],"size":[0.5,224]}},{"h60@boldy_hud.gl":{"offset":[0,209.75],"size":[224,0.5]}},{"v62@boldy_hud.gl":{"offset":[216.75,0],"size":[0.5,224]}},{"h62@boldy_hud.gl":{"offset":[0,216.75],"size":[224,0.5]}},{"v64@boldy_hud.gj":{"offset":[223.75,0],"size":[0.5,224]}},{"h64@boldy_hud.gj":{"offset":[0,223.75],"size":[224,0.5]}},{"mv@boldy_hud.gm":{"offset":[111.5,0],"size":[1,224]}},{"mh@boldy_hud.gm":{"offset":[0,111.5],"size":[224,1]}}]}"##
        )
    };
}

pub(crate) use definitions;

/// The bindings of a hidden crosshair and an empty grid, to publish with the rest of the layer.
pub fn initial_bindings() -> String {
    String::from(
        r##""#crosshair_visible":false,"#crosshair_color":[1,1,1,1],"#crosshair_edge_color":[0,0,0,1],"#drawgrid_color":[1,1,1,1],"#drawgrid_cursor0_visible":false,"#drawgrid_cursor1_visible":false,"#drawgrid_cursor2_visible":false,"#drawgrid_cursor3_visible":false"##,
    )
}

/// The GUI scale `view` is drawn at; an unusable one is 1.
pub(super) fn view_scale(view: &View) -> f32 {
    let scale = view.viewport[2];
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

/// The shape `view` draws.
pub fn view_shape(view: &View) -> Shape {
    shape(&view.look, view_scale(view))
}

/// FNV-1a over 32-bit words: cheap, and enough to tell two versions of a part apart.
pub(super) fn hash(words: impl Iterator<Item = u32>) -> u64 {
    words.fold(0xcbf2_9ce4_8422_2325, |h, word| {
        (h ^ u64::from(word)).wrapping_mul(0x100_0000_01b3)
    })
}

/// What sending a crosshair (or the grid) takes: a header, then each part, each told apart by
/// a hash so only what changed is sent again.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub header: u64,
    pub parts: [u64; PARTS],
}

/// The hash of an empty part, as the HUD document starts with.
pub fn empty_part() -> u64 {
    hash(std::iter::empty())
}

/// The rectangles of part `index`.
pub(super) fn part<T>(rects: &[T], index: usize) -> &[T] {
    let start = (index * PART_RECTS).min(rects.len());
    &rects[start..(start + PART_RECTS).min(rects.len())]
}

/// The plan for `shape`, which is [`view_shape`] of `view`.
pub fn plan(view: &View, shape: &Shape) -> Plan {
    let look = &view.look;
    let header = hash(
        [
            shape.side.to_bits(),
            shape.stroke.to_bits(),
            u32::from(shape.turned),
            u32::from(look.outline),
            u32::from(look.opacity),
            u32::from(look.rgb[0]) << 16 | u32::from(look.rgb[1]) << 8 | u32::from(look.rgb[2]),
        ]
        .into_iter()
        .chain(view.viewport.map(f32::to_bits))
        .chain(
            shape
                .circles
                .iter()
                .flatten()
                .flatten()
                .map(|v| v.to_bits()),
        ),
    );
    let parts = std::array::from_fn(|index| {
        let rects = part(&shape.rects, index);
        if rects.is_empty() {
            return empty_part();
        }
        hash(
            [
                view_scale(view).to_bits(),
                u32::from(look.outline),
                shape.stroke.to_bits(),
            ]
            .into_iter()
            .chain(rects.iter().flatten().map(|v| v.to_bits())),
        )
    });
    Plan { header, parts }
}

/// About how much fuel a header takes to write: some, and more for each circle point.
pub fn header_cost(shape: &Shape) -> u64 {
    let points: usize = shape.circles.iter().map(Vec::len).sum();
    20_000 + 550 * points as u64
}

/// About how much fuel part `index` of `shape` takes to write, measured under wasmtime's fuel
/// count with some to spare.
pub fn part_cost(shape: &Shape, index: usize) -> u64 {
    26_000 + 3_500 * part(&shape.rects, index).len() as u64
}

/// About how much fuel working out a shape and its plan takes, before it is known: finding the
/// rectangles of a large drawing is most of it.
pub fn plan_cost(look: &Crosshair) -> u64 {
    if look.style == CUSTOM {
        20_000 + rects_cost(&look.pixels)
    } else {
        20_000
    }
}

/// A patch showing `view`, or hiding the crosshair for `None`, whole: every part, then the
/// header. For checks and tests; the HUD is sent the steps one a frame, in this order.
pub fn patch(view: Option<&View>) -> String {
    let Some(view) = view else {
        return visibility(false);
    };
    // In the order the HUD is sent them: the parts, then the header that places them.
    let shape = view_shape(view);
    let mut patches: Vec<String> = (0..PARTS)
        .map(|index| part_patch(view, &shape, index))
        .collect();
    patches.push(header_patch(view, &shape));
    patches.join("\n")
}

/// A patch only showing or hiding the crosshair already defined.
pub fn visibility(visible: bool) -> String {
    format!(r##"{{"binding_changes":{{"#crosshair_visible":{visible}}}}}"##)
}

/// Writes `{"name@boldy_hud.template":{"offset":[x,y],"size":[w,h]}},` for `rect`, in physical
/// pixels: the pieces' panel is scaled down by the GUI scale (see [`header_patch`]).
pub(super) fn piece(out: &mut String, name: char, index: usize, template: &str, rect: [f32; 4]) {
    let [x0, y0, x1, y1] = rect;
    out.push_str("{\"");
    out.push(name);
    digits(out, index as u64);
    out.push_str("@boldy_hud.");
    out.push_str(template);
    out.push_str(r#"":{"offset":["#);
    fixed(out, x0);
    out.push(',');
    fixed(out, y0);
    out.push_str(r#"],"size":["#);
    fixed(out, x1 - x0);
    out.push(',');
    fixed(out, y1 - y0);
    out.push_str("]}},");
}

/// The patch defining the crosshair's box (size, place, turn), its rings and colours, and
/// showing it. Its rectangles are in its parts ([`part_patch`]).
pub fn header_patch(view: &View, shape: &Shape) -> String {
    let [width, height, _] = view.viewport;
    let scale = view_scale(view);
    let mut out = String::with_capacity(1500);
    out.push_str(r##"{"controls":{"xhair":{"type":"panel","size":["##);
    let size = shape.side / scale;
    fixed(&mut out, size);
    out.push(',');
    fixed(&mut out, size);
    // Moves the box's corner onto a whole pixel, wherever the screen's middle falls.
    out.push_str(r##"],"offset":["##);
    for (index, length) in [width, height].into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let middle = length * scale / 2.0;
        fixed(&mut out, (middle.round() - middle) / scale);
    }
    out.push_str(r##"],"anchor_from":"center","anchor_to":"center","layer":2,"bindings":[{"binding_name":"#crosshair_visible","binding_name_override":"#visible"}],"controls":["##);
    let outline = view.look.outline;
    if !shape.circles.is_empty() {
        let mut paths = String::new();
        contours(&mut paths, &shape.circles);
        let mut view_box = String::from(r##","view_box":[0,0,"##);
        digits(&mut view_box, shape.side as u64);
        view_box.push(',');
        digits(&mut view_box, shape.side as u64);
        view_box.push(']');
        if outline {
            // Centred on the fill's edge, half of the stroke is hidden under it.
            out.push_str(r##"{"ring_edge":{"type":"custom","renderer":"cinnabar_vector_path","size":["100%","100%"],"layer":1,"closed":true,"stroke_width":"##);
            digits(&mut out, 2 * shape.stroke as u64);
            out.push_str(&view_box);
            out.push_str(r##","paths":"##);
            out.push_str(&paths);
            out.push_str(r##","bindings":[{"binding_name":"#crosshair_edge_color","binding_name_override":"#color"}]}},"##);
        }
        out.push_str(r##"{"ring_fill":{"type":"custom","renderer":"cinnabar_vector_path","size":["100%","100%"],"layer":2"##);
        out.push_str(&view_box);
        out.push_str(r##","paths":"##);
        out.push_str(&paths);
        out.push_str(r##","bindings":[{"binding_name":"#crosshair_color","binding_name_override":"#color"}]}},"##);
    }
    // The rectangles are laid out in physical pixels, as whole numbers, on a panel as many GUI
    // units wide as the box is pixels, drawn scaled down by the GUI scale about its middle (the
    // box's middle). Laid out in GUI units instead, each piece's fractional edges were rounded
    // to the screen's pixels on their own, and pieces gained or lost a pixel. The X is turned
    // here too. (The rings stay outside: a path is drawn at its laid-out size before the scale,
    // and a larger one would not be drawn at all.)
    out.push_str(r##"{"pixels":{"type":"panel","size":["##);
    digits(&mut out, shape.side as u64);
    out.push(',');
    digits(&mut out, shape.side as u64);
    out.push_str(
        r##"],"anchor_from":"center","anchor_to":"center","cinnabar_transform":{"scale":"##,
    );
    out.push_str(&(1.0 / f64::from(scale).clamp(0.25, 64.0)).to_string());
    if shape.turned {
        out.push_str(r##","rotation":45"##);
    }
    out.push_str(r##","origin":[0.5,0.5]},"controls":["##);
    for index in 0..PARTS {
        out.push_str("{\"p");
        digits(&mut out, index as u64);
        out.push_str("@boldy_hud.xpart");
        digits(&mut out, index as u64);
        out.push_str("\":{}},");
    }
    out.pop();
    out.push_str("]}}");
    let alpha = f32::from(view.look.opacity.clamp(MIN_OPACITY, 100)) / 100.0;
    let [r, g, b] = view.look.rgb.map(|c| f32::from(c) / 255.0);
    out.push_str(r##"]}},"binding_changes":{"#crosshair_visible":true,"#crosshair_color":"##);
    colour(&mut out, [r, g, b, alpha]);
    out.push_str(r##","#crosshair_edge_color":"##);
    colour(&mut out, [0.0, 0.0, 0.0, alpha]);
    out.push_str("}}");
    out
}

/// The patch defining part `index` of the crosshair's rectangles, each over its outline.
pub fn part_patch(view: &View, shape: &Shape, index: usize) -> String {
    let rects = part(&shape.rects, index);
    let mut out = String::with_capacity(100 + 220 * rects.len());
    out.push_str(r##"{"controls":{"xpart"##);
    digits(&mut out, index as u64);
    out.push_str(r##"":{"type":"panel","size":["100%","100%"],"controls":["##);
    let o = shape.stroke;
    for (at, &[x0, y0, x1, y1]) in rects.iter().enumerate() {
        if view.look.outline {
            piece(&mut out, 'e', at, "xe", [x0 - o, y0 - o, x1 + o, y1 + o]);
        }
        piece(&mut out, 'f', at, "xf", [x0, y0, x1, y1]);
    }
    if out.ends_with(',') {
        out.pop();
    }
    out.push_str("]}}}");
    out
}

/// The drawing grid's side in GUI units, as the menu embeds it.
pub const GRID_VIEW: f32 = 224.0;

/// The plan for showing `drawing` in `rgb` on the menu's drawing grid: a header (its lines and
/// colour) and its rectangles in parts.
pub fn grid_plan(drawing: &Drawing, rgb: [u8; 3]) -> Plan {
    let rects = rects(drawing);
    let size = drawing.size() as u32;
    let header = hash(
        [
            size,
            u32::from(rgb[0]) << 16 | u32::from(rgb[1]) << 8 | u32::from(rgb[2]),
        ]
        .into_iter(),
    );
    let parts = std::array::from_fn(|index| {
        let cells = part(&rects[..rects.len().min(MAX_RECTS)], index);
        if cells.is_empty() {
            return empty_part();
        }
        hash(std::iter::once(size).chain(cells.iter().map(|r| u32::from_le_bytes(*r))))
    });
    Plan { header, parts }
}

/// The patch showing the grid's lines for `drawing`'s resolution, and its colour.
pub fn grid_header_patch(drawing: &Drawing, rgb: [u8; 3]) -> String {
    let mut out = String::with_capacity(220);
    out.push_str(r##"{"controls":{"gridlines":{"type":"panel","size":["100%","100%"],"layer":5,"controls":[{"l@boldy_hud.lines"##);
    digits(&mut out, drawing.size() as u64);
    out.push_str(r##"":{}}]}},"binding_changes":{"#drawgrid_color":"##);
    let [r, g, b] = rgb.map(|c| f32::from(c) / 255.0);
    colour(&mut out, [r, g, b, 1.0]);
    out.push_str("}}");
    out
}

/// About how much fuel the grid's header takes to write: its lines are fixed, so little.
pub fn grid_header_cost() -> u64 {
    20_000
}

/// About how much fuel working out the grid's plan for `drawing` takes, before it is known.
pub fn grid_plan_cost(drawing: &Drawing) -> u64 {
    20_000 + rects_cost(drawing)
}

/// About how much fuel a part of the grid takes to write.
pub fn grid_part_cost(drawing: &Drawing, index: usize) -> u64 {
    let rects = rects(drawing);
    26_000 + 2_500 * part(&rects[..rects.len().min(MAX_RECTS)], index).len() as u64
}

/// The patch showing part `index` of `drawing`'s rectangles on the grid, in GUI units.
pub fn grid_part_patch(drawing: &Drawing, index: usize) -> String {
    let rects = rects(drawing);
    let cells = part(&rects[..rects.len().min(MAX_RECTS)], index);
    // 14, 7 and 3.5 units a pixel at 16, 32 and 64 pixels.
    let cell = GRID_VIEW / drawing.size() as f32;
    let mut out = String::with_capacity(100 + 70 * cells.len());
    out.push_str(r##"{"controls":{"gcells"##);
    digits(&mut out, index as u64);
    out.push_str(r##"":{"type":"panel","size":["100%","100%"],"layer":3,"controls":["##);
    for (at, &[c0, r0, c1, r1]) in cells.iter().enumerate() {
        if at > 0 {
            out.push(',');
        }
        let units = |out: &mut String, n: u8| fixed(out, f32::from(n) * cell);
        out.push_str("{\"p");
        digits(&mut out, at as u64);
        out.push_str(r#"@boldy_hud.gp":{"offset":["#);
        units(&mut out, c0);
        out.push(',');
        units(&mut out, r0);
        out.push_str(r#"],"size":["#);
        units(&mut out, c1 - c0);
        out.push(',');
        units(&mut out, r1 - r0);
        out.push_str("]}}");
    }
    out.push_str("]}}}");
    out
}

/// A patch moving the highlight to pixel `cell` (row, column) of a grid `size` pixels a side,
/// and with `mirror` to its reflections too; `None` hides it.
pub fn grid_cursor_patch(size: usize, cell: Option<(usize, usize)>, mirror: bool) -> String {
    let side = GRID_VIEW / size.max(1) as f32;
    let mut out = String::with_capacity(400);
    out.push_str(r##"{"binding_changes":{"#drawgrid_cursor_size":"##);
    fixed(&mut out, side);
    let last = size.saturating_sub(1);
    let cells = cell.map(|(row, col)| {
        [
            (row, col),
            (row, last - col.min(last)),
            (last - row.min(last), col),
            (last - row.min(last), last - col.min(last)),
        ]
    });
    for index in 0..4 {
        let at = cells
            .filter(|_| index == 0 || mirror)
            .map(|cells| cells[index]);
        out.push_str(r##","#drawgrid_cursor"##);
        digits(&mut out, index as u64);
        out.push_str(r##"_visible":"##);
        out.push_str(if at.is_some() { "true" } else { "false" });
        if let Some((row, col)) = at {
            out.push_str(r##","#drawgrid_cursor"##);
            digits(&mut out, index as u64);
            out.push_str(r##"_offset":["##);
            fixed(&mut out, col as f32 * side);
            out.push(',');
            fixed(&mut out, row as f32 * side);
            out.push(']');
        }
    }
    out.push_str("}}");
    out
}

/// The grid whole, header and every part: for checks and tests.
pub fn grid_patch(drawing: &Drawing, rgb: [u8; 3]) -> String {
    let mut patches = vec![grid_header_patch(drawing, rgb)];
    patches.extend((0..PARTS).map(|index| grid_part_patch(drawing, index)));
    patches.join("\n")
}

pub(super) fn colour(out: &mut String, values: [f32; 4]) {
    out.push('[');
    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        fixed(out, value);
    }
    out.push(']');
}

/// The longest a point can be written: `[9999.9,9999.9],`.
pub(super) const POINT_BYTES: usize = 16;

/// Writes `paths` as JSON, each coordinate to a tenth of a pixel. A frame has little fuel and the
/// circles have many points, so everything is written by index into one buffer sized up front,
/// with integer arithmetic only.
pub(super) fn contours(out: &mut String, paths: &[Vec<[f32; 2]>]) {
    let points: usize = paths.iter().map(Vec::len).sum();
    let mut bytes = vec![0u8; points * POINT_BYTES + paths.len() * 3 + 2];
    let mut at = 0;
    bytes[at] = b'[';
    at += 1;
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            bytes[at] = b',';
            at += 1;
        }
        bytes[at] = b'[';
        at += 1;
        for (point, &[x, y]) in path.iter().enumerate() {
            if point > 0 {
                bytes[at] = b',';
                at += 1;
            }
            bytes[at] = b'[';
            at += 1;
            put_tenths(&mut bytes, &mut at, x);
            bytes[at] = b',';
            at += 1;
            put_tenths(&mut bytes, &mut at, y);
            bytes[at] = b']';
            at += 1;
        }
        bytes[at] = b']';
        at += 1;
    }
    bytes[at] = b']';
    bytes.truncate(at + 1);
    out.push_str(std::str::from_utf8(&bytes).unwrap_or("[]"));
}

/// Writes `value` to a tenth at `at`. Points are measured from the box's corner, so never below
/// 0 and well under 10,000; anything else (NaN included) is written as the nearest of those.
pub(super) fn put_tenths(bytes: &mut [u8], at: &mut usize, value: f32) {
    // A saturating conversion, rounding half up: no call to a rounding function.
    let n = ((value * 10.0 + 0.5) as u32).min(99_999);
    let (whole, tenth) = (n / 10, n % 10);
    for (place, shown) in [
        (1000, whole >= 1000),
        (100, whole >= 100),
        (10, whole >= 10),
    ] {
        if shown {
            bytes[*at] = b'0' + (whole / place % 10) as u8;
            *at += 1;
        }
    }
    bytes[*at] = b'0' + (whole % 10) as u8;
    *at += 1;
    if tenth != 0 {
        bytes[*at] = b'.';
        bytes[*at + 1] = b'0' + tenth as u8;
        *at += 2;
    }
}
