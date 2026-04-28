// [v2.0.0 Phase R2] 자동 생성된 파일 — 직접 수정하지 마세요!
// build.rs에 의해 monsters.toml / items.toml에서 생성됨

use serde::{Deserialize, Serialize};

/// 몬스터 종류 enum — TOML 데이터에서 자동 생성
/// [v2.0.0 R2] String 비교 대신 패턴 매칭으로 타입 안전성 확보
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    /// "giant ant"
    GiantAnt,
    /// "killer bee"
    KillerBee,
    /// "soldier ant"
    SoldierAnt,
    /// "acid blob"
    AcidBlob,
    /// "kobold"
    Kobold,
    /// "little dog"
    LittleDog,
    /// "poisonous spider"
    PoisonousSpider,
    /// "oracle"
    Oracle,
    /// "baby red dragon"
    BabyRedDragon,
    /// "floating eye"
    FloatingEye,
    /// "priest"
    Priest,
    /// "watchman"
    Watchman,
    /// "yellow light"
    YellowLight,
    /// "orc"
    Orc,
    /// "orc warrior"
    OrcWarrior,
    /// "gold piece"
    GoldPiece,
    /// 알 수 없는 몬스터 (하위 호환용)
    Unknown,
}

impl MonsterKind {
    /// 문자열에서 MonsterKind로 변환 (TOML 로드 시 사용)
    pub fn from_str(s: &str) -> Self {
        match s {
            "giant ant" => Self::GiantAnt,
            "killer bee" => Self::KillerBee,
            "soldier ant" => Self::SoldierAnt,
            "acid blob" => Self::AcidBlob,
            "kobold" => Self::Kobold,
            "little dog" => Self::LittleDog,
            "poisonous spider" => Self::PoisonousSpider,
            "oracle" => Self::Oracle,
            "baby red dragon" => Self::BabyRedDragon,
            "floating eye" => Self::FloatingEye,
            "priest" => Self::Priest,
            "watchman" => Self::Watchman,
            "yellow light" => Self::YellowLight,
            "orc" => Self::Orc,
            "orc warrior" => Self::OrcWarrior,
            "gold piece" => Self::GoldPiece,
            _ => Self::Unknown,
        }
    }

    /// MonsterKind를 원본 문자열로 변환 (표시용)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GiantAnt => "giant ant",
            Self::KillerBee => "killer bee",
            Self::SoldierAnt => "soldier ant",
            Self::AcidBlob => "acid blob",
            Self::Kobold => "kobold",
            Self::LittleDog => "little dog",
            Self::PoisonousSpider => "poisonous spider",
            Self::Oracle => "oracle",
            Self::BabyRedDragon => "baby red dragon",
            Self::FloatingEye => "floating eye",
            Self::Priest => "priest",
            Self::Watchman => "watchman",
            Self::YellowLight => "yellow light",
            Self::Orc => "orc",
            Self::OrcWarrior => "orc warrior",
            Self::GoldPiece => "gold piece",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for MonsterKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 아이템 종류 enum — TOML 데이터에서 자동 생성
/// [v2.0.0 R2] String 비교 대신 패턴 매칭으로 타입 안전성 확보
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    /// "strange object"
    StrangeObject,
    /// "arrow"
    Arrow,
    /// "elven arrow"
    ElvenArrow,
    /// "orcish arrow"
    OrcishArrow,
    /// "silver arrow"
    SilverArrow,
    /// "ya"
    Ya,
    /// "crossbow bolt"
    CrossbowBolt,
    /// "dart"
    Dart,
    /// "shuriken"
    Shuriken,
    /// "boomerang"
    Boomerang,
    /// "dagger"
    Dagger,
    /// "short sword"
    ShortSword,
    /// "long sword"
    LongSword,
    /// "two-handed sword"
    TwoHandedSword,
    /// "mace"
    Mace,
    /// "morning star"
    MorningStar,
    /// "war hammer"
    WarHammer,
    /// "club"
    Club,
    /// "quarterstaff"
    Quarterstaff,
    /// "leather armor"
    LeatherArmor,
    /// "plate mail"
    PlateMail,
    /// "elven cloak"
    ElvenCloak,
    /// "adornment"
    Adornment,
    /// "regeneration"
    Regeneration,
    /// "healing"
    Healing,
    /// "extra healing"
    ExtraHealing,
    /// "identify"
    Identify,
    /// "magic missile"
    MagicMissile,
    /// "striking"
    Striking,
    /// "corpse"
    Corpse,
    /// "food ration"
    FoodRation,
    /// "tinning kit"
    TinningKit,
    /// "tin"
    Tin,
    /// "luckstone"
    Luckstone,
    /// "elven dagger"
    ElvenDagger,
    /// "orcish dagger"
    OrcishDagger,
    /// "scimitar"
    Scimitar,
    /// "orcish short sword"
    OrcishShortSword,
    /// "silver long sword"
    SilverLongSword,
    /// "runesword"
    Runesword,
    /// "spear"
    Spear,
    /// "trident"
    Trident,
    /// "bullwhip"
    Bullwhip,
    /// "knife"
    Knife,
    /// "chain mail"
    ChainMail,
    /// "orcish shield"
    OrcishShield,
    /// "uruk-hai shield"
    UrukHaiShield,
    /// "blindness"
    Blindness,
    /// "confusion"
    Confusion,
    /// "remove curse"
    RemoveCurse,
    /// "water"
    Water,
    /// "blank paper"
    BlankPaper,
    /// "pick-axe"
    PickAxe,
    /// "key"
    Key,
    /// "oil lamp"
    OilLamp,
    /// "magic lamp"
    MagicLamp,
    /// "brass lantern"
    BrassLantern,
    /// "large box"
    LargeBox,
    /// "chest"
    Chest,
    /// "ice box"
    IceBox,
    /// "sack"
    Sack,
    /// "bag of holding"
    BagOfHolding,
    /// "boulder"
    Boulder,
    /// "statue"
    Statue,
    /// "potion of oil"
    PotionOfOil,
    /// "Potion of healing"
    PotionOfHealing,
    /// "Scroll of teleportation"
    ScrollOfTeleportation,
    /// "Scroll of light"
    ScrollOfLight,
    /// "Wand of light"
    WandOfLight,
    /// "unknown"
    Unknown,
    /// "lamp"
    Lamp,
    /// "oilskin bag"
    OilskinBag,
    /// "wand of cancellation"
    WandOfCancellation,
    /// "small shield"
    SmallShield,
    /// "large shield"
    LargeShield,
    /// 알 수 없는 아이템 (하위 호환용)
    UnknownItem,
}

