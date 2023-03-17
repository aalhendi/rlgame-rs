rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

pub struct RexAssets {}

impl RexAssets {
    pub fn new() -> RexAssets {
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_POPULATED, "../../resources/wfc-populated.xp");

        RexAssets {}
    }
}
