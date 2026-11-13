//! Timers over the Java-style status-effect icons.
pub const SLOTS: usize = 32;

macro_rules! document {
    () => {
        concat!(
            r##"{"effect_shadow_extra_0":{"type":"label","text":"#effect_timer_0","size":[24,10],"offset":[-2,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_0"}]}},"##,
            r##"{"effect_shadow_0":{"type":"label","text":"#effect_timer_0","size":[24,10],"offset":[0,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_0"}]}},"##,
            r##"{"effect_timer_0":{"type":"label","text":"#effect_timer_0","size":[24,10],"offset":[-1,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_0"}]}},"##,
            r##"{"effect_shadow_extra_1":{"type":"label","text":"#effect_timer_1","size":[24,10],"offset":[-27,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_1"}]}},"##,
            r##"{"effect_shadow_1":{"type":"label","text":"#effect_timer_1","size":[24,10],"offset":[-25,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_1"}]}},"##,
            r##"{"effect_timer_1":{"type":"label","text":"#effect_timer_1","size":[24,10],"offset":[-26,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_1"}]}},"##,
            r##"{"effect_shadow_extra_2":{"type":"label","text":"#effect_timer_2","size":[24,10],"offset":[-52,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_2"}]}},"##,
            r##"{"effect_shadow_2":{"type":"label","text":"#effect_timer_2","size":[24,10],"offset":[-50,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_2"}]}},"##,
            r##"{"effect_timer_2":{"type":"label","text":"#effect_timer_2","size":[24,10],"offset":[-51,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_2"}]}},"##,
            r##"{"effect_shadow_extra_3":{"type":"label","text":"#effect_timer_3","size":[24,10],"offset":[-77,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_3"}]}},"##,
            r##"{"effect_shadow_3":{"type":"label","text":"#effect_timer_3","size":[24,10],"offset":[-75,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_3"}]}},"##,
            r##"{"effect_timer_3":{"type":"label","text":"#effect_timer_3","size":[24,10],"offset":[-76,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_3"}]}},"##,
            r##"{"effect_shadow_extra_4":{"type":"label","text":"#effect_timer_4","size":[24,10],"offset":[-102,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_4"}]}},"##,
            r##"{"effect_shadow_4":{"type":"label","text":"#effect_timer_4","size":[24,10],"offset":[-100,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_4"}]}},"##,
            r##"{"effect_timer_4":{"type":"label","text":"#effect_timer_4","size":[24,10],"offset":[-101,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_4"}]}},"##,
            r##"{"effect_shadow_extra_5":{"type":"label","text":"#effect_timer_5","size":[24,10],"offset":[-127,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_5"}]}},"##,
            r##"{"effect_shadow_5":{"type":"label","text":"#effect_timer_5","size":[24,10],"offset":[-125,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_5"}]}},"##,
            r##"{"effect_timer_5":{"type":"label","text":"#effect_timer_5","size":[24,10],"offset":[-126,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_5"}]}},"##,
            r##"{"effect_shadow_extra_6":{"type":"label","text":"#effect_timer_6","size":[24,10],"offset":[-152,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_6"}]}},"##,
            r##"{"effect_shadow_6":{"type":"label","text":"#effect_timer_6","size":[24,10],"offset":[-150,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_6"}]}},"##,
            r##"{"effect_timer_6":{"type":"label","text":"#effect_timer_6","size":[24,10],"offset":[-151,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_6"}]}},"##,
            r##"{"effect_shadow_extra_7":{"type":"label","text":"#effect_timer_7","size":[24,10],"offset":[-177,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_7"}]}},"##,
            r##"{"effect_shadow_7":{"type":"label","text":"#effect_timer_7","size":[24,10],"offset":[-175,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_7"}]}},"##,
            r##"{"effect_timer_7":{"type":"label","text":"#effect_timer_7","size":[24,10],"offset":[-176,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_7"}]}},"##,
            r##"{"effect_shadow_extra_8":{"type":"label","text":"#effect_timer_8","size":[24,10],"offset":[-202,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_8"}]}},"##,
            r##"{"effect_shadow_8":{"type":"label","text":"#effect_timer_8","size":[24,10],"offset":[-200,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_8"}]}},"##,
            r##"{"effect_timer_8":{"type":"label","text":"#effect_timer_8","size":[24,10],"offset":[-201,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_8"}]}},"##,
            r##"{"effect_shadow_extra_9":{"type":"label","text":"#effect_timer_9","size":[24,10],"offset":[-227,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_9"}]}},"##,
            r##"{"effect_shadow_9":{"type":"label","text":"#effect_timer_9","size":[24,10],"offset":[-225,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_9"}]}},"##,
            r##"{"effect_timer_9":{"type":"label","text":"#effect_timer_9","size":[24,10],"offset":[-226,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_9"}]}},"##,
            r##"{"effect_shadow_extra_10":{"type":"label","text":"#effect_timer_10","size":[24,10],"offset":[-252,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_10"}]}},"##,
            r##"{"effect_shadow_10":{"type":"label","text":"#effect_timer_10","size":[24,10],"offset":[-250,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_10"}]}},"##,
            r##"{"effect_timer_10":{"type":"label","text":"#effect_timer_10","size":[24,10],"offset":[-251,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_10"}]}},"##,
            r##"{"effect_shadow_extra_11":{"type":"label","text":"#effect_timer_11","size":[24,10],"offset":[-277,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_11"}]}},"##,
            r##"{"effect_shadow_11":{"type":"label","text":"#effect_timer_11","size":[24,10],"offset":[-275,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_11"}]}},"##,
            r##"{"effect_timer_11":{"type":"label","text":"#effect_timer_11","size":[24,10],"offset":[-276,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_11"}]}},"##,
            r##"{"effect_shadow_extra_12":{"type":"label","text":"#effect_timer_12","size":[24,10],"offset":[-302,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_12"}]}},"##,
            r##"{"effect_shadow_12":{"type":"label","text":"#effect_timer_12","size":[24,10],"offset":[-300,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_12"}]}},"##,
            r##"{"effect_timer_12":{"type":"label","text":"#effect_timer_12","size":[24,10],"offset":[-301,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_12"}]}},"##,
            r##"{"effect_shadow_extra_13":{"type":"label","text":"#effect_timer_13","size":[24,10],"offset":[-327,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_13"}]}},"##,
            r##"{"effect_shadow_13":{"type":"label","text":"#effect_timer_13","size":[24,10],"offset":[-325,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_13"}]}},"##,
            r##"{"effect_timer_13":{"type":"label","text":"#effect_timer_13","size":[24,10],"offset":[-326,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_13"}]}},"##,
            r##"{"effect_shadow_extra_14":{"type":"label","text":"#effect_timer_14","size":[24,10],"offset":[-352,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_14"}]}},"##,
            r##"{"effect_shadow_14":{"type":"label","text":"#effect_timer_14","size":[24,10],"offset":[-350,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_14"}]}},"##,
            r##"{"effect_timer_14":{"type":"label","text":"#effect_timer_14","size":[24,10],"offset":[-351,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_14"}]}},"##,
            r##"{"effect_shadow_extra_15":{"type":"label","text":"#effect_timer_15","size":[24,10],"offset":[-377,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_15"}]}},"##,
            r##"{"effect_shadow_15":{"type":"label","text":"#effect_timer_15","size":[24,10],"offset":[-375,16],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_15"}]}},"##,
            r##"{"effect_timer_15":{"type":"label","text":"#effect_timer_15","size":[24,10],"offset":[-376,15],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_15"}]}},"##,
            r##"{"effect_shadow_extra_16":{"type":"label","text":"#effect_timer_16","size":[24,10],"offset":[-2,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_16"}]}},"##,
            r##"{"effect_shadow_16":{"type":"label","text":"#effect_timer_16","size":[24,10],"offset":[0,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_16"}]}},"##,
            r##"{"effect_timer_16":{"type":"label","text":"#effect_timer_16","size":[24,10],"offset":[-1,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_16"}]}},"##,
            r##"{"effect_shadow_extra_17":{"type":"label","text":"#effect_timer_17","size":[24,10],"offset":[-27,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_17"}]}},"##,
            r##"{"effect_shadow_17":{"type":"label","text":"#effect_timer_17","size":[24,10],"offset":[-25,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_17"}]}},"##,
            r##"{"effect_timer_17":{"type":"label","text":"#effect_timer_17","size":[24,10],"offset":[-26,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_17"}]}},"##,
            r##"{"effect_shadow_extra_18":{"type":"label","text":"#effect_timer_18","size":[24,10],"offset":[-52,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_18"}]}},"##,
            r##"{"effect_shadow_18":{"type":"label","text":"#effect_timer_18","size":[24,10],"offset":[-50,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_18"}]}},"##,
            r##"{"effect_timer_18":{"type":"label","text":"#effect_timer_18","size":[24,10],"offset":[-51,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_18"}]}},"##,
            r##"{"effect_shadow_extra_19":{"type":"label","text":"#effect_timer_19","size":[24,10],"offset":[-77,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_19"}]}},"##,
            r##"{"effect_shadow_19":{"type":"label","text":"#effect_timer_19","size":[24,10],"offset":[-75,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_19"}]}},"##,
            r##"{"effect_timer_19":{"type":"label","text":"#effect_timer_19","size":[24,10],"offset":[-76,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_19"}]}},"##,
            r##"{"effect_shadow_extra_20":{"type":"label","text":"#effect_timer_20","size":[24,10],"offset":[-102,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_20"}]}},"##,
            r##"{"effect_shadow_20":{"type":"label","text":"#effect_timer_20","size":[24,10],"offset":[-100,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_20"}]}},"##,
            r##"{"effect_timer_20":{"type":"label","text":"#effect_timer_20","size":[24,10],"offset":[-101,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_20"}]}},"##,
            r##"{"effect_shadow_extra_21":{"type":"label","text":"#effect_timer_21","size":[24,10],"offset":[-127,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_21"}]}},"##,
            r##"{"effect_shadow_21":{"type":"label","text":"#effect_timer_21","size":[24,10],"offset":[-125,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_21"}]}},"##,
            r##"{"effect_timer_21":{"type":"label","text":"#effect_timer_21","size":[24,10],"offset":[-126,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_21"}]}},"##,
            r##"{"effect_shadow_extra_22":{"type":"label","text":"#effect_timer_22","size":[24,10],"offset":[-152,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_22"}]}},"##,
            r##"{"effect_shadow_22":{"type":"label","text":"#effect_timer_22","size":[24,10],"offset":[-150,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_22"}]}},"##,
            r##"{"effect_timer_22":{"type":"label","text":"#effect_timer_22","size":[24,10],"offset":[-151,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_22"}]}},"##,
            r##"{"effect_shadow_extra_23":{"type":"label","text":"#effect_timer_23","size":[24,10],"offset":[-177,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_23"}]}},"##,
            r##"{"effect_shadow_23":{"type":"label","text":"#effect_timer_23","size":[24,10],"offset":[-175,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_23"}]}},"##,
            r##"{"effect_timer_23":{"type":"label","text":"#effect_timer_23","size":[24,10],"offset":[-176,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_23"}]}},"##,
            r##"{"effect_shadow_extra_24":{"type":"label","text":"#effect_timer_24","size":[24,10],"offset":[-202,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_24"}]}},"##,
            r##"{"effect_shadow_24":{"type":"label","text":"#effect_timer_24","size":[24,10],"offset":[-200,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_24"}]}},"##,
            r##"{"effect_timer_24":{"type":"label","text":"#effect_timer_24","size":[24,10],"offset":[-201,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_24"}]}},"##,
            r##"{"effect_shadow_extra_25":{"type":"label","text":"#effect_timer_25","size":[24,10],"offset":[-227,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_25"}]}},"##,
            r##"{"effect_shadow_25":{"type":"label","text":"#effect_timer_25","size":[24,10],"offset":[-225,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_25"}]}},"##,
            r##"{"effect_timer_25":{"type":"label","text":"#effect_timer_25","size":[24,10],"offset":[-226,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_25"}]}},"##,
            r##"{"effect_shadow_extra_26":{"type":"label","text":"#effect_timer_26","size":[24,10],"offset":[-252,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_26"}]}},"##,
            r##"{"effect_shadow_26":{"type":"label","text":"#effect_timer_26","size":[24,10],"offset":[-250,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_26"}]}},"##,
            r##"{"effect_timer_26":{"type":"label","text":"#effect_timer_26","size":[24,10],"offset":[-251,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_26"}]}},"##,
            r##"{"effect_shadow_extra_27":{"type":"label","text":"#effect_timer_27","size":[24,10],"offset":[-277,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_27"}]}},"##,
            r##"{"effect_shadow_27":{"type":"label","text":"#effect_timer_27","size":[24,10],"offset":[-275,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_27"}]}},"##,
            r##"{"effect_timer_27":{"type":"label","text":"#effect_timer_27","size":[24,10],"offset":[-276,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_27"}]}},"##,
            r##"{"effect_shadow_extra_28":{"type":"label","text":"#effect_timer_28","size":[24,10],"offset":[-302,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_28"}]}},"##,
            r##"{"effect_shadow_28":{"type":"label","text":"#effect_timer_28","size":[24,10],"offset":[-300,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_28"}]}},"##,
            r##"{"effect_timer_28":{"type":"label","text":"#effect_timer_28","size":[24,10],"offset":[-301,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_28"}]}},"##,
            r##"{"effect_shadow_extra_29":{"type":"label","text":"#effect_timer_29","size":[24,10],"offset":[-327,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_29"}]}},"##,
            r##"{"effect_shadow_29":{"type":"label","text":"#effect_timer_29","size":[24,10],"offset":[-325,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_29"}]}},"##,
            r##"{"effect_timer_29":{"type":"label","text":"#effect_timer_29","size":[24,10],"offset":[-326,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_29"}]}},"##,
            r##"{"effect_shadow_extra_30":{"type":"label","text":"#effect_timer_30","size":[24,10],"offset":[-352,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_30"}]}},"##,
            r##"{"effect_shadow_30":{"type":"label","text":"#effect_timer_30","size":[24,10],"offset":[-350,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_30"}]}},"##,
            r##"{"effect_timer_30":{"type":"label","text":"#effect_timer_30","size":[24,10],"offset":[-351,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_30"}]}},"##,
            r##"{"effect_shadow_extra_31":{"type":"label","text":"#effect_timer_31","size":[24,10],"offset":[-377,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_31"}]}},"##,
            r##"{"effect_shadow_31":{"type":"label","text":"#effect_timer_31","size":[24,10],"offset":[-375,41],"anchor_from":"top_right","anchor_to":"top_right","color":[0,0,0,1],"shadow":false,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":19,"bindings":[{"binding_name":"#effect_timer_31"}]}},"##,
            r##"{"effect_timer_31":{"type":"label","text":"#effect_timer_31","size":[24,10],"offset":[-376,40],"anchor_from":"top_right","anchor_to":"top_right","color":[1,1,1,1],"shadow":true,"localize":false,"text_alignment":"center","font_scale_factor":0.65,"layer":20,"bindings":[{"binding_name":"#effect_timer_31"}]}}"##,
        )
    };
}
pub(crate) use document;

