use web_sys::KeyboardEvent;

pub fn mask_lower_kebab_case(ev: KeyboardEvent) {
    match ev.char_code() {
        97..=122 => (), // lowercase letters
        48..=57 => (), // numbers
        45 => (), // the hyphen character
        _ => ev.prevent_default(),
    }
}

pub fn mask_numbers(ev: KeyboardEvent) {
    match ev.char_code() {
        48..=57 => (), // numbers
        _ => ev.prevent_default(),
    }
}

