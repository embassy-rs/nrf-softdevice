use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use std::{env, fs};

use regex::{Captures, Regex};
use walkdir::WalkDir;

pub fn gen_bindings(tmp_dir: &PathBuf, src_dir: &PathBuf, dst: &PathBuf, mut f: impl FnMut(String) -> String) {
    let mut wrapper = String::new();

    for entry in WalkDir::new(src_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let _f_name = entry.path().to_string_lossy();
        if entry.file_type().is_file() {
            if entry.file_name().to_string_lossy() == "nrf_nvic.h" {
                continue;
            }

            let data = fs::read_to_string(entry.path()).unwrap();
            let data = f(data);
            fs::write(tmp_dir.join(entry.file_name()), data.as_bytes()).unwrap();

            writeln!(&mut wrapper, "#include \"{}\"", entry.file_name().to_string_lossy()).unwrap();
        }
    }
    fs::write(tmp_dir.join("nrf.h"), &[]).unwrap();
    fs::write(tmp_dir.join("wrapper.h"), wrapper.as_bytes()).unwrap();

    bindgen::Builder::default()
        .use_core()
        .header(tmp_dir.join("wrapper.h").to_str().unwrap())
        .generate_comments(true)
        .size_t_is_usize(true)
        .ctypes_prefix("self")
        .clang_arg("-DSVCALL_AS_NORMAL_FUNCTION")
        .clang_arg("-D__STATIC_INLINE=")
        .clang_arg("--target=thumbv7em-none-eabihf")
        .clang_arg("-mcpu=cortex-m4")
        .clang_arg("-mthumb")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(dst)
        .expect("Couldn't write bindings!");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let src_dir = PathBuf::from(&args[1]);
    let dst_path = PathBuf::from(&args[2]);

    let tmp_dir = PathBuf::from("./tmp");
    let tmp_bindings_path = tmp_dir.join("bindings.rs");

    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    gen_bindings(&tmp_dir, &src_dir, &tmp_bindings_path, |data| {
        let re = Regex::new(
            r"SVCALL\((?P<svc>[A-Za-z0-9_]+),\s*(?P<ret>[A-Za-z0-9_]+),\s*(?P<name>[A-Za-z0-9_]+)\((?P<args>.*)\)\);",
        )
        .unwrap();
        re.replace_all(&data, "uint32_t __svc_$name = $svc;").into()
    });

    let mut svc_nums = HashMap::new();

    let data = fs::read_to_string(&tmp_bindings_path).unwrap();
    let re = Regex::new(r"pub const __svc_(?P<name>[A-Za-z0-9_]+): u32 = (?P<num>\d+);").unwrap();
    for m in re.captures_iter(&data) {
        let name = m.name("name").unwrap().as_str();
        let num = m.name("num").unwrap().as_str().parse::<u32>().unwrap();
        svc_nums.insert(name, num);
    }

    gen_bindings(&tmp_dir, &src_dir, &tmp_bindings_path, |data| {
        // Change all "dynamically-sized" arrays from length 1 to length 0.
        // This avoids UB when creating Rust references to structs when the length is 0.
        //
        // We can't use a "real" flexible array ([] instead of [0]) because
        // they're used inside enums :(
        let data = data.replace("[1];", "[0];");

        let re = Regex::new(
            r"SVCALL\((?P<svc>[A-Za-z0-9_]+),\s*(?P<ret>[A-Za-z0-9_]+),\s*(?P<name>[A-Za-z0-9_]+)\((?P<args>.*)\)\);",
        )
        .unwrap();
        re.replace_all(&data, "$ret $name($args);").into()
    });

    let data = fs::read_to_string(&tmp_bindings_path).unwrap();
    let re = Regex::new(
        r#"extern "C" \{(?P<doc>(?s).*?)pub fn (?P<name>sd_[A-Za-z0-9_]+)\((?P<args>(?s).*?)\) -> u32;\s+\}"#,
    )
    .unwrap();
    let data = re.replace_all(&data, |m: &Captures| {
        let doc = m.name("doc").unwrap().as_str();
        let name = m.name("name").unwrap().as_str();
        let args = m.name("args").unwrap().as_str();
        let num = match svc_nums.get(name) {
            Some(x) => *x,

            // This happens in 2 inline functions in ble_gattc.h
            // They aren't real softdevice calls, so skip them.
            None => return String::new(),
        };

        let mut res = String::new();

        writeln!(
            &mut res,
            "{}\n#[inline(always)]\npub unsafe fn {}({}) -> u32 {{",
            doc, name, args
        )
        .unwrap();

        let arg_names = args
            .split(',')
            .map(|s| s.trim())
            .filter(|s| s.len() != 0)
            .map(|s| s.splitn(2, ':').next().unwrap().trim())
            .collect::<Vec<&str>>();

        writeln!(&mut res, "    let ret: u32;",).unwrap();
        writeln!(&mut res, "    core::arch::asm!(\"svc {}\",", num).unwrap();

        assert!(arg_names.len() <= 4);
        for r in 0..4 {
            if r >= arg_names.len() {
                if r == 0 {
                    writeln!(&mut res, "        lateout(\"r{}\") ret,", r).unwrap();
                } else {
                    writeln!(&mut res, "        lateout(\"r{}\") _,", r).unwrap();
                }
            } else {
                let arg = arg_names[r];
                let out = if r == 0 { "ret" } else { "_" };
                writeln!(&mut res, "        inout(\"r{}\") to_asm({}) => {},", r, arg, out).unwrap();
            }
        }
        writeln!(&mut res, "        lateout(\"r12\") _,").unwrap();
        writeln!(&mut res, "    );").unwrap();
        writeln!(&mut res, "    ret").unwrap();
        writeln!(&mut res, "}}",).unwrap();

        res
    });

    let mut res = Vec::new();
    res.extend(HEADER.as_bytes());
    res.extend(data.as_bytes());

    fs::write(dst_path, &res).unwrap();
}

