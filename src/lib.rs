use lz4_flex::block::{compress_prepend_size, decompress_size_prepended};
use std::collections::BTreeMap;
use wyhash::{wyhash, wyrng};

pub struct SmallSet {
    seeds: Vec<u64>,
    buckets: Vec<BTreeMap<u64, (usize, usize)>>,
    data: Vec<u8>,
}

impl SmallSet {
    pub fn new<'a, Iter: Iterator<Item = &'a [u8]>>(keys: Iter) -> Self {
        let mut rand_seed = 3;

        let mut buf = Vec::new();
        let mut buckets = vec![BTreeMap::new()];
        let mut seeds = vec![wyrng(&mut rand_seed)];

        for key in keys {
            let mut found_spot = false;
            for (seed, bucket) in seeds.iter().zip(buckets.iter_mut()) {
                let hash = wyhash(key, *seed);

                if let std::collections::btree_map::Entry::Vacant(e) = bucket.entry(hash) {
                    let offset = buf.len();
                    buf.extend_from_slice(key);

                    e.insert((offset, offset + key.len()));

                    found_spot = true;
                    break;
                }
            }

            if !found_spot {
                let seed = wyrng(&mut rand_seed);
                let mut bucket = BTreeMap::new();

                let hash = wyhash(key, seed);

                let offset = buf.len();
                buf.extend_from_slice(key);

                bucket.insert(hash, (offset, offset + key.len()));

                buckets.push(bucket);
                seeds.push(seed);
            }
        }

        let data = compress_prepend_size(&buf);

        Self {
            seeds,
            buckets,
            data,
        }
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        let mut offsets = Vec::new();

        for (seed, bucket) in self.seeds.iter().zip(self.buckets.iter()) {
            let hash = wyhash(key, *seed);
            if let Some(offset) = bucket.get(&hash) {
                offsets.push(*offset);
            }
        }

        if offsets.is_empty() {
            return false;
        }

        let data = decompress_size_prepended(&self.data).unwrap();

        for offset in offsets {
            if data.get(offset.0..offset.1).unwrap() == key {
                return true;
            }
        }

        false
    }
}
