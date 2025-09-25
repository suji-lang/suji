//! RNG core backed by rand::rngs::StdRng with global singleton.

use once_cell::sync::OnceCell;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

static RNG: OnceCell<Mutex<StdRng>> = OnceCell::new();
static USER_SEEDED: AtomicBool = AtomicBool::new(false);

fn get_or_init_rng() -> &'static Mutex<StdRng> {
    RNG.get_or_init(|| {
        // Default seed from time for initial creation
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let rng = StdRng::seed_from_u64(now_ms);
        Mutex::new(rng)
    })
}

pub fn rng_seed_with(opt_seed: Option<u64>) {
    match opt_seed {
        Some(s) => {
            USER_SEEDED.store(true, Ordering::Relaxed);
            let mut guard = get_or_init_rng().lock().expect("rng mutex poisoned");
            *guard = StdRng::seed_from_u64(s);
        }
        None => {
            // Only apply default seeding if user has not seeded yet
            if USER_SEEDED.load(Ordering::Relaxed) {
                return;
            }
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let mut guard = get_or_init_rng().lock().expect("rng mutex poisoned");
            *guard = StdRng::seed_from_u64(seed);
        }
    }
}

pub fn rng_f64() -> f64 {
    let mut guard = get_or_init_rng().lock().expect("rng mutex poisoned");
    guard.r#gen::<f64>()
}
