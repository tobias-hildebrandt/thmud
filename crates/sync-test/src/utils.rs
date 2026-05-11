use std::ops::Mul;

pub(crate) fn square<T: Copy + Mul<Output = T>>(num: T) -> T {
    num * num
}
