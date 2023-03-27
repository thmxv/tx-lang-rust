use crate::{
    dyn_array::DynArray,
    opcodes::OpCode,
    value::Value,
    vm::{VmAlloc, VM},
};

pub struct LineStart {
    offset: usize,
    line: usize,
}

pub struct Chunk<'a> {
    pub bytecode: DynArray<'a, u8, VmAlloc>,
    pub constants: DynArray<'a, Value, VmAlloc>,
    pub lines: DynArray<'a, LineStart, VmAlloc>,
}

impl<'a> Chunk<'a> {
    fn write_line(&mut self, tvm: &mut VM, line: usize) {
        if let Some(last) = self.lines.last() {
            if last.line != line {
                unsafe {
                    self.lines.push(
                        &tvm.allocator,
                        LineStart {
                            offset: self.bytecode.len(),
                            line,
                        },
                    );
                }
            }
        }
    }

    pub fn get_line(&self, offset: usize) -> usize {
        let idx = match self
            .lines
            .binary_search_by(|line_start| line_start.offset.cmp(&offset))
        {
            Ok(idx) => idx,
            Err(idx) => idx - 1,
        };
        self.lines[idx].line
    }

    pub fn write_constant(&mut self, tvm: &mut VM, value: Value) -> usize {
        match self.constants.iter().position(|&val| val == value) {
            Some(idx) => idx,
            None => {
                unsafe {
                    self.constants.push(&tvm.allocator, value);
                }
                self.constants.len() - 1
            }
        }
    }

    fn write_byte(&mut self, tvm: &mut VM, byte: u8) {
        unsafe {
            self.bytecode.push(&tvm.allocator, byte);
        }
    }

    pub fn write_instruction<const N: usize>(
        &mut self,
        tvm: &mut VM,
        line: usize,
        opc: OpCode,
        operand: usize,
    ) {
        self.write_line(tvm, line);
        unsafe {
            self.bytecode.reserve(&tvm.allocator, 1 + N);
            self.bytecode.push(&tvm.allocator, opc.into());
        }
        let len = self.bytecode.len();
        unsafe {
            self.bytecode.resize(&tvm.allocator, len + N, 0xff);
        }
        write_multibyte_operand::<N>(
            &mut self.bytecode[len..len + N],
            operand,
        );
    }
}

fn write_multibyte_operand<const N: usize>(slice: &mut [u8], operand: usize) {
    debug_assert!(operand < (1 << (N * 8)));
    debug_assert_eq!(slice.len(), N);
    for i in 0..N {
        slice[i] = ((operand >> (i * 8)) & 0xff) as u8;
    }
}
