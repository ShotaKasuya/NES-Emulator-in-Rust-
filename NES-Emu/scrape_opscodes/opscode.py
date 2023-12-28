import csv
# プログラム自動生成
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
match op.name.as_str() {

'''
ender = '''
    _ => {
        todo!()
    }
  }
}
'''

opscodes = []
with open("data.csv") as f:
    reader = csv.reader(f)
    opscodes = [row for row in reader]

with open("opscode.rs", "w") as f:
    f.write(header)
    for row in opscodes:
        f.write(f"OpCode::new({row[2]}, \"{row[0]}\", {row[3]}, {row[4]}, AddressingMode::{row[1]}),\n")
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
