pub fn align_up(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + alignment - remainder
    }
}

pub fn align_down(value: usize, alignment: usize) -> usize {
    let multiples = value / alignment;
    multiples * alignment
}
