//! [`actix_governor`] key extractor: rate-limit by peer IP when present, otherwise
//! `127.0.0.1`. Test harness requests often omit `peer_addr`; without a fallback the
//! default [`PeerIpKeyExtractor`] returns 500.

use std::net::{IpAddr, Ipv4Addr};

use actix_governor::KeyExtractor;
use actix_web::dev::ServiceRequest;

/// Same IPv6 /56 normalization as [`actix_governor::PeerIpKeyExtractor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerOrFallbackIpKeyExtractor;

fn normalize_ip_key(mut ip: IpAddr) -> IpAddr {
    if let IpAddr::V6(ipv6) = ip {
        let mut octets = ipv6.octets();
        octets[7..16].fill(0);
        ip = IpAddr::V6(octets.into());
    }
    ip
}

impl KeyExtractor for PeerOrFallbackIpKeyExtractor {
    type Key = IpAddr;
    type KeyExtractionError = actix_governor::SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let ip = req
            .peer_addr()
            .map(|s| normalize_ip_key(s.ip()))
            .unwrap_or_else(|| IpAddr::V4(Ipv4Addr::LOCALHOST));
        Ok(ip)
    }
}
