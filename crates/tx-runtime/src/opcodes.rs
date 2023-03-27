#[macro_export]
#[doc(hidden)]
macro_rules! opcodes {
    ( $(($opc:ident, $num_operands:expr, $stack_effect:expr),)* ) => {
        $crate::__opcodes!((0) $($opc,)*);
        $crate::__operands!($($num_operands),*);
        $crate::__stack_effects!($($stack_effect),*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __opcodes {
    ( ($start:expr) $opc:ident, $($rest:tt)* ) => {
        pub const $opc: OpCode = OpCode($start);
        $crate::__opcodes!(($start + 1) $($rest)*);
    };

    ( ($start:expr) ) => {}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __operands {
    ( $($num_operands:expr),* ) => {
        const OPCODE_NUM_OPERANDS: &'static [usize] = &[$($num_operands),*];
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __stack_effects {
    ( $($stack_effect:expr),* ) => {
        const OPCODE_STACK_EFFECT: &'static [isize] = &[$($stack_effect),*];
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct OpCode(u8);

impl OpCode {
    pub const fn get_num_operands(&self) -> usize {
        OPCODE_NUM_OPERANDS[self.0 as usize]
    }

    pub const fn get_stack_effect(&self) -> isize {
        OPCODE_STACK_EFFECT[self.0 as usize]
    }
}

impl From<OpCode> for u8 {
    fn from(opc: OpCode) -> Self {
        opc.0
    }
}

opcodes! {
    (CONSTANT,             1, 1),
    (CONSTANT_LONG,        3, 1),
    (NIL,                  0, 1),
    (TRUE,                 0, 1),
    (FALSE,                0, 1),
    (POP,                  0, -1),
    (GET_LOCAL,            1, 1),
    (GET_LOCAL_LONG,       3, 1),
    (SET_LOCAL,            1, 0),
    (SET_LOCAL_LONG,       3, 0),
    (GET_GLOBAL,           1, 1),
    (GET_GLOBAL_LONG,      3, 1),
    (SET_GLOBAL,           1, 0),
    (SET_GLOBAL_LONG,      3, 0),
    (DEFINE_GLOBAL,        1, -1),
    (DEFINE_GLOBAL_LONG,   3, -1),
    (GET_UPVALUE,          1, 1),
    (GET_UPVALUE_LONG,     3, 1),
    (SET_UPVALUE,          1, 0),
    (SET_UPVALUE_LONG,     3, 0),
    (EQUAL,                0, -1),
    (NOT_EQUAL,            0, -1),
    (GREATER,              0, -1),
    (GREATER_EQUAL,        0, -1),
    (LESS,                 0, -1),
    (LESS_EQUAL,           0, -1),
    (ADD,                  0, -1),
    (SUBSTRACT,            0, -1),
    (MULTIPLY,             0, -1),
    (DIVIDE,               0, -1),
    (NOT,                  0, 0),
    (NEGATE,               0, 0),
    (JUMP,                 2, 0),
    (JUMP_IF_FALSE,        2, 0),
    (LOOP,                 2, 0),
    (CALL,                 1, 0), // 0 for tx fn's but for native fn it is the same as RETURN
    (CLOSURE,              1, 1),
    (CLOSURE_LONG,         3, 1),
    (END_SCOPE,            1, 0), // Stack effect is in the operand
    (END_SCOPE_LONG,       3, 0), // Stack effect is in the operand
    (RETURN,               0, 0), // Stack effect is variable
    // (END,                  0, 0),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values() {
        assert_eq!(CONSTANT, OpCode(0));
    }

    #[test]
    fn test_num_operands() {
        assert_eq!(CONSTANT.get_num_operands(), 1);
        assert_eq!(CONSTANT_LONG.get_num_operands(), 3);
        assert_eq!(POP.get_num_operands(), 0);
    }

    #[test]
    fn test_stack_effect() {
        assert_eq!(CONSTANT.get_stack_effect(), 1);
        assert_eq!(CONSTANT_LONG.get_stack_effect(), 1);
        assert_eq!(POP.get_stack_effect(), -1);
    }
}
