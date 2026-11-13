//! Read-only scientific instrument; graphics never define physical state.
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    text::FontSize,
    window::PresentMode,
};
use qr_neural_memory_system::{
    lattice::Lattice,
    observation::{Recording, Snapshot},
    topology::POPULATION,
};
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
    time::Duration,
};

#[derive(Clone, Default)]
struct Filters {
    axis: Option<usize>,
    plane: usize,
    region: u8,
    nodes: bool,
    edges: bool,
    activity: bool,
    palette: bool,
    logarithmic: bool,
}
impl Filters {
    fn visible(&self, s: &Snapshot, id: usize) -> bool {
        if !self.nodes {
            return false;
        }
        let p = s.neurons[id].position;
        let d = s.case.topology.dimensions();
        if self.axis.is_some_and(|a| p[a] != self.plane) {
            return false;
        }
        let shell = (0..3).any(|a| d[a] > 1 && (p[a] == 0 || p[a] + 1 == d[a]));
        match self.region {
            1 => shell,
            2 => !shell,
            _ => true,
        }
    }
}
#[derive(Resource)]
struct Session {
    recording: Recording,
    shown: Snapshot,
    index: usize,
    live: Option<Lattice>,
    running: bool,
    selected: usize,
    entry: String,
    filters: Filters,
    history: VecDeque<Snapshot>,
    dirty: bool,
}
impl Session {
    fn new(recording: Recording, live: bool) -> io::Result<Self> {
        let first = recording.frames[0].clone();
        let engine = if live {
            if first.synthetic {
                return Err(io::Error::other("synthetic fixtures cannot enter LIVE"));
            }
            Some(Lattice::new(first.case)?)
        } else {
            None
        };
        let shown = engine
            .as_ref()
            .map_or_else(|| first.clone(), Lattice::observe);
        let selected = shown.case.source;
        Ok(Self {
            recording,
            shown,
            index: 0,
            live: engine,
            running: false,
            selected,
            entry: String::new(),
            filters: Filters {
                nodes: true,
                activity: true,
                logarithmic: true,
                ..default()
            },
            history: VecDeque::new(),
            dirty: true,
        })
    }
    fn label(&self) -> &'static str {
        if self.shown.synthetic {
            "SYNTHETIC TEST FIXTURE"
        } else if self.live.is_some() {
            "LIVE"
        } else {
            "REPLAY"
        }
    }
    fn select(&mut self, id: usize) {
        if id < POPULATION {
            self.selected = id;
            self.dirty = true;
        }
    }
    fn seek(&mut self, index: usize) {
        self.live = None;
        self.running = false;
        self.index = index.min(self.recording.frames.len() - 1);
        self.shown = self.recording.frames[self.index].clone();
        self.dirty = true;
    }
    fn step(&mut self) {
        if let Some(engine) = self.live.as_mut() {
            if !engine.advance() {
                self.running = false;
                return;
            }
            self.shown = engine.observe();
            if !self.shown.diagnostic().valid() {
                self.running = false;
            }
            if self.shown.tick.is_multiple_of(8) {
                self.history.push_back(self.shown.clone());
                if self.history.len() > 21 {
                    self.history.pop_front();
                }
            }
            if let Some(i) = self.recording.seek(self.shown.tick) {
                self.index = i;
            }
        } else if self.index + 1 < self.recording.frames.len() {
            self.index += 1;
            self.shown = self.recording.frames[self.index].clone();
        } else {
            self.running = false;
        }
        self.dirty = true;
    }
    fn toggle_live(&mut self) {
        self.running = false;
        if self.live.is_some() {
            let i = self.index;
            self.seek(i);
        } else if !self.shown.synthetic {
            match Lattice::new(self.shown.case) {
                Ok(m) => {
                    self.shown = m.observe();
                    self.live = Some(m);
                    self.history.clear();
                    self.index = 0;
                    self.dirty = true;
                }
                Err(e) => error!("LIVE initialization rejected: {e}"),
            }
        }
    }
}
#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    radius: f32,
    target: Vec3,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.7,
            pitch: 0.45,
            radius: 35.0,
            target: Vec3::ZERO,
        }
    }
}
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct NodeId(usize);
#[derive(Component)]
struct PaletteCell(usize);
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct InfoText;
#[derive(Component)]
struct SeriesText;
#[derive(Resource)]
struct Materials {
    activity: Vec<Handle<StandardMaterial>>,
    palette: Vec<Handle<StandardMaterial>>,
    neutral: Handle<StandardMaterial>,
    invalid: Handle<StandardMaterial>,
}
#[derive(Resource)]
struct FrameCap(u32);
#[derive(Resource)]
struct Witness {
    folder: Option<PathBuf>,
    writer: Option<BufWriter<File>>,
    frame: u64,
    target: usize,
    sample: usize,
    pending: bool,
    audited: bool,
    hold: usize,
    complete_wait: usize,
    live_test: bool,
    reseek: usize,
}
impl Witness {
    fn new(folder: Option<PathBuf>, live: bool) -> io::Result<Self> {
        let writer = if let Some(p) = &folder {
            fs::create_dir(p)?;
            let mut w = BufWriter::new(File::create(p.join("audit.csv"))?);
            writeln!(
                w,
                "tick,id,mode,fnv1a,palette,u27,v27,visible_nodes,axis,plane,region,nodes,edges,activity,palette_view"
            )?;
            Some(w)
        } else {
            None
        };
        Ok(Self {
            folder,
            writer,
            frame: 0,
            target: 0,
            sample: 0,
            pending: false,
            audited: false,
            hold: 0,
            complete_wait: 0,
            live_test: live,
            reseek: 0,
        })
    }
}
fn palette(v: u8) -> Color {
    match v {
        1 => Color::srgb(0.90, 0.19, 0.17),
        2 => Color::srgb(0.14, 0.80, 0.37),
        3 => Color::srgb(0.20, 0.42, 0.96),
        4 => Color::srgb(0.97, 0.80, 0.17),
        _ => Color::srgb(0.035, 0.05, 0.075),
    }
}
fn position(s: &Snapshot, id: usize) -> Vec3 {
    let p = s.neurons[id].position;
    let d = s.case.topology.dimensions();
    Vec3::new(
        p[0] as f32 - (d[0] - 1) as f32 / 2.0,
        p[1] as f32 - (d[1] - 1) as f32 / 2.0,
        p[2] as f32 - (d[2] - 1) as f32 / 2.0,
    )
}
fn activity_value(s: &Snapshot, id: usize) -> Option<f64> {
    let n = &s.neurons[id];
    if n.nonfinite > 0
        || n.peak
            .iter()
            .chain(n.module27.iter())
            .chain(n.velocity27.iter())
            .any(|x| !x.is_finite())
        || !n.squared.is_finite()
    {
        None
    } else {
        Some(n.peak.iter().copied().fold(0.0_f64, f64::max))
    }
}

