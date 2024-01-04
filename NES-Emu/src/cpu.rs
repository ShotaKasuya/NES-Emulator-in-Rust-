use crate::bus::{Bus, Mem};
use crate::opscodes::{call, CPU_OPS_CODES};
use crate::rom::Rom;
use std::fmt::format;
use std::marker::Copy;

#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
  Accumulator,
  Immediate,
  ZeroPage,
  ZeroPage_X,
  ZeroPage_Y,
  Absolute,
  Absolute_X,
  Absolute_Y,
  Indirect,
  Indirect_X,
  Indirect_Y,
  Relative,
  Implied,
  NoneAddressing,
}

#[derive(Clone)]
pub struct OpCode {
  pub code: u8,
  pub name: String,
  pub bytes: u16,
  pub cycles: u16,
  pub addressing_mode: AddressingMode,
}
impl OpCode {
  pub fn new(
    code: u8,
    name: &str,
    bytes: u16,
    cycles: u16,
    addressing_mode: AddressingMode,
  ) -> Self {
    OpCode {
      code: code,
      name: String::from(name),
      bytes: bytes,
      cycles: cycles,
      addressing_mode: addressing_mode,
    }
  }
}

const FLAG_CARRY: u8 = 1;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_INTERRUPT: u8 = 1 << 2;
const FLAG_DECIMAL: u8 = 1 << 3;
const FLAG_BREAK: u8 = 1 << 4;
const FLAG_BREAK2: u8 = 1 << 5;
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_NEGATICE: u8 = 1 << 7;

pub struct CPU {
  pub register_a: u8,
  pub register_x: u8,
  pub register_y: u8,
  pub status: u8,
  pub stack_pointer: u8,
  pub program_counter: u16,
  pub bus: Bus,
}

pub fn trace(cpu: &CPU) -> String {
  // 空実装
  // 0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD
  // 0064 => program_counter
  // A2 01 => binary code
  // LDX #$01 => assembly code
  // A:01 => Accumrator
  // X:02 => Resister X
  // Y:03 => Resister Y
  // P:24 => Status
  // SP:FD => Stack Pointer
  let program_counter = cpu.program_counter - 1;
  let pc = format!("{:<04X}", program_counter);
  let op = cpu.mem_read(program_counter);
  let ops = cpu.find_ops(op).unwrap();
  let mut args: Vec<u8> = vec![];
  for n in 1..ops.bytes {
    let arg = cpu.mem_read(program_counter + n);
    args.push(arg);
  }
  let bin = binary(op, &args);
  let asm = disasm(program_counter, &ops, &args);
  let memacc = memory_access(&cpu, &ops, &args);
  let status = cpu2str(cpu);

  format!(
    "{:<6}{:<10}{:<32}{}",
    pc,
    bin,
    vec![asm, memacc].join(" "),
    status
  )
}

fn binary(op: u8, args: &Vec<u8>) -> String {
  let mut list: Vec<String> = vec![];
  list.push(format!("{:<02X}", op));
  for v in args {
    list.push(format!("{:02X}", v));
  }
  list.join(" ")
}

fn disasm(program_counter: u16, ops: &OpCode, args: &Vec<u8>) -> String {
  format!("{} {}", ops.name, address(program_counter, &ops, args))
}

fn address(program_counter: u16, ops: &OpCode, args: &Vec<u8>) -> String {
  match ops.addressing_mode {
    AddressingMode::Implied => {
      return format!("");
    }
    AddressingMode::Accumulator => {
      return format!("A");
    }
    AddressingMode::Immediate => {
      return format!("#${:<02X}", args[0]);
    }
    AddressingMode::ZeroPage => {
      return format!("${:<02X}", args[0]);
    }
    AddressingMode::Absolute => {
      return format!("${:<02X}{:<02X}", args[1], args[0]);
    }
    AddressingMode::ZeroPage_X => {
      return format!("${:<02X},X", args[0]);
    }
    AddressingMode::ZeroPage_Y => {
      return format!("$${:<02X},Y", args[0]);
    }
    AddressingMode::Absolute_X => {
      return format!("${:<02X}{:<02X},X", args[1], args[0]);
    }
    AddressingMode::Absolute_Y => {
      return format!("${:<02X}{:<02X},Y", args[1], args[0]);
    }
    AddressingMode::Indirect => {
      return format!("(${:<02X}{:<02X})", args[1], args[0]);
    }
    AddressingMode::Indirect_X => {
      return format!("(${:<02X},X)", args[0]);
    }
    AddressingMode::Indirect_Y => {
      return format!("(${:<02X}),Y", args[0]);
    }
    AddressingMode::Relative => {
      return format!("${:<02X}", program_counter + args[0] as u16 + 2);
    }
    AddressingMode::NoneAddressing => {
      panic!("mode {:?} is not supported", ops.addressing_mode);
    }
  }
}

fn memory_access(cpu: &CPU, ops: &OpCode, args: &Vec<u8>) -> String {
  if ops.name.starts_with("J") {
    return format!("");
  }
  match ops.addressing_mode {
    AddressingMode::ZeroPage => {
      let value = cpu.mem_read(args[0] as u16);
      format!("= {:>02X}", value)
    }
    AddressingMode::Absolute => {
      let hi = args[1] as u16;
      let lo = args[0] as u16;
      let addr = hi << 8 | lo;
      let value = cpu.mem_read(addr);
      format!("= {:<02X}", value)
    }
    AddressingMode::Indirect_X => {
      let base = args[0];
      let ptr: u8 = (base as u8).wrapping_add(cpu.register_x);
      let addr = cpu.mem_read_u16(ptr as u16);
      let value = cpu.mem_read(addr);
      format!("= {:>04X} @ {:04X} = {:02X}", ptr, addr, value)
    }
    AddressingMode::Indirect_Y => {
      // = 0400 @ 0400 = AA
      let base = args[0];
      let deref_base = cpu.mem_read_u16(base as u16);
      let deref = deref_base.wrapping_add(cpu.register_y as u16);
      let value = cpu.mem_read(deref);
      format!("= {:>04X} @ {:04X} = {:02X}", deref_base, deref, value)
    }
    AddressingMode::NoneAddressing => {
      panic!("mode {:?} is not supported", ops.addressing_mode);
    }
    _ => {
      format!("")
    }
  }
}

fn cpu2str(cpu: &CPU) -> String {
  format!(
    "A:{:<02X} X:{:02X} Y:{:<02X} P:{:02X} SP:{:02X}",
    cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_pointer,
  )
}

impl Mem for CPU {
  fn mem_read(&self, addr: u16) -> u8 {
    self.bus.mem_read(addr)
  }
  fn mem_write(&mut self, addr: u16, data: u8) {
    self.bus.mem_write(addr, data)
  }
}

