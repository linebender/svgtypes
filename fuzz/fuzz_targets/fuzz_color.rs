// Copyright 2018 the SVG Types Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_main]

#[macro_use] extern crate libfuzzer_sys;
extern crate svgtypes;

use std::str::FromStr;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Must not panic.
        let _ = svgtypes::Color::from_str(s);
    }
});
