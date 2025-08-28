#[unsafe(no_mangle)]
pub extern "C" fn host_pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}
