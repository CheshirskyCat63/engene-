use std::collections::HashMap;

use crate::core::mutation_policy::FixedTickContext;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::SystemDescriptor;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameAction {
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,
    Sprint,
    Jump,
    Crouch,
    Interact,
    Fire,
    Reload,
    Heal,
    Inventory,
    Map,
    QuickSave,
    QuickLoad,
    Pause,
}

#[derive(Clone, Debug)]
pub struct ActionBinding {
    pub action: GameAction,
    pub primary: String,
    pub secondary: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct InputActions {
    pub active: HashMap<GameAction, bool>,
    pub just_activated: HashMap<GameAction, bool>,
}

impl InputActions {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            just_activated: HashMap::new(),
        }
    }

    pub fn is_active(&self, action: GameAction) -> bool {
        self.active.get(&action).copied().unwrap_or(false)
    }

    pub fn was_just_activated(&self, action: GameAction) -> bool {
        self.just_activated.get(&action).copied().unwrap_or(false)
    }

    pub fn clear_frame(&mut self) {
        self.just_activated.clear();
    }
}

pub struct InputActionSystem {
    bindings: Vec<ActionBinding>,
}

impl InputActionSystem {
    pub fn new() -> Self {
        Self {
            bindings: Self::default_bindings(),
        }
    }

    fn default_bindings() -> Vec<ActionBinding> {
        vec![
            ActionBinding { action: GameAction::MoveForward, primary: "KeyW".into(), secondary: None },
            ActionBinding { action: GameAction::MoveBack, primary: "KeyS".into(), secondary: None },
            ActionBinding { action: GameAction::MoveLeft, primary: "KeyA".into(), secondary: None },
            ActionBinding { action: GameAction::MoveRight, primary: "KeyD".into(), secondary: None },
            ActionBinding { action: GameAction::Sprint, primary: "ShiftLeft".into(), secondary: None },
            ActionBinding { action: GameAction::Jump, primary: "Space".into(), secondary: None },
            ActionBinding { action: GameAction::Crouch, primary: "ControlLeft".into(), secondary: None },
            ActionBinding { action: GameAction::Interact, primary: "KeyE".into(), secondary: None },
            ActionBinding { action: GameAction::Fire, primary: "MouseLeft".into(), secondary: None },
            ActionBinding { action: GameAction::Reload, primary: "KeyR".into(), secondary: None },
            ActionBinding { action: GameAction::Heal, primary: "KeyH".into(), secondary: None },
            ActionBinding { action: GameAction::Inventory, primary: "KeyI".into(), secondary: Some("Tab".into()) },
            ActionBinding { action: GameAction::Map, primary: "KeyM".into(), secondary: None },
            ActionBinding { action: GameAction::QuickSave, primary: "F5".into(), secondary: None },
            ActionBinding { action: GameAction::QuickLoad, primary: "F9".into(), secondary: None },
            ActionBinding { action: GameAction::Pause, primary: "Escape".into(), secondary: None },
        ]
    }

    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }
}

impl EngineSystem for InputActionSystem {
    fn name(&self) -> &str {
        "InputActionSystem"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("InputActionSystem")
            .with_parallel(true)
    }

    fn register_resources(&mut self, res: &mut crate::core::registry::Resources) {
        res.insert(InputActions::new());
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        if let Some(actions) = ctx.resources.get_mut::<InputActions>() {
            actions.clear_frame();
        }
    }
}
