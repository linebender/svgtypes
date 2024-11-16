// Copyright 2018 the SVG Types Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_main]

#[macro_use] extern crate libfuzzer_sys;
extern crate svgtypes;

use std::str;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        // Must not panic.
        let mut n = 0;
        for _ in svgtypes::PathParser::from(s) {
            n += 1;

            if n == 1000 {
                panic!("endless loop");
            }
        }
    }
});
