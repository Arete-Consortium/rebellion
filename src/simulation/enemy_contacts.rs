//! Ship contact damage. Detection is read-only; resolution respects the same
//! defenses as projectiles and limits a swarm to one impact per grace period.
use bevy::prelude::*;
use crate::core::{DamageLayer, DamageLayerEvent, DamageType, PlayerDamagedEvent};
use crate::entities::{Enemy, EnemyAI, EnemyBehavior, EnemyStats, Hitbox, Player, PowerupEffects, ShipStats};
use crate::systems::{AbilityEffects, ManeuverState};
use crate::systems::ability::player_incoming_damage;

#[derive(Event)]
pub struct EnemyShipContact { pub enemy: Entity, pub position: Vec2 }

#[derive(Resource, Default)]
pub struct ShipContactGrace(pub f32);

pub fn detect_enemy_ship_contacts(
    player: Query<(&Transform, &Hitbox), With<Player>>,
    enemies: Query<(Entity, &Transform, &EnemyStats, &EnemyAI), With<Enemy>>,
    mut contacts: EventWriter<EnemyShipContact>,
) {
    let Ok((player, hitbox)) = player.get_single() else { return; };
    for (enemy, transform, stats, ai) in &enemies {
        let position = transform.translation.truncate();
        if ai.active && stats.health > 0.0 && position.distance_squared(player.translation.truncate())
            < (hitbox.radius + 20.0).powi(2) {
            contacts.send(EnemyShipContact { enemy, position });
        }
    }
}

pub fn resolve_enemy_ship_contacts(
    time: Res<Time>,
    mut grace: ResMut<ShipContactGrace>,
    mut contacts: EventReader<EnemyShipContact>,
    mut player: Query<(&Transform, &mut ShipStats, &PowerupEffects, &ManeuverState, Option<&AbilityEffects>), With<Player>>,
    mut enemies: Query<(&mut EnemyStats, &EnemyAI), With<Enemy>>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut layer_events: EventWriter<DamageLayerEvent>,
) {
    grace.0 = (grace.0 - time.delta_secs()).max(0.0);
    let Ok((transform, mut stats, powerups, maneuver, ability)) = player.get_single_mut() else {
        contacts.clear();
        grace.0 = 0.0;
        return;
    };
    for contact in contacts.read() {
        if grace.0 > 0.0 { continue; }
        let Ok((mut enemy_stats, ai)) = enemies.get_mut(contact.enemy) else { continue; };
        if enemy_stats.health <= 0.0 || !ai.active { continue; }
        let kamikaze = ai.behavior == EnemyBehavior::Kamikaze;
        let damage = player_incoming_damage(if kamikaze { 25.0 } else { 10.0 }, ability, Some(powerups), Some(maneuver));
        if damage <= 0.0 { continue; }
        let result = stats.take_damage_detailed(damage, DamageType::Kinetic);
        grace.0 = 0.75;
        if kamikaze { enemy_stats.health = 0.0; }
        let position = transform.translation.truncate();
        let direction = (position - contact.position).normalize_or_zero();
        for (layer, amount) in [(DamageLayer::Shield, result.shield_damage),
            (DamageLayer::Armor, result.armor_damage), (DamageLayer::Hull, result.hull_damage)] {
            if amount > 0.0 { layer_events.send(DamageLayerEvent { position, layer, damage: amount, direction }); }
        }
        damage_events.send(PlayerDamagedEvent {
            damage, damage_type: DamageType::Kinetic, source_position: contact.position,
            shield_damage: result.shield_damage, armor_damage: result.armor_damage,
            hull_damage: result.hull_damage, destroyed: result.destroyed,
        });
    }
}
