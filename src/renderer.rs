#![allow(dead_code, clippy::collapsible_if, clippy::unnecessary_cast, clippy::too_many_arguments)]

use std::{
    f32::consts::PI,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    body::{Body, BodySegment},
    elements::element_by_atomic_number,
    gas::GasBillboard,
    quadtree::{Node, Octree},
    render_mode::RenderMode,
    tables::TABLES,
};

use quarkstrom::{egui, winit::event::VirtualKeyCode, winit_input_helper::WinitInputHelper};

use palette::{rgb::Rgba, Hsluv, IntoColor};
use ultraviolet::{Vec2, Vec3};

use once_cell::sync::Lazy;
use parking_lot::Mutex;

pub static PAUSED: Lazy<AtomicBool> = Lazy::new(|| false.into());
pub static UPDATE_LOCK: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub static SPAWN_ENABLED: Lazy<AtomicBool> = Lazy::new(|| true.into());
pub static COLLISIONS_ENABLED: Lazy<AtomicBool> = Lazy::new(|| true.into());
pub static RESET_REQUESTED: Lazy<AtomicBool> = Lazy::new(|| false.into());
pub static CHEMISTRY_COMPASS_COLORING: Lazy<AtomicBool> = Lazy::new(|| false.into());

#[derive(Clone, Copy, Debug)]
pub struct ManualSpawnRequest {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub element_atomic_number: Option<u8>,
}

pub static BODIES: Lazy<Mutex<Vec<Body>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static QUADTREE: Lazy<Mutex<Vec<Node>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static MANUAL_SPAWNS: Lazy<Mutex<Vec<ManualSpawnRequest>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub struct Renderer {
    camera_target: Vec3,
    camera_distance: f32,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_fov: f32,
    camera_speed: f32,
    camera_rotate_speed: f32,
    viewport_size: Vec2,

    settings_window_open: bool,
    element_sample_open: bool,
    selected_atomic_number: u8,
    left_spawn_anchor: Option<Vec3>,
    right_spawn_anchor: Option<Vec3>,

    show_bodies: bool,
    show_quadtree: bool,
    show_spin_axes: bool,

    // Flow-dynamics render mode (controls overlays)
    render_mode: RenderMode,

    // Selected table id from organigram (for HUD details)
    selected_table_id: Option<u64>,

    depth_range: (usize, usize),

    bodies: Vec<Body>,
    quadtree: Vec<Node>,
}

impl quarkstrom::Renderer for Renderer {
    fn new() -> Self {
        Self {
            camera_target: Vec3::zero(),
            camera_distance: 600.0,
            camera_yaw: PI * 0.25,
            camera_pitch: -0.25,
            camera_fov: PI / 3.0,
            camera_speed: 24.0,
            camera_rotate_speed: 0.0035,
            viewport_size: Vec2::zero(),

            settings_window_open: false,
            element_sample_open: false,
            selected_atomic_number: 1,
            left_spawn_anchor: None,
            right_spawn_anchor: None,

            show_bodies: true,
            show_quadtree: false,
            show_spin_axes: true,
            render_mode: RenderMode::Classic,

            // no table selected initially
            selected_table_id: None,

            depth_range: (0, 0),

            bodies: Vec::new(),
            quadtree: Vec::new(),
        }
    }