pub fn initial_bindings() -> String {
    bindings(&[0; SLOTS])
}

/// What each slot shows: 0 is blank, 1 an effect with no end, and anything else the seconds
/// left plus two. A number rather than a `String`, so a poll that changed nothing allocates
/// nothing and is told apart by [`fingerprint`].
pub type Codes = [u32; SLOTS];

pub const BLANK: u64 = fingerprint(&[0; SLOTS]);

/// One slot's code from the ticks an effect has left (`None` for no end).
pub fn code(ticks: Option<u64>) -> u32 {
    match ticks {
        Some(ticks) => (ticks.saturating_add(19) / 20).min(u64::from(u32::MAX) - 2) as u32 + 2,
        None => 1,
    }
}

/// Place each effect in the next free slot of its row; those past the sixteenth are dropped.
pub fn fill(codes: &mut Codes, effects: impl Iterator<Item = (i32, Option<u64>)>) {
    let mut counts = [0usize; 2];
    for (id, ticks) in effects {
        let Some(row) = row(id) else {
            continue;
        };
        let col = counts[row];
        counts[row] += 1;
        if col < ROW_LEN {
            codes[row * ROW_LEN + col] = code(ticks);
        }
    }
}

const ROW_LEN: usize = SLOTS / 2;

/// A cheap summary of what is shown, to tell "changed" from "same" without comparing the lot.
pub const fn fingerprint(codes: &Codes) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0;
    while i < SLOTS {
        hash = (hash ^ codes[i] as u64).wrapping_mul(0x0100_0000_01b3);
        i += 1;
    }
    hash
}