static HEADER: &str = r#"
/*
 * Copyright (c) 2012 - 2019, Nordic Semiconductor ASA
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form, except as embedded into a Nordic
 *    Semiconductor ASA integrated circuit in a product or a software update for
 *    such product, must reproduce the above copyright notice, this list of
 *    conditions and the following disclaimer in the documentation and/or other
 *    materials provided with the distribution.
 *
 * 3. Neither the name of Nordic Semiconductor ASA nor the names of its
 *    contributors may be used to endorse or promote products derived from this
 *    software without specific prior written permission.
 *
 * 4. This software, with or without modification, must only be used with a
 *    Nordic Semiconductor ASA integrated circuit.
 *
 * 5. Any software provided in binary form under this license must not be reverse
 *    engineered, decompiled, modified and/or disassembled.
 *
 * THIS SOFTWARE IS PROVIDED BY NORDIC SEMICONDUCTOR ASA "AS IS" AND ANY EXPRESS
 * OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY, NONINFRINGEMENT, AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL NORDIC SEMICONDUCTOR ASA OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE
 * GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT
 * OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_char = u8;

pub type c_short = i16;
pub type c_ushort = u16;

pub type c_int = i32;
pub type c_uint = u32;

pub type c_long = i32;
pub type c_ulong = u32;

pub type c_longlong = i64;
pub type c_ulonglong = u64;

pub type c_void = core::ffi::c_void;

trait ToAsm {
    fn to_asm(self) -> u32;
}

fn to_asm<T: ToAsm>(t: T) -> u32 {
    t.to_asm()
}

impl ToAsm for u32 {
    fn to_asm(self) -> u32 {
        self
    }
}

impl ToAsm for u16 {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl ToAsm for u8 {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl ToAsm for i8 {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl<T> ToAsm for *const T {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl<T> ToAsm for *mut T {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl<T: ToAsm> ToAsm for Option<T> {
    fn to_asm(self) -> u32 {
        match self {
            Some(x) => x.to_asm(),
            None => 0,
        }
    }
}

impl<X, R> ToAsm for unsafe extern "C" fn(X) -> R {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl<X, Y, R> ToAsm for unsafe extern "C" fn(X, Y) -> R {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

impl<X, Y, Z, R> ToAsm for unsafe extern "C" fn(X, Y, Z) -> R {
    fn to_asm(self) -> u32 {
        self as u32
    }
}

"#;
