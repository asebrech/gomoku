use bevy::prelude::*;

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        // despawn() now automatically despawns children recursively in Bevy 0.15+
        commands.entity(entity).despawn();
    }
}