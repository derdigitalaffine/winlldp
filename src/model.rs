use crate::protocol::{self, Advertisement};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighbor {
    pub adapter: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub advertisement: Advertisement,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub at: DateTime<Utc>,
    pub adapter: String,
    pub action: String,
    pub detail: String,
}
#[derive(Clone, Debug)]
pub struct Packet {
    pub at: DateTime<Utc>,
    pub adapter: String,
    pub bytes: Vec<u8>,
}
#[derive(Default)]
pub struct Monitor {
    pub neighbors: Vec<Neighbor>,
    pub events: VecDeque<Event>,
    pub packets: VecDeque<Packet>,
    pub received: u64,
    pub rejected: u64,
}
impl Monitor {
    pub fn log(&mut self, adapter: &str, action: &str, detail: String) {
        self.events.push_front(Event {
            at: Utc::now(),
            adapter: adapter.into(),
            action: action.into(),
            detail,
        });
        self.events.truncate(2000);
    }
    pub fn ingest(
        &mut self,
        adapter: &str,
        bytes: Vec<u8>,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        let a = protocol::parse(&bytes).inspect_err(|_| self.rejected += 1)?;
        self.received += 1;
        self.packets.push_back(Packet {
            at: now,
            adapter: adapter.into(),
            bytes,
        });
        while self.packets.len() > 2048 {
            self.packets.pop_front();
        }
        let i = self.neighbors.iter().position(|n| {
            n.adapter == adapter
                && n.advertisement.chassis_id == a.chassis_id
                && n.advertisement.port_id == a.port_id
        });
        let detail = format!(
            "{} / {}",
            if a.system_name.is_empty() {
                &a.chassis_id
            } else {
                &a.system_name
            },
            a.port_id
        );
        if a.ttl == 0 {
            if let Some(i) = i {
                self.neighbors.remove(i);
                self.log(adapter, "shutdown", detail);
            }
            return Ok(());
        }
        let expires = now + chrono::Duration::seconds(a.ttl as i64);
        if let Some(i) = i {
            let n = &mut self.neighbors[i];
            let changed = n.advertisement != a;
            n.last_seen = now;
            n.expires = expires;
            n.advertisement = a;
            if changed {
                self.log(adapter, "changed", detail);
            }
        } else {
            if self.neighbors.len() >= 4096 {
                return Err("Neighbor limit reached".into());
            }
            self.neighbors.push(Neighbor {
                adapter: adapter.into(),
                first_seen: now,
                last_seen: now,
                expires,
                advertisement: a,
            });
            self.log(adapter, "discovered", detail);
        }
        Ok(())
    }
    pub fn expire(&mut self, now: DateTime<Utc>) {
        let mut expired = vec![];
        self.neighbors.retain(|n| {
            if n.expires <= now {
                expired.push((
                    n.adapter.clone(),
                    format!(
                        "{} / {}",
                        n.advertisement.chassis_id, n.advertisement.port_id
                    ),
                ));
                false
            } else {
                true
            }
        });
        for (a, d) in expired {
            self.log(&a, "expired", d);
        }
    }
}
