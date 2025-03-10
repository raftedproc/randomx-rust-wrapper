/*
 * Copyright 2024 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::fmt;

use crate::{constants::STORE_L3_CONDITION, registers::REGISTER_COUNT};

static L1_LABEL: &str = "L1";
static L2_LABEL: &str = "L2";
static L3_LABEL: &str = "L3";

#[repr(C, align(8))]
#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub dst: u8,
    pub src: u8,
    pub mod_: u8,
    pub imm32: u32,
}

pub const NAMES_FREQS: [(&str, u8); 30] = [
    ("IADD_RS", 16),
    ("IADD_M", 7),
    ("ISUB_R", 16),
    ("ISUB_M", 7),
    ("IMUL_R", 16),
    ("IMUL_M", 4),
    ("IMULH_R", 4),
    ("IMULH_M", 1),
    ("ISMULH_R", 4),
    ("ISMULH_M", 1),
    ("IMUL_RCP", 8),
    ("INEG_R", 2),
    ("IXOR_R", 15),
    ("IXOR_M", 5),
    ("IROR_R", 8),
    ("IROL_R", 2),
    ("ISWAP_R", 4),
    ("FSWAP_R", 4),
    ("FADD_R", 16),
    ("FADD_M", 5),
    ("FSUB_R", 16),
    ("FSUB_M", 5),
    ("FSCAL_R", 6),
    ("FMUL_R", 32),
    ("FDIV_M", 4),
    ("FSQRT_R", 6),
    ("CBRANCH", 25),
    ("CFROUND", 1),
    ("ISTORE", 16),
    ("NOP", 0),
];

const fn build_instruction_array() -> [&'static str; 256] {
    let mut arr = [""; 256];
    let mut idx = 0;
    let mut i = 0;
    while i < NAMES_FREQS.len() {
        let (name, freq) = NAMES_FREQS[i];
        let mut j = 0;
        while j < freq {
            arr[idx] = name;
            idx += 1;
            j += 1;
        }
        i += 1;
    }
    arr
}

const INSTR_NAMES: [&str; 256] = build_instruction_array();

impl Instruction {
    pub fn new(opcode: u8, dst: u8, src: u8, mod_: u8, imm32: u32) -> Self {
        Self {
            opcode,
            dst,
            src,
            mod_,
            imm32,
        }
    }

    pub fn get_imm32(&self) -> u32 {
        self.imm32
    }

    pub fn get_mod_mem(&self) -> u8 {
        self.mod_ & 0b11
    }

    pub fn get_mod_shift(&self) -> u8 {
        (self.mod_ >> 2) & 0b11
    }

    pub fn get_mod_cond(&self) -> i32 {
        (self.mod_ >> 4) as i32
    }

    pub fn get_address_reg(&self) -> &'static str {
        if self.get_mod_mem() != 0 {
            L1_LABEL
        } else {
            L2_LABEL
        }
    }

    pub fn get_address_reg_dst(&self) -> &'static str {
        if self.get_mod_cond() < STORE_L3_CONDITION {
            if self.get_mod_mem() != 0 {
                L1_LABEL
            } else {
                L2_LABEL
            }
        } else {
            L3_LABEL
        }
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Self {
            opcode: 0,
            dst: 0,
            src: 0,
            mod_: 0,
            imm32: 0,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dst_index = self.dst % REGISTER_COUNT as u8; 
        let src_index = self.src % REGISTER_COUNT as u8;

        match INSTR_NAMES[self.opcode as usize] {
            "ISUB_R" => {
                if dst_index != src_index {
                    write!(f, "ISUB_R r{}, r{}", dst_index, src_index)
                } else {
                    write!(
                        f,
                        "ISUB_R r{}, {}",
                        dst_index, self.get_imm32()
                    )
                }
            }
            "ISTORE" => {
                write!(
                    f,
                    "ISTORE {}[r{} {}], r{}",
                    self.get_address_reg_dst(),
                    dst_index,
                    self.get_imm32() as i32,
                    src_index
                )
            }
            "IADD_RS" | "IMUL_R" | "IMULH_R" | "ISMULH_R" | "IMUL_RCP" | "IXOR_R" | "IROR_R"
            | "IROL_R" | "ISWAP_R" | "FSWAP_R" | "FADD_R" | "FSUB_R" | "FSCAL_R" | "FMUL_R"
            | "FSQRT_R" => {
                write!(
                    f,
                    "{} r{}, r{}",
                    INSTR_NAMES[self.opcode as usize], dst_index, src_index
                )
            }
            "IADD_M" | "ISUB_M" | "IMUL_M" | "IMULH_M" | "ISMULH_M" | "IXOR_M" | "FSUB_M"
            | "FADD_M" | "FDIV_M" => {
                write!(
                    f,
                    "{} r{}, [r{}]",
                    INSTR_NAMES[self.opcode as usize], dst_index, src_index
                )
            }
            "CBRANCH" => {
                write!(f, "CBRANCH {}", self.imm32)
            }
            "CFROUND" => {
                write!(f, "CFROUND {}", self.dst)
            }
            "INEG_R" => {
                write!(f, "INEG_R r{}", self.dst)
            }
            "NOP" => {
                write!(f, "NOP")
            }
            _ => {
                write!(f, "{}", INSTR_NAMES[self.opcode as usize])
            }
        }
    }
}