fn setup(
    mut commands: Commands,
    s: Res<Session>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut assets: ResMut<Assets<StandardMaterial>>,
) {
    let activity = (0..256)
        .map(|i| {
            let t = i as f32 / 255.0;
            assets.add(StandardMaterial {
                base_color: Color::srgb(0.06 + 0.91 * t, 0.13 + 0.63 * t, 0.22 + 0.35 * (1.0 - t)),
                unlit: true,
                ..default()
            })
        })
        .collect::<Vec<_>>();
    let palettes = (0..5)
        .map(|i| {
            assets.add(StandardMaterial {
                base_color: palette(i),
                unlit: true,
                ..default()
            })
        })
        .collect();
    let neutral = assets.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.34, 0.44),
        unlit: true,
        ..default()
    });
    let invalid = assets.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 1.0),
        unlit: true,
        ..default()
    });
    let cube = meshes.add(Cuboid::new(0.19, 0.19, 0.19));
    for id in 0..POPULATION {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(activity[0].clone()),
            Transform::from_translation(position(&s.shown, id)),
            NodeId(id),
        ));
    }
    commands.insert_resource(Materials {
        activity,
        palette: palettes,
        neutral,
        invalid,
    });
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(23.0, 18.0, 26.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));
    commands.spawn((
        Text::new("QR / VOLUMETRIC OBSERVATORY"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.55, 0.87, 0.94)),
        Node {
            position_type: PositionType::Absolute,
            left: px(22),
            top: px(18),
            ..default()
        },
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgb(0.76, 0.84, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            top: px(54),
            max_width: px(950),
            ..default()
        },
        StatusText,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(110),
                width: px(330),
                padding: UiRect::all(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.04, 0.065, 0.97)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("NEURON INSPECTOR"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.87, 0.94)),
            ));
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                InfoText,
            ));
            p.spawn((
                Text::new("ACTUAL PERSISTENT MODULES / 8 x 8"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
            ));
            p.spawn(Node {
                width: px(160),
                height: px(160),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|grid| {
                for i in 0..64 {
                    grid.spawn((
                        Node {
                            width: px(18),
                            height: px(18),
                            margin: UiRect::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(palette(0)),
                        PaletteCell(i),
                    ));
                }
            });
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                SeriesText,
            ));
        });
    commands.spawn((Text::new("Right drag: orbit   Middle drag: pan   Wheel: zoom   R: camera reset\nClick node or type ID + Enter; Up/Down: select ID   Space: pause/play   .: step\nLeft/Right: seek recording   Home/End: first/last   L: LIVE restart / REPLAY\nX/Y/Z: slice axis toggle   [ / ]: slice plane   F: all/shell/interior\nN: nodes   E: selected edges   A: activity   P: palette view   G: linear/log scale\nAxes: X red / Y green / Z blue. No resonance or learning is inferred from color."),TextFont{font_size:FontSize::Px(13.0),..default()},TextColor(Color::srgb(0.59,0.69,0.78)),Node{position_type:PositionType::Absolute,left:px(22),bottom:px(16),..default()}));
}

