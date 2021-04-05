#[cfg(windows)] extern crate winapi;
use std::io::Error;
use std::path::Path;

#[cfg(windows)]
fn press_key(key: u16, flags: u32) -> Result<(), Error> {
    use winapi::um::winuser::{INPUT_u, INPUT, INPUT_KEYBOARD, KEYBDINPUT, SendInput};

    let mut input_u: INPUT_u = unsafe { std::mem::zeroed() };

    unsafe {
        *input_u.ki_mut() = KEYBDINPUT {
            wVk: key,
            dwExtraInfo: 0,
            wScan: 0,
            time: 0,
            dwFlags: flags,
        }
    }

    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: input_u,
    };
    let ipsize = std::mem::size_of::<INPUT>() as i32;
    unsafe {
        SendInput(1, &mut input, ipsize);
    };
    Ok(())
}

#[cfg(windows)]
fn send_char(c: char) -> Result<(), Error> {
    use winapi::um::winuser::VkKeyScanW;
    const KEYUP: u32 = 0x0002;

    let key_enc = unsafe { VkKeyScanW(c as u16) };
    let key = (key_enc & 0xff) as u16;

    if key_enc >> 8 & 0x1 == 1 {
        press_key(0x10, 0)?;
        press_key(key, 0)?;
        press_key(key, KEYUP)?;
        press_key(0x10, KEYUP)?;
    } else {
        press_key(key, 0)?;
        press_key(key, KEYUP)?;
    }
    Ok(())
}

#[cfg(windows)]
fn send_str(s: &str) -> Result<(), Error> {
    for c in s.chars() {
        send_char(c)?;
    }
    send_char('\n')?;
    Ok(())
}

#[cfg(not(windows))]
fn send_str(_s: &str) -> Result<(), Error> {
    println!("--keep flag not yet implemented on *nix");
    Ok(()) // TODO: make it work
}

pub fn cwd_host(path: &Path) -> Result<(), Error> {
    if let Some(s) = path.to_str() {
        let command = format!("pushd {}", s);
        send_str(command.as_str())?;
    }
    Ok(())
}