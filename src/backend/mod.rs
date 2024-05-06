use std::{ops::Deref, sync::Arc};

use dashmap::{DashMap, DashSet};

use crate::{RespFrame, RespNull};

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
    pub(crate) map: DashMap<String, RespFrame>,
    pub(crate) hmap: DashMap<String, DashMap<String, RespFrame>>,
    pub(crate) set: DashMap<String, DashSet<RespFrame>>,
}

impl Deref for Backend {
    type Target = BackendInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Backend(Arc::new(BackendInner::default()))
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        BackendInner {
            map: DashMap::new(),
            hmap: DashMap::new(),
            set: DashMap::new(),
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Backend::default()
    }

    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        let hmap = self.hmap.entry(key).or_default();
        hmap.insert(field, value);
    }

    pub fn hmget(&self, key: &str, fields: &[String]) -> Option<Vec<RespFrame>> {
        match self.hmap.get(key) {
            Some(map) => {
                let ret = fields
                    .iter()
                    .map(|field| match map.get(field) {
                        Some(v) => v.value().clone(),
                        None => RespNull.into(),
                    })
                    .collect();

                Some(ret)
            }
            None => None,
        }
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }

    pub fn sadd(&self, key: String, members: Vec<RespFrame>) -> i64 {
        let set = self.set.entry(key).or_default();
        let mut count = 0;
        for member in members {
            if set.insert(member) {
                count += 1;
            }
        }

        count
    }

    pub fn sismembers(&self, key: &str) -> Option<DashSet<RespFrame>> {
        self.set.get(key).map(|v| v.clone())
    }
}