    fn input(&mut self, input: &WinitInputHelper, width: u16, height: u16) {
        // Guard against a zero-size viewport (e.g. during window minimisation).
        if width == 0 || height == 0 {
            return;
        }

        self.viewport_size = Vec2::new(width as f32, height as f32);
        self.settings_window_open ^= input.key_pressed(VirtualKeyCode::E);
        self.element_sample_open ^= input.key_pressed(VirtualKeyCode::Grave);
        if self.element_sample_open && input.key_pressed(VirtualKeyCode::Up) {
            self.selected_atomic_number = self.selected_atomic_number.saturating_add(1).min(118);
        }
        if self.element_sample_open && input.key_pressed(VirtualKeyCode::Down) {
            self.selected_atomic_number = self.selected_atomic_number.saturating_sub(1).max(1);
        }

        if input.key_pressed(VirtualKeyCode::Space) {
            let val = PAUSED.load(Ordering::Relaxed);
            PAUSED.store(!val, Ordering::Relaxed)
        }

        if input.key_pressed(VirtualKeyCode::X) {
            let enabled = SPAWN_ENABLED.load(Ordering::Relaxed);
            SPAWN_ENABLED.store(!enabled, Ordering::Relaxed);
        }
        if input.key_pressed(VirtualKeyCode::Y) {
            let enabled = COLLISIONS_ENABLED.load(Ordering::Relaxed);
            COLLISIONS_ENABLED.store(!enabled, Ordering::Relaxed);
        }
        if input.key_pressed(VirtualKeyCode::R) {
            RESET_REQUESTED.store(true, Ordering::Relaxed);
        }

        // Cycle render overlay modes with 'M'
        if input.key_pressed(VirtualKeyCode::M) {
            self.render_mode = match self.render_mode {
                RenderMode::Classic => RenderMode::FlowField,
                RenderMode::FlowField => RenderMode::InstitutionGraph,
                RenderMode::InstitutionGraph => RenderMode::AccountLedger,
                RenderMode::AccountLedger => RenderMode::HistoryStability,
                RenderMode::HistoryStability => RenderMode::DoubleExposure,
                RenderMode::DoubleExposure => RenderMode::Classic,
            }
        }

        let move_delta = self.camera_speed * 0.04;
        let (forward, right, up) = self.camera_basis();
        let mut pan = Vec3::zero();
        if input.key_held(VirtualKeyCode::W) {
            pan += forward;
        }
        if input.key_held(VirtualKeyCode::S) {
            pan -= forward;
        }
        if input.key_held(VirtualKeyCode::A) {
            pan -= right;
        }
        if input.key_held(VirtualKeyCode::D) {
            pan += right;
        }
        if input.key_held(VirtualKeyCode::Q) {
            pan -= up;
        }
        if input.key_held(VirtualKeyCode::E) {
            pan += up;
        }
        if pan != Vec3::zero() {
            self.camera_target += pan.normalized() * move_delta;
        }

        if input.mouse_held(2) {
            let (mdx, mdy) = input.mouse_diff();
            self.camera_yaw -= mdx * self.camera_rotate_speed;
            self.camera_pitch =
                (self.camera_pitch - mdy * self.camera_rotate_speed).clamp(-PI * 0.42, PI * 0.42);
        }

        let scroll = input.scroll_diff();
        if scroll != 0.0 {
            self.camera_distance =
                (self.camera_distance * (-scroll * 0.075).exp()).clamp(80.0, 2_500_000.0);
        }

        if !self.settings_window_open {
            if let Some((mouse_x, mouse_y)) = input.mouse() {
                let world_position = self.screen_to_world(Vec2::new(mouse_x, mouse_y));
                if input.mouse_pressed(0) {
                    self.left_spawn_anchor = Some(world_position);
                }
                if input.mouse_released(0) {
                    if let Some(start) = self.left_spawn_anchor.take() {
                        let drag_distance = (world_position - start).mag();
                        MANUAL_SPAWNS.lock().push(ManualSpawnRequest {
                            position: start,
                            velocity: Vec3::zero(),
                            mass: (1.0 + drag_distance * 0.02).clamp(0.1, 1.0e6),
                            element_atomic_number: None,
                        });
                    }
                }
                if self.element_sample_open && input.mouse_pressed(1) {
                    self.right_spawn_anchor = Some(world_position);
                }
                if self.element_sample_open && input.mouse_released(1) {
                    if let Some(start) = self.right_spawn_anchor.take() {
                        MANUAL_SPAWNS.lock().push(ManualSpawnRequest {
                            position: start,
                            velocity: (start - world_position) * 0.08,
                            mass: 1.0,
                            element_atomic_number: Some(self.selected_atomic_number),
                        });
                    }
                }
            }
        }
    }