impl CPU {
  pub fn new(bus: Bus) -> Self {
    CPU {
      register_a: 0,
      register_x: 0,
      register_y: 0,
      status: FLAG_INTERRUPT | FLAG_BREAK2, // Fixme
      stack_pointer: 0xFD,                  // Fixme
      program_counter: 0,
      bus: bus,
    }
  }

  fn get_operand_address(&self, _mode: &AddressingMode) -> u16 {
    match _mode {
      AddressingMode::Implied => {
        panic!("Don't Ask Here Address of Implied");
      }
      AddressingMode::Accumulator => {
        panic!("Don't Ask Here Address of Accumulator");
      }
      AddressingMode::Immediate => self.program_counter,
      AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
      AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
      AddressingMode::ZeroPage_X => {
        let pos = self.mem_read(self.program_counter);
        let addr = pos.wrapping_add(self.register_x) as u16;
        addr
      }
      AddressingMode::ZeroPage_Y => {
        let pos = self.mem_read(self.program_counter);
        let addr = pos.wrapping_add(self.register_y) as u16;
        addr
      }
      AddressingMode::Absolute_X => {
        let base = self.mem_read_u16(self.program_counter);
        let addr = base.wrapping_add(self.register_x as u16);
        addr
      }
      AddressingMode::Absolute_Y => {
        let base = self.mem_read_u16(self.program_counter);
        let addr = base.wrapping_add(self.register_y as u16);
        addr
      }
      AddressingMode::Indirect => {
        let base = self.mem_read_u16(self.program_counter);
        self.mem_read_u16(base as u16)
      }
      AddressingMode::Indirect_X => {
        let base = self.mem_read(self.program_counter);

        let ptr: u8 = (base as u8).wrapping_add(self.register_x);
        self.mem_read_u16(ptr as u16)
      }
      AddressingMode::Indirect_Y => {
        let base = self.mem_read(self.program_counter);

        let deref_base = self.mem_read_u16(base as u16);
        let deref = deref_base.wrapping_add(self.register_y as u16);
        deref
      }
      AddressingMode::Relative => {
        // let base = self.mem_read(self.program_counter);
        // ((base as i16) + (self.program_counter as i16)) as u16
        let base = self.mem_read(self.program_counter);
        let base = base as i8;
        let addr = base as i32 + self.program_counter as i32;
        addr as u16
      }
      AddressingMode::NoneAddressing => {
        panic!("_mode {:?} is not supported", _mode);
      }
    }
  }

  // pub fn mem_read(&self, addr: u16) -> u8 {
  //   self.bus.mem_read(addr)
  // }

  // pub fn mem_write(&mut self, addr: u16, data: u8) {
  //   self.bus.mem_write(addr, data);
  // }

  pub fn mem_read_u16(&self, pos: u16) -> u16 {
    let lo = self.mem_read(pos) as u16;
    let hi = self.mem_read(pos + 1) as u16;
    (hi << 8) | (lo as u16)
  }

  pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.mem_write(pos, lo);
    self.mem_write(pos + 1, hi);
  }

  fn load_and_run(&mut self, program: Vec<u8>) {
    self.load();
    self.reset();
    self.run()
  }

  pub fn load(&mut self) {
    // プログラムメモリは読み込み専用になるのでロードはいらない
    // self.mem_write_u16(0xFFFC, 0x8000);
  }

  pub fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.register_y = 0;
    self.status = FLAG_INTERRUPT | FLAG_BREAK2;
    self.stack_pointer = 0xFD;
    self.program_counter = self.mem_read_u16(0xFFFC);
    // println!("PC: {:X}", self.program_counter);
    self.program_counter = 0xC000;
  }

  pub fn run(&mut self) {
    self.run_with_callback(|_| {});
  }

  pub fn run_with_callback<F>(&mut self, mut callback: F)
  where
    F: FnMut(&mut CPU),
  {
    loop {
      let opscode = self.mem_read(self.program_counter);
      self.program_counter += 1;

      // println!("OPS: {:X}", opscode);

      let op = self.find_ops(opscode);
      match op {
        Some(op) => {
          // FIXME FOR TEST
          if op.name == "BRK" {
            return;
          }
          callback(self);
          call(self, &op);
        }
        _ => {}
      }
    }
  }

  fn find_ops(&self, opscode: u8) -> Option<OpCode> {
    for op in CPU_OPS_CODES.iter() {
      if op.code == opscode {
        return Some(op.clone());
      }
    }
    return None;
  }
  pub fn txs(&mut self, _mode: &AddressingMode) {
    self.stack_pointer = self.register_x;
  }

  pub fn tsx(&mut self, _mode: &AddressingMode) {
    self.register_x = self.stack_pointer;
    self.update_zero_and_negative_flags(self.register_x);
  }
  pub fn tya(&mut self, _mode: &AddressingMode) {
    self.register_a = self.register_y;
    self.update_zero_and_negative_flags(self.register_y);
  }

  // レジスタaの内容をレジスタxにコピーする
  pub fn tax(&mut self, _mode: &AddressingMode) {
    self.register_x = self.register_a;
    self.update_zero_and_negative_flags(self.register_x);
  }
  pub fn tay(&mut self, _mode: &AddressingMode) {
    self.register_y = self.register_a;
    self.update_zero_and_negative_flags(self.register_y);
  }
  pub fn txa(&mut self, _mode: &AddressingMode) {
    self.register_a = self.register_x;
    self.update_zero_and_negative_flags(self.register_x);
  }

  pub fn sty(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    self.mem_write(addr, self.register_y);
  }
  pub fn stx(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    self.mem_write(addr, self.register_x);
  }
  // レジスタaの値をメモリに書き込む
  pub fn sta(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    self.mem_write(addr, self.register_a);
  }

  pub fn rti(&mut self, _mode: &AddressingMode) {
    self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
    self.program_counter = self._pop_u16();
  }

  pub fn plp(&mut self, _mode: &AddressingMode) {
    self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
  }

  pub fn php(&mut self, _mode: &AddressingMode) {
    self._push(self.status | FLAG_BREAK | FLAG_BREAK2);
  }

  pub fn pla(&mut self, _mode: &AddressingMode) {
    self.register_a = self._pop();
    self.update_zero_and_negative_flags(self.register_a);
  }

  pub fn pha(&mut self, _mode: &AddressingMode) {
    self._push(self.register_a);
  }

  pub fn nop(&mut self, _mode: &AddressingMode) {
    // 何もしない
  }

  pub fn ldy(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    self.register_y = value;
    self.update_zero_and_negative_flags(value);
  }
  pub fn ldx(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    self.register_x = value;
    self.update_zero_and_negative_flags(value);
  }

  // レジスタaに値をコピーする
  pub fn lda(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    self.register_a = value;
    self.update_zero_and_negative_flags(value);
  }

  pub fn rts(&mut self, _mode: &AddressingMode) {
    let value = self._pop_u16() + 1;
    self.program_counter = value;
  }

  pub fn jsr(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    self._push_u16(self.program_counter + 2 - 1);
    self.program_counter = addr;
    // 後で+2されるので整合性のため-2する
    self.program_counter -= 2;
  }

  pub fn _push(&mut self, value: u8) {
    let addr = 0x0100 + self.stack_pointer as u16;
    self.mem_write(addr, value);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
  }

  pub fn _pop(&mut self) -> u8 {
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    let addr = 0x0100 + self.stack_pointer as u16;
    self.mem_read(addr)
  }

  pub fn _push_u16(&mut self, value: u16) {
    let addr = 0x0100 + self.stack_pointer.wrapping_sub(1) as u16;
    self.mem_write_u16(addr, value);
    self.stack_pointer = self.stack_pointer.wrapping_sub(2);
  }

  pub fn _pop_u16(&mut self) -> u16 {
    let addr = 0x0100 + self.stack_pointer.wrapping_add(1) as u16;
    let value = self.mem_read_u16(addr);
    self.stack_pointer = self.stack_pointer.wrapping_add(2);
    value
  }

  // レジスタaの値とメモリの値の和をレジスタaに書き込む
  // like SBC
  pub fn adc(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    // n = register_a + value + carry
    let carry = self.status & FLAG_CARRY;
    let (rhs, carry_flag) = value.overflowing_add(carry);
    // n = register_a + rhs
    let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

    let both_minus = (self.register_a & 0x80) == (value & 0x80);
    let value_changed = (value & 0x80) != (n & 0x80);
    // 負の値同士の計算で正の値になってしまった時にこのフラグが立つ
    let overflow = both_minus && value_changed;

    self.register_a = n;

    self.status = if carry_flag || carry_flag2 {
      self.status | FLAG_CARRY
    } else {
      self.status & (!FLAG_CARRY)
    };
    self.status = if overflow {
      self.status | FLAG_OVERFLOW
    } else {
      self.status & (!FLAG_OVERFLOW)
    };

    self.update_zero_and_negative_flags(self.register_a);
  }

  // レジスタaの値とメモリの値の論理積をレジスタaに書き込む
  // like EOR,ORA
  pub fn and(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    self.register_a &= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術左シフト
  // like LSR,ROL,ROR
  pub fn asl(&mut self, _mode: &AddressingMode) {
    let (value, carry) = if _mode == &AddressingMode::Accumulator {
      let (value, carry) = self.register_a.overflowing_mul(2);
      self.register_a = value;
      (value, carry)
    } else {
      let addr = self.get_operand_address(_mode);
      let value = self.mem_read(addr);
      let (value, carry) = value.overflowing_mul(2);
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | FLAG_CARRY
    } else {
      self.status & (!FLAG_CARRY)
    };
    self.update_zero_and_negative_flags(value);
  }

  // キャリーがクリアなら分岐
  pub fn bcc(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_CARRY == 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr
    }
  }

  // キャリーが立っていたら分岐
  pub fn bcs(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_CARRY != 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr
    }
  }

  // キャリーフラグをセット
  pub fn sec(&mut self, _mode: &AddressingMode) {
    self.status |= FLAG_CARRY;
  }

  // デシマルモードをセット
  pub fn sed(&mut self, _mode: &AddressingMode) {
    self.status |= FLAG_DECIMAL;
  }

  // ゼロフラグが立っていたら分岐
  pub fn beq(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_ZERO != 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr
    }
  }

  // レジスタaとメモリの値の論理積が0ならゼロフラグを立てる
  // メモリの7ビットと6ビットを基に
  // オーバーフローフラグとネガティブフラグを立てる
  pub fn bit(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);
    let result = value & self.register_a;
    self.status = if result == 0 {
      self.status | FLAG_ZERO
    } else {
      self.status & (!FLAG_ZERO)
    };
    self.status |= value & FLAG_OVERFLOW;
    self.status |= value & FLAG_NEGATICE;
  }

  // ネガティブフラグが立っていたら分岐
  pub fn bmi(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_NEGATICE != 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr
    }
  }

  // ゼロフラグがクリアなら分岐
  pub fn bne(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_ZERO == 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr
    }
  }

  // ネガティブフラグがクリアなら分岐
  pub fn bpl(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_NEGATICE == 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr;
    }
  }

  // オーバーフローフラグのクリア
  pub fn clv(&mut self, _mode: &AddressingMode) {
    self.status &= !FLAG_OVERFLOW;
  }

  pub fn cmp(&mut self, _mode: &AddressingMode) {
    self._cmp(self.register_a, _mode);
  }

  pub fn cpx(&mut self, _mode: &AddressingMode) {
    self._cmp(self.register_x, _mode);
  }

  // compare register y
  pub fn cpy(&mut self, _mode: &AddressingMode) {
    self._cmp(self.register_y, _mode);
  }

  pub fn _cmp(&mut self, target: u8, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    if target >= value {
      self.sec(&AddressingMode::Implied);
    } else {
      self.clc(&AddressingMode::Implied);
    }

    let (value, _) = target.overflowing_sub(value);
    self.update_zero_and_negative_flags(value);
  }

  // break
  pub fn brk(&mut self, _mode: &AddressingMode) {
    // TODO!
    self._push_u16(self.program_counter);
    self._push(self.status);

    self.program_counter = self.mem_read_u16(0xFFFE);
    self.status |= FLAG_BREAK;
  }

  // オーバーフローフラグがクリアなら分岐
  pub fn bvc(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_OVERFLOW == 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr;
    }
  }

  // オーバーフローフラグが立っていたら分岐
  pub fn bvs(&mut self, _mode: &AddressingMode) {
    if self.status & FLAG_OVERFLOW != 0 {
      let addr = self.get_operand_address(_mode);
      self.program_counter = addr;
    }
  }

  // キャリーをクリア
  pub fn clc(&mut self, _mode: &AddressingMode) {
    self.status &= !FLAG_CARRY;
  }

  // デシマルモードをクリア
  pub fn cld(&mut self, _mode: &AddressingMode) {
    self.status &= !FLAG_DECIMAL;
  }

  // インタラプトをクリア
  pub fn cli(&mut self, _mode: &AddressingMode) {
    self.status &= !FLAG_INTERRUPT;
  }

  // デクリメントメモリ
  pub fn dec(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);
    let value = value.wrapping_sub(1);

    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
  }

  // デクリメントXレジスタ
  pub fn dex(&mut self, _mode: &AddressingMode) {
    self.register_x = self.register_x.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_x);
  }

  // デクリメントYレジスタ
  pub fn dey(&mut self, _mode: &AddressingMode) {
    self.register_y = self.register_y.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_y);
  }

  // デシマルモードをクリア
  pub fn sei(&mut self, _mode: &AddressingMode) {
    self.status |= FLAG_INTERRUPT;
  }

  // レジスタaの値とメモリの値の排他的論理和をレジスタaに書き込む
  // like AND,ORA
  pub fn eor(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    self.register_a ^= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // デクリメントメモリ
  pub fn inc(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);
    let (value, _) = value.overflowing_add(1);

    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
  }

  // デクリメントXレジスタ
  pub fn inx(&mut self, _mode: &AddressingMode) {
    let (value, _) = self.register_x.overflowing_add(1);

    self.register_x = value;
    self.update_zero_and_negative_flags(value);
  }

  // デクリメントYレジスタ
  pub fn iny(&mut self, _mode: &AddressingMode) {
    let (value, _) = self.register_y.overflowing_add(1);

    self.register_y = value;
    self.update_zero_and_negative_flags(value);
  }

  pub fn jmp(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    self.program_counter = addr;
    // 引数の関係上あとで+2するので整合性のため-2する
    self.program_counter -= 2;
    // この後プログラムカウンターはインクリメントしない。
    // 元の6502はターゲットアドレスを正しくフェッチしていません
    // 間接ベクトルがページ境界に該当する場合
    // (例:$ xxFFここで xxは、$ 00から$ FFまでの値です。
    // この場合、LSBを取得します 予想通り$ xxFFからですが、
    // MSBは$ xx00から取得します。これは修正されました
    // 65SC02のような後のいくつかのチップでは、互換性のために常に
    // 間接ベクトルがページの最後にないことを確認します。
  }

  // 算術右シフト
  // like ASR,ROL,ROR
  pub fn lsr(&mut self, _mode: &AddressingMode) {
    let (value, carry) = if _mode == &AddressingMode::Accumulator {
      let carry = (self.register_a & 0x01) != 0; // 最下位ビットが立っているか
      self.register_a /= 2;
      (self.register_a, carry)
    } else {
      let addr = self.get_operand_address(_mode);
      let value = self.mem_read(addr);
      let carry = (value & 0x01) != 0;
      let value = value / 2;
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | FLAG_CARRY
    } else {
      self.status & (!FLAG_CARRY)
    };
    self.update_zero_and_negative_flags(value);
  }

  // レジスタaの値とメモリの値の論理和をレジスタaに書き込む
  // like AND,EOR
  pub fn ora(&mut self, _mode: &AddressingMode) {
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    self.register_a |= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術左シフト(キャリーによる補完あり)
  // like ASL,LSR,ROR
  pub fn rol(&mut self, _mode: &AddressingMode) {
    let (value, carry) = if _mode == &AddressingMode::Accumulator {
      let (value, carry) = self.register_a.overflowing_mul(2);
      let value = value | (self.status & FLAG_CARRY);
      self.register_a = value;
      (value, carry)
    } else {
      let addr = self.get_operand_address(_mode);
      let value = self.mem_read(addr);
      let (value, carry) = value.overflowing_mul(2);
      let value = value | (self.status & FLAG_CARRY);
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | FLAG_CARRY
    } else {
      self.status & (!FLAG_CARRY)
    };
    self.update_zero_and_negative_flags(value);
  }

  // 算術右シフト(キャリーによる補完あり)
  // like ASR,ASL,ROL
  pub fn ror(&mut self, _mode: &AddressingMode) {
    let (value, carry) = if _mode == &AddressingMode::Accumulator {
      let carry = (self.register_a & 0x01) != 0; // 最下位ビットが立っているか
      self.register_a = (self.register_a / 2) | ((self.status & FLAG_CARRY) << 7);
      (self.register_a, carry)
    } else {
      let addr = self.get_operand_address(_mode);
      let value = self.mem_read(addr);
      let carry = (value & 0x01) != 0;
      let value = (value / 2) | ((self.status & FLAG_CARRY) << 7);
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | FLAG_CARRY
    } else {
      self.status & (!FLAG_CARRY)
    };
    self.update_zero_and_negative_flags(value);
  }

  // レジスタaとメモリの値の差をレジスタaに書き込む
  // like ADC
  pub fn sbc(&mut self, _mode: &AddressingMode) {
    // A-M-(1-C)
    let addr = self.get_operand_address(_mode);
    let value = self.mem_read(addr);

    let carry = self.status & FLAG_CARRY;
    let (v1, carry_flag) = self.register_a.overflowing_sub(value);
    let (n, carry_flag2) = v1.overflowing_sub(1 - carry);

    // 違う符号同士の差で符号が変わることはないはずなので
    let overflow =
      ((self.register_a & 0x80) != (value & 0x80)) && ((self.register_a & 0x80) != (n & 0x80));

    self.register_a = n;

    // キャリーがない場合にフラグが立つ
    self.status = if !carry_flag && !carry_flag2 {
      self.status | FLAG_CARRY
    } else {
      self.status & (!FLAG_CARRY)
    };
    self.status = if overflow {
      self.status | FLAG_OVERFLOW
    } else {
      self.status & (!FLAG_OVERFLOW)
    };

    self.update_zero_and_negative_flags(self.register_a);
  }

  // ゼロフラグとネガティブフラグのつけ外し
  fn update_zero_and_negative_flags(&mut self, result: u8) {
    // ネガティブフラグ
    self.status = if result & 0b1000_0000 != 0 {
      self.status | FLAG_NEGATICE
    } else {
      self.status & (!FLAG_NEGATICE)
    };

    // ゼロフラグ
    self.status = if result == 0 {
      self.status | FLAG_ZERO
    } else {
      self.status & (!FLAG_ZERO)
    };
  }
}

