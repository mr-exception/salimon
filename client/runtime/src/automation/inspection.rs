use super::*;

pub(crate) fn inspect_session(app: &GameplaySession) -> Value {
    let prediction=app.navigation_prediction().map(|p|json!({"horizon_seconds":p.horizon_seconds,"closest_altitude_meters":p.closest_altitude_meters,"closest_seconds":p.closest_seconds,"contact_window_seconds":p.contact_window_seconds,"convergence_meters":p.convergence_meters,"age_seconds":p.age_seconds,"warning":p.warning}));
    let snapshot = app.snapshot();
    let hud = crate::survival_hud::Observation::capture(app);
    let ship = snapshot.ship;
    let frame = character_ship_frame(ship.pose);
    let character = snapshot.character;
    let local_eye = frame.world_to_local(character.eye_position_meters);
    let local_look = frame.world_to_local(character.look_target_meters);
    let interaction = app.automation_interaction(local_eye, local_look);
    let bodies: Vec<_> = app.mining.terrain.bodies().iter().map(|body| {
        json!({"name": body.name, "center_meters": body.center.meters(),
            "radius_meters": body.radius_meters,
            "ship_surface_distance_meters": (body.center.meters().iter().zip(ship.pose.position_meters)
                .map(|(center, position)| (*center - position).powi(2)).sum::<f64>()).sqrt() - body.radius_meters})
    }).collect();
    let deposits = app.mining.nearby(character.eye_position_meters, app.seed);
    let (deposits, deposit_error) = match deposits {
        Ok(entries) => (entries.iter().copied().map(|entry| json!({
            "id": format!("{:?}:{}", entry.deposit.id().body, entry.deposit.id().local),
            "resource": entry.deposit.material().resource().key(),
            "position_meters": entry.deposit.position().meters(),
            "ship_local_position_meters": frame.world_to_local(entry.deposit.position().meters()),
            "body_local_position_meters": entry.body_local_position_meters,
            "bounds_radius_meters": entry.bounds_radius_meters,
            "remaining_mass_kg": entry.deposit.remaining_mass_kg(),
            "distance_to_player_meters": entry.deposit.position().offset_from(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).iter().map(|v| v * v).sum::<f64>().sqrt(),
            "state": format!("{:?}", entry.deposit.state()),
            "visual": Value::Null,
            "authored_visual": crate::resource_presentation::deposit_mesh_with_terrain(entry, app.mining.terrain).map(|mesh| json!({"mesh": format!("{:?}", mesh.mesh), "center_meters": mesh.center_meters, "side_meters": mesh.authored_mesh_scale}))
        })).collect::<Vec<_>>(), None),
        Err(error) => (Vec::new(), Some(format!("{error:?}"))),
    };
    let deposits_by_id: serde_json::Map<String, Value> = deposits
        .iter()
        .map(|entry| (entry["id"].as_str().unwrap().to_owned(), entry.clone()))
        .collect();
    let resource_types: std::collections::BTreeSet<_> = deposits
        .iter()
        .filter(|entry| entry["remaining_mass_kg"].as_f64().unwrap_or(0.0) > 0.0)
        .filter_map(|entry| entry["resource"].as_str())
        .collect();
    let nearest_deposit = deposits.iter().min_by(|a, b| {
        a["distance_to_player_meters"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["distance_to_player_meters"].as_f64().unwrap())
    });
    let mining_target = app.mining.target(
        &app.equipment,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open && ship.door_open_fraction >= 0.95,
        app.seed,
    );
    let fragments: Vec<_> = app.mining
        .nearby_fragments(character.eye_position_meters)
        .map(|piece| {
            let mesh = crate::resource_presentation::fragment_visual(piece);
            let sphere = crate::resource_presentation::fragment_mesh(piece);
            json!({
                "id": piece.id().0,
                "reference_frame": if app.mining.ship_fragments.contains_key(&piece.id()) { "ship" } else { "world" },
                "carried": app.mining.session.carried_id() == Some(piece.id()),
                "attached_to_tool": app.mining.session.attached_sphere_id() == Some(piece.id()),
                "sleeping": app.mining.session.carried_id() != Some(piece.id()) && app.mining.fragment_sleep.is_sleeping(piece.id()),
                "distance_to_player_meters": piece.transform().position().offset_from(salimon_world::WorldPosition::new(character.eye_position_meters[0], character.eye_position_meters[1], character.eye_position_meters[2])).iter().map(|v| v * v).sum::<f64>().sqrt(),
                "ship_local_position_meters": app.mining.ship_fragments.get(&piece.id()).copied().map(salimon_character::ShipLocalPoint::meters).unwrap_or_else(|| frame.world_to_local(piece.transform().position().meters())),
                "contact_fragment_ids": if app.mining.session.carried_id() == Some(piece.id()) { Vec::new() } else { app.mining.fragment_sleep.contact_neighbors(piece.id()).iter().map(|id| id.0).collect::<Vec<_>>() },
                "environment_supported": app.mining.session.carried_id() != Some(piece.id()) && app.mining.fragment_sleep.environment_supported(piece.id(), |position, radius| salimon_character::ship_floor_placement(position, radius).is_some()),
                "source_deposit_id": format!("{:?}:{}", piece.source().body, piece.source().local),
                "resource": piece.material().resource().key(),
                "mass_kg": piece.material().mass_kg(), "volume_m3": piece.material().volume_m3(),
                "side_meters": salimon_world::resource_fragments::diameter_meters(piece),
                "position_meters": piece.transform().position().meters(),
                "orientation_xyzw": piece.transform().orientation_xyzw(),
                "sphere_visual": {"asset": "resource.spherical-fragment", "radius_meters": sphere.authored_mesh_scale * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS, "tint": sphere.tint},
                "visual": {"color": mesh.color, "half_extents_meters": mesh.half_extents_meters}
            })
        })
        .collect();
    let fragments_by_id: std::collections::BTreeMap<_, _> = fragments
        .iter()
        .map(|piece| {
            (
                piece["id"]
                    .as_u64()
                    .expect("physical fragment ID")
                    .to_string(),
                piece,
            )
        })
        .collect();
    let fragment_target = crate::carrying::target(
        &app.mining,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open && ship.door_open_fraction >= 0.95,
    );
    let resource_context = crate::resource_context::context(
        &app.mining,
        &app.equipment,
        character,
        frame,
        ship.door_state == salimon_ship::DoorState::Open && ship.door_open_fraction >= 0.95,
        app.seed,
    );
    let player_velocity = app
        .character
        .eva_velocity()
        .unwrap_or(ship.velocity_meters_per_second);
    let relative_velocity =
        std::array::from_fn::<_, 3, _>(|i| player_velocity[i] - ship.velocity_meters_per_second[i]);
    let (ground_entries, ground_hits, ground_misses) = crate::terrain::ground_cache_stats();
    let fabrication = app.fabrication_snapshot();
    let machinery = crate::scout_feedback::capture(app, ship);
    let item = |item: &salimon_fabrication::OutputItem| json!({"recipe":item.recipe.key(), "job":item.job.0, "unit":item.unit, "mass_kg":item.mass.kg(), "category":format!("{:?}",item.category), "effect":app.consumable_effects.get(item.recipe.key())});
    let scanner = json!({"utc_millis":app.scanner.utc_millis,"powered":app.scanner.powered,"parameters":app.scanner.parameters,"selected":app.scanner.selected,"hint":app.scanner_hint(),"active_count":app.scanner.contacts.active(app.scanner.utc_millis).count(),"history":app.scanner.contacts.history().collect::<Vec<_>>(),"error":app.scanner.error});
    let mut observation = json!({
        "consumable_wheel": {"open":app.wheel.open,"held":app.wheel.held,"hovered":app.wheel.hovered.map(salimon_fabrication::ConsumableSlot::index)},
        "paused": app.is_paused(),
        "machinery": {
            "thrust":machinery.thrust, "rudder":machinery.rudder,
            "converting":machinery.converting, "core_charge":machinery.core_charge,
            "fabricator":format!("{:?}",machinery.fabricator),
            "fabrication_progress":machinery.fabrication_progress,
            "phase":machinery.phase, "cockpit_active":machinery.cockpit_active,
        },
        "fabricator": {
            "status":format!("{:?}",fabrication.status), "requested_watts":fabrication.requested_watts,
            "stored":fabrication.stored.iter().map(|bin| json!({"resource":bin.resource.key(),"milligrams":bin.milligrams})).collect::<Vec<_>>(),
            "jobs":fabrication.jobs.iter().map(|job| json!({"id":job.id.0,"recipe":job.recipe.key(),"remaining_quantity":job.remaining_quantity,"completed_quantity":job.completed_quantity,"progress_seconds":job.progress.as_secs_f64(),"unit_seconds":job.unit_duration.as_secs_f64()})).collect::<Vec<_>>(),
            "tray":fabrication.tray.as_ref().map(item), "queue_capacity":fabrication.queue_capacity,
            "ui_open":app.crafting_ui.open,
        },
        "consumables": {"food":app.consumables.food().iter().map(|slot|slot.as_ref().map(item)).collect::<Vec<_>>(), "medical":app.consumables.medical().iter().map(|slot|slot.as_ref().map(item)).collect::<Vec<_>>()},
        "simulation_backlog_millis": snapshot.backlog.as_secs_f64() * 1000.0,
        "resource_queries": app.mining.query_metrics(),
        "ground_cache": {"entries":ground_entries,"hits":ground_hits,"misses":ground_misses},
        "view_mode": app.view_name(),
        "terrain": { "mode": format!("{:?}", app.mining.terrain) },
        "resource_ui": {
            "context": resource_context.as_ref().map(|prompt| &prompt.text),
            "anchor_meters": resource_context.as_ref().and_then(|prompt| match prompt.placement {
                salimon_renderer::OverlayPlacement::World { anchor_meters, .. } => Some(anchor_meters),
                _ => None,
            }),
        },
        "journey_ended": app.survival.snapshot().terminal.then(||app.ended_run().map(|r|crate::journey_ended::Report::from(&r)).expect("validated terminal report")),
        "survival_hud": {"visible":hud.visible,"penalties":hud.penalties,"statuses":hud.statuses},
        "journey": {"active_duration":app.survival.snapshot().active_time, "distance_meters":app.run.journey.distance_meters, "location":crate::session::journey::NamedLocation::with_terrain(app.snapshot().character.eye_position_meters, app.mining.terrain)},
        "survival": {"hunger":app.survival.snapshot().hunger.value(),
            "health":app.survival.snapshot().health.value(), "fatigue":app.survival.snapshot().fatigue.value(),
            "terminal":app.survival.snapshot().terminal,
            "effects":app.survival.active_effects().map(|(id,stacks,remaining)|
                json!({"id":id.0,"stacks":stacks,"remaining_seconds":remaining.map(|d|d.as_secs_f64())})).collect::<Vec<_>>()},
        "survival_restrictions": {
            "hunger_movement_bps": app.restrictions().hunger_movement_bps,
            "fatigue_movement_bps": app.restrictions().fatigue_movement_bps,
            "movement_bps": app.restrictions().movement_bps,
            "interaction_bps": app.restrictions().interaction_bps,
            "exhausted": app.restrictions().exhausted,
            "can_pilot": app.restrictions().permits(salimon_survival::restrictions::Action::Pilot),
            "can_mine": app.restrictions().permits(salimon_survival::restrictions::Action::Mine),
            "can_consume": app.restrictions().permits(salimon_survival::restrictions::Action::Consume)
        },
        "equipment": {"slots": app.equipment.slots().map(|tool| tool.map(|tool| match tool {
            crate::equipment::EquipmentTool::MiningTool => "mining_tool",
        })), "selected_slot": app.equipment.selected().map(|slot| slot.index() + 1)},
        "carrying": {"object_id": app.mining.session.carried_id().map(|id| id.0),
            "target_id": fragment_target.map(|id| id.0),
            "context": crate::carrying::context(&app.mining, fragment_target, character),
            "last_action_feedback": app.mining.carry_feedback},
        "mining": { "equipped": app.equipment.mining_equipped(), "held": app.mining.input.held,
            "active": app.mining.input.held && mining_target.is_some(),
            "held_item_visible": app.held_item(character, mining_target.is_some()).is_some(),
            "extracted_mass_kg": app.mining.session.extracted_mass_kg(),
            "range_meters": salimon_world::mining::MINING_RANGE_METERS,
            "rate_kg_per_second": app.mining.config.rate_kg_per_second(),
            "capacity_kg": app.mining.config.cap_kg(),
            "reserved_mass_kg": app.mining.session.attached_sphere_id().and_then(|id| app.mining.session.fragments().iter().find(|p| p.id() == id)).map_or(0.0, |p| p.material().mass_kg()),
            "attached_fragment_id": app.mining.session.attached_sphere_id().map(|id| id.0),
            "growth_preview": app.mining.growth_preview(&app.equipment, character).map(|mesh| json!({"center_meters": mesh.center_meters, "radius_meters": mesh.authored_mesh_scale * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS, "tint": mesh.tint})),
            "target": mining_target.map(|target| json!({"id": format!("{:?}:{}", target.id.body, target.id.local), "distance_meters": target.distance_meters})) },
        "player": {"nearby_body": app.mining.terrain.nearest_body(character.eye_position_meters, salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS, true).map(|(body, distance)| json!({"name": body.name, "surface_distance_meters": distance})),
            "velocity_meters_per_second": player_velocity,
            "speed_meters_per_second": player_velocity.iter().map(|v| v*v).sum::<f64>().sqrt(),
            "relative_velocity_meters_per_second": relative_velocity,
            "relative_speed_meters_per_second": relative_velocity.iter().map(|v| v*v).sum::<f64>().sqrt(),
            "location": format!("{:?}", character.location),
            "eye_position_meters": character.eye_position_meters,
            "ship_local_eye_position_meters": local_eye,
            "look_target_meters": character.look_target_meters, "up": character.up,
            "local_ship_position_meters": character.local_ship_position_meters},
        "ship": {"position_meters": ship.pose.position_meters, "orientation": ship.pose.orientation,
            "flight_state": format!("{:?}", ship.flight_state), "door_state": format!("{:?}", ship.door_state), "door_open_fraction": ship.door_open_fraction,
            "cockpit_control_active": ship.cockpit_control_active,
            "thruster_percentage": ship.thruster_percentage, "total_mass_kg": ship.total_mass_kg, "speed_meters_per_second": ship.speed_meters_per_second,
            "velocity_meters_per_second": ship.velocity_meters_per_second,
            "power_available": ship.power_available,
            "navigation": {"prediction": prediction, "direction_local": ship.navigation.velocity_direction_local,
                "angle_degrees": ship.navigation.heading_velocity_angle_degrees,
                "acceleration": ship.navigation.acceleration_meters_per_second_squared,
                "braking": format!("{:?}", ship.navigation.braking)},
            "conversion_remaining_joules": ship.core_status.conversion_remaining_joules,
            "consumption_watts": ship.core_status.consumption_watts,
            "energy_core": {"stored_joules": ship.energy_core.stored_joules,
                "capacity_joules": ship.energy_core.capacity_joules},
            "nearby_body": ship.nearby_body.map(|body| json!({"name": body.name,
                "surface_distance_meters": body.surface_distance_meters,
                "radial_speed_meters_per_second": body.radial_speed_meters_per_second})),
            "cockpit_message": ship.cockpit_message.map(|message| message.text())},
        "interaction": interaction,
        "world": {"fragments": fragments, "fragments_by_id": fragments_by_id,
            "fragment_count": app.mining.session.fragments().len(),
            "sleeping_fragment_count": app.mining.session.fragments().iter().filter(|piece| app.mining.session.carried_id() != Some(piece.id()) && app.mining.fragment_sleep.is_sleeping(piece.id())).count(),
            "fragment_mass_kg": app.mining.session.fragments().iter().map(|piece| piece.material().mass_kg()).sum::<f64>(),
            "nearest_deposit": nearest_deposit, "resource_types": resource_types, "deposits": deposits, "deposits_by_id": deposits_by_id, "deposit_query_error": deposit_error, "bodies": bodies, "camera_phase": format!("{:?}", app.camera_phase())}
    });
    observation["core_upgrades"] =
        serde_json::to_value(&app.core_upgrades).expect("validated Core upgrades");
    observation["surface_age"] =
        serde_json::to_value(&app.surface_age).expect("validated surface age");
    observation["finishes"] = serde_json::to_value(&app.finishes).expect("validated finishes");
    observation["cabin"] = serde_json::to_value(&app.habitation.cabin).expect("validated cabin");
    observation["mounts"] = serde_json::to_value(&app.mounts).expect("validated attachment state");
    observation["habitation"] = json!({"bunk_open_fraction":app.habitation.bunk_open_fraction,"bunk_deploying":app.habitation.bunk_deploying});
    observation["cargo_door"] =
        json!({"open_fraction":app.cargo_door.open_fraction,"opening":app.cargo_door.opening});
    observation["lighting"] = app.light_inspect();
    observation["construction"] = app.construction_inspect();
    observation["ship"]["engines_enabled"] = ship.engines_enabled.into();
    observation["archive"] =
        json!({"source":app.archive, "open":app.archive_reader.open, "text":app.archive_text()});
    observation["recovery"] = app.recovery_observation();
    observation["tutorial"] = json!({"state":app.tutorial,"prompt":app.tutorial.text()});
    observation["scanner"] = scanner;
    observation["chart"] = json!({"source":app.chart.checkpoint(),"mode":app.scanner.chart_mode,"note":app.scanner.chart_note,"entry_count":app.chart.entries().count(),"route_count":app.chart.routes().len()});
    observation["field_marks"] =
        json!({"source":app.field_capture().ok().flatten(),"hint":app.field_hint()});
    observation["natural_sites"] = json!({
        "canonical": app.mining.terrain == salimon_world::terrain::TerrainMode::Canonical,
        "count": if app.mining.terrain == salimon_world::terrain::TerrainMode::Canonical { salimon_world::canonical::NATURAL_SITES.sites().len() } else { 0 },
        "publication": if app.mining.terrain == salimon_world::terrain::TerrainMode::Canonical { Some(salimon_world::canonical::NATURAL_SITES.identity()) } else { None },
    });
    observation
}
