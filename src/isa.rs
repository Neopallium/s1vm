
use crate::error::*;
use crate::value::*;

type PC = u32;

/// TODO: try packing instructions.
/// Make enum 1 byte.  Encode u32/u64 parameters as instructions.
/// Need 256 enum values, so add extra DataX values to cover full 0-255 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
  Br(PC),
  BrIfEqz(PC),
  BrIfNez(PC),

  BrTable{
    count: u8,
  },

  Unreachable,
  Return,

  Call(FuncIdx),
  CallIndirect(TypeIdx),

  Drop,
  Select,

  GetLocal(LocalIdx),
  SetLocal(LocalIdx),
  TeeLocal(LocalIdx),

  GetGlobal(GlobalIdx),
  SetGlobal(GlobalIdx),

  I32Load(MemIdx),
  I64Load(MemIdx),
  F32Load(MemIdx),
  F64Load(MemIdx),
  I32Load8S(MemIdx),
  I32Load8U(MemIdx),
  I32Load16S(MemIdx),
  I32Load16U(MemIdx),
  I64Load8S(MemIdx),
  I64Load8U(MemIdx),
  I64Load16S(MemIdx),
  I64Load16U(MemIdx),
  I64Load32S(MemIdx),
  I64Load32U(MemIdx),
  I32Store(MemIdx),
  I64Store(MemIdx),
  F32Store(MemIdx),
  F64Store(MemIdx),
  I32Store8(MemIdx),
  I32Store16(MemIdx),
  I64Store8(MemIdx),
  I64Store16(MemIdx),
  I64Store32(MemIdx),

  CurrentMemory,
  GrowMemory,

  I32Const(ConstI32),
  I64Const(ConstI64),
  F32Const(ConstF32),
  F64Const(ConstF64),

  I32Eqz,
  I32Eq,
  I32Ne,
  I32LtS,
  I32LtU,
  I32GtS,
  I32GtU,
  I32LeS,
  I32LeU,
  I32GeS,
  I32GeU,

  I64Eqz,
  I64Eq,
  I64Ne,
  I64LtS,
  I64LtU,
  I64GtS,
  I64GtU,
  I64LeS,
  I64LeU,
  I64GeS,
  I64GeU,

  F32Eq,
  F32Ne,
  F32Lt,
  F32Gt,
  F32Le,
  F32Ge,

  F64Eq,
  F64Ne,
  F64Lt,
  F64Gt,
  F64Le,
  F64Ge,

  I32Clz,
  I32Ctz,
  I32Popcnt,
  I32Add,
  I32Sub,
  I32Mul,
  I32DivS,
  I32DivU,
  I32RemS,
  I32RemU,
  I32And,
  I32Or,
  I32Xor,
  I32Shl,
  I32ShrS,
  I32ShrU,
  I32Rotl,
  I32Rotr,

  I64Clz,
  I64Ctz,
  I64Popcnt,
  I64Add,
  I64Sub,
  I64Mul,
  I64DivS,
  I64DivU,
  I64RemS,
  I64RemU,
  I64And,
  I64Or,
  I64Xor,
  I64Shl,
  I64ShrS,
  I64ShrU,
  I64Rotl,
  I64Rotr,
  F32Abs,
  F32Neg,
  F32Ceil,
  F32Floor,
  F32Trunc,
  F32Nearest,
  F32Sqrt,
  F32Add,
  F32Sub,
  F32Mul,
  F32Div,
  F32Min,
  F32Max,
  F32Copysign,
  F64Abs,
  F64Neg,
  F64Ceil,
  F64Floor,
  F64Trunc,
  F64Nearest,
  F64Sqrt,
  F64Add,
  F64Sub,
  F64Mul,
  F64Div,
  F64Min,
  F64Max,
  F64Copysign,

  I32WrapI64,
  I32TruncSF32,
  I32TruncUF32,
  I32TruncSF64,
  I32TruncUF64,
  I64ExtendSI32,
  I64ExtendUI32,
  I64TruncSF32,
  I64TruncUF32,
  I64TruncSF64,
  I64TruncUF64,
  F32ConvertSI32,
  F32ConvertUI32,
  F32ConvertSI64,
  F32ConvertUI64,
  F32DemoteF64,
  F64ConvertSI32,
  F64ConvertUI32,
  F64ConvertSI64,
  F64ConvertUI64,
  F64PromoteF32,

  I32ReinterpretF32,
  I64ReinterpretF64,
  F32ReinterpretI32,
  F64ReinterpretI64,
}