impl ItemKind {
    /// 문자열에서 ItemKind로 변환 (TOML 로드 시 사용)
    pub fn from_str(s: &str) -> Self {
        match s {
            "strange object" => Self::StrangeObject,
            "arrow" => Self::Arrow,
            "elven arrow" => Self::ElvenArrow,
            "orcish arrow" => Self::OrcishArrow,
            "silver arrow" => Self::SilverArrow,
            "ya" => Self::Ya,
            "crossbow bolt" => Self::CrossbowBolt,
            "dart" => Self::Dart,
            "shuriken" => Self::Shuriken,
            "boomerang" => Self::Boomerang,
            "dagger" => Self::Dagger,
            "short sword" => Self::ShortSword,
            "long sword" => Self::LongSword,
            "two-handed sword" => Self::TwoHandedSword,
            "mace" => Self::Mace,
            "morning star" => Self::MorningStar,
            "war hammer" => Self::WarHammer,
            "club" => Self::Club,
            "quarterstaff" => Self::Quarterstaff,
            "leather armor" => Self::LeatherArmor,
            "plate mail" => Self::PlateMail,
            "elven cloak" => Self::ElvenCloak,
            "adornment" => Self::Adornment,
            "regeneration" => Self::Regeneration,
            "healing" => Self::Healing,
            "extra healing" => Self::ExtraHealing,
            "identify" => Self::Identify,
            "magic missile" => Self::MagicMissile,
            "striking" => Self::Striking,
            "corpse" => Self::Corpse,
            "food ration" => Self::FoodRation,
            "tinning kit" => Self::TinningKit,
            "tin" => Self::Tin,
            "luckstone" => Self::Luckstone,
            "elven dagger" => Self::ElvenDagger,
            "orcish dagger" => Self::OrcishDagger,
            "scimitar" => Self::Scimitar,
            "orcish short sword" => Self::OrcishShortSword,
            "silver long sword" => Self::SilverLongSword,
            "runesword" => Self::Runesword,
            "spear" => Self::Spear,
            "trident" => Self::Trident,
            "bullwhip" => Self::Bullwhip,
            "knife" => Self::Knife,
            "chain mail" => Self::ChainMail,
            "orcish shield" => Self::OrcishShield,
            "uruk-hai shield" => Self::UrukHaiShield,
            "blindness" => Self::Blindness,
            "confusion" => Self::Confusion,
            "remove curse" => Self::RemoveCurse,
            "water" => Self::Water,
            "blank paper" => Self::BlankPaper,
            "pick-axe" => Self::PickAxe,
            "key" => Self::Key,
            "oil lamp" => Self::OilLamp,
            "magic lamp" => Self::MagicLamp,
            "brass lantern" => Self::BrassLantern,
            "large box" => Self::LargeBox,
            "chest" => Self::Chest,
            "ice box" => Self::IceBox,
            "sack" => Self::Sack,
            "bag of holding" => Self::BagOfHolding,
            "boulder" => Self::Boulder,
            "statue" => Self::Statue,
            "potion of oil" => Self::PotionOfOil,
            "Potion of healing" => Self::PotionOfHealing,
            "Scroll of teleportation" => Self::ScrollOfTeleportation,
            "Scroll of light" => Self::ScrollOfLight,
            "Wand of light" => Self::WandOfLight,
            "unknown" => Self::Unknown,
            "lamp" => Self::Lamp,
            "oilskin bag" => Self::OilskinBag,
            "wand of cancellation" => Self::WandOfCancellation,
            "small shield" => Self::SmallShield,
            "large shield" => Self::LargeShield,
            _ => Self::UnknownItem,
        }
    }

