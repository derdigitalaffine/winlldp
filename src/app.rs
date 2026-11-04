use eframe::egui::{self, Color32, RichText, Vec2};
use std::{
    collections::{BTreeSet, VecDeque},
    time::{Duration, Instant},
};
use winlldp::{
    capture::{self, Adapter, Engine, Notice},
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
            s.spacing.item_spacing = Vec2::new(10., 10.);
            s.spacing.button_padding = Vec2::new(14., 9.);
            s.spacing.interact_size.y = 32.;
            s.visuals.selection.bg_fill = GREEN.gamma_multiply(0.5);
            s.visuals.hyperlink_color = if dark {
                GREEN
            } else {
                Color32::from_rgb(0, 107, 83)
            };
            s.text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(25.));
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(15.));
            s.text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(15.));
        });
    }
    fn start(&mut self) {
        self.ready.clear();
        self.tx_active.clear();
        self.tx_pending.clear();
        let devices = self
            .adapters
            .iter()
            .filter(|a| self.chosen.contains(&a.id))
            .cloned()
            .collect::<Vec<_>>();
        let r = self.engine.start(&devices).map(|_| {
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
                Notice::Status(id, s) => {
                    if s == "capture_started" {
                        self.ready.insert(id.clone());
                    } else {
                        self.ready.remove(&id);
                        self.tx_active.remove(&id);
                        self.tx_pending.remove(&id);
                    }
                    self.monitor.log(&self.name(&id), &s, String::new());
                }
                Notice::Error(id, e) => {
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
    fn title(ui: &mut egui::Ui, title: &str, description: &str) {
        ui.heading(title);
        ui.label(RichText::new(description).weak());
        ui.add_space(14.);
    }
    fn card(ui: &mut egui::Ui, label: &str, value: &str) {
        egui::Frame::group(ui.style())
            .inner_margin(16.)
            .corner_radius(12)
            .show(ui, |ui| {
                ui.label(RichText::new(label).weak());
                ui.label(RichText::new(value).size(26.).strong());
            });
    }
    fn overview(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(
            ui,
            l.t(
                "Dein Anschluss. Alle Details.",
                "Your connection. Every detail.",
                "Je verbinding. Alle details.",
            ),
            l.t(
                "LLDP-Nachbarn in Echtzeit entdecken und verstehen.",
                "Discover and inspect LLDP neighbors in real time.",
                "Ontdek en bekijk LLDP-buren in realtime.",
            ),
        );
        ui.columns(3, |c| {
            Self::card(
                &mut c[0],
                l.t("NACHBARN", "NEIGHBORS", "BUREN"),
                &self.monitor.neighbors.len().to_string(),
            );
            Self::card(
                &mut c[1],
                l.t("AKTIVE ADAPTER", "ACTIVE ADAPTERS", "ACTIEVE ADAPTERS"),
                &self.ready.len().to_string(),
            );
            Self::card(
                &mut c[2],
                l.t("LLDP-PAKETE", "LLDP PACKETS", "LLDP-PAKKETTEN"),
                &self.monitor.received.to_string(),
            );
        });
        ui.add_space(14.);
        ui.horizontal(|ui| {
            ui.label(l.t("Suchen", "Search", "Zoeken"));
            ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(l.t(
                        "Switch, Port oder Adapter …",
                        "Switch, port or adapter …",
                        "Switch, poort of adapter …",
                    ))
                    .desired_width(260.),
            );
            if ui
                .button(l.t("PCAP öffnen", "Open PCAP", "PCAP openen"))
                .clicked()
            {
                self.import();
            }
            ui.menu_button(l.t("Exportieren", "Export", "Exporteren"), |ui| {
                for kind in ["CSV", "JSON", "PCAP"] {
                    if ui.button(kind).clicked() {
                        self.export(kind);
                        ui.close();
                    }
                }
            });
        });
        ui.add_space(8.);
        if self.monitor.neighbors.is_empty() {
            egui::Frame::group(ui.style()).inner_margin(28.).corner_radius(12).show(ui,|ui|{ui.label(RichText::new(l.t("Noch kein Nachbar entdeckt","No neighbors discovered yet","Nog geen buren ontdekt")).size(22.).strong());ui.label(l.t("Ein LLDP-fähiger Switch muss Ankündigungen senden. Prüfe Kabel, Adapterauswahl und Npcap. Die erste Ankündigung kann einige Sekunden dauern.","A switch must advertise LLDP. Check cable, adapter selection and Npcap. The first advertisement can take a few seconds.","Een switch moet LLDP-advertenties sturen. Controleer kabel, adapterselectie en Npcap. De eerste advertentie kan even duren."));ui.hyperlink_to(l.t("Npcap herunterladen","Download Npcap","Npcap downloaden"),"https://npcap.com/#download");});
            return;
        }
        self.selected = self.selected.min(self.monitor.neighbors.len() - 1);
        ui.columns(2, |cols| {
            egui::ScrollArea::vertical()
                .id_salt("neighbors")
                .max_height(cols[0].available_height())
                .show(&mut cols[0], |ui| {
                    let query = self.query.to_lowercase();
                    for (i, n) in self.monitor.neighbors.iter().enumerate() {
                        let a = &n.advertisement;
                        let name = if a.system_name.is_empty() {
                            &a.chassis_id
                        } else {
                            &a.system_name
                        };
                        let label = format!(
                            "{name}\n{}  {}\n{}",
                            l.t("Port", "Port", "Poort"),
                            a.port_id,
                            self.name(&n.adapter)
                        );
                        if !query.is_empty() && !label.to_lowercase().contains(&query) {
                            continue;
                        }
                        let fill = if i == self.selected {
                            GREEN.gamma_multiply(0.16)
                        } else {
                            ui.visuals().faint_bg_color
                        };
                        egui::Frame::new()
                            .fill(fill)
                            .inner_margin(14.)
                            .corner_radius(10)
                            .show(ui, |ui| {
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 78.],
                                        egui::Button::new(RichText::new(label).size(16.))
                                            .selected(i == self.selected)
                                            .frame(false),
                                    )
                                    .clicked()
                                {
                                    self.selected = i;
                                }
                            });
                        ui.add_space(5.);
                    }
                });
            egui::ScrollArea::vertical()
                .id_salt("details")
                .show(&mut cols[1], |ui| {
                    let n = &self.monitor.neighbors[self.selected];
                    let a = &n.advertisement;
                    egui::Frame::group(ui.style())
                        .inner_margin(18.)
                        .corner_radius(12)
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
                });
        });
    }
    fn adapters_ui(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(
            ui,
            l.t("Netzwerkadapter", "Network adapters", "Netwerkadapters"),
            l.t(
                "Wähle die Schnittstellen für den parallelen Empfang.",
                "Select interfaces for concurrent capture.",
                "Selecteer interfaces voor gelijktijdige ontvangst.",
            ),
        );
        ui.horizontal(|ui| {
            if ui
                .button(l.t("Neu erkennen", "Refresh adapters", "Adapters vernieuwen"))
                .clicked()
            {
                self.stop();
                match capture::adapters() {
                    Ok(a) => {
                        self.adapters = a;
                        self.chosen
                            .retain(|id| self.adapters.iter().any(|a| &a.id == id));
                        self.send_index = 0;
                        self.load_profile();
                    }
                    Err(e) => self.notify(Err(e)),
                }
            }
            if ui
                .button(l.t("Auswahl anwenden", "Apply selection", "Selectie toepassen"))
                .clicked()
            {
                self.start();
            }
        });
        for a in &self.adapters {
            egui::Frame::group(ui.style())
                .inner_margin(16.)
                .corner_radius(12)
                .show(ui, |ui| {
                    let mut chosen = self.chosen.contains(&a.id);
                    ui.horizontal(|ui| {
                        if ui
                            .checkbox(&mut chosen, RichText::new(&a.name).strong().size(18.))
                            .changed()
                        {
                            if chosen {
                                self.chosen.insert(a.id.clone());
                            } else {
                                self.chosen.remove(&a.id);
                            }
                        }
                        ui.label(if self.ready.contains(&a.id) {
                            l.t("Empfang aktiv", "Capture active", "Ontvangst actief")
                        } else if a.up {
                            l.t("Verbunden", "Connected", "Verbonden")
                        } else {
                            l.t("Nicht verbunden", "Disconnected", "Niet verbonden")
                        });
                    });
                    ui.label(&a.description);
                    ui.monospace(
                        a.mac
                            .iter()
                            .map(|v| format!("{v:02X}"))
                            .collect::<Vec<_>>()
                            .join(":"),
                    );
                    ui.label(a.ips.join(" · "));
                    ui.add(
                        egui::Label::new(RichText::new(&a.id).small().weak())
                            .selectable(true)
                            .wrap(),
                    );
                });
        }
        if self.adapters.is_empty() {
            ui.label(l.t(
                "Keine Ethernetadapter gefunden.",
                "No Ethernet adapters found.",
                "Geen ethernetadapters gevonden.",
            ));
        }
    }
    fn sender(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(
            ui,
            l.t("LLDP senden", "Advertise LLDP", "LLDP verzenden"),
            l.t(
                "Dein Rechner als LLDP-Nachbar. Nur aktiv, wenn du es einschaltest.",
                "Your computer as an LLDP neighbor. Active only when you enable it.",
                "Je computer als LLDP-buur. Alleen actief als je dit inschakelt.",
            ),
        );
        if self.adapters.is_empty() {
            ui.label(l.t(
                "Zum Senden ist ein Ethernetadapter mit Npcap-Zugriff erforderlich.",
                "Sending requires an Ethernet adapter with Npcap access.",
                "Verzenden vereist een ethernetadapter met Npcap-toegang.",
            ));
            return;
        }
        let old = self.send_index;
        egui::ComboBox::from_id_salt("sender-adapter")
            .selected_text(&self.adapters[self.send_index].name)
            .show_ui(ui, |ui| {
                for (i, a) in self.adapters.iter().enumerate() {
                    ui.selectable_value(&mut self.send_index, i, &a.name);
                }
            });
        if old != self.send_index {
            self.settings
                .profiles
                .insert(self.adapters[old].id.clone(), self.profile.clone());
            self.load_profile();
        }
        let a = self.adapters[self.send_index].clone();
        let active = self.tx_active.contains(&a.id);
        let pending = self.tx_pending.contains(&a.id);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if active {
                    l.t("SENDEN AKTIV", "ADVERTISING ACTIVE", "VERZENDEN ACTIEF")
                } else {
                    l.t("SENDEN AUS", "ADVERTISING OFF", "VERZENDEN UIT")
                })
                .strong()
                .color(if active {
                    GREEN
                } else {
                    ui.visuals().text_color()
                }),
            );
            let valid = protocol::encode(&self.profile, a.mac, false);
            if ui
                .add_enabled(
                    !pending && (active || valid.is_ok()) && self.ready.contains(&a.id),
                    egui::Button::new(if active {
                        l.t("Sender stoppen", "Stop advertising", "Verzenden stoppen")
                    } else {
                        l.t(
                            "Sender einschalten",
                            "Enable advertising",
                            "Verzenden inschakelen",
                        )
                    }),
                )
                .clicked()
            {
                let p = if active {
                    None
                } else {
                    Some(self.profile.clone())
                };
                match self.engine.advertise(&a.id, p) {
                    Ok(()) => {
                        self.tx_pending.insert(a.id.clone());
                    }
                    Err(e) => self.notify(Err(e)),
                }
            }
            if pending {
                ui.spinner();
            }
            if ui
                .button(l.t("Profil speichern", "Save profile", "Profiel opslaan"))
                .clicked()
            {
                self.save();
            }
        });
        if !self.ready.contains(&a.id) {
            ui.label(
                RichText::new(l.t(
                    "Zuerst Empfang für diesen Adapter starten. Rechte und Npcap prüfen.",
                    "Start capture on this adapter first. Check permissions and Npcap.",
                    "Start eerst ontvangst op deze adapter. Controleer rechten en Npcap.",
                ))
                .color(Color32::from_rgb(220, 155, 60)),
            );
        }
        if active {
            ui.label(l.t("Zum Ändern zuerst stoppen. Im Tray wird weitergesendet, nach einem Neustart bleibt Senden aus.","Stop before editing. Advertising continues in the tray; after restart it stays off.","Stop vóór bewerken. Verzenden gaat door in het systeemvak; na herstart blijft het uit."));
        }
        ui.add_enabled_ui(!active&&!pending,|ui|{
   egui::Grid::new("tx-fields").num_columns(2).spacing([16.,10.]).show(ui,|ui|{
    for(label,value)in [(l.t("Gerätename","System name","Systeemnaam"),&mut self.profile.system_name),(l.t("Beschreibung","Description","Beschrijving"),&mut self.profile.description),(l.t("Portbeschreibung","Port description","Poortbeschrijving"),&mut self.profile.port_description)]{ui.label(label);ui.add(egui::TextEdit::singleline(value).desired_width(360.));ui.end_row();}
    ui.label("Chassis ID");ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut self.profile.chassis_subtype).range(1..=7));ui.text_edit_singleline(&mut self.profile.chassis_value);});ui.end_row();
    ui.label("Port ID");ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut self.profile.port_subtype).range(1..=7));ui.text_edit_singleline(&mut self.profile.port_value);});ui.end_row();
    ui.label("TTL / s");ui.add(egui::DragValue::new(&mut self.profile.ttl).range(1..=65535));ui.end_row();ui.label(l.t("Intervall / s","Interval / s","Interval / s"));ui.add(egui::DragValue::new(&mut self.profile.interval).range(1..=3600));ui.end_row();
    ui.label(l.t("Zieladresse","Destination","Bestemming"));egui::ComboBox::from_id_salt("scope").selected_text(&self.profile.destination).show_ui(ui,|ui|{for v in ["01:80:C2:00:00:0E","01:80:C2:00:00:03","01:80:C2:00:00:00"]{ui.selectable_value(&mut self.profile.destination,v.into(),v);}});ui.end_row();
   });
   ui.label(RichText::new(l.t("ID-Subtypen: Chassis MAC=4, IP=5, lokal=7; Port MAC=3, IP=4, Schnittstellenname=5, lokal=7.","ID subtypes: chassis MAC=4, IP=5, local=7; port MAC=3, IP=4, interface name=5, local=7.","ID-subtypen: chassis MAC=4, IP=5, lokaal=7; poort MAC=3, IP=4, interfacenaam=5, lokaal=7.")).small().weak());
   ui.separator();ui.strong(l.t("Erweiterungen · vollständiger TLV-Editor","Extensions · complete TLV editor","Uitbreidingen · volledige TLV-editor"));ui.label(l.t("Typ 7–127. Für Typ 127: OUI (3 Bytes), Subtyp, Daten. Alle Werte als Hexbytes.","Type 7–127. For type 127: OUI (3 bytes), subtype, data. All values as hex bytes.","Type 7–127. Voor type 127: OUI (3 bytes), subtype, gegevens. Alle waarden als hexbytes."));
   let mut remove=None;for(i,(kind,hex))in self.profile.optional.iter_mut().enumerate(){ui.horizontal(|ui|{ui.add(egui::DragValue::new(kind).range(7..=127));ui.add(egui::TextEdit::singleline(hex).font(egui::TextStyle::Monospace).desired_width(380.));if ui.button("×").clicked(){remove=Some(i);}});}if let Some(i)=remove{self.profile.optional.remove(i);}
   ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut self.extra_kind).range(7..=127));ui.add(egui::TextEdit::singleline(&mut self.extra_hex).hint_text("00 80 C2 01 00 2A").desired_width(300.));if ui.button(l.t("Hinzufügen","Add TLV","TLV toevoegen")).clicked(){self.profile.optional.push((self.extra_kind,self.extra_hex.clone()));self.extra_hex.clear();}});
   ui.menu_button(l.t("Vorlage hinzufügen","Add template","Sjabloon toevoegen"),|ui|{for(label,k,v)in [("System capabilities",7,"00 80 00 80"),("IEEE 802.1 PVID (42)",127,"00 80 C2 01 00 2A"),("IEEE 802.3 maximum frame size (1518)",127,"00 12 0F 04 05 EE"),("LLDP-MED capabilities / endpoint class I",127,"00 12 BB 01 00 01 01"),("LLDP-MED inventory / model",127,"00 12 BB 0A 57 69 6E 4C 4C 44 50")]{if ui.button(label).clicked(){self.profile.optional.push((k,v.into()));ui.close();}}});
  });
        match protocol::encode(&self.profile, a.mac, false) {
            Ok(frame) => {
                ui.label(
                    RichText::new(format!(
                        "{} · {} bytes",
                        l.t(
                            "Gültige Ankündigung",
                            "Valid advertisement",
                            "Geldige advertentie"
                        ),
                        frame.len()
                    ))
                    .color(ui.visuals().hyperlink_color),
                );
                egui::CollapsingHeader::new(l.t(
                    "Paketvorschau",
                    "Packet preview",
                    "Pakketvoorbeeld",
                ))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(protocol::hex(&frame)).monospace())
                            .selectable(true)
                            .wrap(),
                    );
                });
            }
            Err(e) => {
                ui.colored_label(
                    Color32::from_rgb(230, 120, 85),
                    format!(
                        "{}: {e}",
                        l.t("Bitte korrigieren", "Please correct", "Corrigeer")
                    ),
                );
            }
        }
    }
    fn history(&mut self, ui: &mut egui::Ui) {
        let l = self.settings.language;
        Self::title(ui,l.t("Änderungsprotokoll","Activity log","Activiteitenlogboek"),l.t("Entdeckungen, Änderungen, Ablauf und Senderstatus. Lokal, maximal 2000 Ereignisse.","Discoveries, changes, expiration and sender status. Local, up to 2000 events.","Ontdekkingen, wijzigingen, verlopen en verzendstatus. Lokaal, maximaal 2000 gebeurtenissen."));
        ui.horizontal(|ui| {
            if ui
                .button(l.t("JSON exportieren", "Export JSON", "JSON exporteren"))
                .clicked()
            {
                self.export("JSON");
            }
            if ui
                .button(l.t("Protokoll leeren", "Clear log", "Logboek wissen"))
                .clicked()
            {
                self.monitor.events.clear();
                let _ = std::fs::remove_file(settings::data_dir().join("history.json"));
            }
        });
        if self.monitor.events.is_empty() {
            ui.label(l.t(
                "Noch keine Ereignisse.",
                "No events yet.",
                "Nog geen gebeurtenissen.",
            ));
        }
        egui::Grid::new("events")
            .num_columns(3)
            .striped(true)
            .min_col_width(110.)
            .show(ui, |ui| {
                for e in &self.monitor.events {
                    ui.label(
                        RichText::new(e.at.format("%d.%m. %H:%M:%S").to_string())
                            .monospace()
                            .small(),
                    );
                    ui.label(&e.adapter);
                    ui.vertical(|ui| {
                        ui.strong(event_name(l, &e.action));
                        ui.add(egui::Label::new(&e.detail).selectable(true).wrap());
                    });
                    ui.end_row();
                }
            });
    }
    fn settings_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.settings.language;
        Self::title(
            ui,
            l.t("Einstellungen", "Settings", "Instellingen"),
            l.t(
                "Dein Arbeitsablauf. Deine Sprache. Dein Design.",
                "Your workflow. Your language. Your theme.",
                "Je werkwijze. Je taal. Je thema.",
            ),
        );
        egui::Frame::group(ui.style()).inner_margin(20.).corner_radius(12).show(ui,|ui|{
   ui.strong(l.t("Sprache","Language","Taal"));ui.horizontal(|ui|{for lang in [L::Deutsch,L::English,L::Nederlands]{if ui.selectable_value(&mut self.settings.language,lang,format!("{lang:?}")).changed(){self.save();}}});
   ui.separator();ui.strong(l.t("Darstellung","Appearance","Weergave"));ui.horizontal(|ui|{for(theme,label)in [(Theme::System,l.t("Wie Windows","Follow Windows","Zoals Windows")),(Theme::Light,l.t("Hell","Light","Licht")),(Theme::Dark,l.t("Dunkel","Dark","Donker"))]{if ui.selectable_value(&mut self.settings.theme,theme,label).changed(){self.apply_theme(ctx);self.save();}}});
   ui.separator();if ui.checkbox(&mut self.settings.close_to_tray,l.t("Beim Schließen in den Infobereich minimieren","Minimize to tray when closing","Bij sluiten minimaliseren naar systeemvak")).changed(){self.save();}ui.label(RichText::new(l.t("Über das Traymenü oder „Beenden“ vollständig schließen. Ein aktiver Sender läuft im Tray weiter.","Use the tray menu or Quit to exit completely. An active sender continues in the tray.","Gebruik het systeemvakmenu of Afsluiten om volledig te stoppen. Een actieve zender blijft in het systeemvak werken.")).weak());
   if ui.checkbox(&mut self.settings.keep_history,l.t("Änderungsprotokoll lokal aufbewahren","Keep activity log locally","Activiteitenlogboek lokaal bewaren")).changed(){if !self.settings.keep_history{let _=std::fs::remove_file(settings::data_dir().join("history.json"));}self.save();}
  });
        ui.add_space(16.);
        egui::Frame::group(ui.style()).inner_margin(20.).corner_radius(12).show(ui,|ui|{ui.strong(l.t("Zugriffsmodus","Access mode","Toegangsmodus"));let admin=crate::platform::is_admin();ui.label(if admin{l.t("Administratorzugriff aktiv","Administrator access active","Beheerderstoegang actief")}else{l.t("Standardzugriff ohne Administratorrechte","Standard access without administrator rights","Standaardtoegang zonder beheerdersrechten")});ui.label(l.t("Live-Empfang und Senden benötigen Npcap. Normaler Zugriff hängt von dessen Berechtigungen ab. Ohne Treiber ist Offlineanalyse möglich.","Live capture and advertising require Npcap. Standard access depends on its permissions. Offline analysis works without a driver.","Live-ontvangst en verzenden vereisen Npcap. Standaardtoegang hangt af van de rechten. Offline-analyse werkt zonder stuurprogramma."));
   if !admin&&ui.button(l.t("Als Administrator neu starten …","Restart as administrator …","Opnieuw starten als beheerder …")).clicked(){match crate::platform::elevate(){Ok(())=>{self.exit=true;ctx.send_viewport_cmd(egui::ViewportCommand::Close);},Err(e)=>self.notify(Err(e))}}
   if admin && ui.button(l.t("Mit Standardzugriff neu starten …", "Restart with standard access …", "Opnieuw starten met standaardtoegang …")).clicked() {
       match crate::platform::standard_access() {
           Ok(()) => { self.exit = true; ctx.send_viewport_cmd(egui::ViewportCommand::Close); },
           Err(e) => self.notify(Err(e)),
       }
   }
   ui.hyperlink_to(l.t("Npcap-Download und Lizenz","Npcap download and license","Npcap-download en licentie"),"https://npcap.com/#download");
  });
        ui.add_space(12.);
        ui.label(l.t(
            "Keine Telemetrie. Daten und Profile bleiben auf diesem Rechner.",
            "No telemetry. Data and profiles stay on this computer.",
            "Geen telemetrie. Gegevens en profielen blijven op deze computer.",
        ));
        ui.add(
            egui::Label::new(settings::data_dir().display().to_string())
                .selectable(true)
                .wrap(),
        );
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
            .exact_height(76.)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.);
                    ui.add(egui::Image::new((self.logo.id(), Vec2::splat(56.))));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("WinLLDP").size(24.).strong());
                        ui.label(
                            RichText::new("NETWORK DISCOVERY")
                                .size(10.)
                                .color(ui.visuals().hyperlink_color),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if self.engine.running() {
                                l.t("Empfang stoppen", "Stop capture", "Ontvangst stoppen")
                            } else {
                                l.t("Empfang starten", "Start capture", "Ontvangst starten")
                            })
                            .clicked()
                        {
                            if self.engine.running() {
                                self.stop();
                            } else {
                                self.start();
                            }
                        }
                        ui.label(
                            RichText::new(if self.tx_active.is_empty() {
                                l.t("Sender aus", "Advertising off", "Verzenden uit")
                            } else {
                                l.t("Sender aktiv", "Advertising active", "Verzenden actief")
                            })
                            .color(if self.tx_active.is_empty() {
                                ui.visuals().weak_text_color()
                            } else {
                                GREEN
                            }),
                        );
                    });
                });
            });
        egui::TopBottomPanel::bottom("status")
            .min_height(30.)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} {} · {} {}",
                            self.ready.len(),
                            l.t("Adapter aktiv", "adapters active", "adapters actief"),
                            self.monitor.received,
                            l.t("Pakete", "packets", "pakketten")
                        ))
                        .small(),
                    );
                    if !self.message.is_empty() {
                        ui.separator();
                        ui.colored_label(
                            if self.is_error {
                                Color32::from_rgb(226, 134, 85)
                            } else {
                                ui.visuals().text_color()
                            },
                            &self.message,
                        );
                    }
                });
            });
        egui::SidePanel::left("navigation")
            .exact_width(192.)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(18.);
                ui.label(
                    RichText::new(l.t("ARBEITSBEREICH", "WORKSPACE", "WERKRUIMTE"))
                        .small()
                        .weak(),
                );
                ui.add_space(12.);
                for (i, label) in [
                    l.t("Übersicht", "Overview", "Overzicht"),
                    l.t("Adapter", "Adapters", "Adapters"),
                    l.t("LLDP senden", "Advertise LLDP", "LLDP verzenden"),
                    l.t("Protokoll", "Activity", "Activiteit"),
                    l.t("Einstellungen", "Settings", "Instellingen"),
                    l.t("Über WinLLDP", "About WinLLDP", "Over WinLLDP"),
                ]
                .iter()
                .enumerate()
                {
                    if ui
                        .add_sized(
                            [ui.available_width(), 42.],
                            egui::Button::new(format!("{:02}   {label}", i + 1))
                                .selected(self.page == i)
                                .frame(self.page == i),
                        )
                        .clicked()
                    {
                        self.page = i;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(16.);
                    if ui.button(l.t("Beenden", "Quit", "Afsluiten")).clicked() {
                        self.exit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.label(RichText::new("v1.0.0 · MIT").small().weak());
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(24.))
            .show(ctx, |ui| {
                if self.page == 0 {
                    self.overview(ui);
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("page")
                        .show(ui, |ui| match self.page {
                            1 => self.adapters_ui(ui),
                            2 => self.sender(ui),
                            3 => self.history(ui),
                            4 => self.settings_ui(ui, ctx),
                            _ => self.about(ui),
                        });
                }
            });
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
