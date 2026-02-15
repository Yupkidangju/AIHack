// [v2.0.0 Phase R2] build.rs — TOML 데이터에서 MonsterKind/ItemKind enum 자동 생성
// 빌드 시 `assets/data/monsters.toml`과 `assets/data/items.toml`을 읽어
// `src/generated/kinds.rs`에 enum 코드를 출력합니다.
//
// 이 방식을 사용하면:
// 1. TOML에 새 몬스터/아이템 추가 시 enum이 자동 확장
// 2. 코드에서 누락된 variant가 있으면 컴파일 에러 발생
// 3. HashMap<String, T> 대신 enum 인덱싱으로 성능 향상

use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // TOML 파일 변경 시에만 재빌드
    println!("cargo:rerun-if-changed=assets/data/monsters.toml");
    println!("cargo:rerun-if-changed=assets/data/items.toml");

    let out_dir = "src/generated";
    fs::create_dir_all(out_dir).expect("generated 디렉토리 생성 실패");

    let mut output = String::new();
    output.push_str("// [v2.0.0 Phase R2] 자동 생성된 파일 — 직접 수정하지 마세요!\n");
    output.push_str("// build.rs에 의해 monsters.toml / items.toml에서 생성됨\n\n");
    output.push_str("use serde::{Deserialize, Serialize};\n\n");

    // === MonsterKind enum 생성 ===
    let monster_names = extract_names("assets/data/monsters.toml");
    // 코드에서 직접 사용되지만 TOML에 없는 몬스터 이름 추가
    let extra_monsters = vec![
        "orc".to_string(),
        "orc warrior".to_string(),
        "gold piece".to_string(),
    ];
    let mut all_monsters = monster_names;
    for extra in extra_monsters {
        if !all_monsters.iter().any(|m| m == &extra) {
            all_monsters.push(extra);
        }
    }

    output.push_str("/// 몬스터 종류 enum — TOML 데이터에서 자동 생성\n");
    output.push_str("/// [v2.0.0 R2] String 비교 대신 패턴 매칭으로 타입 안전성 확보\n");
    output.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
    output.push_str("pub enum MonsterKind {\n");
    for name in &all_monsters {
        let variant = to_variant_name(name);
        output.push_str(&format!("    /// \"{}\"\n", name));
        output.push_str(&format!("    {},\n", variant));
    }
    // Unknown variant 항상 포함 (역직렬화 실패 방지)
    output.push_str("    /// 알 수 없는 몬스터 (하위 호환용)\n");
    output.push_str("    Unknown,\n");
    output.push_str("}\n\n");

    // MonsterKind ↔ &str 변환 구현
    output.push_str("impl MonsterKind {\n");
    output.push_str("    /// 문자열에서 MonsterKind로 변환 (TOML 로드 시 사용)\n");
    output.push_str("    pub fn from_str(s: &str) -> Self {\n");
    output.push_str("        match s {\n");
    for name in &all_monsters {
        let variant = to_variant_name(name);
        output.push_str(&format!("            \"{}\" => Self::{},\n", name, variant));
    }
    output.push_str("            _ => Self::Unknown,\n");
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str("    /// MonsterKind를 원본 문자열로 변환 (표시용)\n");
    output.push_str("    pub fn as_str(&self) -> &'static str {\n");
    output.push_str("        match self {\n");
    for name in &all_monsters {
        let variant = to_variant_name(name);
        output.push_str(&format!("            Self::{} => \"{}\",\n", variant, name));
    }
    output.push_str("            Self::Unknown => \"unknown\",\n");
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Display 구현
    output.push_str("impl std::fmt::Display for MonsterKind {\n");
    output.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("        write!(f, \"{}\", self.as_str())\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // === ItemKind enum 생성 ===
    let item_names = extract_names("assets/data/items.toml");
    // 코드에서 사용되지만 TOML에 없는 아이템 추가
    let extra_items = vec![
        "unknown".to_string(),
        "lamp".to_string(),
        "oilskin bag".to_string(),
        "wand of cancellation".to_string(),
        "oil lamp".to_string(),
        "magic lamp".to_string(),
        "small shield".to_string(),
        "large shield".to_string(),
    ];
    let mut all_items = item_names;
    for extra in extra_items {
        if !all_items.iter().any(|i| i == &extra) {
            all_items.push(extra);
        }
    }

    output.push_str("/// 아이템 종류 enum — TOML 데이터에서 자동 생성\n");
    output.push_str("/// [v2.0.0 R2] String 비교 대신 패턴 매칭으로 타입 안전성 확보\n");
    output.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
    output.push_str("pub enum ItemKind {\n");
    for name in &all_items {
        let variant = to_variant_name(name);
        output.push_str(&format!("    /// \"{}\"\n", name));
        output.push_str(&format!("    {},\n", variant));
    }
    output.push_str("    /// 알 수 없는 아이템 (하위 호환용)\n");
    output.push_str("    UnknownItem,\n");
    output.push_str("}\n\n");

    // ItemKind ↔ &str 변환 구현
    output.push_str("impl ItemKind {\n");
    output.push_str("    /// 문자열에서 ItemKind로 변환 (TOML 로드 시 사용)\n");
    output.push_str("    pub fn from_str(s: &str) -> Self {\n");
    output.push_str("        match s {\n");
    for name in &all_items {
        let variant = to_variant_name(name);
        output.push_str(&format!("            \"{}\" => Self::{},\n", name, variant));
    }
    output.push_str("            _ => Self::UnknownItem,\n");
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str("    /// ItemKind를 원본 문자열로 변환 (표시용)\n");
    output.push_str("    pub fn as_str(&self) -> &'static str {\n");
    output.push_str("        match self {\n");
    for name in &all_items {
        let variant = to_variant_name(name);
        output.push_str(&format!("            Self::{} => \"{}\",\n", variant, name));
    }
    output.push_str("            Self::UnknownItem => \"unknown item\",\n");
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    // is_corpse() 헬퍼 (자주 사용되는 패턴)
    output.push_str("    /// 시체인지 여부 (corpse 체크)\n");
    output.push_str("    pub fn is_corpse(&self) -> bool {\n");
    output.push_str("        matches!(self, Self::Corpse)\n");
    output.push_str("    }\n\n");
    // is_ring() 헬퍼
    output.push_str("    /// 반지류인지 여부 (이름에 ring 포함 — 추후 ItemClass로 대체 예정)\n");
    output.push_str("    pub fn is_ring_name(&self) -> bool {\n");
    output.push_str("        self.as_str().contains(\"ring\")\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Display 구현
    output.push_str("impl std::fmt::Display for ItemKind {\n");
    output.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("        write!(f, \"{}\", self.as_str())\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    let out_path = Path::new(out_dir).join("kinds.rs");
    let mut file = fs::File::create(&out_path).expect("kinds.rs 파일 생성 실패");
    file.write_all(output.as_bytes())
        .expect("kinds.rs 작성 실패");

    println!(
        "cargo:warning=Generated {} with {} monsters and {} items",
        out_path.display(),
        all_monsters.len(),
        all_items.len()
    );
}

/// TOML 파일에서 `name = "xxx"` 패턴으로 이름 목록 추출
fn extract_names(path: &str) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("cargo:warning=TOML 파일을 찾을 수 없음: {}", path);
            return Vec::new();
        }
    };

    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let name = &trimmed[start + 1..start + 1 + end];
                    if seen.insert(name.to_string()) {
                        names.push(name.to_string());
                    }
                }
            }
        }
    }
    names
}

/// 문자열을 PascalCase enum variant 이름으로 변환
/// "giant ant" → "GiantAnt", "baby red dragon" → "BabyRedDragon"
/// "two-handed sword" → "TwoHandedSword", "bag of holding" → "BagOfHolding"
fn to_variant_name(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == ' ' || c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}