    /// ItemKind를 원본 문자열로 변환 (표시용)
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StrangeObject => "strange object",
            Self::Arrow => "arrow",
            Self::ElvenArrow => "elven arrow",
            Self::OrcishArrow => "orcish arrow",
            Self::SilverArrow => "silver arrow",
            Self::Ya => "ya",
            Self::CrossbowBolt => "crossbow bolt",
            Self::Dart => "dart",
            Self::Shuriken => "shuriken",
            Self::Boomerang => "boomerang",
            Self::Dagger => "dagger",
            Self::ShortSword => "short sword",
            Self::LongSword => "long sword",
            Self::TwoHandedSword => "two-handed sword",
            Self::Mace => "mace",
            Self::MorningStar => "morning star",
            Self::WarHammer => "war hammer",
            Self::Club => "club",
            Self::Quarterstaff => "quarterstaff",
            Self::LeatherArmor => "leather armor",
            Self::PlateMail => "plate mail",
            Self::ElvenCloak => "elven cloak",
            Self::Adornment => "adornment",
            Self::Regeneration => "regeneration",
            Self::Healing => "healing",
            Self::ExtraHealing => "extra healing",
            Self::Identify => "identify",
            Self::MagicMissile => "magic missile",
            Self::Striking => "striking",
            Self::Corpse => "corpse",
            Self::FoodRation => "food ration",
            Self::TinningKit => "tinning kit",
            Self::Tin => "tin",
            Self::Luckstone => "luckstone",
            Self::ElvenDagger => "elven dagger",
            Self::OrcishDagger => "orcish dagger",
            Self::Scimitar => "scimitar",
            Self::OrcishShortSword => "orcish short sword",
            Self::SilverLongSword => "silver long sword",
            Self::Runesword => "runesword",
            Self::Spear => "spear",
            Self::Trident => "trident",
            Self::Bullwhip => "bullwhip",
            Self::Knife => "knife",
            Self::ChainMail => "chain mail",
            Self::OrcishShield => "orcish shield",
            Self::UrukHaiShield => "uruk-hai shield",
            Self::Blindness => "blindness",
            Self::Confusion => "confusion",
            Self::RemoveCurse => "remove curse",
            Self::Water => "water",
            Self::BlankPaper => "blank paper",
            Self::PickAxe => "pick-axe",
            Self::Key => "key",
            Self::OilLamp => "oil lamp",
            Self::MagicLamp => "magic lamp",
            Self::BrassLantern => "brass lantern",
            Self::LargeBox => "large box",
            Self::Chest => "chest",
            Self::IceBox => "ice box",
            Self::Sack => "sack",
            Self::BagOfHolding => "bag of holding",
            Self::Boulder => "boulder",
            Self::Statue => "statue",
            Self::PotionOfOil => "potion of oil",
            Self::PotionOfHealing => "Potion of healing",
            Self::ScrollOfTeleportation => "Scroll of teleportation",
            Self::ScrollOfLight => "Scroll of light",
            Self::WandOfLight => "Wand of light",
            Self::Unknown => "unknown",
            Self::Lamp => "lamp",
            Self::OilskinBag => "oilskin bag",
            Self::WandOfCancellation => "wand of cancellation",
            Self::SmallShield => "small shield",
            Self::LargeShield => "large shield",
            Self::UnknownItem => "unknown item",
        }
    }

    /// 시체인지 여부 (corpse 체크)
    pub fn is_corpse(&self) -> bool {
        matches!(self, Self::Corpse)
    }

    /// 반지류인지 여부 (이름에 ring 포함 — 추후 ItemClass로 대체 예정)
    pub fn is_ring_name(&self) -> bool {
        self.as_str().contains("ring")
    }
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
