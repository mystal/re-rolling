use bevy::prelude::*;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct PlayerHealth {
    pub current: u8,
    pub max: u8,
}

impl PlayerHealth {
    pub fn new(max: u8) -> Self {
        Self {
            current: max,
            max,
        }
    }

    pub fn with_current(mut self, current: u8) -> Self {
        self.current = current.min(self.max);
        self
    }

    pub fn missing(&self) -> u8 {
        // Using saturating sub to prevent underflow if current somehow gets higher than max.
        self.max.saturating_sub(self.current)
    }

    /// Returns how much health was actually lost.
    pub fn lose_health(&mut self, amount: u8) -> u8 {
        let lost = amount.min(self.current);
        self.current -= lost;
        lost
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct EnemyHealth {
    pub current: f32,
    pub max: f32,
}

impl EnemyHealth {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    pub fn with_current(mut self, current: f32) -> Self {
        self.current = current.min(self.max);
        self
    }

    pub fn missing(&self) -> f32 {
        (self.max - self.current).max(0.0)
    }

    /// Returns how much health was actually lost.
    pub fn lose_health(&mut self, amount: f32) -> f32 {
        let lost = amount.min(self.current);
        self.current -= lost;
        lost
    }
}
