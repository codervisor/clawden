use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// How an agent was discovered / registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// Manually registered via API.
    Manual,
    /// Discovered via network scan (port probe).
    NetworkScan,
    /// Discovered via DNS-SD / mDNS.
    DnsSd,
}

/// A discovered endpoint that *may* host a claw agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredEndpoint {
    pub host: String,
    pub port: u16,
    pub method: DiscoveryMethod,
    pub runtime_hint: Option<String>,
}

/// Manages known endpoints and discovery state.
#[derive(Default)]
pub struct DiscoveryService {
    endpoints: HashMap<String, DiscoveredEndpoint>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an endpoint manually.
    pub fn register_endpoint(&mut self, endpoint: DiscoveredEndpoint) -> String {
        let key = format!("{}:{}", endpoint.host, endpoint.port);
        self.endpoints.insert(key.clone(), endpoint);
        key
    }

    /// Remove a known endpoint.
    #[allow(dead_code)]
    pub fn remove_endpoint(&mut self, key: &str) -> bool {
        self.endpoints.remove(key).is_some()
    }

    /// List all known endpoints.
    pub fn list_endpoints(&self) -> Vec<&DiscoveredEndpoint> {
        self.endpoints.values().collect()
    }

    /// Simulate a network scan on a set of well-known ports.
    /// In a real implementation this would attempt TCP connects;
    /// here we return any manually-registered endpoints that match.
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

    /// Simulate DNS-SD discovery. Returns all endpoints registered
    /// with `DnsSd` method.
    #[allow(dead_code)]
    pub fn discover_dns_sd(&self) -> Vec<DiscoveredEndpoint> {
        self.endpoints
            .values()
            .filter(|ep| ep.method == DiscoveryMethod::DnsSd)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_list() {
        let mut svc = DiscoveryService::new();
        svc.register_endpoint(DiscoveredEndpoint {
            host: "10.0.0.1".to_string(),
            port: 8080,
            method: DiscoveryMethod::Manual,
            runtime_hint: Some("openclaw".to_string()),
        });

        assert_eq!(svc.list_endpoints().len(), 1);
    }

    #[test]
    fn scan_finds_registered_endpoints() {
        let mut svc = DiscoveryService::new();
        svc.register_endpoint(DiscoveredEndpoint {
            host: "10.0.0.1".to_string(),
            port: 18789,
            method: DiscoveryMethod::NetworkScan,
            runtime_hint: None,
        });

        let found = svc.scan_ports(&["10.0.0.1".to_string()], &[18789, 42617]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].port, 18789);
    }

    #[test]
    fn dns_sd_filters_correctly() {
        let mut svc = DiscoveryService::new();
        svc.register_endpoint(DiscoveredEndpoint {
            host: "agent1.local".to_string(),
            port: 8080,
            method: DiscoveryMethod::DnsSd,
            runtime_hint: Some("zeroclaw".to_string()),
        });
        svc.register_endpoint(DiscoveredEndpoint {
            host: "10.0.0.2".to_string(),
            port: 8080,
            method: DiscoveryMethod::Manual,
            runtime_hint: None,
        });

        let dns_results = svc.discover_dns_sd();
        assert_eq!(dns_results.len(), 1);
        assert_eq!(dns_results[0].host, "agent1.local");
    }
}
