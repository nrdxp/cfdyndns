use cloudflare::endpoints::dns::{DnsRecord, Meta};
pub trait Clone_ {
	fn clone(&self) -> Self;
}

impl Clone_ for DnsRecord {
	fn clone(&self) -> Self {
		Self {
			name: self.name.to_owned(),
			meta: Meta {
				auto_added: self.meta.auto_added,
			},
			locked: self.locked,
			ttl: self.ttl,
			zone_id: self.zone_id.to_owned(),
			modified_on: self.modified_on,
			created_on: self.created_on,
			proxiable: self.proxiable,
			proxied: self.proxied,
			content: self.content.clone(),
			id: self.id.to_owned(),
			zone_name: self.zone_name.to_owned(),
		}
	}
}
