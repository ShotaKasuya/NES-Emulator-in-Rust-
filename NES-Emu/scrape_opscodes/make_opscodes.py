import scrape_opscode

def main():
    url = "https://www.nesdev.org/obelisk-6502-guide/reference.html#INX"
    official_ops = scrape_opscode.get_official_opcode_list(url=url)
    url = "https://www.nesdev.org/undocumented_opcodes.txt"
    unofficial_ops = scrape_opscode.get_unofficial_opcode_list(url=url)
    with open("../src/opscodes.rs", "w") as f:
        f.write(header)
        for opcode in official_ops:
            for addr_mode in opcode.addr:
                f.write(f'OpCode::new({addr_mode.code.replace("$", "0x")}, \"{opcode.name}\", {addr_mode.bytes}, {addr_mode.cycles}, AddressingMode::{addr_mode.addressingmode}),\n')
        for opcode in unofficial_ops:
            for addr_mode in opcode.addr:
                f.write(f'OpCode::new({addr_mode.code.replace("$", "0x")}, \"{"*" + opcode.name}\", {addr_mode.bytes}, {addr_mode.cycles}, AddressingMode::{addr_mode.addressingmode}),\n')
        f.write(mider)
        names = []
        for opcode in official_ops:
            names.append(opcode.name)
            f.write(f"""
                    \"{opcode.name}\" => {{
                        cpu.{opcode.name.lower()}(&op.addressing_mode);
                        cpu.program_counter += op.bytes - 1;
                    }}
                    """)
        for opcode in unofficial_ops:
            if not opcode.name in names:
                names.append(opcode.name)
                f.write(f"""
                        \"{opcode.name}\" => {{
                            cpu.{opcode.name.lower()}(&op.addressing_mode);
                            cpu.program_counter += op.bytes - 1;
                        }}
                        """)
        f.write(ender)

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


if __name__=="__main__":
    main()