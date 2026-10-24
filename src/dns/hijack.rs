use std::net::{Ipv4Addr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::error::AppError;

fn build_dns_response(query: &[u8], ap_ip: Ipv4Addr) -> Option<Vec<u8>> {
    if query.len() < 12 {
        return None;
    }

    let mut resp = Vec::with_capacity(512);
    // Transaction ID
    resp.extend_from_slice(&query[0..2]);
    // Flags: standard response, recursion available, no error
    resp.extend_from_slice(&[0x81, 0x80]);
    // QDCOUNT
    resp.extend_from_slice(&query[4..6]);
    // ANCOUNT = 1
    resp.extend_from_slice(&[0x00, 0x01]);
    // NSCOUNT + ARCOUNT = 0
    resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    let mut pos = 12;
    while pos < query.len() && query[pos] != 0 {
        pos += 1 + query[pos] as usize;
    }
    pos += 5; // null + QTYPE + QCLASS
    if pos > query.len() {
        return None;
    }
    resp.extend_from_slice(&query[12..pos]);

    resp.extend_from_slice(&[0xC0, 0x0C]); // name pointer
    resp.extend_from_slice(&[0x00, 0x01]); // type A
    resp.extend_from_slice(&[0x00, 0x01]); // class IN
    resp.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]); // TTL 60s
    resp.extend_from_slice(&[0x00, 0x04]); // RDLENGTH
    resp.extend_from_slice(&ap_ip.octets());

    Some(resp)
}

pub struct Hijack {
    ap_ip: Ipv4Addr,
    portal_active: Arc<AtomicBool>,
}

impl Hijack {
    pub fn new(ap_ip: Ipv4Addr, portal_active: Arc<AtomicBool>) -> Self {
        Self {
            ap_ip,
            portal_active,
        }
    }

    pub fn start_dns_hijack(self) -> thread::JoinHandle<()> {
        let flag = Arc::clone(&self.portal_active);
        let ap_ip = self.ap_ip;

        thread::Builder::new()
            .name("dns-hijack".into())
            .stack_size(4 * 1024)
            .spawn(move || {
                let socket = UdpSocket::bind("0.0.0.0:53").expect("bind 53");
                socket
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .ok();

                let mut buf = [0u8; 512];
                while flag.load(Ordering::SeqCst) {
                    match socket.recv_from(&mut buf) {
                        Ok((len, src)) => {
                            if let Some(resp) = build_dns_response(&buf[..len], ap_ip) {
                                let _ = socket.send_to(&resp, src);
                            }
                        }
                        Err(ref e)
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                || e.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(e) => log::warn!("dns recv: {}", e),
                    }
                }
            })
            .unwrap()
    }
}