    fn render(&mut self, ctx: &mut quarkstrom::RenderContext) {
        {
            let mut lock = UPDATE_LOCK.lock();
            if *lock {
                std::mem::swap(&mut self.bodies, &mut BODIES.lock());
                if self.show_quadtree {
                    std::mem::swap(&mut self.quadtree, &mut QUADTREE.lock());
                }
            }
            *lock = false;
        }

        ctx.clear_circles();
        ctx.clear_lines();
        ctx.clear_rects();
        ctx.set_view_pos(Vec2::zero());
        ctx.set_view_scale(1.0);

        let cam_pos = self.camera_pos();
        let (forward, right, up) = self.camera_basis();
        let tan_half = (self.camera_fov * 0.5).tan();
        let width = self.viewport_size.x.max(1.0);
        let height = self.viewport_size.y.max(1.0);
        let aspect = width / height;

        if !self.bodies.is_empty() {
            if self.show_bodies {
                for body in &self.bodies {
                    if let Some((pos, z)) = self.project_point_basis(
                        body.pos, cam_pos, forward, right, up, tan_half, aspect,
                    ) {
                        let radius = body.projected_radius() / (z * tan_half).max(0.01);
                        if body.segment == BodySegment::Gas {
                            let billboard =
                                GasBillboard::new(body.gas_density, body.gas_temperature);
                            let [red, green, blue] = billboard.color();
                            let scales = [1.8, 1.35, 1.0];
                            for (layer_index, alpha) in billboard.alpha_layers().iter().enumerate()
                            {
                                ctx.draw_circle(
                                    pos,
                                    radius * scales[layer_index],
                                    [red, green, blue, (alpha * 255.0).round() as u8],
                                );
                            }
                        } else {
                            let color = if CHEMISTRY_COMPASS_COLORING.load(Ordering::Relaxed) {
                                body.element_atomic_number
                                    .and_then(element_by_atomic_number)
                                    .map(|element| {
                                        let [red, green, blue] = element.color();
                                        [
                                            (red * 255.0).round() as u8,
                                            (green * 255.0).round() as u8,
                                            (blue * 255.0).round() as u8,
                                            0xff,
                                        ]
                                    })
                                    .unwrap_or([0xff; 4])
                            } else {
                                [0xff; 4]
                            };
                            ctx.draw_circle(pos, radius, color);
                        }
                        if self.show_spin_axes {
                            let axis_len = body.polar_radius / (z * tan_half).max(0.01) * 0.45;
                            let axis_tip = pos + Vec2::new(0.0, -axis_len);
                            ctx.draw_line(pos, axis_tip, [0x80, 0xff, 0xff, 0xff]);
                        }
                    }
                }
            }
        }

        if self.show_quadtree && !self.quadtree.is_empty() {
            let mut depth_range = self.depth_range;
            if depth_range.0 >= depth_range.1 {
                let mut stack = Vec::new();
                stack.push((Octree::ROOT, 0));

                let mut min_depth = usize::MAX;
                let mut max_depth = 0;
                while let Some((node, depth)) = stack.pop() {
                    let node = &self.quadtree[node];

                    if node.is_leaf() {
                        if depth < min_depth {
                            min_depth = depth;
                        }
                        if depth > max_depth {
                            max_depth = depth;
                        }
                    } else {
                        for i in 0..8 {
                            stack.push((node.children + i, depth + 1));
                        }
                    }
                }

                depth_range = (min_depth, max_depth);
            }
            let (min_depth, max_depth) = depth_range;

            let mut stack = Vec::new();
            stack.push((Octree::ROOT, 0));
            while let Some((node, depth)) = stack.pop() {
                let node = &self.quadtree[node];

                if node.is_branch() && depth < max_depth {
                    for i in 0..8 {
                        stack.push((node.children + i, depth + 1));
                    }
                } else if depth >= min_depth {
                    let oct = node.oct;
                    let half = Vec2::new(0.5, 0.5) * oct.size;
                    let min = oct.center.xy() - half;
                    let max = oct.center.xy() + half;

                    let t = ((depth - min_depth + !node.is_empty() as usize) as f32)
                        / (max_depth - min_depth + 1) as f32;

                    let start_h = -100.0;
                    let end_h = 80.0;
                    let h = start_h + (end_h - start_h) * t;
                    let s = 100.0;
                    let l = t * 100.0;

                    let c = Hsluv::new(h, s, l);
                    let rgba: Rgba = c.into_color();
                    let color = rgba.into_format().into();

                    ctx.draw_rect(min, max, color);
                }
            }

            // FlowField overlay: draw momentum vectors at node positions for visualization
            if matches!(
                self.render_mode,
                RenderMode::FlowField | RenderMode::DoubleExposure
            ) {
                // Sample nodes by mass to avoid overdraw at high node counts.
                let mut samples: Vec<(Vec3, f32, Vec3)> = self
                    .quadtree
                    .iter()
                    .filter(|n| n.mass > 0.0)
                    .map(|n| (n.pos, n.mass, n.momentum))
                    .collect();

                samples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let max_vectors = 1024_usize.min(samples.len());
                let vector_scale = 0.05_f32;

                for (pos_world, mass, momentum) in samples.into_iter().take(max_vectors) {
                    if let Some((pos2d, _z)) = self.project_point_basis(
                        pos_world, cam_pos, forward, right, up, tan_half, aspect,
                    ) {
                        let vel = if mass > 0.0 {
                            momentum / mass
                        } else {
                            Vec3::zero()
                        };
                        let tip_world = pos_world + vel * vector_scale;
                        if let Some((tip2d, _)) = self.project_point_basis(
                            tip_world, cam_pos, forward, right, up, tan_half, aspect,
                        ) {
                            ctx.draw_line(pos2d, tip2d, [0x40, 0x80, 0xff, 0xff]);
                        }
                    }
                }

                // Institution spatial overlay: draw emergent tables (if positioned) and parent edges
                let tables = TABLES.lock().clone();
                for t in &tables {
                    if let Some(pos3) = t.position {
                        if let Some((pos2d, _)) = self.project_point(pos3) {
                            // size heuristic: small but visible, scaled by log mass
                            let mass_log = t.accounts.mass.max(1.0).log10();
                            let radius = (mass_log * 2.0 + 4.0).min(12.0);
                            ctx.draw_circle(pos2d, radius, [0xff, 0xd7, 0x00, 0xff]);

                            if let Some(p_id) = t.parent {
                                if let Some(parent_table) = tables.iter().find(|x| x.id == p_id) {
                                    if let Some(parent_pos3) = parent_table.position {
                                        if let Some((p2d, _)) = self.project_point(parent_pos3) {
                                            ctx.draw_line(pos2d, p2d, [0xaa, 0xaa, 0xaa, 0xff]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn gui(&mut self, ctx: &quarkstrom::egui::Context) {
        if self.element_sample_open {
            let selected_atomic_number = &mut self.selected_atomic_number;
            egui::Window::new("Element sample")
                .open(&mut self.element_sample_open)
                .show(ctx, |ui| {
                    ui.add(
                        egui::Slider::new(selected_atomic_number, 1..=118).text("Atomic number"),
                    );
                    if let Some(element) = element_by_atomic_number(*selected_atomic_number) {
                        ui.label(format!("{} ({})", element.name, element.symbol));
                        ui.label(format!("Family: {:?}", element.family));
                        ui.label(format!("Reactivity: {:.2}", element.reactivity_score()));
                    }
                });
        }

        egui::Area::new("spawn_mode")
            .fixed_pos(egui::pos2(12.0, 12.0))
            .show(ctx, |ui| {
                ui.label(format!(
                    "Spawn Mode: {}",
                    if SPAWN_ENABLED.load(Ordering::Relaxed) {
                        "ON"
                    } else {
                        "OFF"
                    }
                ));
                ui.label("Press X to toggle");
            });

        egui::Window::new("")
            .open(&mut self.settings_window_open)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.show_bodies, "Show Bodies");
                ui.checkbox(&mut self.show_spin_axes, "Show Rotation Axis");
                ui.checkbox(&mut self.show_quadtree, "Show Quadtree");
                ui.label(format!(
                    "Particle spawn: {}",
                    if SPAWN_ENABLED.load(Ordering::Relaxed) {
                        "ON"
                    } else {
                        "OFF"
                    }
                ));
                ui.label("Toggle with X");
                ui.label(format!(
                    "Collisions: {}",
                    if COLLISIONS_ENABLED.load(Ordering::Relaxed) {
                        "ON"
                    } else {
                        "OFF"
                    }
                ));
                ui.label("Toggle with Y");
                ui.label("Reset with R");
                if self.show_quadtree {
                    let range = &mut self.depth_range;
                    ui.horizontal(|ui| {
                        ui.label("Depth Range:");
                        ui.add(egui::DragValue::new(&mut range.0).speed(0.05));
                        ui.label("to");
                        ui.add(egui::DragValue::new(&mut range.1).speed(0.05));
                    });
                }
            });

        // Institution graph overlay as an egui window listing emergent tables
        if matches!(
            self.render_mode,
            RenderMode::InstitutionGraph | RenderMode::DoubleExposure
        ) {
            let tables = TABLES.lock().clone();
            egui::Window::new("Institutions").show(ctx, |ui| {
                ui.label(format!("Registered tables: {}", tables.len()));
                let max_show = 200.min(tables.len());
                for t in tables.iter().take(max_show) {
                    ui.horizontal(|ui| {
                        ui.label(format!("Table #{}", t.id));
                        ui.label(format!("mass: {:.3}", t.accounts.mass));
                        ui.label(format!("timescale: {}", t.timescale));
                    });
                }
                if tables.len() > max_show {
                    ui.label("... (truncated)");
                }
            });

            // Organigram view (radial layout) for a sampled subset of emergent tables
            egui::Window::new("Organigram").show(ctx, |ui| {
                ui.label("Organigram (radial sample)");
                let tables_all = TABLES.lock().clone();
                let sample = tables_all.iter().take(200).cloned().collect::<Vec<_>>();
                let n = sample.len().max(1);
                let desired = egui::Vec2::new(360.0, 360.0);
                let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
                let painter = ui.painter();
                let center = rect.center();
                let radius_layout = (rect.width().min(rect.height()) * 0.4) as f32;

                // Precompute node positions in canvas coordinates
                let mut node_positions: Vec<(u64, egui::Pos2)> = Vec::new();
                for (i, t) in sample.iter().enumerate() {
                    let angle = (i as f32) / (n as f32) * (2.0 * std::f32::consts::PI);
                    let x = center.x + angle.cos() * radius_layout;
                    let y = center.y + angle.sin() * radius_layout;
                    let pos = egui::pos2(x, y);
                    node_positions.push((t.id, pos));
                }

                // Draw edges first
                for (i, t) in sample.iter().enumerate() {
                    let (_id, pos) = node_positions[i];
                    if let Some(p_id) = t.parent {
                        if let Some((idx, _parent)) =
                            sample.iter().enumerate().find(|(_, pt)| pt.id == p_id)
                        {
                            let ppos = node_positions[idx].1;
                            painter.line_segment(
                                [pos, ppos],
                                egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                            );
                        }
                    }
                }

                // Draw nodes
                for (i, t) in sample.iter().enumerate() {
                    let (_id, pos) = node_positions[i];
                    let mass_log = t.accounts.mass.max(1.0).log10();
                    let node_r = (mass_log * 2.0 + 6.0).clamp(4.0, 16.0);
                    painter.circle_filled(pos, node_r, egui::Color32::from_rgb(180, 200, 255));
                }

                // Interaction: clicking within the organigram selects the nearest node
                if response.clicked() {
                    if let Some(click_pos) = response.hover_pos() {
                        let mut best: Option<(u64, f32)> = None;
                        for (i, (id, pos)) in node_positions.iter().enumerate() {
                            let dx = pos.x - click_pos.x;
                            let dy = pos.y - click_pos.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            let t = &sample[i];
                            let mass_log = t.accounts.mass.max(1.0).log10();
                            let node_r = (mass_log * 2.0 + 6.0).clamp(4.0, 16.0);
                            if dist <= node_r * 1.2 {
                                if best.is_none() || dist < best.unwrap().1 {
                                    best = Some((*id, dist));
                                }
                            }
                        }
                        if let Some((sel_id, _)) = best {
                            self.selected_table_id = Some(sel_id);
                        }
                    }
                }
            });
        }

        // Account ledger HUD
        if matches!(
            self.render_mode,
            RenderMode::AccountLedger | RenderMode::DoubleExposure
        ) {
            let tables = TABLES.lock().clone();
            egui::Window::new("Account Ledger")
                .anchor(egui::Align2::LEFT_TOP, [8.0, 8.0])
                .show(ctx, |ui| {
                    ui.label("Top institutions by mass:");
                    let mut by_mass = tables.clone();
                    by_mass.sort_by(|a, b| {
                        b.accounts
                            .mass
                            .partial_cmp(&a.accounts.mass)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    for t in by_mass.iter().take(8) {
                        ui.horizontal(|ui| {
                            ui.label(format!("#{}", t.id));
                            ui.add(
                                egui::ProgressBar::new((t.accounts.mass / 1000.0).min(1.0))
                                    .text(format!("{:.2}", t.accounts.mass)),
                            );
                            ui.label(format!("{:.2}", t.accounts.mass));
                        });
                    }

                    // If the user selected a table in the organigram, show detailed breakdown
                    if let Some(sel_id) = self.selected_table_id {
                        if let Some(t) = tables.iter().find(|x| x.id == sel_id) {
                            ui.separator();
                            ui.label(format!("Selected Table #{} details:", sel_id));
                            ui.label(format!("mass: {:.6}", t.accounts.mass));
                            ui.label(format!("energy: {:.6}", t.accounts.energy));
                            ui.label(format!(
                                "momentum: x:{:.3} y:{:.3} z:{:.3}",
                                t.accounts.momentum.x, t.accounts.momentum.y, t.accounts.momentum.z
                            ));
                            ui.label(format!(
                                "angular_momentum: x:{:.3} y:{:.3} z:{:.3}",
                                t.accounts.angular_momentum.x,
                                t.accounts.angular_momentum.y,
                                t.accounts.angular_momentum.z
                            ));
                            ui.label(format!("timescale: {}", t.timescale));
                            if let Some(pos) = t.position {
                                ui.label(format!(
                                    "position: x:{:.2} y:{:.2} z:{:.2}",
                                    pos.x, pos.y, pos.z
                                ));
                            }
                        }
                    }
                });
        }
    }
}

impl Renderer {
    fn camera_pos(&self) -> Vec3 {
        let cos_pitch = self.camera_pitch.cos();
        let x = self.camera_distance * cos_pitch * self.camera_yaw.cos();
        let y = self.camera_distance * self.camera_pitch.sin();
        let z = self.camera_distance * cos_pitch * self.camera_yaw.sin();
        self.camera_target + Vec3::new(x, y, z)
    }

    fn camera_basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.camera_target - self.camera_pos()).normalized();
        let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
        let up = right.cross(forward).normalized();
        (forward, right, up)
    }

    fn screen_to_ndc(&self, mouse: Vec2) -> Vec2 {
        let width = self.viewport_size.x.max(1.0);
        let height = self.viewport_size.y.max(1.0);
        Vec2::new(mouse.x / width * 2.0 - 1.0, 1.0 - mouse.y / height * 2.0)
    }

    fn project_point(&self, point: Vec3) -> Option<(Vec2, f32)> {
        let cam_pos = self.camera_pos();
        let (forward, right, up) = self.camera_basis();
        self.project_point_basis(
            point,
            cam_pos,
            forward,
            right,
            up,
            (self.camera_fov * 0.5).tan(),
            self.viewport_size.x.max(1.0) / self.viewport_size.y.max(1.0),
        )
    }

    fn project_point_basis(
        &self,
        point: Vec3,
        cam_pos: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
        tan_half: f32,
        aspect: f32,
    ) -> Option<(Vec2, f32)> {
        let rel = point - cam_pos;
        let z = rel.dot(forward);
        if z <= 0.1 {
            return None;
        }
        let x = rel.dot(right);
        let y = rel.dot(up);
        let proj_x = x / (z * tan_half);
        let proj_y = y / (z * tan_half);
        let pos = Vec2::new(proj_x * aspect, proj_y);
        Some((pos, z))
    }

    fn screen_to_world(&self, mouse: Vec2) -> Vec3 {
        let ndc = self.screen_to_ndc(mouse);
        let (forward, right, up) = self.camera_basis();
        let tan_half = (self.camera_fov * 0.5).tan();
        let aspect = self.viewport_size.x / self.viewport_size.y;
        let dir =
            (right * (ndc.x * aspect * tan_half) + up * (ndc.y * tan_half) + forward).normalized();
        let origin = self.camera_pos();
        let plane_z = self.camera_target.z;
        let t = (plane_z - origin.z) / dir.z;
        if t <= 0.0 {
            origin + dir * self.camera_distance * 0.25
        } else {
            origin + dir * t
        }
    }
}
