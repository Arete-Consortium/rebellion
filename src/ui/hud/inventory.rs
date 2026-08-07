//! Inventory HUD — shows active persistent weapon-mod stacks with rarity colors.

use crate::core::CollectibleType;
use crate::entities::collectible::Rarity;
use crate::entities::items::{display_name, required_weapon_type, Inventory};
use crate::entities::player::Weapon;
use crate::entities::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryHudPanel;

#[derive(Component)]
pub struct InventoryHudText;

pub fn spawn_inventory_hud(mut commands: Commands, itch_mode: Res<crate::core::ItchMode>) {
    if itch_mode.enabled {
        return;
    }

    commands
        .spawn((
            InventoryHudPanel,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                top: Val::Px(120.0),
                min_width: Val::Px(150.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BorderColor(Color::srgba(0.2, 0.5, 0.9, 0.5)),
            BackgroundColor(Color::srgba(0.02, 0.05, 0.12, 0.85)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("MODULES"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.8, 1.0, 0.8)),
            ));
            p.spawn((
                InventoryHudText,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.94, 1.0)),
            ));
        });
}

pub fn update_inventory_hud(
    player: Query<(&Inventory, &Weapon), With<Player>>,
    mut text: Query<&mut Text, With<InventoryHudText>>,
) {
    let (Ok((inv, weapon)), Ok(mut t)) = (player.get_single(), text.get_single_mut()) else {
        return;
    };

    if inv.stacks.is_empty() {
        t.0 = "  —".into();
        return;
    }

    let lines: Vec<String> = inv
        .stacks
        .iter()
        .map(|s| {
            let label = display_name(s.id);
            if label.is_empty() {
                return format!("  ? {:?} ×{}", s.id, s.count);
            }
            let rarity = Rarity::for_collectible(s.id);
            let mark = match rarity {
                Rarity::Common => "·",
                Rarity::Uncommon => "◦",
                Rarity::Rare => "✦",
                Rarity::Epic => "★",
            };
            let active = match required_weapon_type(s.id) {
                Some(req) => weapon.weapon_type == req,
                None => true,
            };
            if active {
                format!("{} {} ×{}", mark, label, s.count)
            } else {
                format!("  {} {} ×{}  (inert)", mark, label, s.count)
            }
        })
        .collect();
    t.0 = lines.join("\n");
    let _ = CollectibleType::ScatterLauncher;
}
