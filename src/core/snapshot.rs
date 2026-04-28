use serde::{Deserialize, Serialize};

use crate::{
    core::{
        event::GameEvent,
        ids::{EntityId, LevelId},
        position::Pos,
        session::RunState,
        turn::SnapshotHash,
        world::GameWorld,
    },
    domain::{
        entity::{EntityKind, EntityLocation},
        inventory::{InventoryEntry, InventoryLetter},
        item::ItemKind,
        tile::TileKind,
    },
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub kind: EntityKind,
    pub pos: Option<Pos>,
    pub hp: Option<i16>,
    pub alive: Option<bool>,
    pub item_kind: Option<ItemKind>,
    pub location: Option<EntityLocation>,
    pub assigned_letter: Option<InventoryLetter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSnapshot {
    pub id: LevelId,
    pub map_width: i16,
    pub map_height: i16,
    pub map_tiles: Vec<TileKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventorySnapshot {
    pub owner: EntityId,
    pub entries: Vec<InventoryEntry>,
    pub equipped_melee: Option<EntityId>,
    pub next_letter_index: u8,
}

/// [v0.1.0] Phase 4 snapshot은 item/inventory/equipment 상태를 hash 입력에 포함한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub seed: u64,
    pub turn: u64,
    pub run_state: RunState,
    pub event_count: usize,
    pub last_event: Option<GameEvent>,
    pub current_level: LevelId,
    pub player_pos: Pos,
    pub levels: Vec<LevelSnapshot>,
    pub entities: Vec<EntitySnapshot>,
    pub inventory: InventorySnapshot,
}

impl GameSnapshot {
    pub fn from_world(
        seed: u64,
        turn: u64,
        run_state: RunState,
        event_log: &[GameEvent],
        world: &GameWorld,
    ) -> Self {
        let mut entities = world
            .entities
            .entities()
            .iter()
            .map(|entity| {
                if let Some((_, _, level, pos, stats, alive)) = entity.actor() {
                    EntitySnapshot {
                        id: entity.id,
                        kind: entity.kind(),
                        pos: Some(pos),
                        hp: Some(stats.hp),
                        alive: Some(alive),
                        item_kind: None,
                        location: Some(EntityLocation::OnMap { level, pos }),
                        assigned_letter: None,
                    }
                } else {
                    let (kind, _, location, assigned_letter) =
                        entity.item().expect("actor가 아니면 item payload여야 한다");
                    EntitySnapshot {
                        id: entity.id,
                        kind: entity.kind(),
                        pos: None,
                        hp: None,
                        alive: None,
                        item_kind: Some(kind),
                        location: Some(location),
                        assigned_letter,
                    }
                }
            })
            .collect::<Vec<_>>();
        entities.sort_by_key(|entity| entity.id.0);

        let mut levels = world
            .levels
            .levels
            .iter()
            .map(|level| LevelSnapshot {
                id: level.id,
                map_width: level.map.width,
                map_height: level.map.height,
                map_tiles: level.map.tiles().to_vec(),
            })
            .collect::<Vec<_>>();
        levels.sort_by_key(|level| level.id);

        Self {
            seed,
            turn,
            run_state,
            event_count: event_log.len(),
            last_event: event_log.last().cloned(),
            current_level: world.current_level(),
            player_pos: world.player_pos(),
            levels,
            entities,
            inventory: InventorySnapshot {
                owner: world.inventory.owner,
                entries: world.inventory.entries.clone(),
                equipped_melee: world.inventory.equipped_melee,
                next_letter_index: world.inventory.next_letter_index,
            },
        }
    }

    pub fn stable_hash(&self) -> SnapshotHash {
        // [v0.1.0] serde_json은 구조체 필드 선언 순서를 보존하므로 안정 입력 문자열로 사용한다.
        let payload = serde_json::to_string(self)
            .expect("GameSnapshot은 Phase 4에서 직렬화 실패가 없는 닫힌 타입이다");
        SnapshotHash(format!("{:016x}", fnv1a64(payload.as_bytes())))
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use crate::core::{session::RunState, world::GameWorld};

    use super::GameSnapshot;

    #[test]
    fn 같은_snapshot은_같은_hash를_만든다() {
        let world = GameWorld::fixture_phase4();
        let a = GameSnapshot::from_world(42, 1, RunState::Playing, &[], &world);
        let b = a.clone();

        assert_eq!(a.stable_hash(), b.stable_hash());
    }
}
