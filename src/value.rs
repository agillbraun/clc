use float_cmp::approx_eq;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Display;
use std::ops;

#[rustfmt::skip]
macro_rules! do_cast {
  ($v: expr, $w: expr, $t: tt) => {
    match $w {
      U64 => ($v as u64) as $t, U32 => ($v as u32) as $t, U16 => ($v as u16) as $t, U8 => ($v as u8) as $t,
      I64 => ($v as i64) as $t, I32 => ($v as i32) as $t, I16 => ($v as i16) as $t, I8 => ($v as i8) as $t,
    }
  };
}

#[rustfmt::skip]
macro_rules! unary_op {
  ($v: expr, $w: expr, $op: tt, $t: tt) => {
    match $w {
      U64 => ($op($v as u64)) as $t, U32 => ($op($v as u32)) as $t, U16 => ($op($v as u16)) as $t, U8 => ($op($v as u8)) as $t,
      I64 => ($op($v as i64)) as $t, I32 => ($op($v as i32)) as $t, I16 => ($op($v as i16)) as $t, I8 => ($op($v as i8)) as $t,
    }
  };
}

#[rustfmt::skip]
macro_rules! unary_op_signed {
  ($v: expr, $w: expr, $op: tt, $t: tt) => {
    match $w {
      U64 => ($op($v as i64)) as $t, U32 => ($op($v as i32)) as $t, U16 => ($op($v as i16)) as $t, U8 => ($op($v as i8)) as $t,
      I64 => ($op($v as i64)) as $t, I32 => ($op($v as i32)) as $t, I16 => ($op($v as i16)) as $t, I8 => ($op($v as i8)) as $t,
    }
  };
}

macro_rules! binary_op {
  ($v1: expr, $v2: expr, $w: expr, $op: tt, $t: tt) => {
    match $w {
      U64 => (($v1 as u64) $op ($v2 as u64)) as $t, U32 => (($v1 as u32) $op ($v2 as u32)) as $t,
      U16 => (($v1 as u16) $op ($v2 as u16)) as $t, U8 => (($v1 as u8) $op ($v2 as u8)) as $t,

      I64 => (($v1 as i64) $op ($v2 as i64)) as $t, I32 => (($v1 as i32) $op ($v2 as i32)) as $t,
      I16 => (($v1 as i16) $op ($v2 as i16)) as $t, I8 => (($v1 as i8) $op ($v2 as i8)) as $t,
    }
  }
}

#[rustfmt::skip]
macro_rules! cmp {
  ($v1: expr, $v2: expr, $w: expr) => {
    match $w {
      U64 => (($v1 as u64).cmp(&($v2 as u64))), U32 => (($v1 as u32).cmp(&($v2 as u32))), 
      U16 => (($v1 as u16).cmp(&($v2 as u16))), U8 => (($v1 as u8).cmp(&($v2 as u8))),

      I64 => (($v1 as i64).cmp(&($v2 as i64))), I32 => (($v1 as i32).cmp(&($v2 as i32))),
      I16 => (($v1 as i16).cmp(&($v2 as i16))), I8 => (($v1 as i8).cmp(&($v2 as i8))),
    }
  };
}

#[rustfmt::skip]
macro_rules! value_fmt {
  ($v: expr, $w: expr) => {
    match $w {
      U64 => format!("{}", $v as u64), U32 => format!("{}", $v as u32), 
      U16 => format!("{}", $v as u16), U8 => format!("{}", $v as u8),

      I64 => format!("{}", $v as i64), I32 => format!("{}", $v as i32),
      I16 => format!("{}", $v as i16), I8 => format!("{}", $v as i8),
    }
  };
}

macro_rules! unary_int_op {
  ($v: expr, $w: expr, $op: tt) => {
    unary_op!($v, $w, $op, u64)
  };
}
macro_rules! unary_signed_int_op {
  ($v: expr, $w: expr, $op: tt) => {
    unary_op_signed!($v, $w, $op, u64)
  };
}
macro_rules! binary_int_op {
  ($v1: expr, $v2: expr, $w: expr, $op: tt) => {
    binary_op!($v1, $v2, $w, $op, u64)
  };
}

macro_rules! impl_arithmetic_op {
  ($ops: tt, $func: tt, $op: tt) => {
    impl ops::$ops<Value> for Value {
      type Output = Value;
      fn $func(self, rhs: Value) -> Value {
        use Value::*;
        use Width::*;
        return match self {
          Integer(v1, w) => {
            match rhs {
              Integer(v2, _) => { Integer(binary_int_op!(v1, v2, w, $op), w) },
              Float(v2) => { Integer(binary_int_op!(v1, v2, w, $op), w) },
            }
          },
          Float(v1) => {
            match rhs {
              Integer(v2, _) => { Float(v1 $op (v2 as f64)) },
              Float(v2) => { Float(v1 $op v2) }
            }
          }
        }
      }
    }
  }
}

macro_rules! impl_bitwise_op {
  ($ops: tt, $func: tt, $op: tt) => {
    impl ops::$ops<Value> for Value {
      type Output = Value;
      fn $func(self, rhs: Value) -> Value {
        use Value::*;
        use Width::*;
        return match self {
          Integer(v1, w) => {
            match rhs {
              Integer(v2, _) => { Integer(binary_int_op!(v1, v2, w, $op), w) },
              Float(v2) => { Integer(binary_int_op!(v1, (v2 as u64), w, $op), w) },
            }
          },
          Float(v1) => {
            match rhs {
              Integer(v2, w) => { Integer(binary_int_op!((v1 as u64), (v2 as u64), w, $op), Width::U64) },
              Float(v2) => { Integer((v1 as u64) $op (v2 as u64), Width::U64) },
            }
          }
        }
      }
    }
  }
}

