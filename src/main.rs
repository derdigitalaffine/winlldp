#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
slint::include_modules!();
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    rc::Rc,
    time::{Duration, Instant},
};
use winlldp::{
    capture::{self, CaptureBackend, Engine, Notice},
    model::Monitor,
    protocol::{self, TxProfile},
    settings::{self, Language, Settings, Theme},
};
fn bool_model(values: Vec<bool>) -> ModelRc<bool> {
    ModelRc::new(VecModel::from(values))
}
fn model(values: Vec<String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        values
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
}
fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|s| s == "--cli") {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::System::Console::*;
            let h = GetStdHandle(STD_OUTPUT_HANDLE);
            if h.is_null() || h as isize == -1 {
                AttachConsole(u32::MAX);
            }
        }
        std::process::exit(winlldp::cli::run(&args[1..]));
    }
    if args.iter().any(|s| s == "--self-test") {
        return self_test(&args);
    }
    let settings = settings::load_json::<Settings>("settings.json").unwrap_or_default();
    let mut monitor = Monitor::default();
    if settings.keep_history {
        monitor.events = settings::load_json("history.json").unwrap_or_default();
    }
    if let Some(path) = args
        .iter()
        .position(|s| s == "--open")
        .and_then(|i| args.get(i + 1))
    {
        load_pcap(path, &mut monitor);
    }
    let adapters = capture::adapters().unwrap_or_default();
    let chosen = Rc::new(RefCell::new(
        adapters
            .iter()
            .filter(|a| a.up)
            .map(|a| a.id.clone())
            .collect::<BTreeSet<_>>(),
    ));
    let send_id = Rc::new(RefCell::new(
        adapters
            .iter()
            .find(|a| a.up)
            .or_else(|| adapters.first())
            .map(|a| a.id.clone())
            .unwrap_or_default(),
    ));
    let engine = Rc::new(RefCell::new(Engine::default()));
    let monitor = Rc::new(RefCell::new(monitor));
    let search = Rc::new(RefCell::new(String::new()));
    let selected_neighbor = Rc::new(Cell::new(0usize));
    let backend = Rc::new(Cell::new(settings.capture_backend));
    let mut initial_profile = settings
        .profiles
        .values()
        .next()
        .cloned()
        .unwrap_or_default();
    if let Some(adapter) = adapters.first() {
        if initial_profile.chassis_value.is_empty() {
            initial_profile.chassis_value = protocol::hex(&adapter.mac).replace(" ", ":");
        }
        if initial_profile.port_value.is_empty() {
            initial_profile.port_value = adapter.name.clone();
        }
        if initial_profile.port_description.is_empty() {
            initial_profile.port_description = adapter.description.clone();
        }
    }
    let profile = Rc::new(RefCell::new(initial_profile));
    let app = AppWindow::new()?;
    app.set_language(
        match settings.language {
            Language::English => "English",
            Language::Nederlands => "Nederlands",
            _ => "Deutsch",
        }
        .into(),
    );
    app.set_dark(match settings.theme {
        Theme::Light => false,
        Theme::Dark => true,
        Theme::System => system_dark(),
    });
    app.set_theme(
        match settings.theme {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
        }
        .into(),
    );
    app.set_close_to_tray(settings.close_to_tray);
    app.set_is_windows(cfg!(windows));
    app.set_capture_backend(
        match settings.capture_backend {
            CaptureBackend::Auto => "Auto",
            CaptureBackend::Windows => "Windows Packet Monitor",
            CaptureBackend::Npcap => "Npcap",
        }
        .into(),
    );
    app.set_adapter_checked(bool_model(adapters.iter().map(|a| a.up).collect()));
    app.set_adapters(model(
        adapters
            .iter()
            .map(|a| {
                format!(
                    "{}  ·  {}  ·  {}",
                    a.name,
                    protocol::hex(&a.mac),
                    if a.up { "aktiv" } else { "getrennt" }
                )
            })
            .collect(),
    ));
    let sender_labels = adapters
        .iter()
        .map(|a| format!("{} · {}", a.name, protocol::hex(&a.mac)))
        .collect::<Vec<_>>();
    app.set_sender_adapters(model(sender_labels.clone()));
    app.set_sender_adapter(
        adapters
            .iter()
            .position(|a| a.id == *send_id.borrow())
            .and_then(|i| sender_labels.get(i).cloned())
            .unwrap_or_default()
            .into(),
    );
    app.set_system_name(profile.borrow().system_name.clone().into());
    app.set_port_id(profile.borrow().port_value.clone().into());
    app.set_chassis_id(profile.borrow().chassis_value.clone().into());
    app.set_chassis_subtype(profile.borrow().chassis_subtype.to_string().into());
    app.set_port_subtype(profile.borrow().port_subtype.to_string().into());
    app.set_destination(profile.borrow().destination.clone().into());
    app.set_optional_tlvs(model(
        profile
            .borrow()
            .optional
            .iter()
            .map(|(k, v)| format!("Type {k} · {v}"))
            .collect(),
    ));
    app.set_port_description(profile.borrow().port_description.clone().into());
    app.set_ttl(profile.borrow().ttl.to_string().into());
    app.set_interval(profile.borrow().interval.to_string().into());
    let weak = app.as_weak();
    let engine_c = engine.clone();
    let chosen_c = chosen.clone();
    let backend_c = backend.clone();
    let devices_c = adapters.clone();
    app.on_toggle_adapter({
        let chosen = chosen.clone();
        let devices = adapters.clone();
        move |i| {
            if let Some(a) = devices.get(i as usize) {
                let mut c = chosen.borrow_mut();
                if !c.remove(&a.id) {
                    c.insert(a.id.clone());
                }
            }
        }
    });
    let backend_cb = backend.clone();
    let weak_backend = app.as_weak();
    app.on_choose_backend(move |value| {
        let b = match value.as_str() {
            "Windows Packet Monitor" => CaptureBackend::Windows,
            "Npcap" => CaptureBackend::Npcap,
            _ => CaptureBackend::Auto,
        };
        backend_cb.set(b);
        if let Some(u) = weak_backend.upgrade() {
            u.set_capture_backend(b.label().into());
        }
    });
    app.on_start_capture(move || {
        let Some(ui) = weak.upgrade() else { return };
        if engine_c.borrow().running() {
            engine_c.borrow_mut().stop();
            ui.set_capture_running(false);
            ui.set_status("Empfang gestoppt".into());
        } else {
            let devices = devices_c
                .iter()
                .filter(|a| chosen_c.borrow().contains(&a.id))
                .cloned()
                .collect::<Vec<_>>();
            let r = engine_c.borrow_mut().start_with(&devices, backend_c.get());
            ui.set_capture_running(r.is_ok());
            ui.set_status(
                r.err()
                    .unwrap_or_else(|| "Npcap-Empfang gestartet".into())
                    .into(),
            );
        }
    });
    let weak = app.as_weak();
    let eng = engine.clone();
    let prof = profile.clone();
    let send_target = send_id.clone();
    app.on_toggle_sender(move || {
        let Some(ui) = weak.upgrade() else { return };
        if ui.get_sender_running() {
            let id = send_target.borrow().clone();
            if !id.is_empty() {
                let _ = eng.borrow().advertise(&id, None);
            }
            ui.set_sender_running(false);
            ui.set_status("LLDP-Sender ausgeschaltet".into());
        } else if !eng.borrow().running() {
            ui.set_status("Zuerst den Empfang auf einem Adapter starten (Npcap).".into());
        } else {
            let p = prof.borrow().clone();
            let id = send_target.borrow().clone();
            let ok = !id.is_empty() && eng.borrow().advertise(&id, Some(p)).is_ok();
            ui.set_sender_running(ok);
            ui.set_status(
                if ok {
                    "LLDP-Ankündigung aktiv"
                } else {
                    "Kein aktiver Adapter"
                }
                .into(),
            );
        }
    });
    let send_target = send_id.clone();
    let sender_adapters = adapters.clone();
    let sender_labels = sender_labels.clone();
    let weak = app.as_weak();
    app.on_choose_sender_adapter(move |label| {
        if let Some(i) = sender_labels.iter().position(|x| x == label.as_str()) {
            *send_target.borrow_mut() = sender_adapters[i].id.clone();
            if let Some(ui) = weak.upgrade() {
                ui.set_status(format!("Sendeadapter: {}", sender_adapters[i].name).into());
            }
        }
    });
    let weak = app.as_weak();
    let mon = monitor.clone();
    let query = search.clone();
    app.on_import_pcap(move || {
        if let Some(f) = rfd::FileDialog::new()
            .add_filter("PCAP", &["pcap", "cap"])
            .pick_file()
        {
            let result = std::fs::File::open(&f)
                .map_err(|e| e.to_string())
                .and_then(winlldp::export::read_pcap);
            if let Ok(ps) = result {
                let mut m = mon.borrow_mut();
                for p in ps {
                    let _ = m.ingest("PCAP", p.bytes, p.at);
                }
            }
            if let Some(u) = weak.upgrade() {
                refresh(&u, &mon.borrow(), &query.borrow(), 0);
                u.set_status("PCAP-Aufzeichnung geladen".into());
            }
        }
    });
    let weak = app.as_weak();
    let mon = monitor.clone();
    app.on_export_data(move |kind| {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("winlldp.{}", kind.to_lowercase()))
            .save_file()
        else {
            return;
        };
        let m = mon.borrow();
        let result: Result<(), String> = match kind.as_str() {
            "CSV" => std::fs::write(path, winlldp::export::csv(&m)).map_err(|e| e.to_string()),
            "PCAP" => std::fs::File::create(path)
                .map_err(|e| e.to_string())
                .and_then(|f| {
                    winlldp::export::write_pcap(&m.packets, f).map_err(|e| e.to_string())
                }),
            _ => winlldp::export::json(&m)
                .map_err(|e| e.to_string())
                .and_then(|b| std::fs::write(path, b).map_err(|e| e.to_string())),
        };
        if let Some(u) = weak.upgrade() {
            u.set_status(
                result
                    .map(|_| "Export abgeschlossen".to_string())
                    .unwrap_or_else(|e| e)
                    .into(),
            );
        }
    });
    let prof = profile.clone();
    app.on_update_profile(
        move |name,
              chassis,
              chassis_subtype,
              port,
              port_subtype,
              desc,
              ttl,
              interval,
              destination,
              _| {
            let mut p = prof.borrow_mut();
            p.system_name = name.to_string();
            p.chassis_value = chassis.to_string();
            p.chassis_subtype = chassis_subtype.parse().unwrap_or(4);
            p.port_value = port.to_string();
            p.port_subtype = port_subtype.parse().unwrap_or(5);
            p.port_description = desc.to_string();
            p.ttl = ttl.parse().unwrap_or(120);
            p.interval = interval.parse().unwrap_or(30);
            p.destination = destination.to_string();
        },
    );
    let prof = profile.clone();
    let weak = app.as_weak();
    app.on_add_optional(move |kind, raw| {
        let result = kind
            .parse::<u8>()
            .map_err(|_| "TLV-Typ muss zwischen 7 und 127 liegen".to_string())
            .and_then(|kind| {
                let bytes = protocol::unhex(&raw).map_err(|e| e.to_string())?;
                let mut p = prof.borrow_mut();
                p.optional.push((kind, protocol::hex(&bytes)));
                Ok(())
            });
        if let Some(ui) = weak.upgrade() {
            ui.set_status(
                result
                    .err()
                    .unwrap_or_else(|| "TLV hinzugefügt".into())
                    .into(),
            );
            ui.set_optional_tlvs(model(
                prof.borrow()
                    .optional
                    .iter()
                    .map(|(k, v)| format!("Typ {k} · {v}"))
                    .collect(),
            ));
        }
    });
    let prof = profile.clone();
    let weak = app.as_weak();
    app.on_remove_optional(move |index| {
        let mut p = prof.borrow_mut();
        if (index as usize) < p.optional.len() {
            p.optional.remove(index as usize);
        }
        if let Some(ui) = weak.upgrade() {
            ui.set_optional_tlvs(model(
                p.optional
                    .iter()
                    .map(|(k, v)| format!("Typ {k} · {v}"))
                    .collect(),
            ));
            ui.set_status("TLV entfernt".into());
        }
    });
    let weak = app.as_weak();
    app.on_navigate(move |p| {
        if let Some(u) = weak.upgrade() {
            u.set_page(p);
        }
    });
    let weak = app.as_weak();
    app.on_choose_language(move |v| {
        if let Some(u) = weak.upgrade() {
            u.set_language(v);
        }
    });
    let weak = app.as_weak();
    app.on_choose_theme(move |theme| {
        if let Some(u) = weak.upgrade() {
            u.set_dark(if theme == "System" {
                system_dark()
            } else {
                theme == "Dark"
            });
            u.set_theme(theme);
        }
    });
    let weak = app.as_weak();
    let mon = monitor.clone();
    let query = search.clone();
    let selected = selected_neighbor.clone();
    app.on_select_neighbor(move |index| {
        if let Some(u) = weak.upgrade() {
            selected.set(index as usize);
            u.set_details(details(&mon.borrow(), &query.borrow(), index as usize));
        }
    });
    let weak = app.as_weak();
    let mon = monitor.clone();
    let query = search.clone();
    let selected = selected_neighbor.clone();
    app.on_search_changed(move |q| {
        *query.borrow_mut() = q.to_string();
        selected.set(0);
        if let Some(u) = weak.upgrade() {
            u.set_selected_neighbor(0);
            refresh(&u, &mon.borrow(), &query.borrow(), 0);
        }
    });
    #[cfg(windows)]
    {
        let weak = app.as_weak();
        app.on_restart_admin(move || {
            if let Err(e) = winlldp::platform::elevate() {
                if let Some(u) = weak.upgrade() {
                    u.set_status(e.into());
                }
            } else {
                let _ = slint::quit_event_loop();
            }
        });
        let weak = app.as_weak();
        app.on_restart_standard(move || {
            if let Err(e) = winlldp::platform::standard_access() {
                if let Some(u) = weak.upgrade() {
                    u.set_status(e.into());
                }
            } else {
                let _ = slint::quit_event_loop();
            }
        });
    }
    #[cfg(not(windows))]
    {
        let weak = app.as_weak();
        app.on_restart_admin(move || {
            if let Some(u) = weak.upgrade() {
                u.set_status("Administratorwechsel ist nur unter Windows verfügbar.".into());
            }
        });
        let weak = app.as_weak();
        app.on_restart_standard(move || {
            if let Some(u) = weak.upgrade() {
                u.set_status("Benutzerwechsel ist nur unter Windows verfügbar.".into());
            }
        });
    }
    let weak = app.as_weak();
    app.on_diagnose_npcap(move || {
        let text = match capture::inspect_npcap() {
            Ok(r) => format!(
                "Npcap {} · Geräte: {} · AdminOnly: {:?} · Administrator: {}",
                r.version,
                r.devices.len(),
                r.admin_only,
                admin_state()
            ),
            Err(e) => {
                format!("{e} · Windows Packet Monitor arbeitet ohne Npcap; Senden erfordert Npcap.")
            }
        };
        if let Some(u) = weak.upgrade() {
            u.set_status(text.into());
        }
    });
    #[cfg(windows)]
    app.on_open_npcap(|| {
        let _ = std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", "https://npcap.com/#download"])
            .spawn();
    });
    #[cfg(not(windows))]
    app.on_open_npcap(|| {});
    app.on_quit_app(|| {
        let _ = slint::quit_event_loop();
    });
    refresh(
        &app,
        &monitor.borrow(),
        &search.borrow(),
        selected_neighbor.get(),
    );
    let weak = app.as_weak();
    let eng = engine.clone();
    let mon = monitor.clone();
    let prof = profile.clone();
    let query = search.clone();
    let selected = selected_neighbor.clone();
    let backend_timer = backend.clone();
    let last_save_c = Rc::new(Cell::new(Instant::now()));
    let settings_base = settings.clone();
    #[cfg(windows)]
    let tray = winlldp::platform::Tray::new().ok();
    #[cfg(windows)]
    let tray = Rc::new(tray);
    #[cfg(windows)]
    let tray_available = tray.is_some();
    #[cfg(not(windows))]
    let tray_available = false;
    let weak_close = app.as_weak();
    app.window().on_close_requested(move || {
        if !(weak_close.upgrade().is_some_and(|u| u.get_close_to_tray()) && tray_available) {
            let _ = slint::quit_event_loop();
        }
        slint::CloseRequestResponse::HideWindow
    });
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(500),
        move || {
            #[cfg(windows)]
            if let Some(t) = tray.as_ref() {
                let (show, exit) = t.poll();
                if show {
                    if let Some(u) = weak.upgrade() {
                        u.show().ok();
                    }
                }
                if exit {
                    let _ = slint::quit_event_loop();
                    return;
                }
            }
            let notices = eng.borrow().poll();
            for n in notices {
                match n {
                    Notice::Frame(p) => {
                        let _ = mon.borrow_mut().ingest(&p.adapter, p.bytes, p.at);
                    }
                    Notice::Error(_, e) => {
                        if let Some(u) = weak.upgrade() {
                            u.set_status(e.into());
                        }
                    }
                    Notice::Tx(_, active) => {
                        if let Some(u) = weak.upgrade() {
                            u.set_sender_running(active);
                        }
                    }
                    Notice::LocalFrame(id) => {
                        if let Some(u) = weak.upgrade() {
                            u.set_status(format!("Lokale LLDP-Sendeantwort auf {id}").into());
                        }
                    }
                    Notice::Backend(id, backend_name) => {
                        if let Some(u) = weak.upgrade() {
                            u.set_status(format!("{id}: Empfang über {backend_name}").into());
                        }
                    }
                    Notice::Diagnostic(id, text) => {
                        if let Some(u) = weak.upgrade() {
                            u.set_status(format!("{id}: {text}").into());
                        }
                    }
                    Notice::Status(id, s) => {
                        if s == "capture_started"
                            && weak.upgrade().is_some_and(|u| u.get_sender_running())
                        {
                            let p = prof.borrow().clone();
                            let _ = eng.borrow().advertise(&id, Some(p));
                        }
                    }
                }
            }
            mon.borrow_mut().expire(chrono::Utc::now());
            if let Some(u) = weak.upgrade() {
                refresh(&u, &mon.borrow(), &query.borrow(), selected.get());
                if last_save_c.get().elapsed() >= Duration::from_secs(10) {
                    let mut saved = settings_base.clone();
                    saved.language = match u.get_language().as_str() {
                        "English" => Language::English,
                        "Nederlands" => Language::Nederlands,
                        _ => Language::Deutsch,
                    };
                    saved.theme = match u.get_theme().as_str() {
                        "Light" => Theme::Light,
                        "Dark" => Theme::Dark,
                        _ => Theme::System,
                    };
                    saved.close_to_tray = u.get_close_to_tray();
                    saved.capture_backend = backend_timer.get();
                    saved
                        .profiles
                        .insert("default".into(), prof.borrow().clone());
                    let _ = settings::save_json("settings.json", &saved);
                    let _ = settings::save_json("history.json", &mon.borrow().events);
                    last_save_c.set(Instant::now());
                }
            }
        },
    );
    app.run()?;
    engine.borrow_mut().stop();
    let mut saved = settings;
    saved.language = match app.get_language().as_str() {
        "English" => Language::English,
        "Nederlands" => Language::Nederlands,
        _ => Language::Deutsch,
    };
    saved.close_to_tray = app.get_close_to_tray();
    saved.capture_backend = backend.get();
    saved.theme = match app.get_theme().as_str() {
        "Light" => Theme::Light,
        "Dark" => Theme::Dark,
        _ => Theme::System,
    };
    saved
        .profiles
        .insert("default".into(), profile.borrow().clone());
    let _ = settings::save_json("settings.json", &saved);
    let _ = settings::save_json("history.json", &monitor.borrow().events);
    drop(timer);
    Ok(())
}
fn refresh(ui: &AppWindow, m: &Monitor, query: &str, selected: usize) {
    let l = ui.get_language().to_string();
    let lang = if l == "English" {
        "en"
    } else if l == "Nederlands" {
        "nl"
    } else {
        "de"
    };
    let ns = m
        .neighbors
        .iter()
        .filter(|n| {
            if query.trim().is_empty() {
                return true;
            }
            let a = &n.advertisement;
            format!(
                "{} {} {} {}",
                a.system_name, a.chassis_id, a.port_id, n.adapter
            )
            .to_lowercase()
            .contains(&query.to_lowercase())
        })
        .map(|n| {
            let a = &n.advertisement;
            format!(
                "{}\n{}  ·  {}",
                if a.system_name.is_empty() {
                    &a.chassis_id
                } else {
                    &a.system_name
                },
                a.port_id,
                n.adapter
            )
        })
        .collect::<Vec<_>>();
    ui.set_neighbors(model(ns));
    ui.set_count_label(
        match lang {
            "en" => format!(
                "{} neighbors · {} packets · {} rejected",
                m.neighbors.len(),
                m.received,
                m.rejected
            ),
            "nl" => format!(
                "{} buren · {} pakketten · {} verworpen",
                m.neighbors.len(),
                m.received,
                m.rejected
            ),
            _ => format!(
                "{} Nachbarn · {} Pakete · {} verworfen",
                m.neighbors.len(),
                m.received,
                m.rejected
            ),
        }
        .into(),
    );
    ui.set_events(model(
        m.events
            .iter()
            .map(|e| {
                format!(
                    "{}  {}  {}  {}",
                    e.at.format("%H:%M:%S"),
                    e.action,
                    e.adapter,
                    e.detail
                )
            })
            .collect(),
    ));
    ui.set_details(details(m, query, selected));
}
fn details(m: &Monitor, query: &str, i: usize) -> ModelRc<SharedString> {
    let visible: Vec<_> = m
        .neighbors
        .iter()
        .filter(|n| {
            if query.trim().is_empty() {
                return true;
            }
            let a = &n.advertisement;
            format!(
                "{} {} {} {}",
                a.system_name, a.chassis_id, a.port_id, n.adapter
            )
            .to_lowercase()
            .contains(&query.to_lowercase())
        })
        .collect();
    let Some(n) = visible.get(i).copied() else {
        return model(vec![]);
    };
    let a = &n.advertisement;
    let mut v = vec![
        format!(
            "{}  ·  Port {}",
            if a.system_name.is_empty() {
                &a.chassis_id
            } else {
                &a.system_name
            },
            a.port_id
        ),
        format!("Adapter: {}   ·   MAC: {}", n.adapter, a.source_mac),
        format!("Chassis ID: {}   ·   TTL: {} s", a.chassis_id, a.ttl),
        format!(
            "Zuletzt gesehen: {}",
            n.last_seen.format("%Y-%m-%d %H:%M:%S UTC")
        ),
    ];
    if !a.system_description.is_empty() {
        v.push(a.system_description.clone())
    }
    v.push("ALLE LLDP-TLVS · Rohdaten einschließlich unbekannter Erweiterungen".into());
    for t in &a.tlvs {
        v.push(format!("TLV {} · {}", t.kind, t.name));
        for (k, x) in &t.fields {
            v.push(format!("{k}: {x}"))
        }
        v.push(format!("Hex: {}", t.value_hex));
    }
    model(v)
}
fn load_pcap(path: &str, m: &mut Monitor) {
    if let Ok(f) = std::fs::File::open(path) {
        if let Ok(p) = winlldp::export::read_pcap(f) {
            for v in p {
                let _ = m.ingest("PCAP", v.bytes, v.at);
            }
        }
    }
}
fn admin_state() -> bool {
    #[cfg(windows)]
    {
        winlldp::platform::is_admin()
    }
    #[cfg(not(windows))]
    {
        false
    }
}
fn system_dark() -> bool {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        return RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
            .ok()
            .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme").ok())
            .map(|v| v == 0)
            .unwrap_or(true);
    }
    #[cfg(target_os = "linux")]
    {
        if std::env::var("GTK_THEME")
            .map(|v| v.to_lowercase().contains("dark"))
            .unwrap_or(false)
        {
            return true;
        }
        if let Ok(o) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            return String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains("dark");
        }
        true
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        true
    }
}
fn self_test(args: &[String]) -> Result<(), slint::PlatformError> {
    let p = TxProfile {
        chassis_value: "00:11:22:33:44:55".into(),
        port_value: "self-test".into(),
        ..Default::default()
    };
    let bytes = protocol::encode(&p, [0, 17, 34, 51, 68, 85], false).unwrap();
    assert_eq!(protocol::parse(&bytes).unwrap().port_id, "self-test");
    if let Some(i) = args.iter().position(|x| x == "--report") {
        if let Some(path) = args.get(i + 1) {
            let _ = std::fs::write(path, "WinLLDP Windows self-test OK\n");
        }
    }
    Ok(())
}
