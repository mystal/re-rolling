use bevy::prelude::*;
use heron::prelude::*;

#[derive(PhysicsLayer)]
pub enum CollisionLayer {
    World,
    Hit,
    Hurt,
    Player,
    Enemy,
}

#[derive(Bundle)]
pub struct ColliderBundle {
    #[bundle]
    transform: TransformBundle,
    shape: CollisionShape,
    layers: CollisionLayers,
}

impl ColliderBundle {
    pub fn new(size: Vec2, offset: Vec2, groups: &[CollisionLayer], masks: &[CollisionLayer]) -> Self {
        Self {
            transform: TransformBundle {
                local: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            shape: CollisionShape::Cuboid {
                half_extends: (size / 2.0).extend(0.0),
                border_radius: None,
            },
            layers: CollisionLayers::none()
                .with_groups(groups)
                .with_masks(masks),
        }
    }
}
