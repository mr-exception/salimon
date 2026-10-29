//! Native frame composition; domain visuals belong in scene adapters.
use super::surface_size;
use super::{
    ActiveEventLoop, Arc, CameraFrame, ClientApplication, ControlFlow, IDLE_RETRY_DELAY, Instant,
    OverlayPlacement, RENDER_RETRY_DELAY, RenderOutcome, Renderer, RendererOverlayImage,
    SceneFrame, TIMING_LOG_INTERVAL, ViewMode, Window, camera_body_distances_with_terrain,
    camera_domain_metrics, character_ship_frame, diagnostics, gameplay_domain_metrics,
    gameplay_window_title, map_world_to_renderer, scene,
};
use std::io::Write;
use std::time::Duration;
mod prompts;

impl ClientApplication {
    pub(super) fn redraw(&mut self, event_loop: &ActiveEventLoop, window: Arc<Window>) {
        if self.occluded {
            self.frame_clock.reset_interval();
            self.diagnostics.reset_frame_window();
            return;
        }

        self.render_attempts = self.render_attempts.saturating_add(1);
        let render_started_at = Instant::now();
        let update_started_at = Instant::now();
        let elapsed = if self.automation.is_some() {
            Duration::ZERO
        } else {
            self.session.update_clock.step(update_started_at)
        };
        let (update_delta, ticks) = self.advance_native(elapsed);
        let simulation_time = update_started_at.elapsed();
        self.session.sync_carried();
        self.show_session_feedback();
        let ship_snapshot = self.session.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface_frame = self.session.terrain_frame(ship_snapshot);
        let world_snapshot = self.session.camera_prototype.snapshot();
        let (mut camera, mut scene_instances, mut spheres, mut light) =
            map_world_to_renderer(world_snapshot);
        let mut resource_meshes = Vec::new();
        let character_snapshot = self.session.character.snapshot(ship_frame, surface_frame);
        let presentation = self.session.presentation();
        let render_ship_frame = character_ship_frame(presentation.ship.pose);
        if self.session.view_mode == ViewMode::Gameplay {
            scene_instances.extend(scene::construction_wireframes(
                &self.session,
                render_ship_frame,
                presentation.ship.pose.orientation.map(|v| v as f32),
            ));
        }
        let prompts::PromptFrame {
            monitor_message,
            interaction,
            mining_target,
            contextual_placement,
        } = self.prepare_prompts(
            character_snapshot,
            ship_snapshot,
            ship_frame,
            render_ship_frame,
            &presentation,
        );
        window.set_title(&self.benchmark.as_ref().map_or_else(
            || {
                let title = if self.session.survival.snapshot().terminal {
                    "Salimon - Journey Ended".to_owned()
                } else {
                    gameplay_window_title(monitor_message, interaction)
                };
                self.checkpoints
                    .as_ref()
                    .and_then(|c| c.warning)
                    .map_or(title.clone(), |warning| format!("{title} | {warning}"))
            },
            crate::benchmark::Recorder::window_title,
        ));
        if self.session.view_mode == ViewMode::Gameplay {
            scene_instances.extend(self.session.scanner_geometry(presentation.ship.pose));
            scene_instances.extend(scene::room_lamps(&self.session, presentation.ship));
            scene_instances.extend(scene::cargo_door(&self.session, presentation.ship));
        }
        let ship_mesh = if self.session.view_mode == ViewMode::Gameplay {
            camera = CameraFrame {
                position_meters: presentation.character.eye_position_meters,
                target_meters: presentation.character.look_target_meters,
                up: presentation.character.up,
                vertical_fov_radians: 70.0_f32.to_radians(),
                near_plane_meters: 0.05,
            };
            self.condition_feedback.camera(&mut camera);
            if let Some(view) = &self.scout_reference {
                camera = view.camera(presentation.ship.pose);
            }
            let mesh = scene::gameplay_ship(&self.session, presentation.ship);
            Some(mesh)
        } else {
            None
        };
        if self.session.view_mode == ViewMode::Gameplay {
            match self
                .session
                .mining
                .nearby(camera.position_meters, self.world_seed())
            {
                Ok(deposits) => {
                    resource_meshes.extend(deposits.iter().copied().filter_map(|entry| {
                        crate::resource_presentation::deposit_mesh_with_terrain(
                            entry,
                            self.session.mining.terrain,
                        )
                    }));
                }
                Err(error) => log::error!("deposit presentation query failed: {error:?}"),
            }
        }
        if self.session.view_mode == ViewMode::Gameplay {
            for fragment in self.session.mining.nearby_fragments(camera.position_meters) {
                if self.session.mining.session.attached_sphere_id() == Some(fragment.id()) {
                    continue;
                }
                resource_meshes.push(presentation.fragment_mesh(&self.session, fragment));
            }
        }
        if self.session.view_mode == ViewMode::Gameplay
            && let Some(preview) = self
                .session
                .mining
                .growth_preview(&self.session.equipment, presentation.character)
        {
            resource_meshes.push(preview);
        }
        if let Some(stream) = &mut self.universe_stream {
            let result = if self.session.mining.terrain
                == salimon_world::terrain::TerrainMode::Canonical
            {
                salimon_world::canonical::START
                    .manifest()
                    .distance
                    .system_position
                    .translated(camera.position_meters)
            } else {
                salimon_universe::coordinates::GalacticPosition::new([0; 3], camera.position_meters)
            }
            .and_then(|position| {
                stream.update(position, presentation.ship.velocity_meters_per_second)
            });
            let error = result
                .err()
                .map(|e| e.to_string())
                .or_else(|| stream.last_error.clone());
            if let Some(error) = error {
                self.fail(event_loop, error);
                return;
            }
        }
        let sky = if self.session.mining.terrain == salimon_world::terrain::TerrainMode::Canonical {
            match super::sky::scene(
                camera.position_meters,
                self.session.elapsed,
                self.universe_stream.as_ref(),
            ) {
                Ok(sky) => Some(sky),
                Err(error) => {
                    self.fail(event_loop, error);
                    return;
                }
            }
        } else {
            None
        };
        let atmosphere =
            if self.session.mining.terrain == salimon_world::terrain::TerrainMode::Canonical {
                match super::atmosphere::canonical() {
                    Ok(a) => a,
                    Err(e) => {
                        self.fail(event_loop, e);
                        return;
                    }
                }
            } else {
                None
            };
        let origin = salimon_universe::Epoch::calendar(
            [2000, 1, 1],
            [12, 0, 0, 0],
            salimon_universe::TimeScale::Tt,
        )
        .expect("fixed TT epoch");
        let environment_epoch = match i64::try_from(self.session.elapsed.as_nanos())
            .ok()
            .and_then(|ns| origin.advance_nanos(ns).ok())
        {
            Some(epoch) => epoch,
            None => {
                self.fail(event_loop, "environment clock exceeds supported TT epoch");
                return;
            }
        };
        let weather_frame =
            if self.session.mining.terrain == salimon_world::terrain::TerrainMode::Canonical {
                match super::weather::canonical(environment_epoch) {
                    Ok(frame) => frame,
                    Err(e) => {
                        self.fail(event_loop, e);
                        return;
                    }
                }
            } else {
                None
            };
        let weather = weather_frame
            .as_ref()
            .map(|f| super::weather::scene(f, 16, [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]));
        if let Some(frame) = &weather_frame {
            light.color = light.color.map(|c| c * frame.local_light_transmission());
        }
        let organisms = super::organisms::canonical(camera.position_meters, environment_epoch);
        let render_terrain = if self.session.view_mode == ViewMode::Gameplay {
            self.session.mining.terrain
        } else {
            salimon_world::terrain::TerrainMode::Flat
        };
        if render_terrain == salimon_world::terrain::TerrainMode::Canonical {
            // Declared fixed illumination for canonical terrain presentation;
            // the astronomical sky/ephemeris light adapter remains its own owner.
            light.position_meters = [
                0.,
                salimon_world::canonical::START
                    .manifest()
                    .physical
                    .semi_major_axis_meters,
                0.,
            ];

            spheres = render_terrain
                .bodies()
                .iter()
                .map(|b| salimon_renderer::SphereInstance {
                    center_meters: b.center.meters(),
                    radius_meters: b.radius_meters,
                    material: salimon_renderer::SurfaceMaterial::Slate,
                })
                .collect();
        }
        if let Err(error) = self
            .terrain_stream
            .update(camera.position_meters, render_terrain)
        {
            self.fail(event_loop, error);
            return;
        }
        for sphere in &mut spheres {
            let body = crate::terrain::body_at(sphere.center_meters, sphere.radius_meters);
            sphere.radius_meters =
                crate::terrain::far_radius(body, camera.position_meters, render_terrain);
        }
        let body_distances = camera_body_distances_with_terrain(
            world_snapshot,
            camera.position_meters,
            render_terrain,
        );
        let cpu_update_time = update_started_at.elapsed();
        let terminal = self.session.survival.snapshot().terminal;
        let terminal_report = if terminal {
            match self.session.ended_run() {
                Ok(report) => Some(crate::journey_ended::Report::from(&report)),
                Err(error) => {
                    self.fail(event_loop, error);
                    return;
                }
            }
        } else {
            None
        };
        let crafting_text = if self.session.is_paused() && self.condition_feedback.settings_open {
            Some(self.condition_feedback.menu_text())
        } else {
            self.pause_menu_text()
                .or_else(|| self.session.archive_text())
                .or_else(|| self.session.crafting_text())
        };
        let ended_image = self.journey_ended.image(
            terminal_report,
            [window.inner_size().width, window.inner_size().height],
            window.scale_factor(),
        );
        let crafting_pixels = crafting_text
            .as_ref()
            .map(|text| crate::action_bar::rasterize_action_bar(text, 3));
        if self.session.wheel.open {
            self.wheel_raster.update(self.session.wheel_presentation());
        }
        let wheel_image = self
            .session
            .wheel
            .open
            .then(|| self.wheel_raster.image())
            .flatten();
        let hud_observation = crate::survival_hud::Observation::capture(&self.session);
        let hud_visible = hud_observation.visible && !self.diagnostics.is_visible();
        let hud_frame = self.survival_hud.observe(hud_observation);
        let hud_image = self.survival_hud.image(
            hud_frame,
            [window.inner_size().width, window.inner_size().height],
            window.scale_factor(),
            hud_visible,
        );
        let gameplay_overlay = crafting_pixels
            .as_ref()
            .map(|(width, height, pixels)| RendererOverlayImage {
                width: *width,
                height: *height,
                rgba8: pixels,
                revision: {
                    use std::hash::{Hash, Hasher};
                    let mut hash = std::collections::hash_map::DefaultHasher::new();
                    crafting_text.hash(&mut hash);
                    hash.finish()
                },
                placement: OverlayPlacement::Center,
            })
            .or_else(|| {
                self.diagnostics
                    .overlay()
                    .map(|image| RendererOverlayImage {
                        width: image.width,
                        height: image.height,
                        rgba8: image.rgba8,
                        revision: image.revision,
                        placement: OverlayPlacement::TopLeft,
                    })
            })
            .or(hud_image);
        let overlay_image = if terminal {
            ended_image
        } else {
            wheel_image.or(gameplay_overlay)
        };
        // Modal text and the Q wheel own interaction; world prompts/reticle must
        // not obscure their rows or suggest a competing world action.
        let world_interaction_visible = self.session.view_mode == ViewMode::Gameplay
            && !terminal
            && !self.session.is_paused()
            && crafting_text.is_none()
            && !self.session.wheel.open;
        let action_bar_image = self
            .action_bar
            .image(world_interaction_visible)
            .map(|image| RendererOverlayImage {
                width: image.width,
                height: image.height,
                rgba8: image.rgba8,
                revision: image.revision,
                placement: if self.action_bar.has_transient() {
                    OverlayPlacement::BottomCenter
                } else {
                    contextual_placement
                },
            });
        let held_item = (!terminal)
            .then(|| {
                self.session
                    .held_item(presentation.character, mining_target)
            })
            .flatten();
        let fixture_instances = if self.session.view_mode == ViewMode::Gameplay {
            self.session.fixture_instances(presentation.ship)
        } else {
            Vec::new()
        };
        let outcome = match self
            .renderer
            .as_mut()
            .expect("redraws require an initialized renderer")
            .render(
                SceneFrame {
                    sky: sky.as_ref(),
                    atmosphere,
                    weather: weather.as_ref(),
                    organisms: &organisms,
                    camera,
                    instances: &scene_instances,
                    resource_meshes: &resource_meshes,
                    fixtures: &fixture_instances,
                    spheres: &spheres,
                    terrain: &self.terrain_stream.active,
                    terrain_mask: self.terrain_stream.mask,
                    light: Some(light),
                    ship: ship_mesh,
                    held_item,
                },
                self.condition_feedback.image(),
                overlay_image,
                action_bar_image,
                crate::reticle::image(world_interaction_visible, &self.session.equipment).or(
                    if self.session.wheel.open {
                        hud_image
                    } else {
                        None
                    },
                ),
                (self.session.view_mode == ViewMode::Gameplay
                    && !terminal
                    && !self.session.is_paused())
                .then(|| self.session.equipment_presentation()),
                window.scale_factor(),
                || window.pre_present_notify(),
            ) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.fail(event_loop, error);
                return;
            }
        };
        if self.render_attempts == 1
            || (!matches!(outcome, RenderOutcome::Presented(_))
                && self.render_attempts.is_multiple_of(100))
        {
            log::debug!(
                "render attempt {} completed with {outcome:?}",
                self.render_attempts
            );
        }

        if let Some(capture) = self.renderer.as_mut().and_then(Renderer::take_capture) {
            let result = capture.map_err(|e| e.to_string()).and_then(|bytes| {
                let path = self.capture_path.take().ok_or("capture path missing")?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|e| format!("capture {}: {e}", path.display()))?;
                file.write_all(&bytes)
                    .map_err(|e| format!("capture {}: {e}", path.display()))
            });
            self.capture_status =
                result.map_or_else(|e| format!("failed: {e}"), |()| "captured".into());
        }
        match outcome {
            RenderOutcome::Presented(render_stats) => {
                let ready = window.has_focus()
                    && window.is_visible() == Some(true)
                    && window.is_minimized() != Some(true)
                    && !self.occluded;
                if let Some(benchmark) = self.benchmark.as_mut()
                    && benchmark.native_readiness_changed(ready)
                    && let Err(error) = benchmark.observe_native("presented_window_poll", ready)
                {
                    self.fail(event_loop, error);
                    return;
                }
                self.retry_at = None;
                event_loop.set_control_flow(ControlFlow::Wait);
                let timing = self
                    .frame_clock
                    .record_presented(render_started_at, Instant::now());
                self.observations.record(
                    timing.frame_interval,
                    cpu_update_time,
                    render_stats,
                    [window.inner_size().width, window.inner_size().height],
                );
                self.diagnostics.record_presented(
                    diagnostics::frame_sample(timing, render_stats, cpu_update_time),
                    if self.session.view_mode == ViewMode::Gameplay {
                        gameplay_domain_metrics(
                            character_snapshot.eye_position_meters,
                            ship_snapshot,
                            &body_distances,
                        )
                    } else {
                        camera_domain_metrics(world_snapshot, &body_distances)
                    },
                );
                if timing.frame_number == 1
                    || timing.frame_number.is_multiple_of(TIMING_LOG_INTERVAL)
                    || log::log_enabled!(target: "salimon_client::frame", log::Level::Trace)
                {
                    log::debug!(
                        target: "salimon_client::frame",
                        "frame={} interval_ms={:.3} present_wall_ms={:.3} cpu_update_ms={:.3} cpu_render_ms={:.3} gpu_time={:?} ticks={} backlog_ms={:.3} focused={} occluded={}",
                        timing.frame_number,
                        timing.frame_interval.as_secs_f64() * 1_000.0,
                        timing.submission_wall_time.as_secs_f64() * 1_000.0,
                        cpu_update_time.as_secs_f64() * 1_000.0,
                        render_stats.cpu_render_time.as_secs_f64() * 1_000.0,
                        render_stats.gpu_frame_time,
                        ticks,
                        self.session.update_clock.backlog().as_secs_f64() * 1_000.0,
                        self.native_focused,
                        self.occluded
                    );
                }
                let done = self.benchmark.as_mut().is_some_and(|benchmark| {
                    benchmark.record(
                        update_delta,
                        cpu_update_time,
                        simulation_time,
                        Some(render_stats),
                    )
                });
                if done {
                    let accounting = crate::benchmark::accounting(&self.session);
                    let result = self
                        .benchmark
                        .as_mut()
                        .expect("active benchmark")
                        .finish(accounting);
                    match result {
                        Err(error) => self.fail(event_loop, error),
                        Ok(true) => event_loop.exit(),
                        Ok(false) => {
                            if self.benchmark.as_ref().is_some_and(|b| b.switch_camera()) {
                                self.session
                                    .character
                                    .apply_mouse_delta(std::f64::consts::PI / 0.0022, 0.0);
                            }
                            window.request_redraw();
                        }
                    }
                    return;
                }
                window.request_redraw();
            }
            RenderOutcome::Retry => self.schedule_retry(event_loop, RENDER_RETRY_DELAY),
            RenderOutcome::Idle => {
                self.frame_clock.reset_interval();
                self.reset_update_clock();
                self.diagnostics.reset_frame_window();
                if surface_size(&window).is_drawable() {
                    self.schedule_retry(event_loop, IDLE_RETRY_DELAY);
                } else {
                    self.retry_at = None;
                    event_loop.set_control_flow(ControlFlow::Wait);
                }
            }
            RenderOutcome::SurfaceLost => {
                if let Some(benchmark) = self.benchmark.as_mut()
                    && let Err(error) = benchmark.observe_native("surface_lost", false)
                {
                    self.fail(event_loop, error);
                    return;
                }
                log::warn!("rendering surface was lost; rebuilding renderer");
                self.renderer = None;
                if let Err(error) = self.initialize_renderer() {
                    self.fail(event_loop, error);
                    return;
                }
                self.schedule_retry(event_loop, RENDER_RETRY_DELAY);
            }
        }
    }
}
