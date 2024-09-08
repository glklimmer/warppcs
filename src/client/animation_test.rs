fn animation_system() {
    for event in unit_events.read() {
        for (entity, mut animation_state, _) in &mut query {
            match event {
                UnitEvent::MeleeAttack(attacking_entity) => {
                    if entity == *attacking_entity {
                        atlas.layout = animations.attack.0.clone();
                        current_animation.timer = animations.attack.1.clone();
                    }
                }
                UnitEvent::Moving(attacking_entity) => {
                    if entity == *attacking_entity {
                        atlas.layout = animations.walk.0.clone();
                        current_animation.timer = animations.walk.1.clone();
                        match movement.facing {
                            Facing::Left => {
                                transform.scale.x = transform.scale.x.abs() * -1.;
                            }
                            Facing::Right => {
                                transform.scale.x = transform.scale.x.abs() * 1.;
                            }
                        }
                    }
                }
            }
        }
    }
}
