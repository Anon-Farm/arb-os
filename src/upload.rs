/*
 * Copyright 2021, Offchain Labs, Inc. All rights reserved.
 */

use crate::link::LinkedProgram;
use crate::mavm::{AVMOpcode, Instruction};
use rustc_hex::ToHex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CodeUploader {
    build_buffer: Vec<u8>,
    pub instructions: Vec<Vec<u8>>,
    num_so_far: usize,
    num_total: usize,
}

impl CodeUploader {
    pub fn _new(num_total: usize) -> Self {
        Self {
            build_buffer: vec![],
            instructions: vec![],
            num_so_far: 0,
            num_total,
        }
    }

    pub fn _new_from_file(path: &Path) -> Self {
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {:?}", path.display(), why),
            Ok(file) => file,
        };

        let mut s = String::new();
        s = match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {:?}", path.display(), why),
            Ok(_) => s,
        };

        let parse_result: Result<LinkedProgram, serde_json::Error> = serde_json::from_str(&s);
        match parse_result {
            Ok(prog) => {
                let code_len = prog.code.len();
                let mut ret = CodeUploader::_new(code_len);
                for i in 0..prog.code.len() {
                    ret._serialize_one(&prog.code[code_len - 1 - i]);
                }
                ret
            }
            Err(_) => {
                panic!();
            }
        }
    }

    pub fn push_byte(&mut self, b: u8) {
        self.build_buffer.push(b);
    }

    pub fn push_bytes(&mut self, b: &[u8]) {
        self.build_buffer.extend(b);
    }

    pub fn _serialize_one(&mut self, insn: &Instruction<AVMOpcode>) {
        insn._upload(self);
        self.num_so_far = self.num_so_far + 1;
        self._finish_batch();
    }

    pub fn _translate_pc(&self, pc: usize) -> usize {
        self.num_total - pc
    }

    fn _finish_batch(&mut self) {
        if self.build_buffer.len() > 0 {
            self.instructions.push(self.build_buffer.clone());
            self.build_buffer = vec![];
        }
    }

    pub fn _finalize(&mut self) -> Vec<Vec<u8>> {
        self.instructions.clone()
    }

    pub fn _to_flat_vec(&self) -> Vec<u8> {
        let mut ret = vec![];
        for insn in &self.instructions {
            ret.extend(insn);
        }
        ret
    }

    pub fn _to_code_for_upload(&self) -> CodeForUpload {
        let mut insns = vec![];
        for batch in &self.instructions {
            insns.push(batch.to_hex());
        }
        CodeForUpload {
            instructions: insns,
        }
    }

    pub fn _to_json(&self) -> serde_json::Result<String> {
        self._to_code_for_upload()._to_json()
    }

    pub fn _from_json(s: &str) -> Self {
        let cfu = CodeForUpload::_from_json(s);
        let num = cfu.instructions.clone().len();
        CodeUploader {
            build_buffer: vec![],
            instructions: cfu
                .instructions
                .into_iter()
                .map(|s| hex::decode(s).unwrap())
                .collect(),
            num_so_far: num,
            num_total: num,
        }
    }
}

impl fmt::Display for CodeUploader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for insn in &self.instructions {
            for b in insn {
                write!(f, "{:2x} ", b)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct CodeForUpload {
    instructions: Vec<String>,
}

impl CodeForUpload {
    pub fn _new(insns: Vec<Vec<u8>>) -> Self {
        CodeForUpload {
            instructions: insns.into_iter().map(|v| hex::encode(v)).collect(),
        }
    }

    pub fn _to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }

    pub fn _from_json(s: &str) -> Self {
        let parse_result: Result<Self, serde_json::Error> = serde_json::from_str(s);
        parse_result.unwrap()
    }
}

#[test]
fn test_code_upload_prep() {
    let uploader = CodeUploader::_new_from_file(Path::new("arb_os/arbos.mexe"));
    assert!(uploader.num_total > 5000);
    assert_eq!(uploader.num_total, uploader.num_so_far);
    let reconstituted = CodeUploader::_from_json(&uploader.clone()._to_json().unwrap());
    assert_eq!(uploader, reconstituted);
}