macro_rules! impl_from {
  ($t: tt) => {
    impl From<$t> for Value {
      fn from(v: $t) -> Self {
        return Value::Float(v as f64);
      }
    }
  };
  ($t: tt, $w: expr) => {
    impl From<$t> for Value {
      fn from(v: $t) -> Self {
        return Value::Integer(v as u64, $w);
      }
    }
  };
}

macro_rules! impl_from_value {
  ($t: tt) => {
    impl From<Value> for $t {
      fn from(src: Value) -> $t {
        use Value::*;
        return match src {
          Integer(v, _) => v as $t,
          Float(v) => v as $t,
        };
      }
    }
  };
}

//
//
//

#[derive(Debug, Clone, Copy)]
pub enum Width {
  U64,
  U32,
  U16,
  U8,

  I64,
  I32,
  I16,
  I8,
}

#[derive(Debug, Copy, Clone)]
pub enum Value {
  Integer(u64, Width),
  Float(f64),
}

impl_arithmetic_op!(Add, add, +);
impl_arithmetic_op!(Sub, sub, -);
impl_arithmetic_op!(Mul, mul, *);
impl_arithmetic_op!(Div, div, /);
impl_arithmetic_op!(Rem, rem, %);

impl_bitwise_op!(BitAnd, bitand, &);
impl_bitwise_op!(BitOr, bitor, |);
impl_bitwise_op!(BitXor, bitxor, ^);
impl_bitwise_op!(Shl, shl, <<);
impl_bitwise_op!(Shr, shr, >>);

impl ops::Neg for Value {
  type Output = Value;

  fn neg(self) -> Value {
    use Value::*;
    use Width::*;
    return match self {
      Integer(v, w) => Integer(unary_signed_int_op!(v, w, -), w),
      Float(v) => Float(-v),
    };
  }
}

impl ops::Not for Value {
  type Output = Value;

  fn not(self) -> Value {
    use Value::*;
    use Width::*;
    return match self {
      Integer(v, w) => Integer(unary_int_op!(v, w, !), w),
      Float(v) => Integer((v != 0f64) as u64, Width::U8),
    };
  }
}

// Comparison traits
impl Eq for Value {}

impl Ord for Value {
  fn cmp(&self, other: &Self) -> Ordering {
    use Value::*;
    use Width::*;
    return match self {
      Integer(v1, w) => match other {
        Integer(v2, _) => cmp!(*v1, *v2, w),
        Float(v2) => cmp!(*v1, *v2, w),
      },
      Float(v1) => match other {
        Integer(v2, w) => {
          let vf = do_cast!(*v2, w, f64);
          if approx_eq!(f64, *v1, vf) {
            Ordering::Equal
          } else if v1 < &vf {
            Ordering::Less
          } else {
            Ordering::Greater
          }
        }
        Float(v2) => {
          if approx_eq!(f64, *v1, *v2) {
            Ordering::Equal
          } else if v1 < &v2 {
            Ordering::Less
          } else {
            Ordering::Greater
          }
        }
      },
    };
  }
}

impl PartialEq<Value> for Value {
  fn eq(&self, other: &Value) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl PartialOrd<Value> for Value {
  fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

// `From` traits
impl_from!(u64, Width::U64);
impl_from!(u32, Width::U32);
impl_from!(u16, Width::U16);
impl_from!(u8, Width::U8);

impl_from!(i64, Width::I64);
impl_from!(i32, Width::I32);
impl_from!(i16, Width::I16);
impl_from!(i8, Width::I8);

impl_from!(bool, Width::U8);
impl_from!(f64);

impl_from_value!(u64);
impl_from_value!(u32);
impl_from_value!(u16);
impl_from_value!(u8);

impl_from_value!(i64);
impl_from_value!(i32);
impl_from_value!(i16);
impl_from_value!(i8);

impl_from_value!(f64);

impl From<Value> for bool {
  fn from(src: Value) -> bool {
    use Value::*;
    return match src {
      Integer(v, _) => v != 0,
      Float(v) => v != 0f64,
    };
  }
}

impl Value {
  pub fn to_signed(&self) -> Value {
    use Value::*;
    use Width::*;
    match self {
      Integer(v, w) => {
        let new_width = match w {
          U64 => I64,
          U32 => I64,
          U16 => I16,
          U8 => I8,
          _ => *w,
        };

        Integer(*v, new_width)
      }
      Float(v) => Float(*v),
    }
  }
}

//

impl Width {
  pub fn as_string(&self) -> &str {
    use Width::*;
    match self {
      U64 => "u64",
      U32 => "u32",
      U16 => "u16",
      U8 => "u8",

      I64 => "i64",
      I32 => "i32",
      I16 => "i16",
      I8 => "i8",
    }
  }
}

impl Display for Width {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.as_string())
  }
}

impl Value {
  pub fn as_string(&self) -> String {
    use Value::*;
    use Width::*;
    match self {
      Integer(v, w) => value_fmt!(*v, w),
      Float(v) => format!("{}", v),
    }
  }

  pub fn as_typed_string(&self) -> String {
    use Value::*;
    match self {
      Integer(v, w) => format!("{} {}", w.as_string(), self.as_string()),
      Float(v) => format!("f64 {}", v),
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.as_string())
  }
}
