fn upgrade_building(
    zones: Query<(Entity, &Transform, &BoxCollider), (With<MainBuildingLevel>, With<TriggerZone>)>,
    player: Query<(Entity, &Transform, &BoxCollider), With<ControlledPlayer>>,
) {
    if let Ok((player_entity, player_transform, player_collider)) = player.get_single() {
        for (zone, zone_transform, zone_collider) in zones.iter() {
            let player_bounds = Aabb2d::new(
                player_transform.translation.truncate(),
                player_collider.half_size(),
            );
            let zone_bounds = Aabb2d::new(
                zone_transform.translation.truncate(),
                zone_collider.half_size(),
            );
            if player_bounds.intersects(&zone_bounds) {
                println!("inside");
            }
        }
    }
}
