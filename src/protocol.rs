//! Bounded IEEE 802.1AB Ethernet/TLV parser. Unknown TLVs are always preserved.
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tlv {
    pub kind: u8,
    pub name: String,
    pub value_hex: String,
    pub fields: Vec<(String, String)>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Advertisement {
    pub source_mac: String,
    pub destination_mac: String,
    pub vlan_tags: Vec<u16>,
    pub chassis_id: String,
    pub port_id: String,
    pub ttl: u16,
    pub system_name: String,
    pub port_description: String,
    pub system_description: String,
    pub tlvs: Vec<Tlv>,
}
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn unhex(s: &str) -> Result<Vec<u8>, String> {
    let s: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':' && *c != '-')
        .collect();
    if !s.len().is_multiple_of(2) || s.len() > 1022 {
        return Err("Hex: expected 0–511 complete bytes".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            s.get(i..i + 2)
                .and_then(|p| u8::from_str_radix(p, 16).ok())
                .ok_or_else(|| "Invalid hex".into())
        })
        .collect()
}
fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b)
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
fn mac(b: &[u8]) -> String {
    b.iter()
        .map(|v| format!("{v:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}
fn be(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn address(b: &[u8]) -> String {
    match b.first() {
        Some(1) if b.len() == 5 => Ipv4Addr::new(b[1], b[2], b[3], b[4]).to_string(),
        Some(2) if b.len() == 17 => {
            Ipv6Addr::from(<[u8; 16]>::try_from(&b[1..]).unwrap()).to_string()
        }
        _ => hex(b),
    }
}
fn id(b: &[u8], chassis: bool) -> Result<String, String> {
    if b.len() < 2 || b.len() > 256 {
        return Err("Invalid ID length".into());
    }
    Ok(if (chassis && b[0] == 4) || (!chassis && b[0] == 3) {
        mac(&b[1..])
    } else if (chassis && b[0] == 5) || (!chassis && b[0] == 4) {
        address(&b[1..])
    } else {
        text(&b[1..])
    })
}
fn capabilities(bits: u16) -> String {
    let names = [
        "Other",
        "Repeater",
        "Bridge",
        "WLAN AP",
        "Router",
        "Telephone",
        "DOCSIS",
        "Station",
        "C-VLAN",
        "S-VLAN",
        "TPMR",
    ];
    names
        .iter()
        .enumerate()
        .filter(|(i, _)| bits & (1 << i) != 0)
        .map(|(_, n)| *n)
        .collect::<Vec<_>>()
        .join(", ")
}
fn decode(kind: u8, b: &[u8]) -> Tlv {
    let name = match kind {
        0 => "End",
        1 => "Chassis ID",
        2 => "Port ID",
        3 => "TTL",
        4 => "Port description",
        5 => "System name",
        6 => "System description",
        7 => "System capabilities",
        8 => "Management address",
        127 => "Organizational extension",
        _ => "Unknown TLV",
    }
    .to_owned();
    let mut f: Vec<(String, String)> = vec![];
    let mut add = |k: &str, v: String| f.push((k.into(), v));
    match kind {
        1 | 2 => {
            if let Some(s) = b.first() {
                add("Subtype", s.to_string());
            }
            if let Ok(v) = id(b, kind == 1) {
                add("ID", v);
            }
        }
        3 if b.len() == 2 => add("Seconds", be(b).to_string()),
        4..=6 => add("Text", text(b)),
        7 if b.len() == 4 => {
            add("Supported", capabilities(be(b)));
            add("Enabled", capabilities(be(&b[2..])));
        }
        8 if b.len() >= 2 => {
            let n = b[0] as usize;
            if n >= 2 && b.len() >= n + 7 {
                add("Address", address(&b[1..1 + n]));
                add("Interface numbering subtype", b[n + 1].to_string());
                add(
                    "Interface number",
                    u32::from_be_bytes(b[n + 2..n + 6].try_into().unwrap()).to_string(),
                );
                let o = b[n + 6] as usize;
                if b.len() >= n + 7 + o {
                    add("OID", hex(&b[n + 7..n + 7 + o]));
                }
            }
        }
        127 if b.len() >= 4 => {
            add("OUI", hex(&b[..3]));
            add("Subtype", b[3].to_string());
            let v = &b[4..];
            match (&b[..3], b[3]) {
                ([0, 0x80, 0xc2], 1) if v.len() == 2 => add("Port VLAN ID", be(v).to_string()),
                ([0, 0x80, 0xc2], 2) if v.len() == 3 => {
                    add("PPVID flags", format!("0x{:02X}", v[0]));
                    add("Protocol VLAN ID", be(&v[1..]).to_string());
                }
                ([0, 0x80, 0xc2], 3) if v.len() >= 3 && v.len() >= 3 + v[2] as usize => {
                    add("VLAN ID", be(v).to_string());
                    add("VLAN name", text(&v[3..3 + v[2] as usize]));
                }
                ([0, 0x80, 0xc2], 4) if !v.is_empty() && v.len() > v[0] as usize => {
                    add("Protocol identity", hex(&v[1..1 + v[0] as usize]))
                }
                ([0, 0x12, 0x0f], 1) if v.len() == 5 => {
                    add("Auto-negotiation", format!("0x{:02X}", v[0]));
                    add("PMD capabilities", format!("0x{:04X}", be(&v[1..])));
                    add("MAU type", be(&v[3..]).to_string());
                }
                ([0, 0x12, 0x0f], 2) if v.len() >= 3 => {
                    add("Power support", hex(&v[..3]));
                    if v.len() > 3 {
                        add("Extended power", hex(&v[3..]));
                    }
                }
                ([0, 0x12, 0x0f], 3) if v.len() == 5 => {
                    add("Aggregation flags", v[0].to_string());
                    add(
                        "Aggregation port ID",
                        u32::from_be_bytes(v[1..].try_into().unwrap()).to_string(),
                    );
                }
                ([0, 0x12, 0x0f], 4) if v.len() == 2 => {
                    add("Maximum frame size", be(v).to_string())
                }
                ([0, 0x12, 0xbb], 1) if v.len() == 3 => {
                    add("MED capabilities", format!("0x{:04X}", be(v)));
                    add("MED device class", v[2].to_string());
                }
                ([0, 0x12, 0xbb], 2) if v.len() == 4 => {
                    let p = ((v[1] as u32) << 16) | ((v[2] as u32) << 8) | v[3] as u32;
                    add("MED application", v[0].to_string());
                    add("Unknown policy", (p & 0x800000 != 0).to_string());
                    add("Tagged", (p & 0x400000 != 0).to_string());
                    add("VLAN", ((p >> 9) & 0xfff).to_string());
                    add("L2 priority", ((p >> 6) & 7).to_string());
                    add("DSCP", (p & 63).to_string());
                }
                ([0, 0x12, 0xbb], 3) if !v.is_empty() => {
                    add("MED location format", v[0].to_string());
                    add("Location data", hex(&v[1..]));
                }
                ([0, 0x12, 0xbb], 4) if v.len() == 3 => {
                    add("MED power flags", format!("0x{:02X}", v[0]));
                    add("Power (0.1 W)", be(&v[1..]).to_string());
                }
                ([0, 0x12, 0xbb], s @ 5..=11) => {
                    let labels = [
                        "Hardware revision",
                        "Firmware revision",
                        "Software revision",
                        "Serial number",
                        "Manufacturer",
                        "Model",
                        "Asset ID",
                    ];
                    add(labels[(s - 5) as usize], text(v));
                }
                _ => add("Organizational data", hex(v)),
            }
        }
        _ => {}
    }
    Tlv {
        kind,
        name,
        value_hex: hex(b),
        fields: f,
    }
}
pub fn parse(frame: &[u8]) -> Result<Advertisement, String> {
    if frame.len() < 14 || frame.len() > 65535 {
        return Err("Invalid Ethernet frame length".into());
    }
    let mut eth = be(&frame[12..]);
    let mut p = 14;
    let mut tags = vec![];
    while eth == 0x8100 || eth == 0x88a8 {
        if tags.len() >= 2 || frame.len() < p + 4 {
            return Err("Invalid VLAN encapsulation".into());
        }
        tags.push(be(&frame[p..]));
        eth = be(&frame[p + 2..]);
        p += 4;
    }
    if eth != 0x88cc {
        return Err("Not LLDP".into());
    }
    let mut tlvs = vec![];
    let mut mandatory = vec![];
    let mut ended = false;
    while p + 2 <= frame.len() {
        let h = be(&frame[p..]);
        p += 2;
        let kind = (h >> 9) as u8;
        let n = (h & 511) as usize;
        if p + n > frame.len() {
            return Err("Truncated TLV".into());
        }
        let b = &frame[p..p + n];
        p += n;
        if kind == 0 {
            if n != 0 {
                return Err("Invalid End TLV".into());
            }
            ended = true;
            tlvs.push(decode(kind, b));
            break;
        }
        if tlvs.len() < 3 && kind != tlvs.len() as u8 + 1 {
            return Err("Mandatory TLV order".into());
        }
        if (1..=3).contains(&kind) {
            if mandatory.contains(&kind) {
                return Err("Duplicate mandatory TLV".into());
            }
            mandatory.push(kind);
        }
        if kind == 7 && n != 4 {
            return Err("Invalid capabilities length".into());
        }
        if tlvs.len() >= 256 {
            return Err("Too many TLVs".into());
        }
        tlvs.push(decode(kind, b));
    }
    if mandatory != [1, 2, 3] || !ended {
        return Err("Missing mandatory TLV or End".into());
    }
    let chassis = id(&unhex(&tlvs[0].value_hex)?, true)?;
    let port = id(&unhex(&tlvs[1].value_hex)?, false)?;
    let t = unhex(&tlvs[2].value_hex)?;
    if t.len() != 2 {
        return Err("Invalid TTL length".into());
    }
    let get = |k| {
        tlvs.iter()
            .find(|v| v.kind == k)
            .and_then(|v| v.fields.first())
            .map(|(_, s)| s.clone())
            .unwrap_or_default()
    };
    Ok(Advertisement {
        source_mac: mac(&frame[6..12]),
        destination_mac: mac(&frame[..6]),
        vlan_tags: tags,
        chassis_id: chassis,
        port_id: port,
        ttl: be(&t),
        system_name: get(5),
        port_description: get(4),
        system_description: get(6),
        tlvs,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxProfile {
    pub system_name: String,
    pub description: String,
    pub port_description: String,
    pub chassis_subtype: u8,
    pub chassis_value: String,
    pub port_subtype: u8,
    pub port_value: String,
    pub ttl: u16,
    pub interval: u16,
    pub destination: String,
    pub optional: Vec<(u8, String)>,
}
impl Default for TxProfile {
    fn default() -> Self {
        Self {
            system_name: std::env::var("COMPUTERNAME").unwrap_or_else(|_| "WinLLDP".into()),
            description: "WinLLDP / Windows".into(),
            port_description: String::new(),
            chassis_subtype: 4,
            chassis_value: String::new(),
            port_subtype: 5,
            port_value: String::new(),
            ttl: 120,
            interval: 30,
            destination: "01:80:C2:00:00:0E".into(),
            optional: vec![],
        }
    }
}
fn add_tlv(out: &mut Vec<u8>, kind: u8, v: &[u8]) -> Result<(), String> {
    if kind > 127 || v.len() > 511 {
        return Err("TLV outside 7-bit type / 9-bit length".into());
    }
    out.extend_from_slice(&(((kind as u16) << 9) | v.len() as u16).to_be_bytes());
    out.extend_from_slice(v);
    Ok(())
}
fn id_bytes(subtype: u8, value: &str, is_mac: bool, is_addr: bool) -> Result<Vec<u8>, String> {
    let mut b = vec![subtype];
    if is_mac {
        let v = unhex(value)?;
        if v.len() != 6 {
            return Err("MAC requires 6 bytes".into());
        }
        b.extend(v);
    } else if is_addr {
        let a: std::net::IpAddr = value.parse().map_err(|_| "Invalid IP ID")?;
        match a {
            std::net::IpAddr::V4(a) => {
                b.push(1);
                b.extend(a.octets());
            }
            std::net::IpAddr::V6(a) => {
                b.push(2);
                b.extend(a.octets());
            }
        }
    } else {
        b.extend(value.as_bytes());
    }
    if b.len() < 2 || b.len() > 256 {
        return Err("ID requires 1–255 bytes".into());
    }
    Ok(b)
}
pub fn encode(p: &TxProfile, source: [u8; 6], shutdown: bool) -> Result<Vec<u8>, String> {
    if !(1..=7).contains(&p.chassis_subtype) || !(1..=7).contains(&p.port_subtype) {
        return Err("ID subtype must be 1–7".into());
    }
    if p.interval < 1
        || p.interval > 3600
        || p.ttl < 1
        || u32::from(p.interval) * 2 > u32::from(p.ttl)
    {
        return Err("Interval 1–3600 seconds; TTL must be at least twice interval".into());
    }
    let dest = unhex(&p.destination)?;
    if ![
        vec![1, 128, 194, 0, 0, 14],
        vec![1, 128, 194, 0, 0, 3],
        vec![1, 128, 194, 0, 0, 0],
    ]
    .contains(&dest)
    {
        return Err("Use an IEEE LLDP multicast destination".into());
    }
    let mut b = dest;
    b.extend(source);
    b.extend([0x88, 0xcc]);
    add_tlv(
        &mut b,
        1,
        &id_bytes(
            p.chassis_subtype,
            &p.chassis_value,
            p.chassis_subtype == 4,
            p.chassis_subtype == 5,
        )?,
    )?;
    add_tlv(
        &mut b,
        2,
        &id_bytes(
            p.port_subtype,
            &p.port_value,
            p.port_subtype == 3,
            p.port_subtype == 4,
        )?,
    )?;
    add_tlv(&mut b, 3, &(if shutdown { 0 } else { p.ttl }).to_be_bytes())?;
    if !shutdown {
        for (k, s) in [
            (4, &p.port_description),
            (5, &p.system_name),
            (6, &p.description),
        ] {
            if s.len() > 255 {
                return Err("Text field exceeds 255 UTF-8 bytes".into());
            }
            if !s.is_empty() {
                add_tlv(&mut b, k, s.as_bytes())?;
            }
        }
        for (k, s) in &p.optional {
            if *k < 7 || *k > 127 {
                return Err("Extra TLVs must use types 7–127".into());
            }
            let v = unhex(s)?;
            if *k == 127 && v.len() < 4 {
                return Err("Organizational TLV needs OUI and subtype".into());
            }
            if *k == 7 && v.len() != 4 {
                return Err("Capabilities require 4 bytes".into());
            }
            add_tlv(&mut b, *k, &v)?;
        }
    }
    add_tlv(&mut b, 0, &[])?;
    if b.len() > 1514 {
        return Err("Advertisement exceeds Ethernet MTU (1500)".into());
    }
    b.resize(b.len().max(60), 0);
    Ok(b)
}
