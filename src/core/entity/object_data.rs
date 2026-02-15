#[allow(unused_imports)]
use crate::core::entity::object::{ItemBits, ItemClass, ItemTemplate, Material};

///
#[allow(unused_macros)]
macro_rules! obj {
    ($name:expr, $bits:expr, $dir:expr, $mat:expr, $class:expr, $prob:expr, $wt:expr, $cost:expr, $sdam:expr, $ldam:expr, $oc1:expr, $oc2:expr, $nutr:expr, $color:expr) => {
        ItemTemplate {
            name: $name.to_string(),
            description: None,
            bits: ItemBits::from_bits_truncate($bits),
            dir: $dir,
            material: $mat,
            class: $class,
            subtype: 0,
            prop: 0,
            delay: 0,
            color: $color,
            prob: $prob,
            weight: $wt,
            cost: $cost,
            wsdam: $sdam,
            wldam: $ldam,
            oc1: $oc1,
            oc2: $oc2,
            nutrition: $nutr,
        }
    };
}

///
pub fn get_nethack_objects() -> Vec<ItemTemplate> {
    vec![
        // Weapons (Simplified for brevity in this step, but fully ported in assets/data/items.toml)
        //
    ]
}
