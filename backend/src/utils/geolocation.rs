use reqwest;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize)]
pub struct GeolocationResponse {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IpApiResponse {
    status: String,
    country: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    city: Option<String>,
    timezone: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    message: Option<String>,
}

pub struct GeolocationService {
    client: reqwest::Client,
}

impl GeolocationService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Resolve location from IP address using ip-api.com (free service)
    pub async fn resolve_location(&self, ip_address: &str) -> Option<String> {
        // Handle local/private IPs
        if self.is_local_ip(ip_address) {
            return Some("Local Network".to_string());
        }

        // Validate IP address format
        if ip_address.parse::<IpAddr>().is_err() {
            return None;
        }

        match self.fetch_geolocation_data(ip_address).await {
            Ok(data) => self.format_location_string(&data),
            Err(_) => None,
        }
    }

    /// Get detailed geolocation data
    #[allow(dead_code)]
    pub async fn get_geolocation_details(&self, ip_address: &str) -> Option<GeolocationResponse> {
        if self.is_local_ip(ip_address) {
            return Some(GeolocationResponse {
                country: Some("Local".to_string()),
                region: Some("Network".to_string()),
                city: None,
                timezone: None,
                lat: None,
                lon: None,
            });
        }

        if ip_address.parse::<IpAddr>().is_err() {
            return None;
        }

        match self.fetch_geolocation_data(ip_address).await {
            Ok(data) => Some(GeolocationResponse {
                country: data.country,
                region: data.region_name,
                city: data.city,
                timezone: data.timezone,
                lat: data.lat,
                lon: data.lon,
            }),
            Err(_) => None,
        }
    }

    async fn fetch_geolocation_data(&self, ip_address: &str) -> Result<IpApiResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("http://ip-api.com/json/{}", ip_address);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        let data: IpApiResponse = response.json().await?;

        if data.status == "success" {
            Ok(data)
        } else {
            let error_msg = data.message.unwrap_or_else(|| "Geolocation lookup failed".to_string());
            Err(error_msg.into())
        }
    }

    fn format_location_string(&self, data: &IpApiResponse) -> Option<String> {
        match (&data.city, &data.region_name, &data.country) {
            (Some(city), Some(region), Some(country)) => {
                if city != region {
                    Some(format!("{}, {}, {}", city, region, country))
                } else {
                    Some(format!("{}, {}", city, country))
                }
            }
            (None, Some(region), Some(country)) => Some(format!("{}, {}", region, country)),
            (Some(city), None, Some(country)) => Some(format!("{}, {}", city, country)),
            (None, None, Some(country)) => Some(country.clone()),
            _ => None,
        }
    }

    fn is_local_ip(&self, ip_address: &str) -> bool {
        match ip_address.parse::<IpAddr>() {
            Ok(IpAddr::V4(ipv4)) => {
                // Check for localhost, private ranges, and link-local
                ipv4.is_loopback()
                    || ipv4.is_private()
                    || ipv4.is_link_local()
                    || ipv4.is_unspecified()
            }
            Ok(IpAddr::V6(ipv6)) => {
                // Check for localhost, private ranges, and link-local
                ipv6.is_loopback()
                    || ipv6.is_unspecified()
                    || (ipv6.segments()[0] & 0xfe00) == 0xfc00  // Unique local addresses
                    || (ipv6.segments()[0] & 0xffc0) == 0xfe80  // Link-local addresses
            }
            Err(_) => ip_address == "127.0.0.1" || ip_address == "::1" || ip_address == "localhost",
        }
    }
}

impl Default for GeolocationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_local_ip() {
        let service = GeolocationService::new();

        // Test IPv4 local addresses
        assert!(service.is_local_ip("127.0.0.1"));
        assert!(service.is_local_ip("192.168.1.1"));
        assert!(service.is_local_ip("10.0.0.1"));
        assert!(service.is_local_ip("172.16.0.1"));

        // Test IPv6 local addresses
        assert!(service.is_local_ip("::1"));

        // Test public IP (should not be local)
        assert!(!service.is_local_ip("8.8.8.8"));
        assert!(!service.is_local_ip("1.1.1.1"));
    }
}