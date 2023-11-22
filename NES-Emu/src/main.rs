fn main() {
  println!("Hello, world!");
}

#[derive(Debug, PartialEq)]
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
  Indirect_X,
  Indirect_Y,
  Relative,
  Implied,
  NoneAddressing,
}

pub enum Flag {
  Carry = 1,
  Zero = 1 << 1,
  InterruptDisable = 1 << 2,
  Decimal = 1 << 3,
  Break = 1 << 4,
  // (No CPU effect; always pushed as 1)
  Overflow = 1 << 6,
  Negative = 1 << 7,
}
impl Flag {
  // Getter
  pub fn carry() -> u8 {
    Flag::Carry as u8
  }
  pub fn zero() -> u8 {
    Flag::Zero as u8
  }
  pub fn interrupt_disable() -> u8 {
    Flag::InterruptDisable as u8
  }
  pub fn decimal() -> u8 {
    Flag::Decimal as u8
  }
  pub fn overflow() -> u8 {
    Flag::Overflow as u8
  }
  pub fn negative() -> u8 {
    Flag::Negative as u8
  }
}

pub struct CPU {
  pub register_a: u8,
  pub register_x: u8,
  pub register_y: u8,
  pub status: u8,
  pub program_counter: u16,
  memory: [u8; 0x10000],
}

impl CPU {
  pub fn new() -> Self {
    CPU {
      register_a: 0,
      register_x: 0,
      register_y: 0,
      status: 0,
      program_counter: 0,
      memory: [0x00; 0x10000],
    }
  }

  fn mem_read(&self, addr: u16) -> u8 {
    self.memory[addr as usize]
  }

  fn mem_read_u16(&self, pos: u16) -> u16 {
    let lo = self.mem_read(pos) as u16;
    let hi = self.mem_read(pos + 1) as u16;
    (hi << 8) | (lo as u16)
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    self.memory[addr as usize] = data;
  }

  fn mem_write_u16(&mut self, pos: u16, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.mem_write(pos, lo);
    self.mem_write(pos + 1, hi);
  }

  fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
    match mode {
      AddressingMode::Accumulator => {
        panic!("Don't Ask Here Address of Accumulator");
      }
      AddressingMode::Implied => {
        panic!("Don't Ask Here Address of Implied");
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
        panic!("mode {:?} is not supported", mode);
      }
    }
  }

  pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
  }

  fn load(&mut self, program: Vec<u8>) {
    self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    self.mem_write_u16(0xFFFC, 0x8000);
  }

  pub fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.register_y = 0;
    self.status = 0;
    self.program_counter = self.mem_read_u16(0xFFFC);
  }

  pub fn run(&mut self) {
    loop {
      let opscode = self.mem_read(self.program_counter);
      self.program_counter += 1;

      match opscode {
        /*--- ADC ---*/
        0x69 => {
          self.adc(&AddressingMode::Immediate);
          self.program_counter += 1;
        }
        /*--- AND ---*/
        0x29 => {
          self.and(&AddressingMode::Immediate);
          self.program_counter += 1;
        }
        /*--- ASL ---*/
        0x0a => {
          self.asl(&AddressingMode::Accumulator);
        }
        0x06 => {
          self.asl(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        /*--- BCC ---*/
        0x90 => {
          self.bcc(&AddressingMode::Relative);
          self.program_counter += 1;
        }
        /*--- BCS ---*/
        0xb0 => {
          self.bcs(&AddressingMode::Relative);
          self.program_counter += 1;
        }
        /*--- BEQ ---*/
        0xF0 => {
          self.beq(&AddressingMode::Relative);
          self.program_counter += 1;
        }
        /*--- BMI ---*/
        0x30 => {
          self.bmi(&AddressingMode::Relative);
          self.program_counter += 1;
        }
        /*--- BNE ---*/
        0xD0 => {
          self.bne(&AddressingMode::Relative);
          self.program_counter += 1;
        }
        /*--- BIT ---*/
        0x24 => {
          self.bit(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        0x2C => {
          self.bit(&AddressingMode::Absolute);
          self.program_counter += 2;
        }
        /*--- BPL ---*/
        0x10 => {
          self.bpl(&AddressingMode::Relative);
          self.program_counter += 1;
        }
        /*--- EOR ---*/
        0x49 => {
          self.eor(&AddressingMode::Immediate);
          self.program_counter += 1;
        }
        /*--- LDA ---*/
        0xA9 => {
          self.lda(&AddressingMode::Immediate);
          self.program_counter += 1;
        }
        0xA5 => {
          self.lda(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        0xB5 => {
          self.lda(&AddressingMode::ZeroPage_X);
          self.program_counter += 1;
        }
        0xAD => {
          self.lda(&AddressingMode::Absolute);
          self.program_counter += 2;
        }
        0xBD => {
          self.lda(&AddressingMode::Absolute_X);
          self.program_counter += 2;
        }
        0xB9 => {
          self.lda(&AddressingMode::Absolute_Y);
          self.program_counter += 2;
        }
        0xA1 => {
          self.lda(&AddressingMode::Indirect_X);
          self.program_counter += 1;
        }
        0xB1 => {
          self.lda(&AddressingMode::Indirect_Y);
          self.program_counter += 1;
        }
        /*--- LSR ---*/
        0x4a => {
          self.lsr(&AddressingMode::Accumulator);
        }
        0x46 => {
          self.lsr(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        /*--- ORA ---*/
        0x09 => {
          self.ora(&AddressingMode::Immediate);
          self.program_counter += 1;
        }
        /*--- ROL ---*/
        0x2a => {
          self.rol(&AddressingMode::Accumulator);
        }
        0x26 => {
          self.rol(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        /*--- ROR ---*/
        0x6a => {
          self.ror(&AddressingMode::Accumulator);
        }
        0x66 => {
          self.ror(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        0x76 => {
          self.ror(&AddressingMode::ZeroPage_X);
          self.program_counter += 1;
        }
        0x6e => {
          self.ror(&AddressingMode::Absolute);
          self.program_counter += 1;
        }
        0x7e => {
          self.ror(&AddressingMode::Absolute_X);
          self.program_counter += 1;
        }
        /*--- SDC ---*/
        0xe9 => {
          self.sbc(&AddressingMode::Immediate);
          self.program_counter += 1;
        }
        /*--- STA ---*/
        0x85 => {
          self.sta(&AddressingMode::ZeroPage);
          self.program_counter += 1;
        }
        0x95 => {
          self.sta(&AddressingMode::ZeroPage_X);
          self.program_counter += 1;
        }
        0x8D => {
          self.sta(&AddressingMode::Absolute);
          self.program_counter += 2;
        }
        0x9D => {
          self.sta(&AddressingMode::Absolute_X);
          self.program_counter += 2;
        }
        0x99 => {
          self.sta(&AddressingMode::Absolute_Y);
          self.program_counter += 2;
        }
        0x81 => {
          self.sta(&AddressingMode::Indirect_X);
          self.program_counter += 1;
        }
        0x91 => {
          self.sta(&AddressingMode::Indirect_Y);
          self.program_counter += 1;
        }
        0xAA => self.tax(),
        0xE8 => self.inx(),

        0x00 => {
          // self.brk(&AddressingMode::Implied);
          break;
          // TODO!
        }

        _ => todo!(""),
      }
    }
  }

  // レジスタaの値とメモリの値の和をレジスタaに書き込む
  // like SBC
  fn adc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    // n = register_a + value + carry
    let carry = self.status & Flag::carry();
    let (rhs, carry_flag) = value.overflowing_add(carry);
    // n = register_a + rhs
    let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

    let both_minus = (self.register_a & 0x80) == (value & 0x80);
    let value_changed = (value & 0x80) != (n & 0x80);
    // 負の値同士の計算で正の値になってしまった時にこのフラグが立つ
    let overflow = both_minus && value_changed;

    self.register_a = n;

    self.status = if carry_flag || carry_flag2 {
      self.status | Flag::carry()
    } else {
      self.status & (!Flag::carry())
    };
    self.status = if overflow {
      self.status | Flag::overflow()
    } else {
      self.status & (!Flag::overflow())
    };

    self.update_zero_and_negative_flags(self.register_a);
  }

  // レジスタaの値とメモリの値の論理積をレジスタaに書き込む
  // like EOR,ORA
  fn and(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a &= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術左シフト
  // like LSR,ROL,ROR
  fn asl(&mut self, mode: &AddressingMode) {
    let (value, carry) = if mode == &AddressingMode::Accumulator {
      let (value, carry) = self.register_a.overflowing_mul(2);
      self.register_a = value;
      (value, carry)
    } else {
      let addr = self.get_operand_address(mode);
      let value = self.mem_read(addr);
      let (value, carry) = value.overflowing_mul(2);
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | Flag::carry()
    } else {
      self.status & (!Flag::carry())
    };
    self.update_zero_and_negative_flags(value);
  }

  // キャリーがクリアなら分岐
  fn bcc(&mut self, mode: &AddressingMode) {
    if self.status & Flag::carry() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // キャリーが立っていたら分岐
  fn bcs(&mut self, mode: &AddressingMode) {
    if self.status & Flag::carry() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // ゼロフラグが立っていたら分岐
  fn beq(&mut self, mode: &AddressingMode) {
    if self.status & Flag::zero() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // レジスタaとメモリの値の論理積が0ならゼロフラグを立てる
  // メモリの7ビットと6ビットを基に
  // オーバーフローフラグとネガティブフラグを立てる
  fn bit(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    let result = value & self.register_a;
    self.status = if result == 0 {
      self.status | Flag::zero()
    } else {
      self.status & (!Flag::zero())
    };
    self.status |= value & Flag::overflow();
    self.status |= value & Flag::negative();
  }

  // ネガティブフラグが立っていたら分岐
  fn bmi(&mut self, mode: &AddressingMode) {
    if self.status & Flag::negative() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // ゼロフラグがクリアなら分岐
  fn bne(&mut self, mode: &AddressingMode) {
    if self.status & Flag::zero() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // ネガティブフラグがクリアなら分岐
  fn bpl(&mut self, mode: &AddressingMode) {
    if self.status & Flag::negative() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // break
  fn brk(&mut self, mode: &AddressingMode) {
    self.program_counter = self.mem_read_u16(0xFFFE);
    // TODO!
    self.status |= Flag::Break as u8;
  }

  // レジスタaの値とメモリの値の排他的論理和をレジスタaに書き込む
  // like AND,ORA
  fn eor(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a ^= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // レジスタaに値をコピーする
  fn lda(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a = value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術右シフト
  // like ASR,ROL,ROR
  fn lsr(&mut self, mode: &AddressingMode) {
    let (value, carry) = if mode == &AddressingMode::Accumulator {
      let carry = (self.register_a & 0x01) != 0; // 最下位ビットが立っているか
      self.register_a /= 2;
      (self.register_a, carry)
    } else {
      let addr = self.get_operand_address(mode);
      let value = self.mem_read(addr);
      let carry = (value & 0x01) != 0;
      let value = value / 2;
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | Flag::carry()
    } else {
      self.status & (!Flag::carry())
    };
    self.update_zero_and_negative_flags(value);
  }

  // レジスタaの値とメモリの値の論理和をレジスタaに書き込む
  // like AND,EOR
  fn ora(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a |= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術左シフト(キャリーによる補完あり)
  // like ASL,LSR,ROR
  fn rol(&mut self, mode: &AddressingMode) {
    let (value, carry) = if mode == &AddressingMode::Accumulator {
      let (value, carry) = self.register_a.overflowing_mul(2);
      let value = value | (self.status & Flag::carry());
      self.register_a = value;
      (value, carry)
    } else {
      let addr = self.get_operand_address(mode);
      let value = self.mem_read(addr);
      let (value, carry) = value.overflowing_mul(2);
      let value = value | (self.status & Flag::carry());
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | Flag::carry()
    } else {
      self.status & (!Flag::carry())
    };
    self.update_zero_and_negative_flags(value);
  }

  // 算術右シフト(キャリーによる補完あり)
  // like ASR,ASL,ROL
  fn ror(&mut self, mode: &AddressingMode) {
    let (value, carry) = if mode == &AddressingMode::Accumulator {
      let carry = (self.register_a & 0x01) != 0; // 最下位ビットが立っているか
      self.register_a = (self.register_a / 2) | ((self.status & Flag::carry()) << 7);
      (self.register_a, carry)
    } else {
      let addr = self.get_operand_address(mode);
      let value = self.mem_read(addr);
      let carry = (value & 0x01) != 0;
      let value = (value / 2) | ((self.status & Flag::carry()) << 7);
      self.mem_write(addr, value);
      (value, carry)
    };

    self.status = if carry {
      self.status | Flag::carry()
    } else {
      self.status & (!Flag::carry())
    };
    self.update_zero_and_negative_flags(value);
  }

  // レジスタxをインクリメント
  fn inx(&mut self) {
    // オーバーフロー制御
    self.register_x = self.register_x.wrapping_add(1);
    self.update_zero_and_negative_flags(self.register_x);
  }

  // レジスタaとメモリの値の差をレジスタaに書き込む
  // like ADC
  fn sbc(&mut self, mode: &AddressingMode) {
    // A-M-(1-C)
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let carry = self.status & Flag::carry();
    let (v1, carry_flag) = self.register_a.overflowing_sub(value);
    let (n, carry_flag2) = v1.overflowing_sub(1 - carry);

    // 違う符号同士の差で符号が変わることはないはずなので
    let overflow =
      ((self.register_a & 0x80) != (value & 0x80)) && ((self.register_a & 0x80) != (n & 0x80));

    self.register_a = n;

    // キャリーがない場合にフラグが立つ
    self.status = if !carry_flag && !carry_flag2 {
      self.status | Flag::carry()
    } else {
      self.status & (!Flag::carry())
    };
    self.status = if overflow {
      self.status | Flag::overflow()
    } else {
      self.status & (!Flag::overflow())
    };

    self.update_zero_and_negative_flags(self.register_a);
  }

  // レジスタaの値をメモリに書き込む
  fn sta(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_a);
  }

  // レジスタaの内容をレジスタxにコピーする
  fn tax(&mut self) {
    self.register_x = self.register_a;
    self.update_zero_and_negative_flags(self.register_x);
  }

  // ゼロフラグとネガティブフラグのつけ外し
  fn update_zero_and_negative_flags(&mut self, result: u8) {
    // ネガティブフラグ
    self.status = if result & 0b1000_0000 != 0 {
      self.status | Flag::negative()
    } else {
      self.status & (!Flag::negative())
    };

    // ゼロフラグ
    self.status = if result == 0 {
      self.status | Flag::zero()
    } else {
      self.status & (!Flag::zero())
    };
  }
}

//===================================================================
// テストコード
//===================================================================
#[cfg(test)]
mod test {
  use std::vec;

  use super::*;

  #[test]
  // LDAを呼ぶテスト
  fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    // レジスタに読み込まれたか
    assert_eq!(cpu.register_a, 0x05);

    // フラグがどちらもたっていない確認
    assert!(cpu.status & Flag::zero() == 0);
    assert!(cpu.status & Flag::negative() == 0);
  }

  #[test]
  // ゼロフラグが正常に立つかのテスト
  fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

    assert!((cpu.status & Flag::zero()) == Flag::zero());
  }

  #[test]
  // ネガティブフラグが正常に立つかのテスト
  fn test_0xa9_lda_negative_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x80, 0x00]);

    assert!(cpu.status & Flag::negative() == Flag::negative());
  }

  #[test]
  // TAXのテスト
  fn test_0xaa_tax_move_a_to_x() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xaa, 0x00]);
    cpu.reset();
    cpu.register_a = 0x10;
    cpu.run();

    assert_eq!(cpu.register_x, cpu.register_a);
  }

  #[test]
  fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    // LDA immediate 0xc0
    // TAX implied
    // INX implied
    cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

    assert_eq!(cpu.register_x, 0xc1);
  }

  #[test]
  // INXのオーバーフローテスト
  fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe8, 0xe8, 0x00]);
    cpu.reset();
    cpu.register_x = 0xff;
    cpu.run();

    assert_eq!(cpu.register_x, 1);
  }

  // ==============================================================================
  // LDAのテスト
  // ==============================================================================
  #[test]
  fn test_lda_from_memory_zeropage() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);

    cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

    assert_eq!(cpu.register_a, 0x55);
  }
  #[test]
  fn test_lda_from_memory_zeropage_x() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xb5, 0x10, 0x00]);
    cpu.reset();

    cpu.register_x = 0x01;
    cpu.mem_write(0x11, 0x65);

    cpu.run();
    assert_eq!(cpu.register_a, 0x65);
  }
  #[test]
  fn test_lda_from_memory_absolute() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xad, 0x10, 0xaa, 0x00]);
    cpu.reset();

    cpu.mem_write(0xaa10, 0x66);

    cpu.run();
    assert_eq!(cpu.register_a, 0x66);
  }
  #[test]
  fn test_lda_from_memory_absolute_x() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xbd, 0x10, 0xab, 0x00]);
    cpu.reset();

    cpu.register_x = 0x11;
    cpu.mem_write(0xab21, 0x76);

    cpu.run();
    assert_eq!(cpu.register_a, 0x76);
  }
  #[test]
  fn test_lda_from_memory_absolute_y() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xb9, 0x10, 0xbb, 0x00]);
    cpu.reset();

    cpu.register_y = 0x11;
    cpu.mem_write(0xbb21, 0x77);

    cpu.run();
    assert_eq!(cpu.register_a, 0x77);
  }
  #[test]
  fn test_lda_from_memory_indirect_x() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xa1, 0x01, 0x00]);
    cpu.reset();

    cpu.register_x = 0x0F;
    cpu.mem_write_u16(0x10, 0x1001);
    cpu.mem_write(0x1001, 0x77);

    cpu.run();
    assert_eq!(cpu.register_a, 0x77);
  }
  #[test]
  fn test_lda_from_memory_indirect_y() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xb1, 0x11, 0x00]);
    cpu.reset();

    cpu.register_y = 0x0F;
    cpu.mem_write_u16(0x11, 0x1001);
    cpu.mem_write(0x1010, 0x77);

    cpu.run();
    assert_eq!(cpu.register_a, 0x77);
  }

  #[test]
  fn test_sta_from_memory() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x85, 0x10, 0x00]);
    cpu.reset();

    cpu.register_a = 0xAF;

    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0xAF);
  }

  #[test]
  // キャリーフラグを建てていない状態のADCテスト
  fn test_adc_no_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x10, 0x00]);
    cpu.reset();
    cpu.register_a = 0x1A;
    cpu.run();

    assert_eq!(cpu.register_a, 0x2A);
    assert_eq!(cpu.status, 0x00);
  }
  #[test]
  // キャリーフラグを持った状態のADCテスト
  fn test_adc_has_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x10, 0x00]);
    cpu.reset();
    cpu.register_a = 0x1A;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x2B);
    assert_eq!(cpu.status, 0x00);
  }
  #[test]
  // キャリーフラグを建てるADCテスト
  fn test_adc_occur_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x02, 0x00]);
    cpu.reset();
    cpu.register_a = 0xFF;
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  // 正の値同士のADCでオーバーフローが発生する際のテスト
  fn test_adc_occur_overflow_plus() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x01, 0x00]);
    cpu.reset();
    cpu.register_a = 0x7F;
    cpu.run();

    assert_eq!(cpu.register_a, 0x80);

    let flags = Flag::overflow() | Flag::negative();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // 負の値同士のADCでオーバーフローが発生する際のテスト
  fn test_adc_occur_overflow_minus() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x85, 0x00]);
    cpu.reset();
    cpu.register_a = 0x85;
    cpu.run();

    assert_eq!(cpu.register_a, 0x0a);

    let flags = Flag::overflow() | Flag::carry();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // キャリーが存在する状態で
  // 正の値同士のADCでオーバーフローが起こる際のテスト
  fn test_adc_occur_overflow_plus_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x6F, 0x00]);
    cpu.reset();
    cpu.register_a = 0x10;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x80);
    let flags = Flag::negative() | Flag::overflow();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // キャリーが存在する状態で
  // 負の値同士のADCでキャリーが発生する際のテスト
  fn test_adc_occur_overflow_minus_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x80, 0x00]);
    cpu.reset();
    cpu.register_a = 0x80;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);

    let flags = Flag::overflow() | Flag::carry();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // 正の値と負の値のADCでオーバーフローが起きる際のテスト
  fn test_adc_no_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x69, 0x7F, 0x00]);
    cpu.reset();
    cpu.register_a = 0x82;
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);

    let flags = Flag::carry();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // キャリーフラグを建てていない状態のSBCテスト
  fn test_sbc_no_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe9, 0x10, 0x00]);
    cpu.reset();
    cpu.register_a = 0x20;
    cpu.run();

    assert_eq!(cpu.register_a, 0x0F);
    // SBCでは特に何もなければキャリーフラグが立つ
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  // キャリーフラグを持った状態のSBCテスト
  fn test_sbc_has_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe9, 0x10, 0x00]);
    cpu.reset();
    cpu.register_a = 0x20;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x10);
    // SBCでは特に何もなければキャリーフラグが立つ
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  // キャリーフラグを建てるSBCテスト
  fn test_sbc_occur_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe9, 0x02, 0x00]);
    cpu.reset();
    cpu.register_a = 0x01;
    cpu.run();

    assert_eq!(cpu.register_a, 0xFE);
    let flags = Flag::negative();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // 正の値同士のSBCでオーバーフローが発生する際のテスト
  fn test_sbc_occur_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe9, 0x81, 0x00]);
    cpu.reset();
    cpu.register_a = 0x7F;
    cpu.run();

    assert_eq!(cpu.register_a, 0xFD);

    let flags = Flag::overflow() | Flag::negative();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // キャリーが存在する状態のSBCでオーバーフローが起こる際のテスト
  fn test_sbc_occur_overflow_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe9, 0x81, 0x00]);
    cpu.reset();
    cpu.register_a = 0x7F;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0xFE);
    let flags = Flag::negative() | Flag::overflow();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  // SBCでオーバーフローが起きる際のテスト
  fn test_sbc_no_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe9, 0x7F, 0x00]);
    cpu.reset();
    cpu.register_a = 0x7E;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0xFF);

    let flags = Flag::negative();
    assert_eq!(cpu.status, flags);
  }
  #[test]
  fn test_and() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x29, 0x03, 0x00]);
    cpu.reset();
    cpu.register_a = 0x05;
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);
  }
  #[test]
  fn test_eor() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x49, 0x03, 0x00]);
    cpu.reset();
    cpu.register_a = 0x05;
    cpu.run();

    assert_eq!(cpu.register_a, 0x06);
  }
  #[test]
  fn test_ora() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x09, 0x03, 0x00]);
    cpu.reset();
    cpu.register_a = 0x05;
    cpu.run();

    assert_eq!(cpu.register_a, 0x07);
  }
  #[test]
  fn test_asl_accumulator() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x0a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x05;
    cpu.run();

    assert_eq!(cpu.register_a, 0x05 * 2);
  }
  #[test]
  fn test_asl_accumulator_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x0a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x81;
    cpu.run();

    assert_eq!(cpu.register_a, 0x02);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_asl_zero_page() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x06, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x03);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
  }
  #[test]
  fn test_asl_zero_page_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x06, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x83);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_lsr_accumulator() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x4a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x2;
    cpu.run();

    assert_eq!(cpu.register_a, 0x02 / 2);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_lsr_zero_page() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x46, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x02);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x02 / 2);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_lsr_accumulator_occur_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x4a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x03;
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_lsr_zero_page_occur_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x46, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x03);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x01);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_lsr_zero_page_zero() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x46, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x01);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x00);
    assert_eq!(cpu.status, Flag::carry() | Flag::zero());
  }
  #[test]
  fn test_rol_accumulator() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x2a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x03;
    cpu.run();

    assert_eq!(cpu.register_a, 0x06);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_rol_zero_page() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x26, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x03);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x06);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_rol_accumulator_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x2a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x03;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x07);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_rol_zero_page_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x26, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x03);
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x07);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_rol_accumulator_zero_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x2a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x00;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_rol_zero_page_zero_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x26, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x00);
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x01);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_ror_accumulator() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x6a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x2;
    cpu.run();

    assert_eq!(cpu.register_a, 0x02 / 2);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_ror_zero_page() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x66, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x02);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x02 / 2);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_ror_accumulator_occur_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x6a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x03;
    cpu.run();

    assert_eq!(cpu.register_a, 0x01);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_ror_zero_page_occur_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x66, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x03);
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x01);
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_ror_accumulator_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x6a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x02;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x81);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_ror_zero_page_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x66, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x02);
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x81);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_ror_accumulator_zero_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x6a, 0x00]);
    cpu.reset();
    cpu.register_a = 0x00;
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.register_a, 0x80);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_ror_zero_page_zero_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x66, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x0001, 0x00);
    cpu.status = Flag::carry();
    cpu.run();

    assert_eq!(cpu.mem_read(0x0001), 0x80);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_bcc() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x90, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bcc_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x90, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::carry();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, Flag::carry());
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bcc_negative_jump() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x90, 0xfc, 0x00]);
    cpu.reset();
    cpu.mem_write(0x7FFF, 0x00);
    cpu.mem_write(0x7FFE, 0xe8);

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8000);
  }
  #[test]
  fn test_bcs() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xb0, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bcs_with_carry() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xb0, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::carry();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, Flag::carry());
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bcs_negative_jump() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xb0, 0xfc, 0x00]);
    cpu.reset();
    cpu.mem_write(0x7FFF, 0x00);
    cpu.mem_write(0x7FFE, 0xe8);
    cpu.status = Flag::carry();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, Flag::carry());
    assert_eq!(cpu.program_counter, 0x8000);
  }
  #[test]
  fn test_beq() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xf0, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_beq_with_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xf0, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::zero();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0); // ZEROはINXで落ちる
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bne() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xd0, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0); // ZEROはINXで落ちる
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bne_with_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xd0, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::zero();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, Flag::zero());
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bit() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x24, 0x00, 0x00]);
    cpu.reset();
    cpu.register_a = 0x00;
    cpu.mem_write(0x0000, 0x00);

    cpu.run();
    assert_eq!(cpu.status, Flag::zero());
  }
  #[test]
  fn test_bit_negative_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x24, 0x00, 0x00]);
    cpu.reset();
    cpu.register_a = 0x00;
    cpu.mem_write(0x0000, 0x80);

    cpu.run();
    assert_eq!(cpu.status, Flag::zero() | Flag::negative());
  }
  #[test]
  fn test_bit_overflow_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x24, 0x00, 0x00]);
    cpu.reset();
    cpu.register_a = 0x40;
    cpu.mem_write(0x0000, 0x40);

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow());
  }
  #[test]
  fn test_bmi() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x30, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bmi_with_negative_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x30, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::negative();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0); // INXしたためnegtiveフラグが落ちる
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bpl() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x10, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bpl_with_negative_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x10, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::negative();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, Flag::negative());
    assert_eq!(cpu.program_counter, 0x8003);
  }
}
