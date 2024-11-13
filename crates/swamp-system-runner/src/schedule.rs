use swamp_system::{IntoSystem, System, SystemParam};
use swamp_system_state::State;

#[derive(Default)]
pub struct Schedule {
    systems: Vec<Box<dyn System>>,
}

impl Schedule {
    #[must_use]
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<F, Params>(&mut self, function: F)
    where
        F: IntoSystem<Params>,
        Params: SystemParam,
    {
        self.systems.push(Box::new(function.into_system()));
    }

    pub fn run_systems(&self, state: &mut State) {
        for system in &self.systems {
            system.run(state);
        }
    }
}
