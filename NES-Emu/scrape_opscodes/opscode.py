# coding=utf8
import csv
# プログラム生成
header = '''
use crate::cpu::AddressingMode;
use crate::cpu::OpCode;
use crate::cpu::CPU;

lazy_static!{
pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
'''
mider = '''
];
}

pub fn call(cpu: &mut CPU, op: &OpCode) {
match op.name.replace("*", "").as_str() {

'''
ender = '''
    _ => {
        todo!()
    }
  }
}
'''

unofficial_ops = [
    'OpCode::new(0x04, "*NOP", 2, 2, AddressingMode::ZeroPage),',
    'OpCode::new(0x44, "*NOP", 2, 2, AddressingMode::ZeroPage),',
    'OpCode::new(0x64, "*NOP", 2, 2, AddressingMode::ZeroPage),',
    'OpCode::new(0x0C, "*NOP", 3, 2, AddressingMode::Absolute),',
    'OpCode::new(0x1C, "*NOP", 3, 2, AddressingMode::Absolute_X),',
    'OpCode::new(0x3C, "*NOP", 3, 2, AddressingMode::Absolute_X),',
    'OpCode::new(0x5C, "*NOP", 3, 2, AddressingMode::Absolute_X),',
    'OpCode::new(0x7C, "*NOP", 3, 2, AddressingMode::Absolute_X),',
    'OpCode::new(0xDC, "*NOP", 3, 2, AddressingMode::Absolute_X),',
    'OpCode::new(0xFC, "*NOP", 3, 2, AddressingMode::Absolute_X),',
    'OpCode::new(0x14, "*NOP", 2, 2, AddressingMode::ZeroPage_X),',
    'OpCode::new(0x34, "*NOP", 2, 2, AddressingMode::ZeroPage_X),',
    'OpCode::new(0x54, "*NOP", 2, 2, AddressingMode::ZeroPage_X),',
    'OpCode::new(0x74, "*NOP", 2, 2, AddressingMode::ZeroPage_X),',
    'OpCode::new(0xD4, "*NOP", 2, 2, AddressingMode::ZeroPage_X),',
    'OpCode::new(0xF4, "*NOP", 2, 2, AddressingMode::ZeroPage_X),',
    'OpCode::new(0x1A, "*NOP", 1, 2, AddressingMode::Implied),',
    'OpCode::new(0x3A, "*NOP", 1, 2, AddressingMode::Implied),',
    'OpCode::new(0x5A, "*NOP", 1, 2, AddressingMode::Implied),',
    'OpCode::new(0x7A, "*NOP", 1, 2, AddressingMode::Implied),',
    'OpCode::new(0xDA, "*NOP", 1, 2, AddressingMode::Implied),',
    'OpCode::new(0xFA, "*NOP", 1, 2, AddressingMode::Implied),',
    'OpCode::new(0x80, "*NOP", 2, 2, AddressingMode::Immediate),',
]

opscodes = []
with open("data.csv") as f:
    reader = csv.reader(f)
    opscodes = [row for row in reader]

with open("../src/opscodes.rs", "w") as f:
    f.write(header)
    for row in opscodes:
        f.write(f'OpCode::new({row[2]}, \"{row[0]}\", {row[3]}, {row[4]}, AddressingMode::{row[1]}),\n')
    for u_ops in unofficial_ops:
        f.write(u_ops + "\n")
    f.write(mider)
    before = ""
    for row in opscodes:
        if row[0] != before:
            f.write(f"""
                    \"{row[0]}\" => {{
                        cpu.{row[0].lower()}(&op.addressing_mode);
                        cpu.program_counter += op.bytes - 1;
                    }}
                    """)
        before = row[0]
    f.write(ender)
