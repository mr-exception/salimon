//! Frame preparation, ordered encoding, submission and asynchronous observations.
use super::{
    CLEAR_COLOR, CUBE_VERTEX_COUNT, EquipmentToolbar, GpuFrameTime, GpuMemoryMetrics, GpuTimer,
    INSTANCE_STRIDE, OverlayImage, OverlayPlacement, RenderOutcome, RenderStats, Renderer,
    RendererError, SceneFrame, capture, encode_f32s, encode_instances, prepare_scene,
    scene_draw_calls,
};

impl Renderer {
    /// Draws a scene, optional diagnostics/action/reticle images and typed equipment HUD.
    ///
    /// `before_present` lets the platform runtime issue its presentation
    /// notification at the exact boundary without introducing a `winit`
    /// dependency into this crate.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        scene: SceneFrame<'_>,
        condition_image: Option<OverlayImage<'_>>,
        overlay_image: Option<OverlayImage<'_>>,
        action_bar_image: Option<OverlayImage<'_>>,
        reticle_image: Option<OverlayImage<'_>>,
        toolbar_state: Option<EquipmentToolbar>,
        scale_factor: f64,
        before_present: impl FnOnce(),
    ) -> Result<RenderOutcome, RendererError> {
        if let Some(gpu_timer) = self.gpu_timer.as_mut() {
            gpu_timer.poll(&self.device);
        }
        if !self.drawable {
            return Ok(RenderOutcome::Idle);
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                drop(frame);
                self.configure_surface();
                return Ok(RenderOutcome::Retry);
            }
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(RenderOutcome::Retry),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(RenderOutcome::Idle),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return Ok(RenderOutcome::Retry);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Ok(RenderOutcome::SurfaceLost),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendererError::new(
                    "failed to acquire surface image",
                    "wgpu reported a validation error",
                ));
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let cpu_render_started_at = std::time::Instant::now();
        let aspect_ratio = self.configuration.width as f32 / self.configuration.height as f32;
        let prepared_scene = prepare_scene(scene, aspect_ratio)
            .map_err(|error| RendererError::new("failed to prepare scene", error))?;
        self.sky.prepare(
            &self.queue,
            scene.camera,
            [self.configuration.width, self.configuration.height],
            scene.sky,
        )?;
        self.atmosphere.prepare(
            &self.device,
            &self.queue,
            scene.camera,
            [self.configuration.width, self.configuration.height],
            scene.atmosphere,
            &self.depth_target.view,
        )?;
        self.terrain.prepare(
            &self.device,
            &self.queue,
            scene,
            prepared_scene.view_projection,
        )?;
        let mut sphere_scene = scene;
        if !self.terrain.ready(scene.terrain) {
            sphere_scene.terrain_mask = None;
        }
        self.spheres.prepare(
            &self.device,
            &self.queue,
            sphere_scene,
            self.configuration.width,
            self.configuration.height,
        )?;

        self.ship_mesh
            .prepare(&self.queue, scene, prepared_scene.view_projection)?;
        self.last_fixture_batch = self.fixtures.prepare(
            &self.device,
            &self.queue,
            scene.camera,
            prepared_scene.view_projection,
            aspect_ratio,
            scene.fixtures,
        )?;
        self.held_item
            .prepare(&self.queue, scene.camera, aspect_ratio, scene.held_item)?;
        let resource_batch = self.resource_meshes.prepare(
            &self.device,
            &self.queue,
            scene.camera,
            prepared_scene.view_projection,
            aspect_ratio,
            scene.resource_meshes,
        )?;
        self.ensure_instance_capacity(prepared_scene.instances.len())?;
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            &encode_f32s(prepared_scene.view_projection),
        );
        if !prepared_scene.instances.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                &encode_instances(&prepared_scene.instances),
            );
        }
        self.condition_overlay
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                condition_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare condition overlay", error))?;
        self.overlay
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                overlay_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare overlay", error))?;
        if let Some(state) = toolbar_state {
            self.toolbar_raster.update(state, scale_factor);
        }
        let toolbar_image = toolbar_state.and_then(|_| self.toolbar_raster.image());
        self.toolbar
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                toolbar_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare equipment toolbar", error))?;
        // Toolbar occupies the lowest HUD band; global/transient text stacks above it.
        let action_bar_image = action_bar_image.map(|mut image| {
            if toolbar_state.is_some() && image.placement == OverlayPlacement::BottomCenter {
                image.placement = OverlayPlacement::BottomCenterInset(
                    self.toolbar_raster
                        .message_inset([self.configuration.width, self.configuration.height]),
                );
            }
            image
        });
        self.action_bar
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                action_bar_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare action bar", error))?;
        self.reticle
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                reticle_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare reticle", error))?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Salimon validation frame encoder"),
            });
        let timing_slot = self.gpu_timer.as_mut().and_then(GpuTimer::acquire_slot);
        {
            let timestamp_writes = timing_slot.and_then(|_| {
                self.gpu_timer.as_ref().map(|timer| {
                    let mut writes = timer.timestamp_writes();
                    writes.end_of_pass_write_index = None;
                    writes
                })
            });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Salimon validation render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_target.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.sky.draw(&mut render_pass);
            if prepared_scene.instance_count > 0 {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.draw(0..CUBE_VERTEX_COUNT, 0..prepared_scene.instance_count);
            }
            self.spheres.draw(&mut render_pass);
            self.terrain.draw(&mut render_pass);
            self.ship_mesh.draw_opaque(&mut render_pass);
            self.fixtures.draw(&mut render_pass);
            self.resource_meshes.draw(&mut render_pass);
            self.ship_mesh.draw_glass(&mut render_pass);
            self.held_item.draw(&mut render_pass);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("atmospheric scattering"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.atmosphere.draw(&mut pass);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Salimon overlay pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                // Preserve the timing interval across scene and overlay passes.
                timestamp_writes: timing_slot.and_then(|_| {
                    self.gpu_timer.as_ref().map(|timer| {
                        let mut writes = timer.timestamp_writes();
                        writes.beginning_of_pass_write_index = None;
                        writes
                    })
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.condition_overlay.draw(&mut render_pass);
            self.overlay.draw(&mut render_pass);
            self.toolbar.draw(&mut render_pass);
            self.action_bar.draw(&mut render_pass);
            self.reticle.draw(&mut render_pass);
        }
        if let (Some(gpu_timer), Some(slot_index)) = (self.gpu_timer.as_ref(), timing_slot) {
            gpu_timer.resolve_and_map(&mut encoder, slot_index);
        }

        let readback = if std::mem::take(&mut self.capture_requested) {
            Some(capture::Readback::encode(
                &self.device,
                &mut encoder,
                &frame.texture,
            ))
        } else {
            None
        };
        self.queue.submit([encoder.finish()]);
        if let Some(readback) = readback {
            self.capture_result = Some(readback.and_then(|r| r.finish(&self.device)));
        }
        let cpu_render_time = cpu_render_started_at.elapsed();
        before_present();
        self.queue.present(frame);
        self.presented_frames = self.presented_frames.saturating_add(1);
        if self.presented_frames == 1 || self.presented_frames.is_multiple_of(60) {
            self.cached_gpu_memory =
                self.device
                    .generate_allocator_report()
                    .map(|report| GpuMemoryMetrics {
                        allocated_bytes: report.total_allocated_bytes,
                        reserved_bytes: report.total_reserved_bytes,
                    });
        }

        let gpu_frame_time = match self.gpu_timer.as_ref() {
            Some(timer) => timer
                .latest()
                .map_or(GpuFrameTime::Pending, GpuFrameTime::Measured),
            None => GpuFrameTime::Unsupported,
        };
        let overlay_draw_calls = u32::from(self.condition_overlay.is_visible())
            + u32::from(self.overlay.is_visible())
            + u32::from(self.action_bar.is_visible())
            + u32::from(self.reticle.is_visible())
            + u32::from(self.toolbar.is_visible());
        let scene_draw_calls = self.atmosphere.draws()
            + self.sky.draws()
            + scene_draw_calls(prepared_scene.instance_count)
            + scene_draw_calls(self.spheres.count())
            + self.terrain.count()
            + self.ship_mesh.draw_count()
            + self.fixtures.draw_count()
            + self.held_item.count()
            + self.resource_meshes.draw_count();
        let object_count = prepared_scene.instance_count
            + self.spheres.count()
            + self.terrain.count()
            + self.ship_mesh.count()
            + self.fixtures.object_count()
            + self.held_item.count()
            + self.resource_meshes.object_count();
        Ok(RenderOutcome::Presented(RenderStats {
            resource_batch,
            cpu_render_time,
            gpu_frame_time,
            visible_objects: object_count,
            rendered_objects: object_count,
            scene_draw_calls,
            total_draw_calls: scene_draw_calls + overlay_draw_calls,
            gpu_memory: self.cached_gpu_memory,
        }))
    }

    fn ensure_instance_capacity(&mut self, instance_count: usize) -> Result<(), RendererError> {
        let required_capacity = instance_count.max(1);
        if required_capacity <= self.instance_capacity {
            return Ok(());
        }

        let new_capacity = required_capacity
            .checked_next_power_of_two()
            .unwrap_or(required_capacity);
        let buffer_size = u64::try_from(new_capacity)
            .ok()
            .and_then(|count| count.checked_mul(INSTANCE_STRIDE))
            .ok_or_else(|| {
                RendererError::new("failed to prepare scene", "instance buffer size overflowed")
            })?;
        if buffer_size > self.device.limits().max_buffer_size {
            return Err(RendererError::new(
                "failed to prepare scene",
                format!(
                    "instance buffer requires {buffer_size} bytes but the device limit is {}",
                    self.device.limits().max_buffer_size
                ),
            ));
        }

        self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon cuboid instances"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = new_capacity;
        Ok(())
    }
}