pub fn bindings(codes: &Codes) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for (i, &value) in codes.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, r##""#effect_timer_{i}":"{}""##, text(value));
    }
    out
}

/// The two rows match Cinnabar's pinned Java HUD registry.
pub fn row(id: i32) -> Option<usize> {
    match id {
        2 | 4 | 9 | 15 | 17..=20 | 24 | 25 | 32..=35 => Some(1),
        1 | 3 | 5 | 8 | 10..=14 | 16 | 21 | 22 | 26..=31 | 36 | 37 => Some(0),
        _ => None,
    }
}

/// The label for a [`code`].
pub fn text(code: u32) -> String {
    match code {
        0 => String::new(),
        1 => "inf".into(),
        _ => {
            let seconds = code - 2;
            format!("{}:{:02}", seconds / 60, seconds % 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_rows_and_timer_format_match_the_hud() {
        assert_eq!(row(16), Some(0));
        assert_eq!(row(19), Some(1));
        assert_eq!(row(6), None);
        assert_eq!(text(code(Some(1201))), "1:01");
        assert_eq!(text(code(Some(20))), "0:01");
        assert_eq!(text(code(Some(0))), "0:00");
        assert_eq!(text(code(None)), "inf");
        assert_eq!(text(0), "");
    }

    #[test]
    fn effects_fill_their_rows_in_order_and_skip_unknown_ones() {
        let mut codes = [0; SLOTS];
        let list = [(16, Some(100)), (6, Some(100)), (19, None), (21, Some(40))];
        fill(&mut codes, list.into_iter());
        assert_eq!(codes[0], code(Some(100)));
        assert_eq!(codes[1], code(Some(40)));
        assert_eq!(codes[16], 1);
        assert_eq!(codes.iter().filter(|&&c| c != 0).count(), 3);
    }

    #[test]
    fn a_full_row_drops_the_rest_instead_of_spilling_into_the_next() {
        let mut codes = [0; SLOTS];
        fill(&mut codes, (0..20).map(|_| (16, Some(20))));
        assert!(codes[..16].iter().all(|&c| c != 0));
        assert!(codes[16..].iter().all(|&c| c == 0));
    }

    #[test]
    fn fingerprint_tells_blank_from_shown_and_one_change_from_another() {
        let mut codes = [0; SLOTS];
        assert_eq!(fingerprint(&codes), BLANK);
        codes[3] = code(Some(100));
        let a = fingerprint(&codes);
        assert_ne!(a, BLANK);
        codes[3] = code(Some(120));
        assert_ne!(fingerprint(&codes), a);
    }

    #[test]
    fn blank_bindings_are_the_initial_ones() {
        assert_eq!(initial_bindings(), bindings(&[0; SLOTS]));
        assert!(initial_bindings().starts_with(r##""#effect_timer_0":"","##));
    }
}
