#[cfg(target_os = "windows")]
pub fn beep() {
    win_beep::beep_with_hz_and_millis(1000, 100);
}

#[cfg(not(target_os = "windows"))]
pub fn beep() {
    println!("{}", '\x07')
}