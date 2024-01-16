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
bytes_table = {
    "ZeroPage":2,
    "Absolute":3,
    "Absolute_X":3,
    "Absolute_Y":3,
    "ZeroPage_X":2,
    "ZeroPage_Y":2,
    "Implied":1,
    "Immediate":2,
    "Indirect_X":2,
    "Indirect_Y":2,
}
unofficial_ops = {
    "*NOP" :{
        "ZeroPage":[ "0x04", "0x44", "0x64"],
        "Absolute":[ "0x0C"],
        "Absolute_X":[ "0x1C", "0x3C", "0x5C", "0x7C", "0xDC", "0xFC"],
        "ZeroPage_X":[ "0x14", "0x34", "0x54", "0x74", "0xD4", "0xF4"],
        "Implied":[ "0x1A", "0x3A", "0x5A", "0x7A", "0xDA", "0xFA", "0xEA"],
        "Immediate":["0x80"]
    },
    "*SBC":{
        "Immediate":["0xEB"],
    },
    "*LAX":{
        "Indirect_X":["0xA3"],
        "Indirect_Y":["0xB3"],
        "ZeroPage":["0xA7"],
        "ZeroPage_Y":["0xB7"],
        "Absolute":["0xAF"],
        "Absolute_Y":["0xBF"],
    },
    "*SAX":{
        "Indirect_X":["0x83"],
        "ZeroPage":["0x87"],
        "ZeroPage_Y":["0x97"],
        "Absolute":["0x8F"],
    }
}


opscodes = []
with open("data.csv") as f:
    reader = csv.reader(f)
    opscodes = [row for row in reader]

def append_unofficial_opscodes():
    global unofficial_ops
    global opscodes
    ops = [ opcode[0] for opcode in opscodes]
    print(ops)
    for u_ops in unofficial_ops.keys():
        print(u_ops.replace("*", ""))
        if not u_ops.replace("*", "") in ops:
            opscodes.append([u_ops, "", "", "", ""])

with open("../src/opscodes.rs", "w") as f:
    f.write(header)
    for row in opscodes:
        f.write(f'OpCode::new({row[2]}, \"{row[0]}\", {row[3]}, {row[4]}, AddressingMode::{row[1]}),\n')
    cycles = 2
    for ops in unofficial_ops.keys():
        for addr in unofficial_ops[ops].keys():
            for opcode in unofficial_ops[ops][addr]:
                f.write(f'OpCode::new({opcode}, \"{ops}\", {bytes_table[addr]}, {cycles}, AddressingMode::{addr}),\n')
    f.write(mider)
    before = ""

    append_unofficial_opscodes()
    for row in opscodes:
        if row[0] != before:
            f.write(f"""
                    \"{row[0].replace("*", "")}\" => {{
                        cpu.{row[0].replace("*", "").lower()}(&op.addressing_mode);
                        cpu.program_counter += op.bytes - 1;
                    }}
                    """)
        before = row[0]
    # for ops in unofficial_ops.keys():
    #     f.write(f"""
    #             \"{ops.replace("*", "")}\" => {{
    #                     cpu.{ops.replace("*", "").lower()}(&op.addressing_mode);
    #                     cpu.program_counter += op.bytes -1;
    #                 }}
    #             """)
    
    
    f.write(ender)