//===================================================================
// テストコード
//===================================================================
#[cfg(test)]
mod test {

  use super::*;
  use crate::bus::Bus;
  use crate::cartridge::test::test_rom;

  #[test]
  fn test_format_trace() {
    let mut bus = Bus::new(test_rom());
    bus.mem_write(100, 0xa2);
    bus.mem_write(101, 0x01);
    bus.mem_write(102, 0xca);
    bus.mem_write(103, 0x88);
    bus.mem_write(104, 0x00);

    let mut cpu = CPU::new(bus);
    cpu.program_counter = 0x64;
    cpu.register_a = 1;
    cpu.register_x = 2;
    cpu.register_y = 3;
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
      result.push(trace(cpu));
    });
    assert_eq!(
      "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
      result[0]
    );
    assert_eq!(
      "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
      result[1]
    );
    assert_eq!(
      "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
      result[2]
    );
  }

  #[test]
  fn test_format_mem_access() {
    let mut bus = Bus::new(test_rom());
    // ORA ($33), Y
    bus.mem_write(100, 0x11);
    bus.mem_write(101, 0x33);

    // data
    bus.mem_write(0x33, 0x00);
    bus.mem_write(0x34, 0x04);

    // target cell
    bus.mem_write(0x400, 0xAA);

    let mut cpu = CPU::new(bus);
    cpu.program_counter = 0x64;
    cpu.register_y = 0;
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
      result.push(trace(cpu));
    });
    assert_eq!(
      "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
      result[0]
    );
  }

  fn run<F>(program: Vec<u8>, f: F) -> CPU
  where
    F: Fn(&mut CPU),
  {
    let mut cpu = CPU::new(Bus::new(Rom::empty()));
    cpu.load();
    cpu.reset();

    f(&mut cpu);
    cpu.run();

    cpu
  }

  fn assert_status(cpu: &CPU, flag: u8) {
    assert_eq!(cpu.status, flag);
  }

  /*
  // LDA
  #[test]
  // LDAを呼ぶテスト
  fn test_0xa9_lda_immediate_load_data() {
    let cpu = run(vec![0xa9, 0x05, 0x00], |_| {});

    // レジスタAに読み込まれたか
    assert_eq!(cpu.register_a, 0x05);

    // フラグがどちらもたっていない確認
    assert_status(&cpu, 0);
  }

  #[test]
  // ゼロフラグが正常に立つかのテスト
  fn test_0xa9_lda_zero_flag() {
    let cpu = run(vec![0xa9, 0x00, 0x00], |_| {});

    assert_status(&cpu, FLAG_ZERO);
  }

  #[test]
  // ネガティブフラグが正常に立つかのテスト
  fn test_0xa9_lda_negative_flag() {
    let cpu = run(vec![0xa, 0x80, 0x00], |_| {});
    assert_status(&cpu, FLAG_NEGATICE);
  }

  #[test]
  fn test_5_ops_working_together() {
    let cpu = run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00], |_| {});
    // LDA immediate 0xc0
    // TAX implied   0xaa
    // INX implied   0xe8

    assert_eq!(cpu.register_x, 0xc1);
  }

  #[test]
  fn test_lda_from_memory_zeropage() {
    let cpu = run(vec![0xa5, 0x10, 0x00], |cpu| {
      cpu.mem_write(0x10, 0x55);
    });
    assert_eq!(cpu.register_a, 0x55);
  }
  #[test]
  fn test_lda_from_memory_zeropage_x() {
    let cpu = run(vec![0xb5, 0x10, 0x00], |cpu| {
      cpu.register_x = 0x01;
      cpu.mem_write(0x11, 0x65);
    });
    assert_eq!(cpu.register_a, 0x65);
  }
  #[test]
  fn test_lda_from_memory_absolute() {
    let cpu = run(vec![0xad, 0x10, 0xaa, 0x00], |cpu| {
      cpu.mem_write(0xaa10, 0x66);
    });
    assert_eq!(cpu.register_a, 0x66);
  }
  #[test]
  fn test_lda_from_memory_absolute_x() {
    let cpu = run(vec![0xbd, 0x10, 0xab, 0x00], |cpu| {
      cpu.register_x = 0x11;
      cpu.mem_write(0xab21, 0x76);
    });
    assert_eq!(cpu.register_a, 0x76);
  }
  #[test]
  fn test_lda_from_memory_absolute_y() {
    let cpu = run(vec![0xb9, 0x10, 0xbb, 0x00], |cpu| {
      cpu.register_y = 0x11;
      cpu.mem_write(0xbb21, 0x77);
    });
    assert_eq!(cpu.register_a, 0x77);
  }
  #[test]
  fn test_lda_from_memory_indirect_x() {
    let cpu = run(vec![0xa1, 0x01, 0x00], |cpu| {
      cpu.register_x = 0x0F;
      cpu.mem_write_u16(0x10, 0x1001);
      cpu.mem_write(0x1001, 0x77);
    });
    assert_eq!(cpu.register_a, 0x77);
  }
  #[test]
  fn test_lda_from_memory_indirect_y() {
    let cpu = run(vec![0xb1, 0x11, 0x00], |cpu| {
      cpu.register_y = 0x0F;
      cpu.mem_write_u16(0x11, 0x1001);
      cpu.mem_write(0x1010, 0x77);
    });
    assert_eq!(cpu.register_a, 0x77);
  }

  #[test]
  // TAXのテスト
  fn test_0xaa_tax_move_a_to_x() {
    let cpu = run(vec![0xaa, 0x00], |cpu| {
      cpu.register_a = 0x10;
    });

    assert_eq!(cpu.register_x, cpu.register_a);
  }

  #[test]
  fn test_sta_from_memory() {
    let cpu = run(vec![0x85, 0x10, 0x00], |cpu| {
      cpu.register_a = 0xAF;
    });
    assert_eq!(cpu.mem_read(0x10), 0xAF);
  }

  #[test]
  // キャリーフラグを建てていない状態のADCテスト
  fn test_adc_no_carry() {
    let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
      cpu.register_a = 0x1A;
    });
    assert_eq!(cpu.register_a, 0x2A);
    assert_status(&cpu, 0x00);
  }
  #[test]
  // キャリーフラグを持った状態のADCテスト
  fn test_adc_has_carry() {
    let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
      cpu.register_a = 0x1A;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x2B);
    assert_status(&cpu, 0x00);
  }
  #[test]
  // キャリーフラグを建てるADCテスト
  fn test_adc_occur_carry() {
    let cpu = run(vec![0x69, 0x02, 0x00], |cpu| {
      cpu.register_a = 0xFF;
    });
    assert_eq!(cpu.register_a, 0x01);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  // 正の値同士のADCでオーバーフローが発生する際のテスト
  fn test_adc_occur_overflow_plus() {
    let cpu = run(vec![0x69, 0x01, 0x00], |cpu| {
      cpu.register_a = 0x7F;
    });
    assert_eq!(cpu.register_a, 0x80);
    assert_status(&cpu, FLAG_OVERFLOW | FLAG_NEGATICE);
  }
  #[test]
  // 負の値同士のADCでオーバーフローが発生する際のテスト
  fn test_adc_occur_overflow_minus() {
    let cpu = run(vec![0x69, 0x85, 0x00], |cpu| {
      cpu.register_a = 0x85;
    });
    assert_eq!(cpu.register_a, 0x0a);

    let flags = FLAG_OVERFLOW | FLAG_CARRY;
    assert_status(&cpu, flags);
  }
  #[test]
  // キャリーが存在する状態で
  // 正の値同士のADCでオーバーフローが起こる際のテスト
  fn test_adc_occur_overflow_plus_with_carry() {
    let cpu = run(vec![0x69, 0x6F, 0x00], |cpu| {
      cpu.register_a = 0x10;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x80);
    let flags = FLAG_NEGATICE | FLAG_OVERFLOW;
    assert_status(&cpu, flags);
  }
  #[test]
  // キャリーが存在する状態で
  // 負の値同士のADCでキャリーが発生する際のテスト
  fn test_adc_occur_overflow_minus_with_carry() {
    let cpu = run(vec![0x69, 0x80, 0x00], |cpu| {
      cpu.register_a = 0x80;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x01);
    let flags = FLAG_OVERFLOW | FLAG_CARRY;
    assert_status(&cpu, flags);
  }
  #[test]
  // 正の値と負の値のADCでオーバーフローが起きる際のテスト
  fn test_adc_no_overflow() {
    let cpu = run(vec![0x69, 0x7F, 0x00], |cpu| {
      cpu.register_a = 0x82;
    });
    assert_eq!(cpu.register_a, 0x01);
    let flags = FLAG_CARRY;
    assert_status(&cpu, flags);
  }
  #[test]
  // キャリーフラグを建てていない状態のSBCテスト
  fn test_sbc_no_carry() {
    let cpu = run(vec![0xe9, 0x10, 0x00], |cpu| {
      cpu.register_a = 0x20;
    });
    assert_eq!(cpu.register_a, 0x0F);
    // SBCでは特に何もなければキャリーフラグが立つ
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  // キャリーフラグを持った状態のSBCテスト
  fn test_sbc_has_carry() {
    let cpu = run(vec![0xe9, 0x10, 0x00], |cpu| {
      cpu.register_a = 0x20;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x10);
    // SBCでは特に何もなければキャリーフラグが立つ
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  // キャリーフラグを建てるSBCテスト
  fn test_sbc_occur_carry() {
    let cpu = run(vec![0xe9, 0x02, 0x00], |cpu| {
      cpu.register_a = 0x01;
    });
    assert_eq!(cpu.register_a, 0xFE);
    let flags = FLAG_NEGATICE;
    assert_status(&cpu, flags);
  }
  #[test]
  // 正の値同士のSBCでオーバーフローが発生する際のテスト
  fn test_sbc_occur_overflow() {
    let cpu = run(vec![0xe9, 0x81, 0x00], |cpu| {
      cpu.register_a = 0x7F;
    });
    assert_eq!(cpu.register_a, 0xFD);
    let flags = FLAG_OVERFLOW | FLAG_NEGATICE;
    assert_status(&cpu, flags);
  }
  #[test]
  // キャリーが存在する状態のSBCでオーバーフローが起こる際のテスト
  fn test_sbc_occur_overflow_with_carry() {
    let cpu = run(vec![0xe9, 0x81, 0x00], |cpu| {
      cpu.register_a = 0x7F;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0xFE);
    let flags = FLAG_NEGATICE | FLAG_OVERFLOW;
    assert_status(&cpu, flags);
  }
  #[test]
  // SBCでオーバーフローが起きる際のテスト
  fn test_sbc_no_overflow() {
    let cpu = run(vec![0xe9, 0x7F, 0x00], |cpu| {
      cpu.register_a = 0x7E;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0xFF);
    let flags = FLAG_NEGATICE;
    assert_status(&cpu, flags);
  }
  #[test]
  fn test_and() {
    let cpu = run(vec![0x29, 0x03, 0x00], |cpu| {
      cpu.register_a = 0x05;
    });
    assert_eq!(cpu.register_a, 0x01);
  }
  #[test]
  fn test_eor() {
    let cpu = run(vec![0x49, 0x03, 0x00], |cpu| {
      cpu.register_a = 0x05;
    });
    assert_eq!(cpu.register_a, 0x06);
  }
  #[test]
  fn test_ora() {
    let cpu = run(vec![0x09, 0x03, 0x00], |cpu| {
      cpu.register_a = 0x05;
    });
    assert_eq!(cpu.register_a, 0x07);
  }
  #[test]
  fn test_asl_accumulator() {
    let cpu = run(vec![0x0a, 0x00], |cpu| {
      cpu.register_a = 0x05;
    });
    assert_eq!(cpu.register_a, 0x05 * 2);
  }
  #[test]
  fn test_asl_accumulator_carry() {
    let cpu = run(vec![0x0a, 0x00], |cpu| {
      cpu.register_a = 0x81;
    });
    assert_eq!(cpu.register_a, 0x02);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_asl_zero_page() {
    let cpu = run(vec![0x06, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x03);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
  }
  #[test]
  fn test_asl_zero_page_carry() {
    let cpu = run(vec![0x06, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x83);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_lsr_accumulator() {
    let cpu = run(vec![0x4a, 0x00], |cpu| {
      cpu.register_a = 0x2;
    });
    assert_eq!(cpu.register_a, 0x02 / 2);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_lsr_zero_page() {
    let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x02);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x02 / 2);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_lsr_accumulator_occur_carry() {
    let cpu = run(vec![0x4a, 0x00], |cpu| {
      cpu.register_a = 0x03;
    });
    assert_eq!(cpu.register_a, 0x01);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_lsr_zero_page_occur_carry() {
    let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x03);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x01);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_lsr_zero_page_zero() {
    let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x01);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x00);
    assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
  }
  #[test]
  fn test_rol_accumulator() {
    let cpu = run(vec![0x2a, 0x00], |cpu| {
      cpu.register_a = 0x03;
    });
    assert_eq!(cpu.register_a, 0x06);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_rol_zero_page() {
    let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x03);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x06);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_rol_accumulator_with_carry() {
    let cpu = run(vec![0x2a, 0x00], |cpu| {
      cpu.register_a = 0x03;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x07);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_rol_zero_page_with_carry() {
    let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x03);
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.mem_read(0x0001), 0x07);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_rol_accumulator_zero_with_carry() {
    let cpu = run(vec![0x2a, 0x00], |cpu| {
      cpu.register_a = 0x00;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x01);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_rol_zero_page_zero_with_carry() {
    let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x00);
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.mem_read(0x0001), 0x01);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_ror_accumulator() {
    let cpu = run(vec![0x6a, 0x00], |cpu| {
      cpu.register_a = 0x2;
    });
    assert_eq!(cpu.register_a, 0x02 / 2);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_ror_zero_page() {
    let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x02);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x02 / 2);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_ror_accumulator_occur_carry() {
    let cpu = run(vec![0x6a, 0x00], |cpu| {
      cpu.register_a = 0x03;
    });
    assert_eq!(cpu.register_a, 0x01);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_ror_zero_page_occur_carry() {
    let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x03);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x01);
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_ror_accumulator_with_carry() {
    let cpu = run(vec![0x6a, 0x00], |cpu| {
      cpu.register_a = 0x02;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x81);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_ror_zero_page_with_carry() {
    let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x02);
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.mem_read(0x0001), 0x81);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_ror_accumulator_zero_with_carry() {
    let cpu = run(vec![0x6a, 0x00], |cpu| {
      cpu.register_a = 0x00;
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_a, 0x80);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_ror_zero_page_zero_with_carry() {
    let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x0001, 0x00);
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.mem_read(0x0001), 0x80);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_bcc() {
    let cpu = run(vec![0x90, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bcc_with_carry() {
    let cpu = run(vec![0x90, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, FLAG_CARRY);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bcc_negative_jump() {
    let cpu = run(vec![0x90, 0xfc, 0x00], |cpu| {
      cpu.mem_write(0x7FFF, 0x00);
      cpu.mem_write(0x7FFE, 0xe8);
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8000);
  }
  #[test]
  fn test_bcs() {
    let cpu = run(vec![0xb0, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bcs_with_carry() {
    let cpu = run(vec![0xb0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, FLAG_CARRY);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bcs_negative_jump() {
    let cpu = run(vec![0xb0, 0xfc, 0x00], |cpu| {
      cpu.mem_write(0x7FFF, 0x00);
      cpu.mem_write(0x7FFE, 0xe8);
      cpu.status = FLAG_CARRY;
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, FLAG_CARRY);
    assert_eq!(cpu.program_counter, 0x8000);
  }
  #[test]
  fn test_beq() {
    let cpu = run(vec![0xf0, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_beq_with_zero_flag() {
    let cpu = run(vec![0xf0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_ZERO;
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0); // ZEROはINXで落ちる
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bne() {
    let cpu = run(vec![0xd0, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0); // ZEROはINXで落ちる
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bne_with_zero_flag() {
    let cpu = run(vec![0xd0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_ZERO;
    });
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, FLAG_ZERO);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bit() {
    let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
      cpu.register_a = 0x00;
      cpu.mem_write(0x0000, 0x00);
    });
    assert_status(&cpu, FLAG_ZERO);
  }
  #[test]
  fn test_bit_negative_flag() {
    let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
      cpu.register_a = 0x00;
      cpu.mem_write(0x0000, 0x80);
    });
    assert_status(&cpu, FLAG_ZERO | FLAG_NEGATICE);
  }
  #[test]
  fn test_bit_overflow_flag() {
    let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
      cpu.register_a = 0x40;
      cpu.mem_write(0x0000, 0x40);
    });
    assert_status(&cpu, FLAG_OVERFLOW);
  }
  #[test]
  fn test_bmi() {
    let cpu = run(vec![0x30, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bmi_with_negative_flag() {
    let cpu = run(vec![0x30, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_NEGATICE;
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0); // INXしたためnegtiveフラグが落ちる
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bpl() {
    let cpu = run(vec![0x10, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bpl_with_negative_flag() {
    let cpu = run(vec![0x10, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_NEGATICE;
    });
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, FLAG_NEGATICE);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bvc() {
    let cpu = run(vec![0x50, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bvc_with_overflow_flag() {
    let cpu = run(vec![0x50, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW;
    });
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, FLAG_OVERFLOW);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bvs() {
    let cpu = run(vec![0x70, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x00);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bvs_with_overflow_flag() {
    let cpu = run(vec![0x70, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW;
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, FLAG_OVERFLOW);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_clc() {
    let cpu = run(vec![0x18, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW | FLAG_CARRY;
    });
    assert_status(&cpu, FLAG_OVERFLOW);
  }
  #[test]
  fn test_sec() {
    let cpu = run(vec![0x38, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW;
    });
    assert_status(&cpu, FLAG_OVERFLOW | FLAG_CARRY);
  }
  #[test]
  fn test_cld() {
    let cpu = run(vec![0xd8, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW | FLAG_DECIMAL;
    });
    assert_status(&cpu, FLAG_OVERFLOW);
  }
  #[test]
  fn test_sed() {
    let cpu = run(vec![0xf8, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW;
    });
    assert_status(&cpu, FLAG_OVERFLOW | FLAG_DECIMAL);
    assert_eq!(cpu.status, FLAG_OVERFLOW | FLAG_DECIMAL);
  }
  #[test]
  fn test_cli() {
    let cpu = run(vec![0x58, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW | FLAG_INTERRUPT;
    });
    assert_status(&cpu, FLAG_OVERFLOW);
  }
  #[test]
  fn test_sei() {
    let cpu = run(vec![0x78, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW;
    });
    assert_status(&cpu, FLAG_OVERFLOW | FLAG_INTERRUPT);
  }

  #[test]
  fn test_clv() {
    let cpu = run(vec![0xB8, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW;
    });
    assert_status(&cpu, 0);
  }

  #[test]
  fn test_cmp() {
    let cpu = run(vec![0xC9, 0x01, 0x00], |cpu| {
      cpu.register_a = 0x02;
    });
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_cmp_eq() {
    let cpu = run(vec![0xC9, 0x02, 0x00], |cpu| {
      cpu.register_a = 0x02;
    });
    assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
  }
  #[test]
  fn test_cmp_negative() {
    let cpu = run(vec![0xC9, 0x03, 0x00], |cpu| {
      cpu.register_a = 0x02;
    });
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_cpx() {
    let cpu = run(vec![0xE0, 0x01, 0x00], |cpu| {
      cpu.register_x = 0x02;
    });
    assert_status(&cpu, FLAG_CARRY);
  }
  #[test]
  fn test_cpy() {
    let cpu = run(vec![0xC0, 0x01, 0x00], |cpu| {
      cpu.register_y = 0x02;
    });
    assert_status(&cpu, FLAG_CARRY);
  }

  #[test]
  fn test_dec() {
    let cpu = run(vec![0xC6, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x01, 0x05);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x04);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_dec_with_overflow() {
    let cpu = run(vec![0xC6, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x01, 0x00);
    });
    assert_eq!(cpu.mem_read(0x0001), 0xFF);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_dex() {
    let cpu = run(vec![0xCA, 0x00], |cpu| {
      cpu.register_x = 0x05;
    });
    assert_eq!(cpu.register_x, 0x04);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_dex_overflow() {
    let cpu = run(vec![0xCA, 0x00], |cpu| {
      cpu.register_x = 0x00;
    });
    assert_eq!(cpu.register_x, 0xFF);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_dey() {
    let cpu = run(vec![0x88, 0x00], |cpu| {
      cpu.register_y = 0x05;
    });
    assert_eq!(cpu.register_y, 0x04);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_dey_overflow() {
    let cpu = run(vec![0x88, 0x00], |cpu| {
      cpu.register_y = 0x00;
    });
    assert_eq!(cpu.register_y, 0xFF);
    assert_status(&cpu, FLAG_NEGATICE);
  }

  #[test]
  fn test_inc() {
    let cpu = run(vec![0xE6, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x01, 0x05);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x06);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_inc_with_overflow() {
    let cpu = run(vec![0xE6, 0x01, 0x00], |cpu| {
      cpu.mem_write(0x01, 0x7F);
    });
    assert_eq!(cpu.mem_read(0x0001), 0x80);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_inx() {
    let cpu = run(vec![0xE8, 0x00], |cpu| {
      cpu.register_x = 0x05;
    });
    assert_eq!(cpu.register_x, 0x06);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_inx_overflow() {
    let cpu = run(vec![0xE8, 0x00], |cpu| {
      cpu.register_x = 0x7F;
    });
    assert_eq!(cpu.register_x, 0x80);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_iny() {
    let cpu = run(vec![0xc8, 0x00], |cpu| {
      cpu.register_y = 0x05;
    });
    assert_eq!(cpu.register_y, 0x06);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_iny_overflow() {
    let cpu = run(vec![0xc8, 0x00], |cpu| {
      cpu.register_y = 0x7F;
    });
    assert_eq!(cpu.register_y, 0x80);
    assert_status(&cpu, FLAG_NEGATICE);
  }

  #[test]
  fn test_jmp() {
    let cpu = run(vec![0x4C, 0x30, 0x40, 0x00], |cpu| {
      cpu.mem_write(0x4030, 0xE8);
      cpu.mem_write(0x4031, 0x00);
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x4032);
  }
  #[test]
  fn test_jmp_indirect() {
    let cpu = run(vec![0x6C, 0x30, 0x40, 0x00], |cpu| {
      cpu.mem_write(0x4030, 0x80);
      cpu.mem_write(0x4031, 0x00);
      println!("{:?}", cpu.program_counter);
      cpu.mem_write(0x8000, 0xE8);
      cpu.mem_write(0x8001, 0x00);
    });
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_jsr() {
    let cpu = run(vec![0x20, 0x30, 0x40, 0x00], |cpu| {
      cpu.mem_write(0x4030, 0xE8);
      cpu.mem_write(0x4031, 0x00);
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.program_counter, 0x4032);
    assert_eq!(cpu.mem_read_u16(0x01FE), 0x8003);
  }

  #[test]
  fn test_rts() {
    let cpu = run(vec![0x60, 0x00], |cpu| {
      cpu.mem_write(0x01FF, 0x05);
      cpu.mem_write(0x01FE, 0x06);

      cpu.mem_write(0x0506, 0xe8);
      cpu.mem_write(0x0507, 0x00);
      cpu.stack_pointer = 0xFD;
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.program_counter, 0x0508);
  }
  #[test]
  fn test_jsr_rts() {
    let cpu = run(vec![0x20, 0x30, 0x40, 0x00], |cpu| {
      cpu.mem_write(0x4030, 0xe8);
      cpu.mem_write(0x4031, 0x60); // RTS
      cpu.mem_write(0x4032, 0x00);
    });
    assert_eq!(cpu.register_x, 0x01);
    assert_status(&cpu, 0);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.program_counter, 0x8004);
  }
  #[test]
  fn test_ldx() {
    let cpu = run(vec![0xa2, 0x05, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0x05);
  }
  #[test]
  fn test_ldy() {
    let cpu = run(vec![0xa0, 0x05, 0x00], |_| {});
    assert_eq!(cpu.register_y, 0x05);
  }

  #[test]
  fn test_nop() {
    let cpu = run(vec![0xea, 0x00], |_| {});
    assert_eq!(cpu.program_counter, 0x8002);
  }

  #[test]
  fn test_pha() {
    let cpu = run(vec![0x48, 0x00], |cpu| {
      cpu.register_a = 0x07;
    });
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_a, 0x07);
    assert_eq!(cpu.stack_pointer, 0xFE);
    assert_eq!(cpu.mem_read(0x01FF), 0x07);
  }

  #[test]
  fn test_pla() {
    let cpu = run(vec![0x68, 0x00], |cpu| {
      cpu.mem_write(0x01FF, 0x07);
      cpu.stack_pointer = 0xFE;
    });
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_a, 0x07);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_status(&cpu, 0x00);
  }
  #[test]
  fn test_pla_flag_zero() {
    let cpu = run(vec![0x68, 0x00], |cpu| {
      cpu.mem_write(0x01FF, 0x00);
      cpu.stack_pointer = 0xFE;
    });
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_a, 0x00);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_status(&cpu, FLAG_ZERO);
  }

  #[test]
  fn test_pla_and_pha() {
    let cpu = run(vec![0x48, 0xa9, 0x60, 0x68, 0x00], |cpu| {
      cpu.register_a = 0x80;
    });
    assert_eq!(cpu.program_counter, 0x8005);
    assert_eq!(cpu.register_a, 0x80);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_php() {
    let cpu = run(vec![0x08, 0x00], |cpu| {
      cpu.status = FLAG_NEGATICE | FLAG_OVERFLOW;
    });
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.stack_pointer, 0xFE);
    assert_eq!(cpu.mem_read(0x01FF), FLAG_NEGATICE | FLAG_OVERFLOW);
  }
  #[test]
  fn test_plp() {
    let cpu = run(vec![0x28, 0x00], |cpu| {
      cpu.mem_write(0x01FF, FLAG_CARRY | FLAG_ZERO);
      cpu.stack_pointer = 0xFE;
    });
    assert_eq!(cpu.program_counter, 0x8002);
    assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
    assert_eq!(cpu.stack_pointer, 0xFF);
  }
  #[test]
  fn test_plp_and_php() {
    let cpu = run(vec![0x08, 0xa9, 0x80, 0x28, 0x00], |cpu| {
      cpu.status = FLAG_OVERFLOW | FLAG_CARRY;
    });
    assert_eq!(cpu.program_counter, 0x8005);
    assert_status(&cpu, FLAG_OVERFLOW | FLAG_CARRY);
    assert_eq!(cpu.stack_pointer, 0xFF);
  }

  #[test]
  fn test_stx() {
    let cpu = run(vec![0x86, 0x10, 0x00], |cpu| {
      cpu.register_x = 0xBA;
    });
    assert_eq!(cpu.mem_read(0x10), 0xBA);
  }
  #[test]
  fn test_sty() {
    let cpu = run(vec![0x84, 0x10, 0x00], |cpu| {
      cpu.register_y = 0xBA;
    });
    assert_eq!(cpu.mem_read(0x10), 0xBA);
  }
  #[test]
  fn test_txa() {
    let cpu = run(vec![0x8a, 0x00], |cpu| {
      cpu.register_x = 0x10;
    });
    assert_eq!(cpu.register_a, 0x10);
  }
  #[test]
  fn test_tay() {
    let cpu = run(vec![0xa8, 0x00], |cpu| {
      cpu.register_a = 0x10;
    });
    assert_eq!(cpu.register_y, 0x10);
  }
  #[test]
  fn test_tya() {
    let cpu = run(vec![0x98, 0x00], |cpu| {
      cpu.register_y = 0x10;
    });
    assert_eq!(cpu.register_a, 0x10);
  }
  #[test]
  fn test_tsx() {
    let cpu = run(vec![0xba, 0x00], |_| {});
    assert_eq!(cpu.register_x, 0xFF);
    assert_status(&cpu, FLAG_NEGATICE);
  }
  #[test]
  fn test_tsx_no_flag() {
    let cpu = run(vec![0xba, 0x00], |cpu| {
      cpu.stack_pointer = 0x75;
    });
    assert_eq!(cpu.register_x, 0x75);
    assert_status(&cpu, 0);
  }
  #[test]
  fn test_txs() {
    let cpu = run(vec![0x9a, 0x00], |cpu| {
      cpu.register_x = 0x80;
    });
    assert_eq!(cpu.stack_pointer, 0x80);
  }
   */
}
