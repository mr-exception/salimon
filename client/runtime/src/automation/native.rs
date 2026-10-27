use super::*;

pub(super) fn execute_native(app: &mut ClientApplication, line: &str) -> Value {
    // Portable parser validates every envelope before any shell side effects.
    if app.e2e_step().is_none() {
        return error(Value::Null, "disabled", "automation requires --e2e");
    }
    let command = match parse_envelope(line) {
        Ok(command) => command,
        Err(response) => return response,
    };
    let id = command["id"].clone();
    let result = match command.get("op").and_then(Value::as_str) {
        Some("scout_reference") => match command.get("view").and_then(Value::as_str) {
            Some("clear") => {
                app.scout_reference = None;
                Ok(json!({"view":"clear"}))
            }
            Some(name) => match crate::scout_reference::find(name) {
                Some(view) => {
                    app.scout_reference = Some(view);
                    Ok(json!({"view":name}))
                }
                None => Err(("invalid_view", "unknown Scout reference camera")),
            },
            None => Err(("invalid_view", "view is required")),
        },
        Some("restart") => Ok(json!({"applied":app.restart_terminal()})),
        Some("window") => {
            let action = command.get("action").and_then(Value::as_str).unwrap_or("");
            let size = command
                .get("width")
                .and_then(Value::as_u64)
                .zip(command.get("height").and_then(Value::as_u64))
                .and_then(|(w, h)| u32::try_from(w).ok().zip(u32::try_from(h).ok()))
                .map(|(w, h)| [w, h]);
            app.automation_window(action, size)
                .map(|()| json!({"requested":true}))
                .map_err(|_| {
                    (
                        "invalid_window",
                        "native window action or dimensions invalid",
                    )
                })
        }
        Some("capture") => match command.get("path").and_then(Value::as_str) {
            Some(path) => app
                .request_capture(path)
                .map(|()| json!({"scheduled":true}))
                .map_err(|_| {
                    (
                        "invalid_capture",
                        "new absolute .png path and capture-capable surface required",
                    )
                }),
            None => Err(("invalid_capture", "path is required")),
        },
        Some("performance_reset") => {
            app.observations.clear();
            Ok(json!({"reset":true}))
        }
        Some("inspect") if command.get("scope").and_then(Value::as_str) == Some("capture") => {
            Ok(json!({"capture_status":app.capture_status}))
        }
        Some("inspect") => {
            let mut result = inspect_session(&app.session);
            result["universe_stream"] = app.universe_stream.as_ref().map_or(Value::Null, |stream| json!({
                "ready": stream.ready(), "active_cells": stream.active().len(),
                "wanted_cells": stream.wanted().len(), "resident_encoded_bytes": stream.resident_encoded_bytes,
                "io": stream.stats, "error": stream.last_error,
            }));
            let upload_times = app.terrain_upload_times();
            let (uploaded_bytes, peak_upload_bytes) = app.terrain_uploads();
            let native = json!({ "native_input_isolated":app.e2e_input_isolated, "terrain_upload_preparation_micros": upload_times,
                "visual_context": app.visual_context(),
                        "survival_details_open":app.survival_hud.details_open,
                "window": app.window_state(),
                "capture_status": app.capture_status,
                "audio_output":app.audio_telemetry(),
            "ship_resources":app.ship_resource_stats().map(|s|json!({"resident_vertex_bytes":s[0],"uniform_bytes":s[1],"triangles":s[2],"draws":s[3]})),
            "fixtures": app.fixture_batch_stats().map(|s| {json!({"layout_version":salimon_renderer::FIXTURE_LAYOUT_VERSION,"draws":s.draw_calls,"visible_instances":s.visible_instances,"triangles":s.triangles,"resident_geometry_bytes":s.resident_geometry_bytes,"instance_capacity_bytes":s.instance_capacity_bytes,"upload_bytes":s.upload_bytes,"prepare_micros":s.prepare_time.as_micros()})}),
                "performance": app.observations.snapshot(),
                "terrain": { "mode": format!("{:?}", app.session.mining.terrain), "active_patches": app.terrain_stream.active.len(),
                    "cache_patches": app.terrain_stream.cached(), "pending_patches": app.terrain_stream.pending,
                    "gpu_ready": app.terrain_gpu_ready(), "presentation_ready":app.terrain_presentation_ready(), "triangles": app.terrain_stream.triangles(), "cpu_cache_bytes": app.terrain_stream.cpu_bytes(),
                    "gpu_cache_bytes": app.terrain_gpu_bytes(), "generated_last_frame": app.terrain_stream.generated,
                    "selection_last_frame_micros": app.terrain_stream.selection_micros,
                    "preparation_last_frame_micros": app.terrain_stream.preparation_micros,
                    "preparation_peak_micros": app.terrain_stream.preparation_peak_micros,
                    "preparation_overruns": app.terrain_stream.preparation_overruns,
                    "generation_last_frame_micros": app.terrain_stream.generation_micros,
                    "generation_peak_micros":app.terrain_stream.generation_peak_micros,
                    "generation_total_micros":app.terrain_stream.generation_total_micros,
                    "generated_peak":app.terrain_stream.generated_peak, "uploaded_last_frame_bytes":uploaded_bytes, "peak_upload_bytes":peak_upload_bytes, "total_generated": app.terrain_stream.total_generated },

                    });
            for (key, value) in native.as_object().expect("native observation object") {
                result[key] = value.clone();
            }
            return json!({"protocol":PROTOCOL,"id":id,"ok":true,"result":result});
        }
        Some("key")
            if matches!(
                command.get("key").and_then(Value::as_str),
                Some(
                    "pause"
                        | "resume"
                        | "diagnostics"
                        | "status"
                        | "status_previous"
                        | "status_next"
                )
            ) =>
        {
            if let Some(pressed) = command.get("pressed").and_then(Value::as_bool) {
                let key = match command["key"].as_str() {
                    Some("pause") => KeyCode::Escape,
                    Some("resume") => KeyCode::Enter,
                    Some("status") => KeyCode::KeyH,
                    Some("status_previous") => KeyCode::BracketLeft,
                    Some("status_next") => KeyCode::BracketRight,
                    _ => KeyCode::F3,
                };
                app.automation_key(PhysicalKey::Code(key), pressed);
                Ok(json!({"applied":true}))
            } else {
                return execute_session(&mut app.session, line);
            }
        }
        _ => return execute_session(&mut app.session, line),
    };
    match result {
        Ok(result) => json!({"protocol":PROTOCOL,"id":id,"ok":true,"result":result}),
        Err((code, message)) => error(id, code, message),
    }
}
