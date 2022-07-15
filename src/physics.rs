use heron::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum CollisionLayer {
    Collision,
    Hit,
    Hurt,
    Player,
    Enemy,
}