fn record_adapter(info: Res<bevy::render::renderer::RenderAdapterInfo>, witness: Res<Witness>) {
    let text = format!(
        "name={}\nbackend={:?}\ndevice_type={:?}\ndriver={}\ndriver_info={}\nwindow_backend=X11\n",
        info.name, info.backend, info.device_type, info.driver, info.driver_info
    );
    eprintln!("Renderer adapter:\n{text}");
    if let Some(folder) = &witness.folder {
        fs::write(folder.join("adapter.txt"), text).expect("adapter receipt");
    }
}
#[derive(bevy::ecs::system::SystemParam)]
struct MouseState<'w> {
    buttons: Res<'w, ButtonInput<MouseButton>>,
    motion: Res<'w, AccumulatedMouseMotion>,
    scroll: Res<'w, AccumulatedMouseScroll>,
}

fn input_commands(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: MouseState,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut s: ResMut<Session>,
    mut orbit: ResMut<Orbit>,
) {
    if mouse.buttons.pressed(MouseButton::Right) {
        orbit.yaw -= mouse.motion.delta.x * 0.006;
        orbit.pitch = (orbit.pitch + mouse.motion.delta.y * 0.006).clamp(-1.4, 1.4);
    }
    if mouse.buttons.pressed(MouseButton::Middle) {
        let yaw = orbit.yaw;
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        orbit.target +=
            right * (-mouse.motion.delta.x * 0.025) + Vec3::Y * (mouse.motion.delta.y * 0.025);
    }
    orbit.radius = (orbit.radius * (-mouse.scroll.delta.y * 0.08).exp()).clamp(8.0, 150.0);
    if keys.just_pressed(KeyCode::KeyR) {
        *orbit = Orbit::default();
    }
    if keys.just_pressed(KeyCode::Space) {
        s.running = !s.running;
        s.dirty = true;
    }
    if keys.just_pressed(KeyCode::Period) {
        s.running = false;
        s.step();
    }
    if keys.just_pressed(KeyCode::KeyL) {
        s.toggle_live();
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        let i = s.index.saturating_sub(1);
        s.seek(i);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        let i = s.index + 1;
        s.seek(i);
    }
    if keys.just_pressed(KeyCode::Home) {
        s.seek(0);
    }
    if keys.just_pressed(KeyCode::End) {
        let i = s.recording.frames.len() - 1;
        s.seek(i);
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        let id = (s.selected + 1) % POPULATION;
        s.select(id);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        let id = (s.selected + POPULATION - 1) % POPULATION;
        s.select(id);
    }
    for (key, digit) in [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ] {
        if keys.just_pressed(key) && s.entry.len() < 5 {
            s.entry.push(digit);
            s.dirty = true;
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        s.entry.pop();
        s.dirty = true;
    }
    if keys.just_pressed(KeyCode::Enter) {
        if let Ok(id) = s.entry.parse() {
            s.select(id);
        }
        s.entry.clear();
        s.dirty = true;
    }
    for (key, axis) in [(KeyCode::KeyX, 0), (KeyCode::KeyY, 1), (KeyCode::KeyZ, 2)] {
        if keys.just_pressed(key) {
            s.filters.axis = if s.filters.axis == Some(axis) {
                None
            } else {
                Some(axis)
            };
            s.filters.plane = 0;
            s.dirty = true;
        }
    }
    let limit = s
        .filters
        .axis
        .map_or(16, |a| s.shown.case.topology.dimensions()[a]);
    if keys.just_pressed(KeyCode::BracketRight) {
        s.filters.plane = (s.filters.plane + 1) % limit;
        s.dirty = true;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        s.filters.plane = (s.filters.plane + limit - 1) % limit;
        s.dirty = true;
    }
    if keys.just_pressed(KeyCode::KeyF) {
        s.filters.region = (s.filters.region + 1) % 3;
        s.dirty = true;
    }
    for key in [
        KeyCode::KeyN,
        KeyCode::KeyE,
        KeyCode::KeyA,
        KeyCode::KeyP,
        KeyCode::KeyG,
    ] {
        if keys.just_pressed(key) {
            match key {
                KeyCode::KeyN => s.filters.nodes = !s.filters.nodes,
                KeyCode::KeyE => s.filters.edges = !s.filters.edges,
                KeyCode::KeyA => s.filters.activity = !s.filters.activity,
                KeyCode::KeyP => s.filters.palette = !s.filters.palette,
                KeyCode::KeyG => s.filters.logarithmic = !s.filters.logarithmic,
                _ => (),
            };
            s.dirty = true;
        }
    }
    if mouse.buttons.just_pressed(MouseButton::Left)
        && let Some(cursor) = window.cursor_position()
        && cursor.x < window.width() - 350.0
        && cursor.y > 90.0
        && cursor.y < window.height() - 125.0
        && let Ok(ray) = camera.0.viewport_to_world(camera.1, cursor)
    {
        let direction = *ray.direction;
        let mut best = None;
        for id in 0..POPULATION {
            if s.filters.visible(&s.shown, id) {
                let delta = position(&s.shown, id) - ray.origin;
                let t = delta.dot(direction);
                if t > 0.0
                    && (delta - direction * t).length() < 0.28
                    && best.is_none_or(|(_, distance)| t < distance)
                {
                    best = Some((id, t));
                }
            }
        }
        if let Some((id, _)) = best {
            s.select(id);
        }
    }
}
fn fixed_tick(mut s: ResMut<Session>) {
    if s.running {
        s.step();
    }
}
fn camera_update(orbit: Res<Orbit>, mut camera: Single<&mut Transform, With<MainCamera>>) {
    let offset = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.cos() * orbit.pitch.cos(),
    ) * orbit.radius;
    **camera = Transform::from_translation(orbit.target + offset).looking_at(orbit.target, Vec3::Y);
}
type StatusFilter = (With<StatusText>, Without<InfoText>, Without<SeriesText>);
type InfoFilter = (With<InfoText>, Without<StatusText>, Without<SeriesText>);
type SeriesFilter = (With<SeriesText>, Without<InfoText>, Without<StatusText>);
fn refresh_scene(
    mut s: ResMut<Session>,
    materials: Res<Materials>,
    mut nodes: Query<(
        &NodeId,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
    mut cells: Query<(&PaletteCell, &mut BackgroundColor)>,
    mut status: Single<&mut Text, StatusFilter>,
    mut info: Single<&mut Text, InfoFilter>,
    mut series: Single<&mut Text, SeriesFilter>,
) {
    if !s.dirty {
        return;
    }
    for (id, mut material, mut visibility) in &mut nodes {
        *visibility = if s.filters.visible(&s.shown, id.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        material.0 = if activity_value(&s.shown, id.0).is_none() {
            materials.invalid.clone()
        } else if s.filters.palette {
            let mut counts = [0; 5];
            for value in s.shown.neurons[id.0].palette {
                counts[usize::from(value)] += 1;
            }
            let best = (0..5).max_by_key(|&i| (counts[i], 5 - i)).unwrap();
            materials.palette[best].clone()
        } else if s.filters.activity {
            let value = activity_value(&s.shown, id.0).unwrap_or(0.0);
            let scaled = if s.filters.logarithmic {
                if value <= 0.0 {
                    0.0
                } else {
                    ((value.log10() + 8.0) / 8.0).clamp(0.0, 1.0)
                }
            } else {
                value.clamp(0.0, 1.0)
            };
            materials.activity[(scaled * 255.0).round() as usize].clone()
        } else {
            materials.neutral.clone()
        };
    }
    let n = &s.shown.neurons[s.selected];
    for (cell, mut color) in &mut cells {
        color.0 = palette(n.palette[cell.0]);
    }
    let legend = if s.filters.palette {
        "PERSISTENT PALETTE / glyph = dominant module color"
    } else if !s.filters.activity {
        "ACTIVITY OVERLAY OFF"
    } else if s.filters.logarithmic {
        "ACTIVITY max-module/channel |u| / FIXED log10 1e-8..1; below floor dark"
    } else {
        "ACTIVITY max-module/channel |u| / FIXED linear 0..1"
    };
    let d = s.shown.diagnostic();
    status.0 = format!(
        "EXPERIMENT 0003 / {} / {} / {}\nTick {:03} / t={:.1} / 4,096 addressable nodes / nonfinite={}\n{}\nSlice {:?}={} / region {} / ID entry: {}",
        s.label(),
        s.shown.case.topology.name(),
        if s.running { "RUNNING" } else { "PAUSED" },
        s.shown.tick,
        s.shown.tick as f64 * 0.1,
        d.nonfinite,
        legend,
        s.filters.axis,
        s.filters.plane,
        s.filters.region,
        s.entry
    );
    info.0 = format!(
        "ID {} / ({},{},{}) / degree {}\nSeed {} / {}\nmax |u| = {:.6e}\nmax |v| = {:.6e}\nu27 = [{:.6e}, {:.6e},\n       {:.6e}, {:.6e}]\nv27 = [{:.6e}, {:.6e},\n       {:.6e}, {:.6e}]\nNONFINITE {}",
        n.id,
        n.position[0],
        n.position[1],
        n.position[2],
        s.shown.case.topology.neighbors(n.id).len(),
        s.shown.case.seed,
        s.shown.case.configuration.name(),
        n.peak.iter().copied().fold(0.0, f64::max),
        n.velocity_peak.iter().copied().fold(0.0, f64::max),
        n.module27[0],
        n.module27[1],
        n.module27[2],
        n.module27[3],
        n.velocity27[0],
        n.velocity27[1],
        n.velocity27[2],
        n.velocity27[3],
        n.nonfinite
    );
    let mut text = String::from("MEASURED HISTORY / max |u|\n");
    let history: Vec<_> = if s.live.is_some() {
        s.history.iter().collect()
    } else {
        s.recording.frames[..=s.index].iter().collect()
    };
    for frame in history.iter().rev().take(6).rev() {
        let n = &frame.neurons[s.selected];
        text.push_str(&format!(
            "t{:03}  {:.5e}\n",
            frame.tick,
            n.peak.iter().copied().fold(0.0, f64::max)
        ));
    }
    series.0 = text;
    s.dirty = false;
}
fn overlays(s: Res<Session>, mut gizmos: Gizmos) {
    let start = Vec3::splat(-8.5);
    gizmos.arrow(start, start + Vec3::X * 5.0, Color::srgb(0.95, 0.2, 0.2));
    gizmos.arrow(start, start + Vec3::Y * 5.0, Color::srgb(0.2, 0.9, 0.4));
    gizmos.arrow(start, start + Vec3::Z * 5.0, Color::srgb(0.3, 0.5, 1.0));
    let selected = position(&s.shown, s.selected);
    gizmos.cube(
        Transform::from_translation(selected).with_scale(Vec3::splat(0.45)),
        Color::srgb(1.0, 0.95, 0.45),
    );
    if s.filters.edges {
        for j in s.shown.case.topology.neighbors(s.selected) {
            gizmos.line(
                selected,
                position(&s.shown, j),
                Color::srgba(0.4, 0.9, 0.9, 0.7),
            );
        }
    }
}
fn witness_commands(
    mut commands: Commands,
    mut w: ResMut<Witness>,
    mut s: ResMut<Session>,
    mut orbit: ResMut<Orbit>,
) {
    if w.folder.is_none() {
        return;
    }
    w.frame += 1;
    if w.frame < 30 {
        return;
    }
    if w.hold > 0 {
        w.hold -= 1;
        return;
    }
    if w.audited {
        w.audited = false;
        w.sample += 1;
        if w.sample >= 3 {
            w.sample = 0;
            w.target += 1;
        }
    }
    let total = s.recording.frames.len();
    if w.target >= total {
        let seeks = [total - 1, 0, total / 2, 1];
        if w.reseek < seeks.len() {
            s.seek(seeks[w.reseek]);
            s.select(0);
            w.pending = true;
            w.reseek += 1;
            w.hold = 1;
            return;
        }
        w.complete_wait += 1;
        if w.complete_wait > 20 {
            if let Some(writer) = w.writer.as_mut() {
                writer.flush().expect("witness flush");
            }
            fs::write(w.folder.as_ref().unwrap().join("COMPLETE"),"Real Bevy session completed UI binding and tick/hash audits; compare with headless evidence.\n").expect("witness completion");
            commands.write_message(AppExit::Success);
        }
        return;
    }
    let index = w.target;
    let target = s.recording.frames[index].tick;
    if index + 1 == total && s.shown.tick < target {
        s.running = true;
        s.dirty = true;
        return;
    }
    s.running = false;
    if w.live_test && s.live.is_some() {
        if s.shown.tick < target {
            s.running = false;
            s.step();
            return;
        }
    } else {
        s.seek(index);
    }
    let ids = [0, s.shown.case.source, 4095];
    s.select(ids[w.sample]);
    s.filters.axis = if index % 5 == 1 {
        Some(index % 3)
    } else {
        None
    };
    s.filters.plane = 8;
    s.filters.region = if index % 5 == 2 {
        1
    } else if index % 5 == 3 {
        2
    } else {
        0
    };
    s.filters.edges = index.is_multiple_of(2);
    s.filters.palette = index % 4 == 3;
    s.filters.nodes = index % 7 != 4;
    s.filters.activity = index % 6 != 3;
    s.dirty = true;
    orbit.yaw += 0.003;
    orbit.radius = if index.is_multiple_of(2) { 35.0 } else { 38.0 };
    orbit.target.x = if index.is_multiple_of(3) { 0.5 } else { 0.0 };
    w.pending = true;
    if w.sample == 1 && [0, total / 2, total - 1].contains(&index) {
        let path = w
            .folder
            .as_ref()
            .unwrap()
            .join(format!("tick-{target:03}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        w.hold = 8;
    }
}
fn witness_audit(
    mut w: ResMut<Witness>,
    s: Res<Session>,
    cells: Query<(&PaletteCell, &BackgroundColor)>,
    info: Single<&Text, With<InfoText>>,
    nodes: Query<(&NodeId, &Visibility)>,
) {
    if !w.pending {
        return;
    }
    w.pending = false;
    let snapshot = &s.shown;
    let id = s.selected;
    let n = &snapshot.neurons[id];
    let mut count = 0;
    for (cell, color) in &cells {
        assert_eq!(
            color.0,
            palette(n.palette[cell.0]),
            "displayed module mismatch"
        );
        count += 1;
    }
    assert_eq!(count, 64);
    assert_eq!(nodes.iter().count(), 4096);
    assert!(info.0.contains(&format!("ID {id} /")));
    assert!(
        info.0.contains(&format!("{:.6e}", n.module27[0])),
        "displayed number mismatch"
    );
    let bytes = n
        .palette
        .iter()
        .map(|v| format!("{v:x}"))
        .collect::<String>();
    let visible = nodes
        .iter()
        .filter(|(_, v)| **v != Visibility::Hidden)
        .count();
    writeln!(
        w.writer.as_mut().unwrap(),
        "{},{},{},{:016x},{},{:.17e},{:.17e},{},{:?},{},{},{},{},{},{}",
        snapshot.tick,
        id,
        s.label(),
        snapshot.fingerprint().unwrap(),
        bytes,
        n.module27[0],
        n.velocity27[0],
        visible,
        s.filters.axis,
        s.filters.plane,
        s.filters.region,
        s.filters.nodes,
        s.filters.edges,
        s.filters.activity,
        s.filters.palette
    )
    .expect("witness write");
    if w.target < s.recording.frames.len() {
        w.audited = true;
    }
}
fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}
fn limit_fps(cap: Res<FrameCap>) {
    if cap.0 > 0 {
        std::thread::sleep(Duration::from_secs_f64(1.0 / f64::from(cap.0)));
    }
}
fn main() -> io::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut path = PathBuf::from("results/experiment_0003-a/case-0000");
    let mut live = false;
    let mut witness = None;
    let mut fps = 60;
    let mut camera_b = false;
    let mut synthetic = false;
    let mut adapter = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--recording" => {
                path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing path"))?,
                )
            }
            "--live" => live = true,
            "--witness" => {
                witness = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing witness path"))?,
                ))
            }
            "--fps" => {
                fps = args
                    .next()
                    .ok_or_else(|| io::Error::other("missing fps"))?
                    .parse()
                    .map_err(io::Error::other)?
            }
            "--camera-b" => camera_b = true,
            "--synthetic-label-test" => synthetic = true,
            "--adapter" => {
                adapter = Some(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing adapter substring"))?,
                )
            }
            _ => return Err(io::Error::other(format!("unknown argument {a}"))),
        }
    }
    let mut recording = Recording::load(&path).map_err(|e| {
        io::Error::other(format!(
            "Recording rejected before engine/window startup: {e}"
        ))
    })?;
    if synthetic {
        for f in &mut recording.frames {
            f.synthetic = true;
            f.neurons[0].nonfinite = 1;
        }
    }
    let s = Session::new(recording, live)?;
    eprintln!(
        "Observatory opened {}: 4096 nodes, engine_instances={}",
        s.label(),
        usize::from(live)
    );
    let w = Witness::new(witness, live)?;
    let orbit = if camera_b {
        Orbit {
            yaw: -1.4,
            pitch: 0.8,
            ..default()
        }
    } else {
        Orbit::default()
    };
    let mut render_settings = bevy::render::settings::WgpuSettings::default();
    if std::env::var_os("WGPU_POWER_PREF").is_none() {
        render_settings.power_preference = bevy::render::settings::PowerPreference::LowPower;
    }
    render_settings.adapter_name = adapter;
    App::new()
        .insert_resource(s)
        .insert_resource(w)
        .insert_resource(orbit)
        .insert_resource(FrameCap(fps))
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .insert_resource(ClearColor(Color::srgb(0.018, 0.025, 0.04)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "QR Neural Network | Experiment 0003 Observatory".into(),
                        resolution: (1380, 940).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: render_settings.into(),
                    ..default()
                }),
        )
        .add_systems(Startup, (setup, record_adapter))
        .add_systems(FixedUpdate, fixed_tick)
        .add_systems(
            Update,
            (
                input_commands,
                witness_commands,
                refresh_scene,
                camera_update,
                overlays,
                witness_audit,
                exit_on_escape,
            )
                .chain(),
        )
        .add_systems(Last, limit_fps)
        .run();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use qr_neural_memory_system::lattice::Case;
    #[test]
    fn selection_filters_and_seek_do_not_execute_engine() {
        let mut engine = Lattice::new(Case::default()).unwrap();
        let first = engine.observe();
        engine.advance();
        let second = engine.observe();
        let mut s = Session::new(
            Recording {
                frames: vec![first.clone(), second.clone()],
            },
            false,
        )
        .unwrap();
        assert!(s.live.is_none());
        s.seek(1);
        s.select(4095);
        assert_eq!(s.shown, second);
        s.filters.axis = Some(0);
        s.filters.plane = 0;
        assert!(s.filters.visible(&s.shown, 0));
        assert!(!s.filters.visible(&s.shown, 4095));
        s.seek(0);
        assert_eq!(s.shown, first);
        assert!(s.live.is_none());
        s.shown.synthetic = true;
        assert_eq!(s.label(), "SYNTHETIC TEST FIXTURE");
    }
}
