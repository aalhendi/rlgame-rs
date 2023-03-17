rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");

pub struct RexAssets {}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../../resources/wfc-demo1.xp");

        RexAssets {}
    }
}
