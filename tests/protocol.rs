use chrono::Utc;
use winlldp::{export, model::Monitor, protocol::*};
fn profile() -> TxProfile {
    TxProfile {
        chassis_value: "00:11:22:33:44:55".into(),
        port_value: "Ethernet".into(),
        system_name: "switch".into(),
        ..Default::default()
    }
}
#[test]
fn roundtrip_and_unknown_extensions() {
    let mut p = profile();
    p.optional = vec![
        (127, "00 80 C2 01 00 2A".into()),
        (126, "DE AD BE EF".into()),
        (127, "00 12 BB 02 01 40 54 AE".into()),
    ];
    let b = encode(&p, [1; 6], false).unwrap();
    let a = parse(&b).unwrap();
    assert_eq!(a.port_id, "Ethernet");
    assert_eq!(a.ttl, 120);
    assert!(a
        .tlvs
        .iter()
        .any(|t| t.fields.contains(&("Port VLAN ID".into(), "42".into()))));
    assert!(a
        .tlvs
        .iter()
        .any(|t| t.kind == 126 && t.value_hex == "DE AD BE EF"));
}
#[test]
fn all_truncations_and_invalid_inputs_do_not_panic() {
    let b = encode(&profile(), [2; 6], false).unwrap();
    for i in 0..b.len() {
        let _ = parse(&b[..i]);
    }
    for n in 0..4096usize {
        let data = (0..n % 256).map(|i| (i * 73 + n) as u8).collect::<Vec<_>>();
        let _ = parse(&data);
    }
    assert!(parse(&[0; 20]).is_err());
    assert!(unhex("é").is_err());
    assert!(unhex("1").is_err());
}
#[test]
fn mandatory_order_duplicate_end_and_ttl() {
    let mut b = encode(&profile(), [2; 6], false).unwrap();
    b[14] = 4;
    assert!(parse(&b).is_err());
    let mut p = profile();
    p.ttl = 1;
    assert!(encode(&p, [1; 6], false).is_err());
    p = profile();
    p.optional = vec![(1, "01 02".into())];
    assert!(encode(&p, [1; 6], false).is_err());
}
#[test]
fn vlan_qinq() {
    let b = encode(&profile(), [1; 6], false).unwrap();
    let mut v = b[..12].to_vec();
    v.extend([0x88, 0xa8, 0, 10, 0x81, 0, 0, 20]);
    v.extend(&b[12..]);
    let a = parse(&v).unwrap();
    assert_eq!(a.vlan_tags, vec![10, 20]);
}
#[test]
fn expiration_shutdown_and_isolated_adapters() {
    let p = profile();
    let b = encode(&p, [1; 6], false).unwrap();
    let now = Utc::now();
    let mut m = Monitor::default();
    m.ingest("a", b.clone(), now).unwrap();
    m.ingest("b", b, now).unwrap();
    assert_eq!(m.neighbors.len(), 2);
    m.ingest("a", encode(&p, [1; 6], true).unwrap(), now)
        .unwrap();
    assert_eq!(m.neighbors.len(), 1);
    m.expire(now + chrono::Duration::seconds(121));
    assert!(m.neighbors.is_empty());
}
#[test]
fn export_roundtrip_and_csv_safety() {
    let mut m = Monitor::default();
    let mut p = profile();
    p.system_name = "=SUM(1,1)".into();
    m.ingest("a", encode(&p, [1; 6], false).unwrap(), Utc::now())
        .unwrap();
    let mut bytes = vec![];
    export::write_pcap(&m.packets, &mut bytes).unwrap();
    let packets = export::read_pcap(&bytes[..]).unwrap();
    assert_eq!(packets[0].bytes, m.packets[0].bytes);
    assert!(export::csv(&m).contains("'=SUM(1,1)"));
    assert!(serde_json::from_slice::<serde_json::Value>(&export::json(&m).unwrap()).is_ok());
    bytes.pop();
    assert!(export::read_pcap(&bytes[..]).is_err());
}
#[test]
fn oversize_and_bad_org_are_rejected() {
    let mut p = profile();
    p.optional = vec![(127, "01 02".into())];
    assert!(encode(&p, [1; 6], false).is_err());
    p.optional = vec![(126, "AB ".repeat(512))];
    assert!(encode(&p, [1; 6], false).is_err());
}

#[test]
fn capture_rejects_empty_selection_and_inactive_sender() {
    let mut engine = winlldp::capture::Engine::default();
    assert!(engine.start(&[]).is_err());
    assert!(!engine.running());
    assert!(engine.advertise("missing", Some(profile())).is_err());
    engine.stop();
    assert!(engine.poll().is_empty());
}

#[test]
fn settings_roundtrip_and_no_persisted_send_switch() {
    use winlldp::settings::*;
    let mut s = Settings::default();
    assert!(s.close_to_tray);
    s.language = Language::Nederlands;
    s.theme = Theme::Dark;
    s.profiles.insert("Ethernet".into(), profile());
    let text = serde_json::to_string(&s).unwrap();
    let loaded: Settings = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded.language, Language::Nederlands);
    assert_eq!(loaded.theme, Theme::Dark);
    assert!(!text.contains("tx_active"));
    assert!(!text.contains("sending_enabled"));
    assert_eq!(Settings::default().language.t("ja", "yes", "ja"), "ja");
}
