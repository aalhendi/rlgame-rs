#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum HorizontalPlacement {
    Left,
    Center,
    Right,
}

#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum VerticalPlacement {
    Top,
    Center,
    Bottom,
}

#[derive(PartialEq, Clone, Copy)]
pub struct PrefabSection {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub placement: (HorizontalPlacement, VerticalPlacement),
}

pub const UNDERGROUND_FORT: PrefabSection = PrefabSection {
    template: RIGHT_FORT,
    width: 15,
    height: 43,
    placement: (HorizontalPlacement::Right, VerticalPlacement::Top),
};

const RIGHT_FORT: &str = "     #
  #######
  #     #
  #     #######
  #  g        #
  #     #######
  #     #
  ### ###
    # #
    # #
    # ##
    ^
    ^
    # ##
    # #
    # #
    # #
    # #
  ### ###
  #     #
  #     #
  #  g  #
  #     #
  #     #
  ### ###
    # #
    # #
    # #
    # ##
    ^
    ^
    # ##
    # #
    # #
    # #
  ### ###
  #     #
  #     #######
  #  g        #
  #     #######
  #     #
  #######
     #
";
