use super::GROUP_LIMIT;

/// Data struct containing all permission categories as final, evaluated masks.
#[derive(Default)]
pub struct Mask {
    pub groups: [u64; GROUP_LIMIT as usize]
}

impl Mask {
    pub fn can(&self, group: usize, permission: i32) -> bool {
        self.groups[group] & (1 << permission) > 0
    }
}