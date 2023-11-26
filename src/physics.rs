use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub mod groups {
    use bevy_rapier2d::geometry::Group;

    pub const WORLD : Group = Group::GROUP_1;
    pub const HIT : Group = Group::GROUP_2;
    pub const HURT : Group = Group::GROUP_3;
    pub const PLAYER : Group = Group::GROUP_4;
    pub const ENEMY : Group = Group::GROUP_5;
}

#[derive(Bundle)]
pub struct ColliderBundle {
    transform: TransformBundle,
    shape: Collider,
    layers: CollisionGroups,
}

impl ColliderBundle {
    pub fn new(size: Vec2, offset: Vec2, groups: Group, masks: Group) -> Self {
        let half_extents = size / 2.0;
        Self {
            transform: TransformBundle {
                local: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            shape: Collider::cuboid(half_extents.x, half_extents.y),
            layers: CollisionGroups::new(groups, masks),
        }
    }
}
