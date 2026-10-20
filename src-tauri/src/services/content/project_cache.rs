//! Caché en memoria de fichas de proyecto (`ContentProjectDetail`) — la
//! ventana de log y el resto del launcher no cachean nada hoy, pero una
//! ficha pesa bastante más que un `ContentSearchHit` (descripción completa,
//! galería) y se vuelve a pedir seguido si el usuario abre/cierra el mismo
//! proyecto mientras explora. Sin esto, cada reapertura sería un round-trip
//! a Modrinth por nada.
//!
//! TTL corto (10 minutos: alcanza para una sesión de exploración, no deja
//! datos desactualizados por días) y tope de entradas con desalojo LRU
//! simple — "límites claros", no un `HashMap` sin techo.

use super::types::ContentProjectDetail;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

/// Única instancia para todo el proceso — `ContentService::new()` crea un
/// servicio liviano por comando (ver `content::service()`), así que la
/// caché NO puede vivir ahí adentro o se tiraría entera en cada llamada.
/// Mismo patrón que `core::http_client::HTTP`.
pub static PROJECT_CACHE: LazyLock<ProjectCache> = LazyLock::new(ProjectCache::new);

const TTL: Duration = Duration::from_secs(10 * 60);
const MAX_ENTRIES: usize = 64;

struct CacheEntry {
    detail: ContentProjectDetail,
    fetched_at: Instant,
}

#[derive(Default)]
pub struct ProjectCache {
    // Orden de inserción/acceso más reciente al final — el primero en
    // `order` es el próximo candidato a desalojar.
    entries: Mutex<(HashMap<String, CacheEntry>, Vec<String>)>,
}

impl ProjectCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// `None` si no hay entrada vigente (nunca cacheada, o vencida).
    pub fn get(&self, project_id: &str) -> Option<ContentProjectDetail> {
        let mut guard = self.entries.lock().unwrap();
        let (map, order) = &mut *guard;
        let entry = map.get(project_id)?;
        if entry.fetched_at.elapsed() > TTL {
            map.remove(project_id);
            order.retain(|id| id != project_id);
            return None;
        }
        let detail = entry.detail.clone();
        order.retain(|id| id != project_id);
        order.push(project_id.to_string());
        Some(detail)
    }

    pub fn put(&self, project_id: String, detail: ContentProjectDetail) {
        let mut guard = self.entries.lock().unwrap();
        let (map, order) = &mut *guard;
        if !map.contains_key(&project_id) && map.len() >= MAX_ENTRIES {
            if let Some(oldest) = order.first().cloned() {
                map.remove(&oldest);
                order.remove(0);
            }
        }
        order.retain(|id| id != &project_id);
        order.push(project_id.clone());
        map.insert(
            project_id,
            CacheEntry {
                detail,
                fetched_at: Instant::now(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(id: &str) -> ContentProjectDetail {
        ContentProjectDetail {
            project_id: id.to_string(),
            title: "T".to_string(),
            icon_url: None,
            description: "d".to_string(),
            body: String::new(),
            categories: vec![],
            game_versions: vec![],
            loaders: vec![],
            downloads: 0,
            followers: 0,
            published: String::new(),
            updated: String::new(),
            source_url: None,
            wiki_url: None,
            discord_url: None,
            issues_url: None,
            gallery: vec![],
        }
    }

    #[test]
    fn miss_then_hit_after_put() {
        let cache = ProjectCache::new();
        assert!(cache.get("p1").is_none());
        cache.put("p1".to_string(), sample("p1"));
        assert_eq!(cache.get("p1").unwrap().project_id, "p1");
    }

    #[test]
    fn evicts_oldest_beyond_capacity() {
        let cache = ProjectCache::new();
        for i in 0..MAX_ENTRIES {
            cache.put(format!("p{i}"), sample(&format!("p{i}")));
        }
        // Todavía no se accedió a "p0" de nuevo, así que sigue siendo el
        // más antiguo en orden de acceso.
        cache.put("new".to_string(), sample("new"));
        assert!(cache.get("p0").is_none());
        assert!(cache.get("new").is_some());
        assert_eq!(cache.get("p1").unwrap().project_id, "p1");
    }

    #[test]
    fn accessing_an_entry_protects_it_from_eviction() {
        let cache = ProjectCache::new();
        for i in 0..MAX_ENTRIES {
            cache.put(format!("p{i}"), sample(&format!("p{i}")));
        }
        // Tocar "p0" lo manda al final del orden de acceso.
        assert!(cache.get("p0").is_some());
        cache.put("new".to_string(), sample("new"));
        assert!(cache.get("p0").is_some());
        assert!(cache.get("p1").is_none());
    }
}