type LabelID = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BrKind {
  Br,
  BrIfEqz,
  BrIfNez,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Block {
  Block {
    end_label: LabelID,
  },
  Loop {
    head: LabelID,
  },
  If {
    end_label: LabelID,
    if_not: LabelID,
  },
  Else {
    end_label: LabelID,
  },
}

impl Block {
  fn br_destination(&self) -> LabelID {
    match *self {
      Block::Block{ end_label } => end_label,
      Block::Loop{ head } => head,
      Block::If{ end_label, .. } => end_label,
      Block::Else{ end_label } => end_label,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Label {
  pc: Option<PC>,
  reloc: Vec<PC>,
}

#[derive(Debug, Clone, PartialEq)]
struct Sink {
  code: Vec<Instruction>,
  blocks: Vec<Block>,
  labels: Vec<Label>,
}

impl Sink {
  fn new(size: usize) -> Sink {
    Sink {
      code: Vec::with_capacity(size),
      blocks: vec![],
      labels: vec![],
    }
  }

  fn pc(&self) -> u32 {
    self.code.len() as u32
  }

  fn push_block(&mut self, block: Block) {
    self.blocks.push(block);
  }

  fn ref_block(&self, depth: u32) -> Result<&Block> {
    // Checked calculation: idx = ((len - 1) - depth)
    let idx = self.blocks
      .len()
      .checked_sub(1)
      .and_then(|x| x.checked_sub(depth as usize))
      .ok_or_else(|| Error::ValidationError(format!("missing block")))?;

    Ok(&self.blocks[idx])
  }

  fn pop_block(&mut self) -> Result<Block> {
    self.blocks.pop()
      .ok_or_else(|| Error::ValidationError(format!("missing block")))
  }

  fn is_stack_empty(&mut self) -> bool {
    self.blocks.len() == 0
  }

  fn br_destination(&self, depth: u32) -> Result<LabelID> {
    let block = self.ref_block(depth)?;
    Ok(block.br_destination())
  }

  fn new_label(&mut self) -> LabelID {
    let id = self.labels.len() as LabelID;
    self.labels.push(Default::default());
    id
  }

  fn ref_label(&mut self, id: LabelID) -> PC {
    let ref_pc = self.pc();
    let label = &mut self.labels[id as usize];
    if let Some(pc) = label.pc {
      pc
    } else {
      label.reloc.push(ref_pc);
      PC::max_value()
    }
  }

  fn resolve_label(&mut self, id: LabelID) {
    let label_pc = self.pc();
    let label = &mut self.labels[id as usize];
    label.pc = Some(label_pc);
    for ref_pc in label.reloc.iter() {
      let op = &mut self.code[*ref_pc as usize];
      match op {
        Instruction::Br(ref mut pc) => *pc = label_pc,
        Instruction::BrIfEqz(ref mut pc) => *pc = label_pc,
        Instruction::BrIfNez(ref mut pc) => *pc = label_pc,
        op => {
          unreachable!("Invalid patch instruction '{:?}' at pc:{}", op, ref_pc);
        },
      }
    }
  }

  fn emit_br(&mut self, kind: BrKind, label: LabelID) -> Result<()> {
    let pc = self.ref_label(label);
    let op = match kind {
      BrKind::Br => Instruction::Br(pc),
      BrKind::BrIfEqz => Instruction::BrIfEqz(pc),
      BrKind::BrIfNez => Instruction::BrIfNez(pc),
    };
    self.code.push(op);
    Ok(())
  }

  fn emit(&mut self, op: Instruction) {
    self.code.push(op);
  }

  fn into_inner(self) -> Vec<Instruction> {
    self.code
  }
}

/// Compile wasm instructions into internal ISA
pub fn compile_function(func: &bwasm::Function) -> Result<Vec<Instruction>> {
  let code = func.instructions();
  let mut sink = Sink::new(code.len());

  // Push function block.
  let block = self::Block::Block {
    end_label: sink.new_label(),
  };
  sink.push_block(block);

  // compile function opcodes.
  for op in code.into_iter() {
    use parity_wasm::elements::Instruction::*;
    //eprintln!("compile op: {:?}", op);
    let res = match op {
	    Block(_) => {
        let block = self::Block::Block {
          end_label: sink.new_label(),
        };
        sink.push_block(block);
        None
      },
	    Loop(_) => {
        let head = sink.new_label();
        let block = self::Block::Loop {
          head,
        };
        sink.push_block(block);
        // resolve loop label here.
        sink.resolve_label(head);
        None
      },
	    If(_) => {
        let if_not = sink.new_label();
        let block = self::Block::If {
          end_label: sink.new_label(),
          if_not,
        };
        sink.push_block(block);
        sink.emit_br(BrKind::BrIfEqz, if_not)?;
        None
      },
	    Else => {
        let block = sink.pop_block()?;
        match block {
          self::Block::If{ end_label, if_not } => {
            // jump to the end of the 'else' block.
            sink.emit_br(BrKind::Br, end_label)?;
            // resolve label for start of 'else' block.
            sink.resolve_label(if_not);
            // start 'else' block
            sink.push_block(self::Block::Else {
              end_label: end_label,
            });
          },
          _ => unreachable!("Validation ensures the correct nesting."),
        }
        None
      },
	    End => {
        let block = sink.pop_block()?;
        match block {
          self::Block::Block{ end_label } => {
            sink.resolve_label(end_label);
          },
          self::Block::Loop{ .. } => (),
          self::Block::If{ end_label, if_not } => {
            sink.resolve_label(if_not);
            sink.resolve_label(end_label);
          },
          self::Block::Else{ end_label } => {
            sink.resolve_label(end_label);
          },
        }
        if sink.is_stack_empty() {
          sink.emit(Instruction::Return);
        }
        None
      },
	    Br(block_depth) => {
        let dst = sink.br_destination(*block_depth)?;
        sink.emit_br(BrKind::Br, dst)?;
        None
      },
	    BrIf(block_depth) => {
        let dst = sink.br_destination(*block_depth)?;
        sink.emit_br(BrKind::BrIfNez, dst)?;
        todo!("");
      },
	    BrTable(ref br_table) => {
        for block_depth in br_table.table.iter() {
          let dst = sink.br_destination(*block_depth)?;
          sink.emit_br(BrKind::Br, dst)?;
        }
        let dst = sink.br_destination(br_table.default)?;
        sink.emit_br(BrKind::Br, dst)?;
        todo!("");
      },
	    Return => {
        Some(Instruction::Return)
      },

	    Call(func_idx) => {
        Some(Instruction::Call(*func_idx))
      },
	    CallIndirect(idx, _reserved) => {
        Some(Instruction::CallIndirect(*idx))
      },

	    Unreachable => Some(Instruction::Unreachable),
	    Nop => None,

	    Drop => Some(Instruction::Drop),
	    Select => Some(Instruction::Select),

	    GetLocal(local_idx) => Some(Instruction::GetLocal(*local_idx)),
	    SetLocal(local_idx) => Some(Instruction::SetLocal(*local_idx)),
	    TeeLocal(local_idx) => Some(Instruction::TeeLocal(*local_idx)),
	    GetGlobal(local_idx) => Some(Instruction::GetGlobal(*local_idx)),
	    SetGlobal(local_idx) => Some(Instruction::SetGlobal(*local_idx)),

	    I32Load(_align, offset) => Some(Instruction::I32Load(*offset)),
	    I64Load(_align, offset) => Some(Instruction::I64Load(*offset)),
	    F32Load(_align, offset) => Some(Instruction::F32Load(*offset)),
	    F64Load(_align, offset) => Some(Instruction::F64Load(*offset)),
	    I32Load8S(_align, offset) => Some(Instruction::I32Load8S(*offset)),
	    I32Load8U(_align, offset) => Some(Instruction::I32Load8U(*offset)),
	    I32Load16S(_align, offset) => Some(Instruction::I32Load16S(*offset)),
	    I32Load16U(_align, offset) => Some(Instruction::I32Load16U(*offset)),
	    I64Load8S(_align, offset) => Some(Instruction::I64Load8S(*offset)),
	    I64Load8U(_align, offset) => Some(Instruction::I64Load8U(*offset)),
	    I64Load16S(_align, offset) => Some(Instruction::I64Load16S(*offset)),
	    I64Load16U(_align, offset) => Some(Instruction::I64Load16U(*offset)),
	    I64Load32S(_align, offset) => Some(Instruction::I64Load32S(*offset)),
	    I64Load32U(_align, offset) => Some(Instruction::I64Load32U(*offset)),
	    I32Store(_align, offset) => Some(Instruction::I32Store(*offset)),
	    I64Store(_align, offset) => Some(Instruction::I64Store(*offset)),
	    F32Store(_align, offset) => Some(Instruction::F32Store(*offset)),
	    F64Store(_align, offset) => Some(Instruction::F64Store(*offset)),
	    I32Store8(_align, offset) => Some(Instruction::I32Store8(*offset)),
	    I32Store16(_align, offset) => Some(Instruction::I32Store16(*offset)),
	    I64Store8(_align, offset) => Some(Instruction::I64Store8(*offset)),
	    I64Store16(_align, offset) => Some(Instruction::I64Store16(*offset)),
	    I64Store32(_align, offset) => Some(Instruction::I64Store32(*offset)),

	    CurrentMemory(_mem_idx) => Some(Instruction::CurrentMemory),
	    GrowMemory(_mem_idx) => Some(Instruction::GrowMemory),

	    I32Const(val) => Some(Instruction::I32Const(*val)),
	    I64Const(val) => Some(Instruction::I64Const(*val)),
	    F32Const(val) => Some(Instruction::F32Const(f32::from_bits(*val as _))),
	    F64Const(val) => Some(Instruction::F64Const(f64::from_bits(*val as _))),

	    I32Eqz => Some(Instruction::I32Eqz),
	    I32Eq => Some(Instruction::I32Eq),
	    I32Ne => Some(Instruction::I32Ne),
	    I32LtS => Some(Instruction::I32LtS),
	    I32LtU => Some(Instruction::I32LtU),
	    I32GtS => Some(Instruction::I32GtS),
	    I32GtU => Some(Instruction::I32GtU),
	    I32LeS => Some(Instruction::I32LeS),
	    I32LeU => Some(Instruction::I32LeU),
	    I32GeS => Some(Instruction::I32GeS),
	    I32GeU => Some(Instruction::I32GeU),

	    I64Eqz => Some(Instruction::I64Eqz),
	    I64Eq => Some(Instruction::I64Eq),
	    I64Ne => Some(Instruction::I64Ne),
	    I64LtS => Some(Instruction::I64LtS),
	    I64LtU => Some(Instruction::I64LtU),
	    I64GtS => Some(Instruction::I64GtS),
	    I64GtU => Some(Instruction::I64GtU),
	    I64LeS => Some(Instruction::I64LeS),
	    I64LeU => Some(Instruction::I64LeU),
	    I64GeS => Some(Instruction::I64GeS),
	    I64GeU => Some(Instruction::I64GeU),

	    F32Eq => Some(Instruction::F32Eq),
	    F32Ne => Some(Instruction::F32Ne),
	    F32Lt => Some(Instruction::F32Lt),
	    F32Gt => Some(Instruction::F32Gt),
	    F32Le => Some(Instruction::F32Le),
	    F32Ge => Some(Instruction::F32Ge),

	    F64Eq => Some(Instruction::F64Eq),
	    F64Ne => Some(Instruction::F64Ne),
	    F64Lt => Some(Instruction::F64Lt),
	    F64Gt => Some(Instruction::F64Gt),
	    F64Le => Some(Instruction::F64Le),
	    F64Ge => Some(Instruction::F64Ge),

	    I32Clz => Some(Instruction::I32Clz),
	    I32Ctz => Some(Instruction::I32Ctz),
	    I32Popcnt => Some(Instruction::I32Popcnt),
	    I32Add => Some(Instruction::I32Add),
	    I32Sub => Some(Instruction::I32Sub),
	    I32Mul => Some(Instruction::I32Mul),
	    I32DivS => Some(Instruction::I32DivS),
	    I32DivU => Some(Instruction::I32DivU),
	    I32RemS => Some(Instruction::I32RemS),
	    I32RemU => Some(Instruction::I32RemU),
	    I32And => Some(Instruction::I32And),
	    I32Or => Some(Instruction::I32Or),
	    I32Xor => Some(Instruction::I32Xor),
	    I32Shl => Some(Instruction::I32Shl),
	    I32ShrS => Some(Instruction::I32ShrS),
	    I32ShrU => Some(Instruction::I32ShrU),
	    I32Rotl => Some(Instruction::I32Rotl),
	    I32Rotr => Some(Instruction::I32Rotr),

	    I64Clz => Some(Instruction::I64Clz),
	    I64Ctz => Some(Instruction::I64Ctz),
	    I64Popcnt => Some(Instruction::I64Popcnt),
	    I64Add => Some(Instruction::I64Add),
	    I64Sub => Some(Instruction::I64Sub),
	    I64Mul => Some(Instruction::I64Mul),
	    I64DivS => Some(Instruction::I64DivS),
	    I64DivU => Some(Instruction::I64DivU),
	    I64RemS => Some(Instruction::I64RemS),
	    I64RemU => Some(Instruction::I64RemU),
	    I64And => Some(Instruction::I64And),
	    I64Or => Some(Instruction::I64Or),
	    I64Xor => Some(Instruction::I64Xor),
	    I64Shl => Some(Instruction::I64Shl),
	    I64ShrS => Some(Instruction::I64ShrS),
	    I64ShrU => Some(Instruction::I64ShrU),
	    I64Rotl => Some(Instruction::I64Rotl),
	    I64Rotr => Some(Instruction::I64Rotr),
	    F32Abs => Some(Instruction::F32Abs),
	    F32Neg => Some(Instruction::F32Neg),
	    F32Ceil => Some(Instruction::F32Ceil),
	    F32Floor => Some(Instruction::F32Floor),
	    F32Trunc => Some(Instruction::F32Trunc),
	    F32Nearest => Some(Instruction::F32Nearest),
	    F32Sqrt => Some(Instruction::F32Sqrt),
	    F32Add => Some(Instruction::F32Add),
	    F32Sub => Some(Instruction::F32Sub),
	    F32Mul => Some(Instruction::F32Mul),
	    F32Div => Some(Instruction::F32Div),
	    F32Min => Some(Instruction::F32Min),
	    F32Max => Some(Instruction::F32Max),
	    F32Copysign => Some(Instruction::F32Copysign),
	    F64Abs => Some(Instruction::F64Abs),
	    F64Neg => Some(Instruction::F64Neg),
	    F64Ceil => Some(Instruction::F64Ceil),
	    F64Floor => Some(Instruction::F64Floor),
	    F64Trunc => Some(Instruction::F64Trunc),
	    F64Nearest => Some(Instruction::F64Nearest),
	    F64Sqrt => Some(Instruction::F64Sqrt),
	    F64Add => Some(Instruction::F64Add),
	    F64Sub => Some(Instruction::F64Sub),
	    F64Mul => Some(Instruction::F64Mul),
	    F64Div => Some(Instruction::F64Div),
	    F64Min => Some(Instruction::F64Min),
	    F64Max => Some(Instruction::F64Max),
	    F64Copysign => Some(Instruction::F64Copysign),

	    I32WrapI64 => Some(Instruction::I32WrapI64),
	    I32TruncSF32 => Some(Instruction::I32TruncSF32),
	    I32TruncUF32 => Some(Instruction::I32TruncUF32),
	    I32TruncSF64 => Some(Instruction::I32TruncSF64),
	    I32TruncUF64 => Some(Instruction::I32TruncUF64),
	    I64ExtendSI32 => Some(Instruction::I64ExtendSI32),
	    I64ExtendUI32 => Some(Instruction::I64ExtendUI32),
	    I64TruncSF32 => Some(Instruction::I64TruncSF32),
	    I64TruncUF32 => Some(Instruction::I64TruncUF32),
	    I64TruncSF64 => Some(Instruction::I64TruncSF64),
	    I64TruncUF64 => Some(Instruction::I64TruncUF64),
	    F32ConvertSI32 => Some(Instruction::F32ConvertSI32),
	    F32ConvertUI32 => Some(Instruction::F32ConvertUI32),
	    F32ConvertSI64 => Some(Instruction::F32ConvertSI64),
	    F32ConvertUI64 => Some(Instruction::F32ConvertUI64),
	    F32DemoteF64 => Some(Instruction::F32DemoteF64),
	    F64ConvertSI32 => Some(Instruction::F64ConvertSI32),
	    F64ConvertUI32 => Some(Instruction::F64ConvertUI32),
	    F64ConvertSI64 => Some(Instruction::F64ConvertSI64),
	    F64ConvertUI64 => Some(Instruction::F64ConvertUI64),
	    F64PromoteF32 => Some(Instruction::F64PromoteF32),

	    I32ReinterpretF32 => Some(Instruction::I32ReinterpretF32),
	    I64ReinterpretF64 => Some(Instruction::I64ReinterpretF64),
	    F32ReinterpretI32 => Some(Instruction::F32ReinterpretI32),
	    F64ReinterpretI64 => Some(Instruction::F64ReinterpretI64),
    };
    // push compiled op.
    if let Some(op) = res {
      sink.emit(op);
    }
  }

  Ok(sink.into_inner())
}

