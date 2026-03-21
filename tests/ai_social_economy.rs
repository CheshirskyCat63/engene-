//! Integration tests for AI, social, and economy systems in the ENGENE engine.
//! Split into domain sections to reduce giant-file cognitive load.

use engene::core::ecs::Ecs;
use engene::core::persistent_id::PersistentEntityId;
use engene::game::ai::body::{is_night, time_of_day_mult, BodyState};
use engene::game::ai::decision::{decide_monster, decide_npc};
use engene::game::ai::emotions::{
    apply_monster_personality, apply_npc_personality, DominantEmotion, Emotions,
};
use engene::game::ai::goals::{pick_best, ScoredGoal};
use engene::game::ai::memory::{
    context_for_kind, CellTag, EventKind, EventMemory, Lesson, LessonAction, LessonContext, Memory,
};
use engene::game::ai::perception::{
    distance2, find_allies, find_predator, find_prey, npcs_nearby, PerceptionCache,
};
use engene::game::ai::plan::Plan;
use engene::game::economy::item_registry::{ItemCategory, ItemRarity, ItemRegistry, ItemTemplate};
use engene::game::economy::resource_flow::snapshot;
use engene::game::economy::trader_economy::{TraderInventorySlot, TraderState};
use engene::game::economy::trading::attempt_trade;
use engene::simulation::camp_simulation::CampState;
use engene::simulation::role_simulation::{NpcRole, RoleBehavior};
use engene::simulation::world_milestones::WorldMilestoneTracker;
use engene::world::components::{
    EcosystemNeeds, EntityKind, Goal, Job, MonsterSpecies, MonsterTraits, NpcEconomy, NpcTraits,
    PersonalNeeds, SocialNeeds, Transform,
};

#[path = "ai_social_economy/section_1.rs"]
mod section_1;
#[path = "ai_social_economy/section_2.rs"]
mod section_2;
#[path = "ai_social_economy/section_3.rs"]
mod section_3;
#[path = "ai_social_economy/section_4.rs"]
mod section_4;
#[path = "ai_social_economy/section_5.rs"]
mod section_5;
#[path = "ai_social_economy/section_6.rs"]
mod section_6;
