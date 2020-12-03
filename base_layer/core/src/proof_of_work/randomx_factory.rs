use std::collections::HashMap;
use randomx_rs::{RandomXVM, RandomXCache, RandomXDataset, RandomXFlag};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;
use crate::proof_of_work::monero_rx::MergeMineError;

const MAX_VMS :usize = 5;

pub struct RandomXFactory {
    vms : HashMap<Vec<u8>, (Instant, Mutex<RandomXVM>)>
}

impl RandomXFactory {
    pub fn create(&mut self, key: &Vec<u8>) -> Result<MutexGuard<RandomXVM>, MergeMineError>  {

        if let Some(entry) = self.vms.get_mut(key) {
            let vm = entry.1.lock().unwrap();
            entry.0 = Instant::now();
            return Ok(vm);
        }

        if self.vms.len() > MAX_VMS {

            let mut oldest_value = Instant::now();
            let  mut oldest_key = None;
            for (k, v) in  self.vms {
                if v.0 < oldest_value {
                    oldest_key = Some(k.clone());
                }
            }
            if let Some(k) = oldest_key {
                self.vms.remove(&k);
            }
        }

        let flags = RandomXFlag::get_recommended_flags();
        let cache = RandomXCache::new(flags, &key)?;
        let dataset = RandomXDataset::new(flags, &cache, 0)?;
        let vm = RandomXVM::new(flags, Some(&cache), Some(&dataset))?;

        let  mutex = Mutex::new(vm);
        let res = mutex.lock().unwrap();
        self.vms.insert(key.clone(),  (Instant::now(), mutex));

        Ok(res)
    }
}
