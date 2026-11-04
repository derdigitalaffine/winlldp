use eframe::egui::{self, Color32, RichText, Vec2};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::{Duration, Instant},
};
use winlldp::{
    capture::{self, Adapter, CaptureBackend, Engine, Notice},
    model::{Event, Monitor},
    protocol::{self, TxProfile},
    settings::{self, Language as L, Settings, Theme},
};
const GREEN: Color32 = Color32::from_rgb(30, 184, 146);

pub struct App {
    settings: Settings,
    monitor: Monitor,
    engine: Engine,
    adapters: Vec<Adapter>,
    chosen: BTreeSet<String>,
    ready: BTreeSet<String>,
    tx_active: BTreeSet<String>,
    tx_pending: BTreeSet<String>,
    page: usize,
    selected: usize,
    query: String,
    detail_open: bool,
    selection_dirty: bool,
    confirm_clear_history: bool,
    selected_event: Option<Event>,
    detail_key: Option<(String, String, String)>,
    npcap: Result<capture::NpcapReport, String>,
    capture_errors: BTreeMap<String, String>,
    backend_names: BTreeMap<String, String>,
    backend_notes: BTreeMap<String, String>,
    native_report: Option<Result<String, String>>,
    own_frames: u64,
    message: String,
    is_error: bool,
    logo: egui::TextureHandle,
    send_index: usize,
    profile: TxProfile,
    extra_kind: u8,
    extra_hex: String,
    last_save: Instant,
    exit: bool,
    qa_output: Option<String>,
    qa_frames: u32,
    #[cfg(windows)]
    tray: Option<crate::platform::Tray>,
}
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings: Settings = settings::load_json("settings.json").unwrap_or_default();
        let mut monitor = Monitor::default();
        if settings.keep_history {
            monitor.events =
                settings::load_json::<VecDeque<Event>>("history.json").unwrap_or_default();
            monitor.events.truncate(2000);
        }
        let im = image::load_from_memory(include_bytes!("../assets/winlldp-otter.png"))
            .unwrap()
            .into_rgba8();
        let logo = cc.egui_ctx.load_texture(
            "otter",
            egui::ColorImage::from_rgba_unmultiplied(
                [im.width() as usize, im.height() as usize],
                &im,
            ),
            Default::default(),
        );
        let adapters = capture::adapters().unwrap_or_default();
        let chosen = adapters
            .iter()
            .filter(|a| a.up)
            .map(|a| a.id.clone())
            .collect();
        let mut app = Self {
            settings,
            monitor,
            engine: Engine::default(),
            adapters,
            chosen,
            ready: BTreeSet::new(),
            tx_active: BTreeSet::new(),
            tx_pending: BTreeSet::new(),
            page: 0,
            selected: 0,
            query: String::new(),
            detail_open: false,
            selection_dirty: false,
            confirm_clear_history: false,
            selected_event: None,
            detail_key: None,
            npcap: capture::inspect_npcap(),
            capture_errors: BTreeMap::new(),
            backend_names: BTreeMap::new(),
            backend_notes: BTreeMap::new(),
            native_report: None,
            own_frames: 0,
            message: String::new(),
            is_error: false,
            logo,
            send_index: 0,
            profile: TxProfile::default(),
            extra_kind: 127,
            extra_hex: String::new(),
            last_save: Instant::now(),
            exit: false,
            qa_output: None,
            qa_frames: 0,
            #[cfg(windows)]
            tray: None,
        };
        #[cfg(windows)]
        {
            match crate::platform::Tray::new() {
                Ok(t) => app.tray = Some(t),
                Err(e) => app.notify(Err(e)),
            }
        }
        let args: Vec<String> = std::env::args().collect();
        let argument = |key: &str| {
            args.iter()
                .position(|v| v == key)
                .and_then(|i| args.get(i + 1))
                .cloned()
        };
        app.qa_output = argument("--screenshot");
        if app.qa_output.is_some() {
            app.settings = Settings::default();
            app.monitor = Monitor::default();
            app.page = argument("--page").and_then(|s| s.parse().ok()).unwrap_or(0);
            app.settings.language = match argument("--language").as_deref() {
                Some("en") => L::English,
                Some("nl") => L::Nederlands,
                _ => L::Deutsch,
            };
            if args.iter().any(|v| v == "--dark") {
                app.settings.theme = Theme::Dark;
            }
            if args.iter().any(|v| v == "--light") {
                app.settings.theme = Theme::Light;
            }
            app.chosen.clear();
            if args.iter().any(|v| v == "--qa-adapter") {
                app.adapters = vec![Adapter {
                    id: "QA-SYNTHETIC".into(),
                    name: "Ethernet · USB-C".into(),
                    description: "Synthetic QA adapter (no live capture)".into(),
                    mac: [0, 17, 34, 51, 68, 85],
                    ips: vec!["192.0.2.20".into()],
                    up: true,
                }];
            }
        }
        if let Some(path) = argument("--open") {
            match std::fs::File::open(path)
                .map_err(|e| e.to_string())
                .and_then(winlldp::export::read_pcap)
            {
                Ok(packets) => {
                    for p in packets {
                        let _ = app.monitor.ingest("PCAP", p.bytes, p.at);
                    }
                }
                Err(e) => app.notify(Err(e)),
            }
        }
        app.load_profile();
        app.apply_theme(&cc.egui_ctx);
        if !app.chosen.is_empty() {
            app.start();
        }
        app
    }
    fn notify(&mut self, result: Result<String, String>) {
        match result {
            Ok(s) => {
                self.message = s;
                self.is_error = false;
            }
            Err(s) => {
                self.message = s;
                self.is_error = true;
            }
        }
    }
    fn apply_theme(&self, ctx: &egui::Context) {
        ctx.set_theme(match self.settings.theme {
            Theme::System => egui::ThemePreference::System,
            Theme::Light => egui::ThemePreference::Light,
            Theme::Dark => egui::ThemePreference::Dark,
        });
        ctx.all_styles_mut(|s| {
            let dark = s.visuals.dark_mode;
            s.visuals.override_text_color = Some(if dark {
                Color32::from_rgb(229, 237, 245)
            } else {
                Color32::from_rgb(24, 42, 58)
            });
            s.visuals.weak_text_color = Some(if dark {
                Color32::from_rgb(166, 185, 201)
            } else {
                Color32::from_rgb(73, 94, 112)
            });
            s.visuals.panel_fill = if dark {
                Color32::from_rgb(16, 24, 33)
            } else {
                Color32::from_rgb(245, 248, 251)
            };
            s.visuals.window_fill = if dark {
                Color32::from_rgb(23, 35, 47)
            } else {
                Color32::WHITE
            };
            s.visuals.faint_bg_color = if dark {
                Color32::from_rgb(23, 35, 47)
            } else {
                Color32::from_rgb(233, 240, 245)
            };
            s.visuals.extreme_bg_color = if dark {
                Color32::from_rgb(10, 17, 24)
            } else {
                Color32::WHITE
            };
            // One dense design language for table, toolbar and controls.
            s.spacing.item_spacing = Vec2::new(7., 6.);
            s.spacing.button_padding = Vec2::new(10., 5.);
            s.spacing.interact_size.y = 27.;
            s.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(7);
            s.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(7);
            s.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(7);
            s.visuals.widgets.active.corner_radius = egui::CornerRadius::same(7);
            s.visuals.selection.bg_fill = GREEN.gamma_multiply(0.5);
            s.visuals.hyperlink_color = if dark {
                GREEN
            } else {
                Color32::from_rgb(0, 107, 83)
            };
            s.text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(19.));
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(14.));
            s.text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(13.));
        });
    }
    fn start(&mut self) {
        self.ready.clear();
        self.capture_errors.clear();
        self.backend_names.clear();
        self.backend_notes.clear();
        self.npcap = capture::inspect_npcap();
        self.tx_active.clear();
        self.tx_pending.clear();
        let devices = self
            .adapters
            .iter()
            .filter(|a| self.chosen.contains(&a.id))
            .cloned()
            .collect::<Vec<_>>();
        let r = self.engine.start_with(&devices, self.settings.capture_backend).map(|_| {
            self.settings
                .language
                .t(
                    "Empfang wird gestartet …",
                    "Starting capture …",
                    "Ontvangst starten …",
                )
                .into()
        });
        self.notify(r);
    }
    fn stop(&mut self) {
        self.engine.stop();
        self.ready.clear();
        self.tx_active.clear();
        self.tx_pending.clear();
    }
    fn load_profile(&mut self) {
        if let Some(a) = self.adapters.get(self.send_index) {
            self.profile = self
                .settings
                .profiles
                .get(&a.id)
                .cloned()
                .unwrap_or_else(|| TxProfile {
                    chassis_value: a
                        .mac
                        .iter()
                        .map(|b| format!("{b:02X}"))
                        .collect::<Vec<_>>()
                        .join(":"),
                    port_value: a.name.clone(),
                    port_description: a.description.clone(),
                    ..Default::default()
                });
        }
    }
    fn save(&mut self) {
        if self.qa_output.is_some() {
            return;
        }
        if let Some(a) = self.adapters.get(self.send_index) {
            self.settings
                .profiles
                .insert(a.id.clone(), self.profile.clone());
        }
        if let Err(e) = settings::save_json("settings.json", &self.settings) {
            self.notify(Err(e));
        }
        if self.settings.keep_history {
            if let Err(e) = settings::save_json("history.json", &self.monitor.events) {
                self.notify(Err(e));
            }
        }
        self.last_save = Instant::now();
    }
    fn name(&self, id: &str) -> String {
        self.adapters
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| id.into())
    }
    fn import(&mut self) {
        let l = self.settings.language;
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("PCAP", &["pcap", "cap"])
            .pick_file()
        {
            let result = (|| {
                let f = std::fs::File::open(&p).map_err(|e| e.to_string())?;
                let packets = winlldp::export::read_pcap(f)?;
                let mut accepted = 0;
                let mut rejected = 0;
                for p in packets {
                    if self.monitor.ingest("PCAP", p.bytes, p.at).is_ok() {
                        accepted += 1;
                    } else {
                        rejected += 1;
                    }
                }
                self.page = 0;
                Ok(format!(
                    "{}: {accepted} LLDP / {rejected} {}",
                    l.t("Importiert", "Imported", "Geïmporteerd"),
                    l.t("andere/ungültig", "other/invalid", "overig/ongeldig")
                ))
            })();
            self.notify(result);
        }
    }
    fn export(&mut self, kind: &str) {
        if let Some(p) = rfd::FileDialog::new()
            .set_file_name(format!("WinLLDP.{}", kind.to_lowercase()))
            .add_filter(kind, &[&kind.to_lowercase()])
            .save_file()
        {
            let result = (|| -> Result<String, String> {
                let b = match kind {
                    "CSV" => winlldp::export::csv(&self.monitor).into_bytes(),
                    "JSON" => winlldp::export::json(&self.monitor).map_err(|e| e.to_string())?,
                    _ => {
                        let mut b = vec![];
                        winlldp::export::write_pcap(&self.monitor.packets, &mut b)
                            .map_err(|e| e.to_string())?;
                        b
                    }
                };
                std::fs::write(&p, b).map_err(|e| e.to_string())?;
                Ok(format!(
                    "{}: {}",
                    self.settings
                        .language
                        .t("Gespeichert", "Saved", "Opgeslagen"),
                    p.display()
                ))
            })();
            self.notify(result);
        }
    }
    fn poll(&mut self) {
        for e in self.engine.poll() {
            match e {
                Notice::Frame(p) => {
                    let _ = self.monitor.ingest(&p.adapter, p.bytes, p.at);
                }
                Notice::LocalFrame(_id) => self.own_frames += 1,
                Notice::Backend(id, backend) => {
                    self.backend_names.insert(id.clone(), backend);
                    self.monitor.log(&self.name(&id), "backend_selected", String::new());
                }
                Notice::Diagnostic(id, detail) => {
                    self.backend_notes.insert(id.clone(), detail.clone());
                    self.monitor.log(&self.name(&id), "capture_diagnostic", detail);
                },
                Notice::Status(id, s) => {
                    if s == "capture_started" {
                        self.capture_errors.remove(&id);
                        self.ready.insert(id.clone());
                    } else {
                        self.ready.remove(&id);
                        self.tx_active.remove(&id);
                        self.tx_pending.remove(&id);
                    }
                    self.monitor.log(&self.name(&id), &s, String::new());
                }
                Notice::Error(id, e) => {
                    self.capture_errors.insert(id.clone(), e.clone());
                    self.tx_pending.remove(&id);
                    self.monitor.log(&self.name(&id), "error", e.clone());
                    self.notify(Err(format!("{}: {e}", self.name(&id))));
                }
                Notice::Tx(id, active) => {
                    self.tx_pending.remove(&id);
                    if active {
                        self.tx_active.insert(id.clone());
                    } else {
                        self.tx_active.remove(&id);
                    }
                    self.monitor.log(
                        &self.name(&id),
                        if active { "tx_started" } else { "tx_stopped" },
                        String::new(),
                    );
                }
            }
        }
        // Offline records retain their capture-time TTL; only live neighbors expire in real time.
        let mut offline = vec![];
        self.monitor.neighbors.retain(|n| {
            if n.adapter == "PCAP" {
                offline.push(n.clone());
                false
            } else {
                true
            }
        });
        self.monitor.expire(chrono::Utc::now());
        self.monitor.neighbors.extend(offline);
    }
    // Shared border, padding and type scale across all secondary surfaces.
    fn surface(ui: &egui::Ui) -> egui::Frame {
        let (fill, border) = if ui.visuals().dark_mode {
            (Color32::from_rgb(24, 36, 49), Color32::from_rgb(43, 62, 77))
        } else {
            (Color32::WHITE, Color32::from_rgb(220, 229, 235))
        };
        egui::Frame::new()
            .fill(fill)
            .stroke(egui::Stroke::new(1., border))
            .inner_margin(11.)
            .corner_radius(8)
    }
    fn title(ui: &mut egui::Ui, title: &str, _description: &str) {
        ui.label(RichText::new(title).size(19.).strong());
        ui.add_space(7.);
    }
    fn refresh_adapters(&mut self) {
        let was_running = self.engine.running();
        if let Some(a) = self.adapters.get(self.send_index) {
            self.settings.profiles.insert(a.id.clone(), self.profile.clone());
        }
        let old_id = self.adapters.get(self.send_index).map(|a| a.id.clone());
        match capture::adapters() {
            Ok(adapters) => {
                self.stop();
                self.adapters = adapters;
                self.chosen.retain(|id| self.adapters.iter().any(|a| &a.id == id));
                self.send_index = old_id
                    .and_then(|id| self.adapters.iter().position(|a| a.id == id))
                    .unwrap_or(0);
                self.load_profile();
                self.npcap = capture::inspect_npcap();
                self.native_report = None;
                self.selection_dirty = false;
                if was_running && !self.chosen.is_empty() {
                    self.start();
                }
            }
            Err(error) => self.notify(Err(error)),
        }
    }
    fn restart_elevated(&mut self, ctx: &egui::Context) {
        match crate::platform::elevate() {
            Ok(()) => {
                self.exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(error) => self.notify(Err(error)),
        }
    }
    fn file_menu(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        ui.menu_button(l.t("Datei", "File", "Bestand"), |ui| {
            if ui.button(l.t("PCAP öffnen …", "Open PCAP …", "PCAP openen …")).clicked() {
                self.import();
                ui.close();
            }
            ui.separator();
            for (kind, label) in [
                ("CSV", l.t("CSV exportieren", "Export CSV", "CSV exporteren")),
                ("JSON", l.t("JSON exportieren", "Export JSON", "JSON exporteren")),
                ("PCAP", l.t("PCAP exportieren", "Export PCAP", "PCAP exporteren")),
            ] {
                if ui.button(label).clicked() {
                    self.export(kind);
                    ui.close();
                }
            }
        });
    }
    fn overview(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        ui.horizontal(|ui| {
            ui.label(RichText::new(l.t(
                "Nachbarn", "Neighbors", "Buren"
            )).size(19.).strong());
            ui.separator();
            ui.label(RichText::new(format!(
                "{} {}", self.monitor.neighbors.len(),
                l.t("gefunden", "found", "gevonden"),
            )).small().weak());
        });
        ui.add_space(5.);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query)
                .hint_text(l.t("Switch, Port oder Adapter suchen",
                    "Search switch, port or adapter",
                    "Zoek switch, poort of adapter"))
                .desired_width(285.));
            if !self.query.is_empty()
                && ui.small_button(l.t("Leeren", "Clear", "Wissen")).clicked() {
                self.query.clear();
            }
            self.file_menu(ui);
        });
        ui.add_space(4.);
        let query = self.query.trim().to_lowercase();
        let rows: Vec<(usize, String, String, String, String, String)> = self.monitor.neighbors
            .iter().enumerate().filter_map(|(index, neighbor)| {
                let a = &neighbor.advertisement;
                let switch = if a.system_name.is_empty() {
                    a.chassis_id.clone()
                } else {
                    a.system_name.clone()
                };
                let adapter = self.name(&neighbor.adapter);
                let searchable = format!("{} {} {} {} {} {}",
                    switch, a.port_id, a.port_description, a.chassis_id,
                    a.source_mac, adapter).to_lowercase();
                if !query.is_empty() && !searchable.contains(&query) {
                    return None;
                }
                Some((index, switch, a.port_id.clone(), adapter,
                    format!("{} s", a.ttl),
                    neighbor.last_seen.format("%d.%m. %H:%M:%S").to_string()))
            }).collect();
        if rows.is_empty() {
            ui.add_space(14.);
            Self::surface(ui).show(ui, |ui| {
                ui.add_space(12.);
                ui.vertical_centered(|ui| {
                    if self.query.is_empty() {
                        ui.label(RichText::new(l.t(
                            "Noch keine LLDP-Nachbarn",
                            "No LLDP neighbors yet",
                            "Nog geen LLDP-buren"
                        )).strong());
                        ui.label(RichText::new(l.t(
                            "Aktiven Netzwerkadapter und LLDP am Switch prüfen.",
                            "Check the active adapter and LLDP on the switch.",
                            "Controleer de adapter en LLDP op de switch."
                        )).small().weak());
                        ui.add_space(5.);
                        if ui.button(l.t(
                            "Adapter konfigurieren", "Configure adapters", "Adapters instellen"
                        )).clicked() {
                            self.page = 1;
                        }
                    } else {
                        ui.label(l.t(
                            "Keine Treffer. Suchbegriff ändern oder leeren.",
                            "No matches. Try another search.",
                            "Geen resultaten. Probeer een andere zoekterm."
                        ));
                    }
                });
                ui.add_space(12.);
            });
            return;
        }
        if !rows.iter().any(|row| row.0 == self.selected) {
            self.selected = rows[0].0;
        }
        ui.label(RichText::new(format!(
            "{} {}  ·  {}",
            rows.len(), l.t("Treffer", "matches", "resultaten"),
            l.t("Doppelklick für Details", "Double-click for details",
                "Dubbelklik voor details")
        )).small().weak());
        ui.add_space(2.);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8., 3.);
            ui.spacing_mut().interact_size.y = 23.;
            egui::ScrollArea::both().id_salt("neighbors-table")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("neighbors-grid")
                        .num_columns(5)
                        .striped(true)
                        .spacing([15., 4.])
                        .show(ui, |ui| {
                            for heading in [
                                l.t("Switch", "Switch", "Switch"),
                                l.t("Port", "Port", "Poort"),
                                l.t("Adapter", "Adapter", "Adapter"),
                                "TTL",
                                l.t("Zuletzt (UTC)", "Last (UTC)", "Laatst (UTC)"),
                            ] {
                                ui.label(RichText::new(heading).small().strong().weak());
                            }
                            ui.end_row();
                            for (index, switch, port, adapter, ttl, seen) in &rows {
                                let selected = *index == self.selected;
                                for (value, hover) in [
                                    (switch.as_str(), switch.as_str()),
                                    (port.as_str(), port.as_str()),
                                    (adapter.as_str(), adapter.as_str()),
                                    (ttl.as_str(), ttl.as_str()),
                                    (seen.as_str(), seen.as_str()),
                                ] {
                                    let response = ui.selectable_label(selected, value)
                                        .on_hover_text(hover);
                                    if response.clicked() { self.selected = *index; }
                                    if response.double_clicked() ||
                                        (response.has_focus() &&
                                         ui.input(|input| input.key_pressed(egui::Key::Enter))) {
                                        self.selected = *index;
                                        let neighbor = &self.monitor.neighbors[*index];
                                        self.detail_key = Some((
                                            neighbor.adapter.clone(),
                                            neighbor.advertisement.chassis_id.clone(),
                                            neighbor.advertisement.port_id.clone(),
                                        ));
                                        self.detail_open = true;
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
        });
    }
    fn neighbor_details(&self, ui: &mut egui::Ui, index: usize) {
        let l = self.settings.language;
                    let n = &self.monitor.neighbors[index];
                    let a = &n.advertisement;
                    Self::surface(ui)
                        .inner_margin(18.)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(l.t("SWITCH", "SWITCH", "SWITCH"))
                                    .color(ui.visuals().hyperlink_color)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(if a.system_name.is_empty() {
                                    &a.chassis_id
                                } else {
                                    &a.system_name
                                })
                                .size(24.)
                                .strong(),
                            );
                            ui.add_space(8.);
                            ui.label(
                                RichText::new(format!(
                                    "{}  {}",
                                    l.t("Port", "Port", "Poort"),
                                    a.port_id
                                ))
                                .size(21.),
                            );
                            if !a.port_description.is_empty() {
                                ui.label(&a.port_description);
                            }
                            ui.separator();
                            egui::Grid::new("summary")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    for (k, v) in [
                                        ("Chassis ID", a.chassis_id.clone()),
                                        (
                                            l.t("Adapter", "Adapter", "Adapter"),
                                            self.name(&n.adapter),
                                        ),
                                        ("MAC", a.source_mac.clone()),
                                        ("TTL", format!("{} s", a.ttl)),
                                        (
                                            l.t("Zuletzt gesehen", "Last seen", "Laatst gezien"),
                                            n.last_seen.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                                        ),
                                    ] {
                                        ui.label(RichText::new(k).weak());
                                        ui.add(egui::Label::new(v).selectable(true).wrap());
                                        ui.end_row();
                                    }
                                });
                            if n.adapter == "PCAP" {
                                ui.label(l.t(
                                    "Offline-Aufzeichnung · Zeitstempel aus Datei",
                                    "Offline capture · timestamps from file",
                                    "Offline-opname · tijdstempels uit bestand",
                                ));
                            }
                            if !a.system_description.is_empty() {
                                ui.add_space(6.);
                                ui.add(
                                    egui::Label::new(&a.system_description)
                                        .selectable(true)
                                        .wrap(),
                                );
                            }
                        });
                    ui.add_space(12.);
                    ui.strong(l.t(
                        "Vollständige LLDP-Daten",
                        "Complete LLDP data",
                        "Volledige LLDP-gegevens",
                    ));
                    ui.label(
                        RichText::new(l.t(
                            "Alle TLVs, einschließlich unbekannter Erweiterungen.",
                            "All TLVs, including unknown extensions.",
                            "Alle TLV's, inclusief onbekende uitbreidingen.",
                        ))
                        .weak(),
                    );
                    for (i, t) in a.tlvs.iter().enumerate() {
                        egui::CollapsingHeader::new(format!(
                            "{:02}  {} · {}",
                            t.kind,
                            t.name,
                            t.value_hex.split_whitespace().count()
                        ))
                        .id_salt(i)
                        .show(ui, |ui| {
                            for (k, v) in &t.fields {
                                ui.horizontal_wrapped(|ui| {
                                    ui.strong(k);
                                    ui.add(egui::Label::new(v).selectable(true).wrap());
                                });
                            }
                            ui.label(
                                RichText::new(l.t(
                                    "Rohdaten (Hex)",
                                    "Raw data (hex)",
                                    "Ruwe gegevens (hex)",
                                ))
                                .weak(),
                            );
                            ui.add(
                                egui::Label::new(RichText::new(&t.value_hex).monospace())
                                    .selectable(true)
                                    .wrap(),
                            );
                            if ui
                                .small_button(l.t("Kopieren", "Copy", "Kopiëren"))
                                .clicked()
                            {
                                ui.ctx().copy_text(t.value_hex.clone());
                            }
                        });
                    }

    }
    fn capture_backend_ui(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        ui.horizontal_wrapped(|ui| {
            ui.label(l.t("Empfangsweg", "Capture method", "Ontvangstmethode"));
            let old = self.settings.capture_backend;
            egui::ComboBox::from_id_salt("capture-backend")
                .selected_text(match old {
                    CaptureBackend::Auto => l.t("Automatisch", "Automatic", "Automatisch"),
                    CaptureBackend::Windows => "Windows Packet Monitor",
                    CaptureBackend::Npcap => "Npcap",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.settings.capture_backend,
                        CaptureBackend::Auto,
                        l.t("Automatisch (Windows / Npcap)",
                            "Automatic (Windows / Npcap)",
                            "Automatisch (Windows / Npcap)"));
                    ui.selectable_value(&mut self.settings.capture_backend,
                        CaptureBackend::Windows, "Windows Packet Monitor");
                    ui.selectable_value(&mut self.settings.capture_backend,
                        CaptureBackend::Npcap, "Npcap");
                });
            if old != self.settings.capture_backend {
                if self.engine.running() {
                    self.start();
                }
                self.save();
            }
            if !crate::platform::is_admin() &&
                self.settings.capture_backend != CaptureBackend::Npcap {
                ui.label(RichText::new(l.t(
                    "Windows-Empfang: Adminrechte erforderlich",
                    "Windows capture: admin rights required",
                    "Windows-ontvangst: beheerder vereist"
                )).small().weak());
                if ui.small_button(l.t(
                    "Als Admin starten …", "Run as admin …", "Als beheerder starten …"
                )).clicked() {
                    self.restart_elevated(ui.ctx());
                }
            }
        });
    }
    fn npcap_setup_ui(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        let status = match &self.npcap {
            Ok(report) => format!("{} · {} {}", report.version, report.devices.len(),
                l.t("Geräte", "devices", "apparaten")),
            Err(_) => l.t("Nicht installiert oder nicht verfügbar",
                "Not installed or unavailable", "Niet geïnstalleerd of niet beschikbaar").into(),
        };
        ui.label(RichText::new(format!("Npcap: {status}")).small().weak());
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(l.t(
                "Offizielle Downloadseite", "Official download", "Officiële download"
            ), "https://npcap.com/#download");
            if ui.small_button(l.t(
                "Installer ausführen …", "Run installer …", "Installatie starten …"
            )).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Npcap installer", &["exe"]).pick_file() {
                    self.stop();
                    let result = crate::platform::launch_npcap_installer(&path)
                        .map(|_| l.t(
                            "Npcap-Setup gestartet. Danach Diagnose erneut prüfen.",
                            "Npcap setup launched. Recheck diagnostics after installation.",
                            "Npcap-installatie gestart. Controleer daarna de diagnose.",
                        ).into());
                    self.notify(result);
                }
            }
        });
        ui.label(RichText::new(l.t(
            "Optional für Empfang, erforderlich zum Senden. Nur signierte Installationsdateien von npcap.com verwenden; Lizenz und UAC beachten.",
            "Optional for capture, required for advertising. Use signed installers only from npcap.com; review license and UAC.",
            "Optioneel voor ontvangst, vereist voor verzenden. Gebruik alleen ondertekende installatiebestanden van npcap.com."
        )).small().weak());
    }
    fn adapters_ui(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(ui,
            l.t("Netzwerkadapter", "Network adapters", "Netwerkadapters"), "");
        ui.horizontal(|ui| {
            if ui.small_button(l.t(
                "Aktualisieren", "Refresh", "Vernieuwen"
            )).clicked() {
                self.refresh_adapters();
            }
            if self.selection_dirty {
                if ui.button(l.t(
                    "Auswahl übernehmen", "Apply selection", "Selectie toepassen"
                )).clicked() {
                    if self.chosen.is_empty() {
                        self.stop();
                        self.selection_dirty = false;
                    } else {
                        self.start();
                    }
                }
                ui.label(RichText::new(l.t(
                    "Auswahl geändert", "Selection changed", "Selectie gewijzigd"
                )).small().weak());
            } else {
                ui.label(RichText::new(format!(
                    "{} / {} {}",
                    self.chosen.len(), self.adapters.len(),
                    l.t("ausgewählt", "selected", "geselecteerd"),
                )).small().weak());
            }
        });
        ui.add_space(5.);
        Self::surface(ui).show(ui, |ui| {
            self.capture_backend_ui(ui);
            egui::CollapsingHeader::new(l.t(
                "Diagnose und Npcap",
                "Diagnostics and Npcap",
                "Diagnose en Npcap"
            )).id_salt("capture-diagnostics").default_open(false).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button(l.t(
                        "Jetzt prüfen", "Check now", "Nu controleren"
                    )).clicked() {
                        self.native_report = Some(capture::inspect_packet_monitor());
                        self.npcap = capture::inspect_npcap();
                    }
                    ui.label(RichText::new(l.t(
                        "Windows Packet Monitor / Npcap",
                        "Windows Packet Monitor / Npcap",
                        "Windows Packet Monitor / Npcap"
                    )).small().weak());
                });
                if let Some(report) = &self.native_report {
                    match report {
                        Ok(info) => {
                            ui.add(egui::Label::new(
                                RichText::new(format!("Windows: {info}")).small()
                            ).selectable(true).wrap());
                        }
                        Err(problem) => {
                            ui.colored_label(Color32::from_rgb(204, 129, 69),
                                format!("Windows: {problem}"));
                        }
                    }
                }
                self.npcap_setup_ui(ui);
                for (id, problem) in &self.capture_errors {
                    ui.colored_label(Color32::from_rgb(204, 129, 69),
                        format!("{}: {problem}", self.name(id)));
                }
                for (id, detail) in &self.backend_notes {
                    ui.label(RichText::new(format!("{}: {detail}", self.name(id)))
                        .small().weak());
                }
            });
        });
        ui.add_space(10.);
        if self.adapters.is_empty() {
            ui.label(l.t(
                "Keine Netzwerkadapter gefunden. Über „Aktualisieren“ erneut suchen.",
                "No network adapters found. Try Refresh.",
                "Geen netwerkadapters gevonden. Probeer Vernieuwen."
            ));
            return;
        }
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8., 4.);
            egui::ScrollArea::both().id_salt("adapter-table")
                .auto_shrink([false, true]).show(ui, |ui| {
                    egui::Grid::new("adapter-grid").striped(true)
                        .num_columns(4).spacing([16., 6.]).show(ui, |ui| {
                        for heading in [
                            l.t("Ein", "On", "Aan"),
                            l.t("Schnittstelle", "Interface", "Interface"),
                            l.t("Verbindung", "Link", "Verbinding"),
                            l.t("Empfang", "Capture", "Ontvangst"),
                        ] {
                            ui.label(RichText::new(heading).small().strong().weak());
                        }
                        ui.end_row();
                        for a in &self.adapters {
                            let mut checked = self.chosen.contains(&a.id);
                            if ui.checkbox(&mut checked, "").on_hover_text(l.t(
                                "Empfang auf diesem Adapter", "Capture on this interface",
                                "Ontvangst op deze adapter"
                            )).changed() {
                                if checked { self.chosen.insert(a.id.clone()); }
                                else { self.chosen.remove(&a.id); }
                                self.selection_dirty = true;
                            }
                            ui.label(RichText::new(&a.name).strong())
                                .on_hover_text(format!("{}\n{}\n{}\n{:02X?}",
                                    a.description, a.id, a.ips.join(", "), a.mac));
                            ui.label(if a.up {
                                l.t("Verbunden", "Connected", "Verbonden")
                            } else {
                                l.t("Getrennt", "Disconnected", "Verbroken")
                            });
                            if let Some(error) = self.capture_errors.get(&a.id) {
                                ui.colored_label(Color32::from_rgb(211, 127, 76),
                                    l.t("Fehler", "Error", "Fout")).on_hover_text(error);
                            } else if self.ready.contains(&a.id) {
                                ui.label(self.backend_names.get(&a.id)
                                    .map(String::as_str).unwrap_or("Aktiv"));
                            } else {
                                ui.label(RichText::new(l.t(
                                    "Inaktiv", "Idle", "Inactief"
                                )).weak());
                            }
                            ui.end_row();
                        }
                    });
                });
        });
    }
    fn sender(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(ui,
            l.t("LLDP senden", "Advertise LLDP", "LLDP verzenden"), "");
        if self.adapters.is_empty() {
            ui.label(l.t(
                "Kein Netzwerkadapter verfügbar. Adapterliste aktualisieren.",
                "No network adapter available. Refresh adapters.",
                "Geen netwerkadapter beschikbaar. Vernieuw de adapters."
            ));
            if ui.button(l.t(
                "Zu den Adaptern", "Go to adapters", "Naar adapters"
            )).clicked() { self.page = 1; }
            return;
        }
        // Switching the sender's adapter must not discard unsaved profile edits.
        let previous = self.send_index;
        ui.horizontal_wrapped(|ui| {
            ui.label(l.t("Adapter", "Adapter", "Adapter"));
            egui::ComboBox::from_id_salt("sender-adapter")
                .selected_text(&self.adapters[self.send_index].name)
                .show_ui(ui, |ui| {
                    for (i, adapter) in self.adapters.iter().enumerate() {
                        ui.selectable_value(&mut self.send_index, i, &adapter.name);
                    }
                });
        });
        if previous != self.send_index {
            self.settings.profiles.insert(
                self.adapters[previous].id.clone(), self.profile.clone()
            );
            self.load_profile();
            self.save();
        }
        let adapter = self.adapters[self.send_index].clone();
        let active = self.tx_active.contains(&adapter.id);
        let pending = self.tx_pending.contains(&adapter.id);
        let prepared = self.settings.capture_backend == CaptureBackend::Npcap
            && self.ready.contains(&adapter.id);
        let valid = protocol::encode(&self.profile, adapter.mac, false);
        ui.add_space(5.);
        Self::surface(ui).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let state = if active {
                    l.t("Sender aktiv", "Advertising active", "Verzenden actief")
                } else {
                    l.t("Sender aus", "Advertising off", "Verzenden uit")
                };
                ui.label(RichText::new(state).strong().color(if active {
                    GREEN
                } else {
                    ui.visuals().text_color()
                }));
                ui.separator();
                if prepared || active {
                    if ui.add_enabled(!pending && (active || valid.is_ok()),
                        egui::Button::new(if active {
                            l.t("Senden stoppen", "Stop advertising", "Stop verzenden")
                        } else {
                            l.t("Senden einschalten", "Enable advertising", "Verzenden starten")
                        })
                    ).clicked() {
                        let profile = if active { None } else { Some(self.profile.clone()) };
                        match self.engine.advertise(&adapter.id, profile) {
                            Ok(()) => { self.tx_pending.insert(adapter.id.clone()); }
                            Err(error) => self.notify(Err(error)),
                        }
                    }
                } else if ui.button(l.t(
                    "Sender vorbereiten", "Prepare advertising", "Verzenden voorbereiden"
                )).clicked() {
                    // Prepare only: never transmit without a separate explicit click.
                    self.settings.capture_backend = CaptureBackend::Npcap;
                    self.chosen.insert(adapter.id.clone());
                    self.start();
                    self.save();
                }
                if pending { ui.spinner(); }
            });
            if !prepared && !active {
                ui.label(RichText::new(l.t(
                    "Zum Senden wird Npcap benötigt. „Sender vorbereiten“ wählt Npcap und aktiviert diesen Adapter, sendet aber noch nichts.",
                    "Sending requires Npcap. Prepare selects Npcap and enables this adapter; it does not transmit.",
                    "Verzenden vereist Npcap. Voorbereiden selecteert Npcap en de adapter, maar verzendt nog niets."
                )).small().weak());
                if self.npcap.is_err() {
                    ui.hyperlink_to(l.t(
                        "Npcap herunterladen", "Download Npcap", "Npcap downloaden"
                    ), "https://npcap.com/#download");
                }
            } else if active {
                ui.label(RichText::new(l.t(
                    "Profil während des Sendens gesperrt. Im Tray läuft der Sender weiter.",
                    "Profile locked during advertising. The tray keeps advertising active.",
                    "Profiel geblokkeerd tijdens verzenden. Systeemvak houdt verzending actief."
                )).small().weak());
            }
        });
        ui.add_space(10.);
        Self::surface(ui).show(ui, |ui| {
            ui.label(RichText::new(l.t(
                "Ankündigungsprofil", "Advertisement profile", "Advertentieprofiel"
            )).strong());
            ui.add_space(5.);
            ui.add_enabled_ui(!active && !pending, |ui| {
                egui::Grid::new("tx-basic-fields")
                    .num_columns(2).spacing([13., 7.]).show(ui, |ui| {
                    for (label, value) in [
                        (l.t("Systemname", "System name", "Systeemnaam"), &mut self.profile.system_name),
                        (l.t("Beschreibung", "Description", "Beschrijving"), &mut self.profile.description),
                        (l.t("Portbeschreibung", "Port description", "Poortbeschrijving"),
                            &mut self.profile.port_description),
                    ] {
                        ui.label(label);
                        ui.add(egui::TextEdit::singleline(value).desired_width(285.));
                        ui.end_row();
                    }
                });
                egui::CollapsingHeader::new(l.t(
                    "Technische Einstellungen",
                    "Technical settings",
                    "Technische instellingen"
                )).id_salt("tx-advanced").default_open(false).show(ui, |ui| {
                    egui::Grid::new("tx-advanced-fields")
                        .num_columns(2).spacing([14., 7.]).show(ui, |ui| {
                        ui.label("Chassis ID");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.profile.chassis_subtype)
                                .range(1..=7));
                            ui.add(egui::TextEdit::singleline(&mut self.profile.chassis_value)
                                .desired_width(225.));
                        });
                        ui.end_row();
                        ui.label("Port ID");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.profile.port_subtype)
                                .range(1..=7));
                            ui.add(egui::TextEdit::singleline(&mut self.profile.port_value)
                                .desired_width(225.));
                        });
                        ui.end_row();
                        ui.label("TTL (s)");
                        ui.add(egui::DragValue::new(&mut self.profile.ttl).range(1..=65535));
                        ui.end_row();
                        ui.label(l.t("Intervall (s)", "Interval (s)", "Interval (s)"));
                        ui.add(egui::DragValue::new(&mut self.profile.interval).range(1..=3600));
                        ui.end_row();
                        ui.label(l.t("Ziel-MAC", "Destination MAC", "Doel-MAC"));
                        egui::ComboBox::from_id_salt("tx-destination")
                            .selected_text(&self.profile.destination).show_ui(ui, |ui| {
                            for mac in [
                                "01:80:C2:00:00:0E",
                                "01:80:C2:00:00:03",
                                "01:80:C2:00:00:00",
                            ] {
                                ui.selectable_value(&mut self.profile.destination,
                                    mac.to_owned(), mac);
                            }
                        });
                        ui.end_row();
                    });
                    ui.label(RichText::new(l.t(
                        "Subtypen: Chassis MAC=4, IP=5, lokal=7 · Port MAC=3, IP=4, Name=5, lokal=7",
                        "Subtypes: chassis MAC=4, IP=5, local=7 · port MAC=3, IP=4, name=5, local=7",
                        "Subtypen: chassis MAC=4, IP=5, lokaal=7 · poort MAC=3, IP=4, naam=5, lokaal=7"
                    )).small().weak());
                    egui::CollapsingHeader::new(l.t(
                        "Zusätzliche TLVs", "Additional TLVs", "Extra TLV's"
                    )).id_salt("tx-tlvs").show(ui, |ui| {
                        let mut remove = None;
                        for (i, (kind, hex)) in self.profile.optional.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(kind).range(7..=127));
                                ui.add(egui::TextEdit::singleline(hex)
                                    .font(egui::TextStyle::Monospace).desired_width(260.));
                                if ui.small_button("×").clicked() { remove = Some(i); }
                            });
                        }
                        if let Some(index) = remove { self.profile.optional.remove(index); }
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.extra_kind).range(7..=127));
                            ui.add(egui::TextEdit::singleline(&mut self.extra_hex)
                                .hint_text("00 80 C2 01 00 2A").desired_width(260.));
                            if ui.button(l.t(
                                "TLV hinzufügen", "Add TLV", "TLV toevoegen"
                            )).clicked() {
                                self.profile.optional.push((
                                    self.extra_kind, self.extra_hex.clone()
                                ));
                                self.extra_hex.clear();
                            }
                        });
                        ui.menu_button(l.t(
                            "Vorlage", "Template", "Sjabloon"
                        ), |ui| {
                            for (name, kind, hex) in [
                                ("System capabilities", 7, "00 80 00 80"),
                                ("IEEE 802.1 PVID (42)", 127, "00 80 C2 01 00 2A"),
                                ("IEEE 802.3 max frame size (1518)", 127, "00 12 0F 04 05 EE"),
                                ("LLDP-MED capabilities", 127, "00 12 BB 01 00 01 01"),
                                ("LLDP-MED model", 127, "00 12 BB 0A 57 69 6E 4C 4C 44 50"),
                            ] {
                                if ui.button(name).clicked() {
                                    self.profile.optional.push((kind, hex.into()));
                                    ui.close();
                                }
                            }
                        });
                    });
                });
            });
            ui.add_space(5.);
            match valid {
                Ok(frame) => {
                    ui.label(RichText::new(format!(
                        "{} · {} Byte",
                        l.t("Paket gültig", "Valid packet", "Geldig pakket"), frame.len()
                    )).small().weak());
                    egui::CollapsingHeader::new(l.t(
                        "Paketvorschau", "Packet preview", "Pakketvoorbeeld"
                    )).id_salt("tx-preview").show(ui, |ui| {
                        ui.add(egui::Label::new(
                            RichText::new(protocol::hex(&frame)).monospace()
                        ).selectable(true).wrap());
                    });
                }
                Err(error) => {
                    ui.colored_label(Color32::from_rgb(211, 127, 76),
                        format!("{}: {error}",
                            l.t("Paket ungültig", "Invalid packet", "Ongeldig pakket")));
                }
            }
        });
        ui.add_space(4.);
        ui.label(RichText::new(l.t(
            "Änderungen werden automatisch gespeichert.",
            "Changes are saved automatically.",
            "Wijzigingen worden automatisch opgeslagen."
        )).small().weak());
    }
    fn history(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(ui,
            l.t("Protokoll", "Activity log", "Logboek"), "");
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!(
                "{} {}", self.monitor.events.len(),
                l.t("Ereignisse", "events", "gebeurtenissen"),
            )).small().weak());
            self.file_menu(ui);
            if !self.monitor.events.is_empty() &&
                ui.small_button(l.t("Leeren …", "Clear …", "Wissen …")).clicked() {
                self.confirm_clear_history = true;
            }
        });
        ui.add_space(6.);
        if self.monitor.events.is_empty() {
            ui.label(l.t(
                "Noch keine Ereignisse.", "No events yet.", "Nog geen gebeurtenissen."
            ));
            return;
        }
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8., 4.);
            egui::Grid::new("event-table").striped(true)
                .num_columns(3).spacing([16., 5.]).show(ui, |ui| {
                for heading in [
                    l.t("Zeit (UTC)", "Time (UTC)", "Tijd (UTC)"),
                    l.t("Adapter", "Adapter", "Adapter"),
                    l.t("Ereignis", "Event", "Gebeurtenis"),
                ] {
                    ui.label(RichText::new(heading).small().strong().weak());
                }
                ui.end_row();
                for event in self.monitor.events.iter().take(250) {
                    ui.monospace(event.at.format("%d.%m. %H:%M:%S").to_string());
                    ui.label(&event.adapter);
                    let response = ui.selectable_label(false, event_name(l, &event.action))
                        .on_hover_text(&event.detail);
                    if response.double_clicked() {
                        self.selected_event = Some(event.clone());
                    }
                    ui.end_row();
                }
            });
        });
        if self.monitor.events.len() > 250 {
            ui.label(RichText::new(l.t(
                "Die neuesten 250 Ereignisse werden angezeigt.",
                "Showing the latest 250 events.",
                "De nieuwste 250 gebeurtenissen worden getoond."
            )).small().weak());
        }
    }
    fn settings_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.settings.language;
        Self::title(ui, l.t(
            "Einstellungen", "Settings", "Instellingen"
        ), "");
        let old_language = self.settings.language;
        let old_theme = self.settings.theme;
        let old_tray = self.settings.close_to_tray;
        let old_history = self.settings.keep_history;
        Self::surface(ui).show(ui, |ui| {
            egui::Grid::new("preferences-grid")
                .num_columns(2).spacing([18., 12.]).show(ui, |ui| {
                ui.label(l.t("Sprache", "Language", "Taal"));
                egui::ComboBox::from_id_salt("setting-language")
                    .selected_text(match self.settings.language {
                        L::Deutsch => "Deutsch",
                        L::English => "English",
                        L::Nederlands => "Nederlands",
                    }).show_ui(ui, |ui| {
                        for (lang, label) in [
                            (L::Deutsch, "Deutsch"),
                            (L::English, "English"),
                            (L::Nederlands, "Nederlands"),
                        ] {
                            ui.selectable_value(&mut self.settings.language, lang, label);
                        }
                    });
                ui.end_row();
                ui.label(l.t("Design", "Appearance", "Uiterlijk"));
                egui::ComboBox::from_id_salt("setting-theme")
                    .selected_text(match self.settings.theme {
                        Theme::System => l.t("System", "System", "Systeem"),
                        Theme::Light => l.t("Hell", "Light", "Licht"),
                        Theme::Dark => l.t("Dunkel", "Dark", "Donker"),
                    }).show_ui(ui, |ui| {
                        for (theme, label) in [
                            (Theme::System, l.t("System", "System", "Systeem")),
                            (Theme::Light, l.t("Hell", "Light", "Licht")),
                            (Theme::Dark, l.t("Dunkel", "Dark", "Donker")),
                        ] {
                            ui.selectable_value(&mut self.settings.theme, theme, label);
                        }
                    });
                ui.end_row();
            });
            ui.separator();
            ui.checkbox(&mut self.settings.close_to_tray, l.t(
                "Beim Schließen im Infobereich weiterlaufen",
                "Continue in tray when window closes",
                "Verdergaan in systeemvak bij sluiten"
            ));
            ui.checkbox(&mut self.settings.keep_history, l.t(
                "Protokoll lokal aufbewahren",
                "Keep activity log locally",
                "Logboek lokaal bewaren"
            ));
        });
        if old_language != self.settings.language ||
            old_theme != self.settings.theme ||
            old_tray != self.settings.close_to_tray ||
            old_history != self.settings.keep_history {
            if old_history && !self.settings.keep_history {
                let _ = std::fs::remove_file(settings::data_dir().join("history.json"));
            }
            if old_theme != self.settings.theme {
                self.apply_theme(ctx);
            }
            self.save();
        }
        ui.add_space(10.);
        egui::CollapsingHeader::new(l.t(
            "Zugriff und Datenschutz", "Access and privacy", "Toegang en privacy"
        )).id_salt("access-privacy").show(ui, |ui| {
            let admin = crate::platform::is_admin();
            ui.label(if admin {
                l.t("Administratorrechte aktiv",
                    "Running as administrator", "Beheerdersrechten actief")
            } else {
                l.t("Standardbenutzer",
                    "Standard user", "Standaardgebruiker")
            });
            if !admin {
                if ui.small_button(l.t(
                    "Als Administrator neu starten …",
                    "Restart as administrator …",
                    "Opnieuw starten als beheerder …"
                )).clicked() {
                    self.restart_elevated(ctx);
                }
            } else if ui.small_button(l.t(
                "Als Standardbenutzer neu starten …",
                "Restart as standard user …",
                "Herstart als standaardgebruiker …"
            )).clicked() {
                match crate::platform::standard_access() {
                    Ok(()) => {
                        self.exit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Err(error) => self.notify(Err(error)),
                }
            }
            ui.label(RichText::new(l.t(
                "Keine Telemetrie. Einstellungen und Protokoll bleiben lokal.",
                "No telemetry. Preferences and activity remain local.",
                "Geen telemetrie. Instellingen en logboek blijven lokaal."
            )).small().weak());
            ui.add(egui::Label::new(settings::data_dir().display().to_string())
                .selectable(true).wrap());
        });
    }
    fn about(&self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        ui.horizontal(|ui| {
            ui.add(egui::Image::new((self.logo.id(), Vec2::splat(128.))));
            ui.vertical(|ui| {
                ui.label(RichText::new("WinLLDP").size(38.).strong());
                ui.label(format!(
                    "Version {} · Windows 10 / 11 · x64",
                    env!("CARGO_PKG_VERSION")
                ));
                ui.label(l.t(
                    "Ein offenes Ohr für dein Netzwerk.",
                    "An open ear for your network.",
                    "Een luisterend oor voor je netwerk.",
                ));
            });
        });
        ui.add_space(20.);
        ui.label(
            RichText::new(l.t(
                "Entwickelt von Dominik Tröster",
                "Developed by Dominik Tröster",
                "Ontwikkeld door Dominik Tröster",
            ))
            .size(20.)
            .strong(),
        );
        ui.label("Verbandsgemeinde Otterbach-Otterberg");
        ui.add_space(12.);
        ui.strong(l.t(
            "Freie Software · MIT-Lizenz",
            "Free software · MIT License",
            "Vrije software · MIT-licentie",
        ));
        ui.label(l.t("Der eigene Programmcode steht unter der MIT-Lizenz. Die Software wird ohne Gewährleistung bereitgestellt.","The application code is licensed under MIT. The software is provided without warranty.","De programmacode valt onder de MIT-licentie. De software wordt zonder garantie geleverd."));
        ui.hyperlink_to(
            "GitHub · WinLLDP",
            "https://github.com/derdigitalaffine/winlldp",
        );
        egui::CollapsingHeader::new(l.t(
            "MIT-Lizenz lesen",
            "Read MIT License",
            "MIT-licentie lezen",
        ))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(include_str!("../LICENSE"))
                    .selectable(true)
                    .wrap(),
            );
        });
        egui::CollapsingHeader::new(l.t(
            "Drittanbieter und Hinweise",
            "Third-party notices",
            "Derden en kennisgevingen",
        ))
        .show(ui, |ui| {
            ui.label(include_str!("../THIRD_PARTY_NOTICES.md"));
        });
    }
}
fn event_name(l: L, s: &str) -> &str {
    match s {
        "discovered" => l.t("Nachbar entdeckt", "Neighbor discovered", "Buur ontdekt"),
        "changed" => l.t("Nachbar geändert", "Neighbor changed", "Buur gewijzigd"),
        "expired" => l.t(
            "Nachbar abgelaufen / Verbindung verloren",
            "Neighbor expired / connection lost",
            "Buur verlopen / verbinding verloren",
        ),
        "shutdown" => l.t("Nachbar abgemeldet", "Neighbor shutdown", "Buur afgemeld"),
        "capture_started" => l.t("Empfang gestartet", "Capture started", "Ontvangst gestart"),
        "capture_stopped" => l.t("Empfang gestoppt", "Capture stopped", "Ontvangst gestopt"),
        "tx_started" => l.t(
            "Sender eingeschaltet",
            "Advertising enabled",
            "Verzenden ingeschakeld",
        ),
        "tx_stopped" => l.t(
            "Sender ausgeschaltet",
            "Advertising disabled",
            "Verzenden uitgeschakeld",
        ),
        "error" => l.t("Fehler", "Error", "Fout"),
        _ => s,
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some(path) = &self.qa_output {
            self.qa_frames += 1;
            if self.qa_frames == 8 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
            let screenshot = ctx.input(|i| {
                i.events.iter().find_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(im) = screenshot {
                let rgba: Vec<u8> = im.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    path,
                    &rgba,
                    im.width() as u32,
                    im.height() as u32,
                    image::ColorType::Rgba8,
                )
                .expect("QA screenshot save");
                self.exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.poll();
        let l = self.settings.language;
        #[cfg(windows)]
        if let Some(t) = &self.tray {
            t.show
                .set_text(l.t("WinLLDP öffnen", "Open WinLLDP", "WinLLDP openen"));
            t.exit.set_text(l.t("Beenden", "Quit", "Afsluiten"));
            let (show, exit) = t.poll();
            if show {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            if exit {
                self.exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if ctx.input(|i| i.viewport().close_requested())
            && !self.exit
            && self.settings.close_to_tray
        {
            #[cfg(windows)]
            if self.tray.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }
        egui::TopBottomPanel::top("top")
            .exact_height(49.)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add(egui::Image::new((self.logo.id(), Vec2::splat(27.))));
                    ui.label(RichText::new("WinLLDP").size(16.).strong());
                    ui.separator();
                    for (page, name) in [
                        (0, l.t("Nachbarn", "Neighbors", "Buren")),
                        (1, l.t("Adapter", "Adapters", "Adapters")),
                        (2, l.t("Senden", "Advertise", "Verzenden")),
                    ] {
                        ui.selectable_value(&mut self.page, page, name);
                    }
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.menu_button(l.t("Mehr", "More", "Meer"), |ui| {
                                for (page, label) in [
                                    (3, l.t("Protokoll", "Activity log", "Logboek")),
                                    (4, l.t("Einstellungen", "Settings", "Instellingen")),
                                    (5, l.t("Über WinLLDP", "About", "Over WinLLDP")),
                                ] {
                                    if ui.button(label).clicked() {
                                        self.page = page;
                                        ui.close();
                                    }
                                }
                                ui.separator();
                                if ui.button(l.t("Beenden", "Quit", "Afsluiten")).clicked() {
                                    self.exit = true;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    ui.close();
                                }
                            });
                            let running = self.engine.running();
                            let label = if running {
                                l.t("Stopp", "Stop", "Stop")
                            } else {
                                l.t("Empfang starten", "Start capture", "Ontvangst starten")
                            };
                            let action = egui::Button::new(if running {
                                RichText::new(label)
                            } else {
                                RichText::new(label).color(Color32::WHITE)
                            });
                            let action = if running { action } else {
                                action.fill(Color32::from_rgb(0, 111, 86))
                            };
                            if ui.add_enabled(running || !self.chosen.is_empty(), action)
                                .clicked() {
                                if running {
                                    self.stop();
                                } else {
                                    self.start();
                                }
                            }
                        }
                    );
                });
            });
        egui::TopBottomPanel::bottom("status")
            .exact_height(27.)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let status = if !self.capture_errors.is_empty() {
                        l.t("Empfangsfehler", "Capture error", "Ontvangstfout")
                    } else if !self.ready.is_empty() {
                        l.t("Empfang aktiv", "Listening", "Ontvangst actief")
                    } else {
                        l.t("Empfang inaktiv", "Not listening", "Ontvangst inactief")
                    };
                    ui.label(RichText::new(status).small().color(
                        if !self.capture_errors.is_empty() {
                            Color32::from_rgb(211, 127, 76)
                        } else if !self.ready.is_empty() {
                            GREEN
                        } else {
                            ui.visuals().weak_text_color()
                        }
                    ));
                    ui.separator();
                    ui.label(RichText::new(format!(
                        "{} {} · {} {}",
                        self.ready.len(),
                        l.t("Adapter", "adapters", "adapters"),
                        self.monitor.neighbors.len(),
                        l.t("Nachbarn", "neighbors", "buren"),
                    )).small().weak());
                    if !self.tx_active.is_empty() {
                        ui.separator();
                        ui.label(RichText::new(l.t(
                            "Sender aktiv", "Advertising", "Verzenden actief"
                        )).small().color(GREEN));
                    }
                    if !self.message.is_empty() {
                        let short = self.message.chars().take(86).collect::<String>();
                        ui.separator();
                        ui.label(RichText::new(short).small().color(
                            if self.is_error {
                                Color32::from_rgb(211, 127, 76)
                            } else {
                                ui.visuals().weak_text_color()
                            }
                        )).on_hover_text(&self.message);
                    }
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(12.))
            .show(ctx, |ui| {
                if self.page == 0 {
                    self.overview(ui);
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt(("workspace", self.page))
                        .auto_shrink([false, false])
                        .show(ui, |ui| match self.page {
                            1 => self.adapters_ui(ui),
                            2 => self.sender(ui),
                            3 => self.history(ui),
                            4 => self.settings_ui(ui, ctx),
                            _ => self.about(ui),
                        });
                }
            });
        // Keep LLDP metadata off the main table. Double-click (or Enter) opens detail.
        if self.detail_open {
            // Track a stable adapter/chassis/port identity rather than a vector index:
            // TTL expiration can reorder neighbors while the dialog is open.
            let selected_index = self.detail_key.as_ref().and_then(|(adapter, chassis, port)| {
                self.monitor.neighbors.iter().position(|n| {
                    &n.adapter == adapter
                        && &n.advertisement.chassis_id == chassis
                        && &n.advertisement.port_id == port
                })
            });
            if let Some(selected_index) = selected_index {
                let mut open = self.detail_open;
                egui::Window::new(l.t("LLDP-Nachbar · Details", "LLDP neighbor · details", "LLDP-buur · details"))
                    .id(egui::Id::new("neighbor-detail-dialog"))
                    .open(&mut open)
                    .resizable(true)
                    .default_size([610., 440.])
                    .min_width(410.)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().id_salt("neighbor-detail-scroll")
                            .show(ui, |ui| self.neighbor_details(ui, selected_index));
                    });
                self.detail_open = open;
            } else {
                self.detail_open = false;
                self.detail_key = None;
            }
        }
        if self.last_save.elapsed() > Duration::from_secs(10) {
            self.save();
        }
        ctx.request_repaint_after(Duration::from_millis(200));
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.stop();
        self.save();
    }
}
