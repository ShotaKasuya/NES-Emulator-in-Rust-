use crate::opscodes::{call, CPU_OPS_CODES};

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
  Indirect,
  Indirect_X,
  Indirect_Y,
  Relative,
  Implied,
  NoneAddressing,
}

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
  pub stack_pointer: u8,
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
      stack_pointer: 0xFF,
      program_counter: 0,
      memory: [0x00; 0x10000],
    }
  }

  fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
    match mode {
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
        panic!("mode {:?} is not supported", mode);
      }
    }
  }

  fn mem_read(&self, addr: u16) -> u8 {
    self.memory[addr as usize]
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    self.memory[addr as usize] = data;
  }

  fn mem_read_u16(&self, pos: u16) -> u16 {
    let lo = self.mem_read(pos) as u16;
    let hi = self.mem_read(pos + 1) as u16;
    (hi << 8) | (lo as u16)
  }

  fn mem_write_u16(&mut self, pos: u16, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.mem_write(pos, lo);
    self.mem_write(pos + 1, hi);
  }

  fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
  }

  fn load(&mut self, program: Vec<u8>) {
    self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    self.mem_write_u16(0xFFFC, 0x8000);
  }

  fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.register_y = 0;
    self.status = 0;
    self.stack_pointer = 0xFF;
    self.program_counter = self.mem_read_u16(0xFFFC);
  }

  fn run(&mut self) {
    loop {
      let opscode = self.mem_read(self.program_counter);
      self.program_counter += 1;

      println!("OPS: {:X}", opscode);

      for op in CPU_OPS_CODES.iter() {
        if op.code == opscode {
          // FIX ME FOR TEST
          if op.name == "BRK" {
            return;
          }
          call(self, &op);
          break;
        }
      }
    }
  }
  pub fn txs(&mut self, mode: &AddressingMode) {}
  pub fn tsx(&mut self, mode: &AddressingMode) {}
  pub fn tya(&mut self, mode: &AddressingMode) {}

  // レジスタaの内容をレジスタxにコピーする
  pub fn tax(&mut self, mode: &AddressingMode) {
    self.register_x = self.register_a;
    self.update_zero_and_negative_flags(self.register_x);
  }
  pub fn tay(&mut self, mode: &AddressingMode) {}
  pub fn txa(&mut self, mode: &AddressingMode) {}

  pub fn sty(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_y);
  }
  pub fn stx(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_x);
  }
  // レジスタaの値をメモリに書き込む
  pub fn sta(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_a);
  }

  pub fn rti(&mut self, mode: &AddressingMode) {
    todo!("BRKが出来ていないので保留")
  }

  pub fn plp(&mut self, mode: &AddressingMode) {
    self.status = self._pop();
  }

  pub fn php(&mut self, mode: &AddressingMode) {
    self._push(self.status);
  }

  pub fn pla(&mut self, mode: &AddressingMode) {
    self.register_a = self._pop();
    self.update_zero_and_negative_flags(self.register_a);
  }

  pub fn pha(&mut self, mode: &AddressingMode) {
    self._push(self.register_a);
  }

  pub fn nop(&mut self, mode: &AddressingMode) {
    // 何もしない
  }

  pub fn ldy(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_y = value;
    self.update_zero_and_negative_flags(value);
  }
  pub fn ldx(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_x = value;
    self.update_zero_and_negative_flags(value);
  }

  // レジスタaに値をコピーする
  pub fn lda(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a = value;
    self.update_zero_and_negative_flags(value);
  }

  pub fn rts(&mut self, mode: &AddressingMode) {
    let value = self._pop_u16();
    self.program_counter = value;
  }

  pub fn jsr(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self._push_u16(self.program_counter + 2);
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
  pub fn adc(&mut self, mode: &AddressingMode) {
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
  pub fn and(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a &= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術左シフト
  // like LSR,ROL,ROR
  pub fn asl(&mut self, mode: &AddressingMode) {
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
  pub fn bcc(&mut self, mode: &AddressingMode) {
    if self.status & Flag::carry() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // キャリーが立っていたら分岐
  pub fn bcs(&mut self, mode: &AddressingMode) {
    if self.status & Flag::carry() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // キャリーフラグをセット
  pub fn sec(&mut self, mode: &AddressingMode) {
    self.status |= Flag::carry();
  }

  // デシマルモードをセット
  pub fn sed(&mut self, mode: &AddressingMode) {
    self.status |= Flag::decimal();
  }

  // ゼロフラグが立っていたら分岐
  pub fn beq(&mut self, mode: &AddressingMode) {
    if self.status & Flag::zero() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // レジスタaとメモリの値の論理積が0ならゼロフラグを立てる
  // メモリの7ビットと6ビットを基に
  // オーバーフローフラグとネガティブフラグを立てる
  pub fn bit(&mut self, mode: &AddressingMode) {
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
  pub fn bmi(&mut self, mode: &AddressingMode) {
    if self.status & Flag::negative() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // ゼロフラグがクリアなら分岐
  pub fn bne(&mut self, mode: &AddressingMode) {
    if self.status & Flag::zero() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr
    }
  }

  // ネガティブフラグがクリアなら分岐
  pub fn bpl(&mut self, mode: &AddressingMode) {
    if self.status & Flag::negative() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr;
    }
  }

  // オーバーフローフラグのクリア
  pub fn clv(&mut self, mode: &AddressingMode) {
    self.status &= !Flag::overflow();
  }

  pub fn cmp(&mut self, mode: &AddressingMode) {
    self._cmp(self.register_a, mode);
  }

  pub fn cpx(&mut self, mode: &AddressingMode) {
    self._cmp(self.register_x, mode);
  }

  // compare register y
  pub fn cpy(&mut self, mode: &AddressingMode) {
    self._cmp(self.register_y, mode);
  }

  pub fn _cmp(&mut self, target: u8, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
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
  pub fn brk(&mut self, mode: &AddressingMode) {
    self.program_counter = self.mem_read_u16(0xFFFE);
    // TODO!
    self.status |= Flag::Break as u8;
  }

  // オーバーフローフラグがクリアなら分岐
  pub fn bvc(&mut self, mode: &AddressingMode) {
    if self.status & Flag::overflow() == 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr;
    }
  }

  // オーバーフローフラグが立っていたら分岐
  pub fn bvs(&mut self, mode: &AddressingMode) {
    if self.status & Flag::overflow() != 0 {
      let addr = self.get_operand_address(mode);
      self.program_counter = addr;
    }
  }

  // キャリーをクリア
  pub fn clc(&mut self, mode: &AddressingMode) {
    self.status &= !Flag::carry();
  }

  // デシマルモードをクリア
  pub fn cld(&mut self, mode: &AddressingMode) {
    self.status &= !Flag::decimal();
  }

  // インタラプトをクリア
  pub fn cli(&mut self, mode: &AddressingMode) {
    self.status &= !Flag::interrupt_disable();
  }

  // デクリメントメモリ
  pub fn dec(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    let value = value.wrapping_sub(1);

    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
  }

  // デクリメントXレジスタ
  pub fn dex(&mut self, mode: &AddressingMode) {
    self.register_x = self.register_x.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_x);
  }

  // デクリメントYレジスタ
  pub fn dey(&mut self, mode: &AddressingMode) {
    self.register_y = self.register_y.wrapping_sub(1);
    self.update_zero_and_negative_flags(self.register_y);
  }

  // デシマルモードをクリア
  pub fn sei(&mut self, mode: &AddressingMode) {
    self.status |= Flag::interrupt_disable();
  }

  // レジスタaの値とメモリの値の排他的論理和をレジスタaに書き込む
  // like AND,ORA
  pub fn eor(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a ^= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // デクリメントメモリ
  pub fn inc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    let (value, _) = value.overflowing_add(1);

    self.mem_write(addr, value);
    self.update_zero_and_negative_flags(value);
  }

  // デクリメントXレジスタ
  pub fn inx(&mut self, mode: &AddressingMode) {
    let (value, _) = self.register_x.overflowing_add(1);

    self.register_x = value;
    self.update_zero_and_negative_flags(value);
  }

  // デクリメントYレジスタ
  pub fn iny(&mut self, mode: &AddressingMode) {
    let (value, _) = self.register_y.overflowing_add(1);

    self.register_y = value;
    self.update_zero_and_negative_flags(value);
  }

  pub fn jmp(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
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
  pub fn lsr(&mut self, mode: &AddressingMode) {
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
  pub fn ora(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    self.register_a |= value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // 算術左シフト(キャリーによる補完あり)
  // like ASL,LSR,ROR
  pub fn rol(&mut self, mode: &AddressingMode) {
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
  pub fn ror(&mut self, mode: &AddressingMode) {
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

  // レジスタaとメモリの値の差をレジスタaに書き込む
  // like ADC
  pub fn sbc(&mut self, mode: &AddressingMode) {
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
  #[test]
  fn test_bvc() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x50, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_bvc_with_overflow_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x50, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, Flag::overflow());
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bvs() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x70, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x00);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x8003);
  }
  #[test]
  fn test_bvs_with_overflow_flag() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x70, 0x02, 0x00, 0x00, 0xe8, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow();

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, Flag::overflow());
    assert_eq!(cpu.program_counter, 0x8006);
  }
  #[test]
  fn test_clc() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x18, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow() | Flag::carry();

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow());
  }
  #[test]
  fn test_sec() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x38, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow();

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow() | Flag::carry());
  }
  #[test]
  fn test_cld() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xd8, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow() | Flag::decimal();

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow());
  }
  #[test]
  fn test_sed() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xf8, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow();

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow() | Flag::decimal());
  }
  #[test]
  fn test_cli() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x58, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow() | Flag::interrupt_disable();

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow());
  }
  #[test]
  fn test_sei() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x78, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow();

    cpu.run();
    assert_eq!(cpu.status, Flag::overflow() | Flag::interrupt_disable());
  }

  #[test]
  fn test_clv() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xB8, 0x00]);
    cpu.reset();

    cpu.status = Flag::overflow();

    cpu.run();
    assert_eq!(cpu.status, 0);
  }

  #[test]
  fn test_cmp() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xC9, 0x01, 0x00]);
    cpu.reset();
    cpu.register_a = 0x02;

    cpu.run();
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_cmp_eq() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xC9, 0x02, 0x00]);
    cpu.reset();
    cpu.register_a = 0x02;

    cpu.run();
    assert_eq!(cpu.status, Flag::carry() | Flag::zero());
  }
  #[test]
  fn test_cmp_negative() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xC9, 0x03, 0x00]);
    cpu.reset();
    cpu.register_a = 0x02;

    cpu.run();
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_cpx() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xE0, 0x01, 0x00]);
    cpu.reset();
    cpu.register_x = 0x02;

    cpu.run();
    assert_eq!(cpu.status, Flag::carry());
  }
  #[test]
  fn test_cpy() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xC0, 0x01, 0x00]);
    cpu.reset();
    cpu.register_y = 0x02;

    cpu.run();
    assert_eq!(cpu.status, Flag::carry());
  }

  #[test]
  fn test_dec() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xC6, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01, 0x05);

    cpu.run();
    assert_eq!(cpu.mem_read(0x0001), 0x04);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_dec_with_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xC6, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01, 0x00);

    cpu.run();
    assert_eq!(cpu.mem_read(0x0001), 0xFF);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_dex() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xCA, 0x00]);
    cpu.reset();
    cpu.register_x = 0x05;

    cpu.run();
    assert_eq!(cpu.register_x, 0x04);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_dex_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xCA, 0x00]);
    cpu.reset();
    cpu.register_x = 0x00;

    cpu.run();
    assert_eq!(cpu.register_x, 0xFF);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_dey() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x88, 0x00]);
    cpu.reset();
    cpu.register_y = 0x05;

    cpu.run();
    assert_eq!(cpu.register_y, 0x04);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_dey_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x88, 0x00]);
    cpu.reset();
    cpu.register_y = 0x00;

    cpu.run();
    assert_eq!(cpu.register_y, 0xFF);
    assert_eq!(cpu.status, Flag::negative());
  }

  #[test]
  fn test_inc() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xE6, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01, 0x05);

    cpu.run();
    assert_eq!(cpu.mem_read(0x0001), 0x06);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_inc_with_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xE6, 0x01, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01, 0x7F);

    cpu.run();
    assert_eq!(cpu.mem_read(0x0001), 0x80);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_inx() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xE8, 0x00]);
    cpu.reset();
    cpu.register_x = 0x05;

    cpu.run();
    assert_eq!(cpu.register_x, 0x06);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xE8, 0x00]);
    cpu.reset();
    cpu.register_x = 0x7F;

    cpu.run();
    assert_eq!(cpu.register_x, 0x80);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_iny() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xc8, 0x00]);
    cpu.reset();
    cpu.register_y = 0x05;

    cpu.run();
    assert_eq!(cpu.register_y, 0x06);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_iny_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xc8, 0x00]);
    cpu.reset();
    cpu.register_y = 0x7F;

    cpu.run();
    assert_eq!(cpu.register_y, 0x80);
    assert_eq!(cpu.status, Flag::negative());
  }

  #[test]
  fn test_jmp() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x4C, 0x30, 0x40, 0x00]);
    cpu.reset();
    cpu.mem_write(0x4030, 0xE8);
    cpu.mem_write(0x4031, 0x00);

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x4032);
  }
  #[test]
  fn test_jmp_indirect() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x6C, 0x30, 0x40, 0x00]);
    cpu.reset();
    cpu.mem_write(0x4030, 0x80);
    cpu.mem_write(0x4031, 0x00);
    println!("{:?}", cpu.program_counter);
    cpu.mem_write(0x8000, 0xE8);
    cpu.mem_write(0x8001, 0x00);

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
  }
  #[test]
  fn test_jsr() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x20, 0x30, 0x40, 0x00]);
    cpu.reset();
    cpu.mem_write(0x4030, 0xE8);
    cpu.mem_write(0x4031, 0x00);

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.program_counter, 0x4032);
    assert_eq!(cpu.mem_read_u16(0x01FE), 0x8003);
  }

  #[test]
  fn test_rts() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x60, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01FF, 0x05);
    cpu.mem_write(0x01FE, 0x06);

    cpu.mem_write(0x0506, 0xe8);
    cpu.mem_write(0x0507, 0x00);
    cpu.stack_pointer = 0xFD;

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.program_counter, 0x0508);
  }
  #[test]
  fn test_jsr_rts() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x20, 0x30, 0x40, 0x00]);
    cpu.reset();
    cpu.mem_write(0x4030, 0xe8);
    cpu.mem_write(0x4031, 0x60); // RTS
    cpu.mem_write(0x4032, 0x00);

    cpu.run();
    assert_eq!(cpu.register_x, 0x01);
    assert_eq!(cpu.status, 0);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.program_counter, 0x8004);
  }
  #[test]
  fn test_ldx() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xa2, 0x05, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_x, 0x05);
  }
  #[test]
  fn test_ldy() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xa0, 0x05, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.register_y, 0x05);
  }

  #[test]
  fn test_nop() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xea, 0x00]);
    cpu.reset();

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
  }

  #[test]
  fn test_pha() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x48, 0x00]);
    cpu.reset();
    cpu.register_a = 0x07;

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_a, 0x07);
    assert_eq!(cpu.stack_pointer, 0xFE);
    assert_eq!(cpu.mem_read(0x01FF), 0x07);
  }

  #[test]
  fn test_pla() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x68, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01FF, 0x07);
    cpu.stack_pointer = 0xFE;

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_a, 0x07);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.status, 0x00);
  }
  #[test]
  fn test_pla_flag_zero() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x68, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01FF, 0x00);
    cpu.stack_pointer = 0xFE;

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.register_a, 0x00);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.status, Flag::zero());
  }

  #[test]
  fn test_pla_and_pha() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x48, 0xa9, 0x60, 0x68, 0x00]);
    cpu.reset();
    cpu.register_a = 0x80;

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8005);
    assert_eq!(cpu.register_a, 0x80);
    assert_eq!(cpu.stack_pointer, 0xFF);
    assert_eq!(cpu.status, Flag::negative());
  }
  #[test]
  fn test_php() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x08, 0x00]);
    cpu.reset();
    cpu.status = Flag::negative() | Flag::overflow();

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.stack_pointer, 0xFE);
    assert_eq!(cpu.mem_read(0x01FF), Flag::negative() | Flag::overflow());
  }
  #[test]
  fn test_plp() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x28, 0x00]);
    cpu.reset();
    cpu.mem_write(0x01FF, Flag::carry() | Flag::zero());
    cpu.stack_pointer = 0xFE;

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
    assert_eq!(cpu.status, Flag::carry() | Flag::zero());
    assert_eq!(cpu.stack_pointer, 0xFF);
  }
  #[test]
  fn test_plp_and_php() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x08, 0xa9, 0x80, 0x28, 0x00]);
    cpu.reset();
    cpu.status = Flag::overflow() | Flag::carry();

    cpu.run();
    assert_eq!(cpu.program_counter, 0x8005);
    assert_eq!(cpu.status, Flag::overflow() | Flag::carry());
    assert_eq!(cpu.stack_pointer, 0xFF);
  }

  #[test]
  fn test_stx() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x86, 0x10, 0x00]);
    cpu.reset();

    cpu.register_x = 0xBA;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0xBA);
  }
  #[test]
  fn test_sty() {
    let mut cpu = CPU::new();
    cpu.load(vec![0x84, 0x10, 0x00]);
    cpu.reset();

    cpu.register_y = 0xBA;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0xBA);
  }
}
