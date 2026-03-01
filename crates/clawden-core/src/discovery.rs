use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    Manual,
    NetworkScan,
    DnsSd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredEndpoint {
    pub host: String,
    pub port: u16,
    pub method: DiscoveryMethod,
    pub runtime_hint: Option<String>,
}

#[derive(Default)]
pub struct DiscoveryService {
    endpoints: HashMap<String, DiscoveredEndpoint>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_endpoint(&mut self, endpoint: DiscoveredEndpoint) -> String {
        let key = format!("{}:{}", endpoint.host, endpoint.port);
        self.endpoints.insert(key.clone(), endpoint);
        key
    }

    pub fn list_endpoints(&self) -> Vec<&DiscoveredEndpoint> {
        self.endpoints.values().collect()
    }

    pub fn scan_ports(&self, hosts: &[String], ports: &[u16]) -> Vec<DiscoveredEndpoint> {
        let mut results = Vec::new();
        for host in hosts {
            for &port in ports {
                let key = format!("{host}:{port}");
                if let Some(ep) = self.endpoints.get(&key) {
                    results.push(ep.clone());
                }
            }
        }
        results
    }
}
