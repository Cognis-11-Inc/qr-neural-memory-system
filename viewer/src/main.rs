//! Read-only scientific instrument; graphics never define physical state.
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    text::FontSize,
    window::PresentMode,
};
use qr_neural_memory_system::{
    decay_0004::DecayRecording,
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
struct Morphology {
    before: Vec<[u8; 64]>,
    after: Vec<[u8; 64]>,
    counts: Vec<usize>,
    overlay: bool,
    module: usize,
    energy: Vec<(u64, f64, f64)>,
}
impl Morphology {
    fn load(path: &std::path::Path, module: usize) -> io::Result<Self> {
        let load = |name: &str| -> io::Result<Vec<[u8; 64]>> {
            let bytes = fs::read(path.join(name))?;
            if bytes.len() != 4096 * 64 || bytes.iter().any(|&x| x > 4) {
                return Err(io::Error::other("QR6 morphology palette invalid"));
            }
            Ok(bytes.as_chunks::<64>().0.to_vec())
        };
        let before = load("before.palette")?;
        let after = load("after.palette")?;
        let counts = before
            .iter()
            .zip(&after)
            .map(|(a, b)| a.iter().zip(b).filter(|(a, b)| a != b).count())
            .collect();
        let data = fs::read_to_string(path.join("metrics.csv"))?;
        if data.len() > 15_000_000 {
            return Err(io::Error::other("QR6 metrics too large"));
        }
        let mut energy = Vec::new();
        for line in data.lines().skip(1) {
            let fields: Vec<_> = line.split(',').collect();
            if fields.len() != 11 {
                return Err(io::Error::other("QR6 metrics schema"));
            }
            let value = |i: usize| fields[i].parse::<f64>().map_err(io::Error::other);
            let total = value(2)? + value(3)? + value(4)?;
            let work = value(6)?;
            if !total.is_finite() || !work.is_finite() {
                return Err(io::Error::other("QR6 nonfinite metrics"));
            }
            energy.push((fields[0].parse().map_err(io::Error::other)?, total, work));
        }
        Ok(Self {
            before,
            after,
            counts,
            overlay: true,
            module,
            energy,
        })
    }
}
#[derive(Component)]
struct BeforePaletteCell(usize);
#[derive(bevy::ecs::system::SystemParam)]
struct PalettePanels<'w, 's> {
    after: Query<
        'w,
        's,
        (&'static PaletteCell, &'static mut BackgroundColor),
        Without<BeforePaletteCell>,
    >,
    before: Query<
        'w,
        's,
        (&'static BeforePaletteCell, &'static mut BackgroundColor),
        Without<PaletteCell>,
    >,
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
    experiment4: Option<(String, f64)>,
    experiment5: bool,
    experiment6: Option<Morphology>,
    node_spacing: f32,
    decay: Option<DecayRecording>,
    playback_error: Option<String>,
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
            experiment4: None,
            experiment5: false,
            experiment6: None,
            node_spacing: 1.8,
            decay: None,
            playback_error: None,
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
    fn frame_count(&self) -> usize {
        self.decay
            .as_ref()
            .map_or(self.recording.frames.len(), DecayRecording::len)
    }
    fn seek(&mut self, index: usize) {
        self.live = None;
        self.running = false;
        let next = index.min(self.frame_count() - 1);
        if let Some(replay) = &mut self.decay {
            match replay.frame(next) {
                Ok(frame) => {
                    if self.index + 1 != next {
                        self.history.clear();
                    }
                    self.shown = frame;
                    self.history.push_back(self.shown.clone());
                    while self.history.len() > 6 {
                        self.history.pop_front();
                    }
                    self.index = next;
                    self.playback_error = None;
                }
                Err(error) => {
                    error!("Playback stopped: {error}");
                    self.playback_error = Some(error.to_string());
                }
            }
        } else {
            self.index = next;
            self.shown = self.recording.frames[next].clone();
        }
        self.dirty = true;
    }
    fn step(&mut self) {
        if self.decay.is_some() {
            if self.dirty {
                return;
            }
            if self.index + 1 < self.frame_count() {
                let running = self.running;
                self.seek(self.index + 1);
                if self.playback_error.is_none() {
                    self.running = running;
                }
            } else {
                self.running = false;
                self.dirty = true;
            }
            return;
        }
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
        if self.experiment4.is_some() {
            return;
        }
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
struct DecayAudit {
    path: Option<PathBuf>,
    writer: Option<BufWriter<File>>,
    last: Option<u64>,
    hold: u32,
}
impl DecayAudit {
    fn new(path: Option<PathBuf>) -> io::Result<Self> {
        let writer = if let Some(p) = &path {
            let mut w = BufWriter::new(
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(p)?,
            );
            writeln!(w, "tick,fnv1a,id,peak,velocity_peak,u27,v27,contiguous")?;
            Some(w)
        } else {
            None
        };
        Ok(Self {
            path,
            writer,
            last: None,
            hold: 0,
        })
    }
}
fn audit_decay(s: Res<Session>, mut audit: ResMut<DecayAudit>, mut commands: Commands) {
    if audit.writer.is_none() || s.decay.is_none() {
        return;
    }
    let tick = s.shown.tick;
    if audit.last == Some(tick) {
        if s.index + 1 == s.frame_count() {
            audit.hold += 1;
            if audit.hold == 5 {
                let path = audit.path.as_ref().unwrap().with_extension("png");
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));
            }
            if audit.hold == 30 {
                audit
                    .writer
                    .as_mut()
                    .unwrap()
                    .flush()
                    .expect("decay audit flush");
                fs::write(audit.path.as_ref().unwrap().with_extension("complete"),"Native full-tick replay audit completed; independently verify contiguous tick rows and hashes.\n").expect("decay audit marker");
                commands.write_message(AppExit::Success);
            }
        }
        return;
    }
    let contiguous = audit.last.map_or(tick == 0, |last| tick == last + 1);
    let hash = qr_neural_memory_system::decay_0004::observation_checksum(&s.shown)
        .expect("validated observed fields");
    let d = s.shown.diagnostic();
    let n = &s.shown.neurons[s.selected];
    writeln!(
        audit.writer.as_mut().unwrap(),
        "{tick},{hash:016x},{},{:.17e},{:.17e},{:.17e},{:.17e},{contiguous}",
        s.selected,
        d.peak,
        d.velocity_peak,
        n.module27[0],
        n.velocity27[0]
    )
    .expect("decay audit row");
    audit.last = Some(tick);
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
fn position(s: &Snapshot, id: usize, spacing: f32) -> Vec3 {
    let p = s.neurons[id].position;
    let d = s.case.topology.dimensions();
    Vec3::new(
        p[0] as f32 - (d[0] - 1) as f32 / 2.0,
        p[1] as f32 - (d[1] - 1) as f32 / 2.0,
        p[2] as f32 - (d[2] - 1) as f32 / 2.0,
    ) * spacing
}
fn fitted_orbit(snapshot: &Snapshot, spacing: f32) -> Orbit {
    let extent = snapshot
        .case
        .topology
        .dimensions()
        .into_iter()
        .max()
        .unwrap_or(16) as f32;
    Orbit {
        radius: 35.0 * extent / 16.0 * spacing,
        ..Orbit::default()
    }
}
fn change_spacing(s: &mut Session, orbit: &mut Orbit, value: f32) {
    let value = value.clamp(1.0, 3.0);
    let ratio = value / s.node_spacing;
    orbit.radius *= ratio;
    orbit.target *= ratio;
    s.node_spacing = value;
    s.dirty = true;
}
fn pick(s: &Session, origin: Vec3, direction: Vec3) -> Option<usize> {
    let mut best = None;
    for id in 0..POPULATION {
        if s.filters.visible(&s.shown, id) {
            let delta = position(&s.shown, id, s.node_spacing) - origin;
            let t = delta.dot(direction);
            if t > 0.0
                && (delta - direction * t).length() < 0.28
                && best.is_none_or(|(_, distance)| t < distance)
            {
                best = Some((id, t));
            }
        }
    }
    best.map(|(id, _)| id)
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
            Transform::from_translation(position(&s.shown, id, s.node_spacing)),
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
            if s.experiment6.is_some() {
                p.spawn((
                    Text::new("RECORDED BEFORE / FINAL AFTER (8 x 8)"),
                    TextFont {
                        font_size: FontSize::Px(12.),
                        ..default()
                    },
                ));
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|row| {
                    for before in [true, false] {
                        row.spawn(Node {
                            width: px(128),
                            height: px(128),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        })
                        .with_children(|grid| {
                            for module in 0..64 {
                                let mut cell = grid.spawn((
                                    Node {
                                        width: px(14),
                                        height: px(14),
                                        margin: UiRect::all(px(1)),
                                        ..default()
                                    },
                                    BackgroundColor(palette(0)),
                                ));
                                if before {
                                    cell.insert(BeforePaletteCell(module));
                                } else {
                                    cell.insert(PaletteCell(module));
                                }
                            }
                        });
                    }
                });
            } else {
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
            }
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                SeriesText,
            ));
        });
    commands.spawn((Text::new("Right drag: orbit   Middle drag: pan   Wheel: zoom   R: camera reset\nClick node or type ID + Enter; Up/Down: select ID   Space: pause/play   .: step\nLeft/Right: seek recording   Home/End: first/last   L: LIVE restart / REPLAY\nX/Y/Z: slice axis toggle   [ / ]: slice plane   F: all/shell/interior\nN: nodes   E: selected edges   A: activity   P: palette view   G: linear/log scale\nAxes: X red / Y green / Z blue. No resonance or learning is inferred from color.".replace("L: LIVE restart / REPLAY", if s.experiment4.is_some() { "REPLAY ONLY / L disabled" } else { "L: LIVE restart / REPLAY" })),TextFont{font_size:FontSize::Px(13.0),..default()},TextColor(Color::srgb(0.59,0.69,0.78)),Node{position_type:PositionType::Absolute,left:px(22),bottom:px(16),..default()}));
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
        orbit.target += right * (-mouse.motion.delta.x * 0.025 * s.node_spacing)
            + Vec3::Y * (mouse.motion.delta.y * 0.025 * s.node_spacing);
    }
    orbit.radius = (orbit.radius * (-mouse.scroll.delta.y * 0.08).exp()).clamp(
        2.0 * s.node_spacing,
        fitted_orbit(&s.shown, s.node_spacing).radius * 5.0,
    );
    if keys.just_pressed(KeyCode::KeyR) {
        *orbit = fitted_orbit(&s.shown, s.node_spacing);
    }
    for (key, delta) in [(KeyCode::Minus, -0.1), (KeyCode::Equal, 0.1)] {
        if keys.just_pressed(key) {
            let value = s.node_spacing + delta;
            change_spacing(&mut s, &mut orbit, value);
        }
    }
    if keys.just_pressed(KeyCode::Backslash) {
        change_spacing(&mut s, &mut orbit, 1.8);
    }
    if keys.just_pressed(KeyCode::Space) {
        s.running = !s.running;
        s.dirty = true;
    }
    if keys.just_pressed(KeyCode::Period) {
        s.running = false;
        s.step();
    }
    if keys.just_pressed(KeyCode::KeyM)
        && let Some(morph) = &mut s.experiment6
    {
        morph.overlay = !morph.overlay;
        s.dirty = true;
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
        let i = s.frame_count() - 1;
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
        && let Some(id) = pick(&s, ray.origin, *ray.direction)
    {
        s.select(id);
    }
}
fn fixed_tick(mut s: ResMut<Session>) {
    if s.running && (s.decay.is_none() || !s.dirty) {
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
        &mut Transform,
    )>,
    mut panels: PalettePanels,
    mut status: Single<&mut Text, StatusFilter>,
    mut info: Single<&mut Text, InfoFilter>,
    mut series: Single<&mut Text, SeriesFilter>,
) {
    if !s.dirty {
        return;
    }
    for (id, mut material, mut visibility, mut transform) in &mut nodes {
        transform.translation = position(&s.shown, id.0, s.node_spacing);
        *visibility = if s.filters.visible(&s.shown, id.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        material.0 = if s.experiment6.as_ref().is_some_and(|m| m.overlay) {
            match s.experiment6.as_ref().expect("QR6 overlay").counts[id.0] {
                0 => materials.neutral.clone(),
                1 => materials.palette[3].clone(),
                2..=3 => materials.palette[4].clone(),
                _ => materials.palette[1].clone(),
            }
        } else if activity_value(&s.shown, id.0).is_none() {
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
    for (cell, mut color) in &mut panels.after {
        color.0 = palette(
            s.experiment6
                .as_ref()
                .map_or(n.palette[cell.0], |m| m.after[s.selected][cell.0]),
        );
    }
    for (cell, mut color) in &mut panels.before {
        color.0 =
            palette(s.experiment6.as_ref().expect("QR6 before panel").before[s.selected][cell.0]);
    }
    let legend = if s.experiment6.as_ref().is_some_and(|m| m.overlay) {
        "RECORDED FINAL MORPHOLOGY DIFF / gray0 blue1 yellow2-3 red4+ / M toggles"
    } else if s.filters.palette {
        "PERSISTENT PALETTE / glyph = dominant module color"
    } else if !s.filters.activity {
        "ACTIVITY OVERLAY OFF"
    } else if s.filters.logarithmic {
        "ACTIVITY max-module/channel |u| / FIXED log10 1e-8..1; below floor dark"
    } else {
        "ACTIVITY max-module/channel |u| / FIXED linear 0..1"
    };
    let d = s.shown.diagnostic();
    let experiment = if let Some(morph) = &s.experiment6 {
        let (label, dt) = s.experiment4.as_ref().expect("QR6 metadata");
        format!(
            "EXPERIMENT 0006 / CHECKPOINT REPLAY / {label} / dt{dt} / input module{}",
            morph.module
        )
    } else if s.experiment5 {
        let (label, dt) = s.experiment4.as_ref().expect("0005 replay metadata");
        format!("EXPERIMENT 0005 / CHECKPOINT REPLAY / {label} / dt{dt}")
    } else if s.decay.is_some() {
        "EXPERIMENT 0004 EXTENSION / FROZEN / A2182 B2186 / AB+".to_string()
    } else {
        s.experiment4.as_ref().map_or_else(
            || "EXPERIMENT 0003".to_string(),
            |(arm, dt)| format!("EXPERIMENT 0004 / FROZEN / A2182 B2186 / {arm} / dt{dt}"),
        )
    };
    let dt = s.experiment4.as_ref().map_or(0.1, |(_, dt)| *dt);
    let tick_label = if s.decay.is_some() {
        format!("{:04}/3072", s.shown.tick)
    } else {
        format!("{:03}", s.shown.tick)
    };
    status.0 = format!(
        "{experiment} / {} / {} / {}\nTick {tick_label} / t={:.1} / 4,096 addressable nodes / nonfinite={}\n{}\nSlice {:?}={} / region {} / ID entry: {}",
        s.label(),
        s.shown.case.topology.name(),
        if s.running { "RUNNING" } else { "PAUSED" },
        s.shown.tick as f64 * dt,
        d.nonfinite,
        legend,
        s.filters.axis,
        s.filters.plane,
        s.filters.region,
        s.entry
    );
    status.0.push_str(&format!(
        "\nDisplay spacing {:.1} (visual only) / - = adjust / Backslash default / R camera",
        s.node_spacing
    ));
    if s.decay.is_some() {
        status.0.push_str(&format!(
            "\nEVERY TICK / global max|u|={:.4e} max|v|={:.4e} / both <1e-6: {}",
            d.peak,
            d.velocity_peak,
            d.peak < 1e-6 && d.velocity_peak < 1e-6
        ));
    }
    if let Some(error) = &s.playback_error {
        status.0.push_str(&format!("\nPLAYBACK STOPPED: {error}"));
    }
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
    if let Some(morph) = &s.experiment6 {
        info.0.push_str(&format!(
            "\nRecorded changed positions: {} / 64",
            morph.counts[s.selected]
        ));
        if let Some((_, energy, work)) = morph
            .energy
            .iter()
            .find(|(tick, _, _)| *tick == s.shown.tick)
        {
            info.0
                .push_str(&format!("\nGlobal E={energy:.6e}\nSwitch work={work:.6e}"));
        }
    }
    let mut text = String::from("MEASURED HISTORY / max |u|\n");
    let history: Vec<_> = if s.live.is_some() || s.decay.is_some() {
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
    let d = s.shown.case.topology.dimensions();
    let start = Vec3::new(
        -(d[0] as f32 + 1.) / 2.,
        -(d[1] as f32 + 1.) / 2.,
        -(d[2] as f32 + 1.) / 2.,
    ) * s.node_spacing;
    gizmos.arrow(
        start,
        start + Vec3::X * 5.0 * s.node_spacing,
        Color::srgb(0.95, 0.2, 0.2),
    );
    gizmos.arrow(
        start,
        start + Vec3::Y * 5.0 * s.node_spacing,
        Color::srgb(0.2, 0.9, 0.4),
    );
    gizmos.arrow(
        start,
        start + Vec3::Z * 5.0 * s.node_spacing,
        Color::srgb(0.3, 0.5, 1.0),
    );
    let selected = position(&s.shown, s.selected, s.node_spacing);
    gizmos.cube(
        Transform::from_translation(selected).with_scale(Vec3::splat(0.45)),
        Color::srgb(1.0, 0.95, 0.45),
    );
    if s.filters.edges {
        for j in s.shown.case.topology.neighbors(s.selected) {
            gizmos.line(
                selected,
                position(&s.shown, j, s.node_spacing),
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
    orbit.radius = fitted_orbit(&s.shown, s.node_spacing).radius
        * if index.is_multiple_of(2) {
            1.0
        } else {
            38.0 / 35.0
        };
    orbit.target.x = if index.is_multiple_of(3) {
        0.5 * s.node_spacing
    } else {
        0.0
    };
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
fn display_checksum(snapshot: &Snapshot) -> u64 {
    let mut bytes = b"QR-DISPLAY-RECEIPT-v1".to_vec();
    for x in [
        snapshot.tick,
        snapshot.case.seed,
        snapshot.case.topology as u64,
        snapshot.case.configuration as u64,
        snapshot.case.source as u64,
        snapshot.case.channel as u64,
        snapshot.case.drive as u64,
        u64::from(snapshot.synthetic),
    ] {
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    for neuron in &snapshot.neurons {
        for x in [
            neuron.id,
            neuron.position[0],
            neuron.position[1],
            neuron.position[2],
        ] {
            bytes.extend_from_slice(&(x as u64).to_le_bytes());
        }
        bytes.extend_from_slice(&neuron.palette);
        for x in neuron
            .peak
            .iter()
            .chain(&neuron.velocity_peak)
            .chain(&neuron.module27)
            .chain(&neuron.velocity27)
            .chain([&neuron.squared, &neuron.velocity_squared])
        {
            bytes.extend_from_slice(&x.to_le_bytes());
        }
        bytes.extend_from_slice(&neuron.nonfinite.to_le_bytes());
    }
    qr_neural_memory_system::observation::checksum(&bytes)
}
#[derive(Resource)]
struct Capture {
    path: Option<PathBuf>,
    ticks: u32,
}
fn capture_frame(
    mut c: ResMut<Capture>,
    s: Res<Session>,
    orbit: Res<Orbit>,
    mut commands: Commands,
) {
    let Some(path) = c.path.clone() else {
        return;
    };
    c.ticks += 1;
    if c.ticks == 30 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
        let receipt = format!(
            "tick={} snapshot_hash={:016x} spacing={} selected={} position={:?} radius={} engine_instances={}\n",
            s.shown.tick,
            display_checksum(&s.shown),
            s.node_spacing,
            s.selected,
            position(&s.shown, s.selected, s.node_spacing),
            orbit.radius,
            usize::from(s.live.is_some())
        );
        let receipt = if let Some(m) = &s.experiment6 {
            format!(
                "{receipt}morphology_before={:016x} morphology_after={:016x} changed_modules={} module={} overlay={}\n",
                qr_neural_memory_system::observation::checksum(&m.before[s.selected]),
                qr_neural_memory_system::observation::checksum(&m.after[s.selected]),
                m.counts[s.selected],
                m.module,
                m.overlay
            )
        } else {
            receipt
        };
        fs::write(path.with_extension("txt"), receipt).expect("capture receipt");
    }
    if c.ticks == 60 {
        commands.write_message(AppExit::Success);
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
    let mut experiment4 = false;
    let mut experiment5 = false;
    let mut experiment6 = false;
    let mut node_spacing = 1.8_f32;
    let mut capture = None;
    let mut decay = false;
    let mut decay_audit = None;
    let mut selected = None;
    let mut slice_z = None;
    let mut region = 0_u8;
    let mut start_frame = 0;
    let mut play = false;
    let mut replay_hz = 20.0;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--recording" => {
                path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing path"))?,
                )
            }
            "--select" => {
                selected = Some(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing ID"))?
                        .parse::<usize>()
                        .map_err(io::Error::other)?,
                )
            }
            "--region" => {
                region = match args.next().as_deref() {
                    Some("all") => 0,
                    Some("shell") => 1,
                    Some("interior") => 2,
                    _ => return Err(io::Error::other("region must be all,shell,interior")),
                };
            }
            "--slice-z" => {
                slice_z = Some(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing plane"))?
                        .parse::<usize>()
                        .map_err(io::Error::other)?,
                )
            }
            "--frame" => {
                start_frame = args
                    .next()
                    .ok_or_else(|| io::Error::other("missing frame index"))?
                    .parse::<usize>()
                    .map_err(io::Error::other)?
            }
            "--play" => play = true,
            "--replay-hz" => {
                replay_hz = args
                    .next()
                    .ok_or_else(|| io::Error::other("missing replay cadence"))?
                    .parse::<f64>()
                    .map_err(io::Error::other)?
            }
            "--node-spacing" => {
                node_spacing = args
                    .next()
                    .ok_or_else(|| io::Error::other("missing spacing"))?
                    .parse::<f32>()
                    .map_err(io::Error::other)?
            }
            "--capture" => {
                capture = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing capture path"))?,
                ))
            }
            "--experiment-0006" => experiment6 = true,
            "--experiment-0005" => experiment5 = true,
            "--live" => live = true,
            "--experiment-0004" => experiment4 = true,
            "--decay-0004" => {
                experiment4 = true;
                decay = true;
            }
            "--decay-audit" => {
                decay_audit = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| io::Error::other("missing audit path"))?,
                ))
            }
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
    if [experiment4, experiment5, experiment6]
        .into_iter()
        .filter(|&x| x)
        .count()
        > 1
    {
        return Err(io::Error::other("choose one experiment replay mode"));
    }
    if !node_spacing.is_finite() || !(1.0..=3.0).contains(&node_spacing) {
        return Err(io::Error::other("node spacing must be finite in1..3"));
    }
    if (experiment4 || experiment5 || experiment6) && (live || synthetic || witness.is_some()) {
        return Err(io::Error::other(
            "0004/0005/0006 support measured REPLAY only; LIVE/synthetic/0003 witness prohibited",
        ));
    }
    if decay_audit.is_some() && !decay {
        return Err(io::Error::other("decay audit requires --decay-0004"));
    }
    let mut morphology = None;
    let (mut recording, metadata, stream) = if experiment6 {
        let (r, f) = qr_neural_memory_system::morphic_0006::load_replay(&path)?;
        if !path.join("COMPLETE").exists() {
            return Err(io::Error::other("QR6 replay requires completed case"));
        }
        let m = Morphology::load(&path, f.module)?;
        if r.frames[0]
            .neurons
            .iter()
            .map(|n| n.palette)
            .collect::<Vec<_>>()
            != m.before
            || r.frames
                .last()
                .expect("nonempty QR6")
                .neurons
                .iter()
                .map(|n| n.palette)
                .collect::<Vec<_>>()
                != m.after
        {
            return Err(io::Error::other(
                "QR6 endpoint palettes disagree with morphology metadata",
            ));
        }
        morphology = Some(m);
        (r, Some((f.label, f.dt)), None)
    } else if decay {
        let mut replay = DecayRecording::open(&path)?;
        let first = replay.frame(0)?;
        (
            Recording {
                frames: vec![first],
            },
            Some(("AB+ / EVERY TICK".to_string(), 0.1)),
            Some(replay),
        )
    } else if experiment5 {
        let (r, f) = qr_neural_memory_system::morphic_0005::load_replay(&path)?;
        (r, Some((f.label, f.dt)), None)
    } else if experiment4 {
        let (r, f) = qr_neural_memory_system::propagation_0004::load_replay(&path)?;
        (r, Some((f.arm.name().to_string(), f.dt)), None)
    } else {
        (
            Recording::load(&path).map_err(|e| {
                io::Error::other(format!(
                    "Recording rejected before engine/window startup: {e}"
                ))
            })?,
            None,
            None,
        )
    };
    if synthetic {
        for f in &mut recording.frames {
            f.synthetic = true;
            f.neurons[0].nonfinite = 1;
        }
    }
    let mut s = Session::new(recording, live)?;
    s.experiment4 = metadata;
    s.experiment5 = experiment5;
    s.experiment6 = morphology;
    s.node_spacing = node_spacing;
    s.decay = stream;
    if start_frame >= s.frame_count()
        || selected.is_some_and(|id| id >= POPULATION)
        || slice_z.is_some_and(|z| z >= s.shown.case.topology.dimensions()[2])
        || !replay_hz.is_finite()
        || !(0.1..=60.0).contains(&replay_hz)
        || (live && (start_frame != 0 || replay_hz.to_bits() != 20.0_f64.to_bits()))
    {
        return Err(io::Error::other(
            "invalid startup selection/slice/frame/cadence; alternate cadence is replay-only",
        ));
    }
    if !live {
        s.seek(start_frame);
    }
    if let Some(error) = &s.playback_error {
        return Err(io::Error::other(error.clone()));
    }
    if let Some(id) = selected {
        s.select(id);
    }
    if let Some(z) = slice_z {
        s.filters.axis = Some(2);
        s.filters.plane = z;
    }
    s.filters.region = region;
    s.running = play;
    eprintln!(
        "Startup view: selected={} tick={} Z={:?} play={} replay_hz={}",
        s.selected, s.shown.tick, slice_z, play, replay_hz
    );
    eprintln!(
        "Observatory opened {}: 4096 nodes, engine_instances={}",
        s.label(),
        usize::from(live)
    );
    let w = Witness::new(witness, live)?;
    let audit = DecayAudit::new(decay_audit)?;
    let base_orbit = fitted_orbit(&s.shown, node_spacing);
    let orbit = if camera_b {
        Orbit {
            yaw: -1.4,
            pitch: 0.8,
            ..base_orbit
        }
    } else {
        base_orbit
    };
    let mut render_settings = bevy::render::settings::WgpuSettings::default();
    if std::env::var_os("WGPU_POWER_PREF").is_none() {
        render_settings.power_preference = bevy::render::settings::PowerPreference::LowPower;
    }
    render_settings.adapter_name = adapter;
    App::new()
        .insert_resource(Capture {
            path: capture,
            ticks: 0,
        })
        .insert_resource(s)
        .insert_resource(w)
        .insert_resource(audit)
        .insert_resource(orbit)
        .insert_resource(FrameCap(fps))
        .insert_resource(Time::<Fixed>::from_hz(replay_hz))
        .insert_resource(ClearColor(Color::srgb(0.018, 0.025, 0.04)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: if experiment6 {
                            "QR Neural Network | Experiment 0006 Morphology Observatory"
                        } else if experiment5 {
                            "QR Neural Network | Experiment 0005 Checkpoint Replay"
                        } else if experiment4 {
                            "QR Neural Network | Experiment 0004 Frozen Medium"
                        } else {
                            "QR Neural Network | Experiment 0003 Observatory"
                        }
                        .into(),
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
        .add_systems(Last, (audit_decay, capture_frame, limit_fps).chain())
        .run();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use qr_neural_memory_system::lattice::Case;
    #[test]
    fn authoritative_morphology_panels_and_replay_guard() {
        let folder = std::env::temp_dir().join(format!("qr6-viewmodel-{}", std::process::id()));
        fs::create_dir(&folder).unwrap();
        let first = Lattice::new(Case::default()).unwrap().observe();
        let before: Vec<_> = first.neurons.iter().map(|n| n.palette).collect();
        let mut after = before.clone();
        for color in after[2182].iter_mut().take(8) {
            *color = (*color + 1) % 5;
        }
        after[2183][9] = (after[2183][9] + 1) % 5;
        fs::write(folder.join("before.palette"), before.concat()).unwrap();
        fs::write(folder.join("after.palette"), after.concat()).unwrap();
        fs::write(folder.join("metrics.csv"),"tick,time,kinetic,onsite,coupling,loss,switch_work,drift,peak,velocity_peak,nonfinite\n0,0,0,1,0,0,0,0,1,0,0\n").unwrap();
        let m = Morphology::load(&folder, 9).unwrap();
        assert_eq!(m.counts[2182], 8);
        assert_eq!(m.counts[2183], 1);
        assert_eq!(m.counts[0], 0);
        assert_eq!(m.before[2182], before[2182]);
        assert_eq!(m.after[2182], after[2182]);
        let mut session = Session::new(
            Recording {
                frames: vec![first.clone()],
            },
            false,
        )
        .unwrap();
        session.experiment4 = Some(("unit-only-qr6".into(), 0.025));
        session.experiment6 = Some(m);
        session.toggle_live();
        assert!(session.live.is_none());
        assert_eq!(session.shown, first);
        assert_eq!(session.node_spacing, 1.8);
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn spacing_is_pure_display_and_picking_uses_display_endpoints() {
        use qr_neural_memory_system::topology::Topology;
        for topology in [Topology::Cubic6, Topology::Cubic26, Topology::Grid2D8] {
            let snapshot = Lattice::new(Case {
                topology,
                ..Case::default()
            })
            .unwrap()
            .observe();
            let original = snapshot.encode().unwrap();
            let mut session = Session::new(
                Recording {
                    frames: vec![snapshot.clone()],
                },
                false,
            )
            .unwrap();
            let mut orbit = fitted_orbit(&snapshot, session.node_spacing);
            for spacing in [1.0, 1.8, 3.0] {
                change_spacing(&mut session, &mut orbit, spacing);
                assert_eq!(session.shown.encode().unwrap(), original);
                session.filters.axis = Some(2);
                session.filters.plane = snapshot.neurons[2184].position[2];
                let selected = position(&snapshot, 2184, spacing);
                assert_eq!(
                    pick(&session, selected + Vec3::Z * 100., -Vec3::Z),
                    Some(2184)
                );
                for other in topology.neighbors(2184) {
                    let from = position(&snapshot, 2184, spacing);
                    let to = position(&snapshot, other, spacing);
                    let raw = Vec3::from_array(snapshot.neurons[other].position.map(|x| x as f32))
                        - Vec3::from_array(snapshot.neurons[2184].position.map(|x| x as f32));
                    assert!((to - from - raw * spacing).length() < 1e-4);
                    assert_eq!(topology.hops(2184, other), 1);
                }
                assert!(fitted_orbit(&snapshot, spacing).radius > 15. * spacing);
            }
            let byte_check = session.shown.encode().unwrap();
            change_spacing(&mut session, &mut orbit, 1.8);
            assert_eq!(session.shown.encode().unwrap(), byte_check);
        }
    }
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
        s.experiment4 = Some(("AB+".to_string(), 0.05));
        let before = s.shown.clone();
        s.toggle_live();
        assert!(s.live.is_none());
        assert_eq!(s.shown, before);
        s.shown.synthetic = true;
        assert_eq!(s.label(), "SYNTHETIC TEST FIXTURE");
    }
}
