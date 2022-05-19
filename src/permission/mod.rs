mod test;

const GROUP_LIMIT: u32 = 16;
const PERM_LIMIT: u32 = u64::BITS;

pub enum PermissionError {
    CategoryOverflow,
}


#[derive(Clone, Copy, Default)]
/// A single permission in the bitmask.
pub struct Permission {
    id: i32,
    position: u8,
}

/// Identifier for a group of permissions.
/// Bitmasks are limited to 64 per group.
pub struct PermissionCategory {
    id: i32,
    position: u8,
    permissions: [Permission; PERM_LIMIT as usize],
}

impl Default for PermissionCategory {
    /// Bafflingly, Default is manually implemented for generic types up to a length of 32
    /// See: https://doc.rust-lang.org/std/default/trait.Default.html#impl-Default-71
    fn default() -> Self {
        PermissionCategory {
            id: 0,
            position: 0,
            permissions: [Permission::default(); PERM_LIMIT as usize]
        }
    }
}

impl PermissionCategory {
    fn add_permission(&mut self, id: i32) -> Result<&mut Permission, PermissionError> {
        let mut i: u8 = 0;

        for permission in self.permissions.iter_mut() {
            // Is a default permission
            if permission.id == 0 {
                *permission = Permission { id, position: i };
                return Ok(permission);
            }

            i += 1;
        }

        Err(PermissionError::CategoryOverflow)
    }
}

/// Per-value permission masks.
/// Organized this way for complex binary operations that intersect each other.
/// 24 byte struct
#[derive(Default)]
pub struct PermissionGroupValues {
    yes: u64,
    no: u64,
    never: u64,
}

impl PermissionGroupValues {
    /// Combines group values laterally.
    /// Explicit YES permissions override explicit NO permissions.
    fn join(&self, left: &PermissionGroupValues) -> PermissionGroupValues {
        // Combine NEVER
        let never = self.never | left.never;
        // Combine NO, remove explicit YES
        let no = (self.no | left.no) & !(self.yes | left.yes);
        // Combine YES, remove NO and NEVER
        let yes = (self.yes | left.yes) & !(no | never);
        PermissionGroupValues { yes, no, never, }
    }

    /// Combines group values vertically.
    /// NO permissions override YES permissions.
    fn stack(&self, below: &PermissionGroupValues) -> PermissionGroupValues {
        // Combine NEVER
        let never = self.never | below.never;
        // Replace NO
        let no = self.no;
        // Combine YES, remove NO and NEVER
        let yes = (self.yes | below.yes) & !(no | never);
        PermissionGroupValues { yes, no, never, }
    }
}

/// Permission mask made of bitmasks organized by permission group.
#[derive(Default)]
pub struct PermissionMask {
    groups: [u64; GROUP_LIMIT as usize]
}

impl PermissionMask {
    fn can(&self, group: usize, permission: i32) -> bool {
        self.groups[group] & (1 << permission) > 0
    }
}

/// Set of permission values organized by permission group.
#[derive(Default)]
pub struct PermissionSet {
    /// Group ID -> PermissionGroupValues
    groups: [PermissionGroupValues; GROUP_LIMIT as usize]
}

impl PermissionSet {
    /// Combines permission sets at the same depth.
    /// Explicit YES permissions override explicit NO permissions.
    fn join(&self, left: &PermissionSet) -> PermissionSet {
        let mut i: usize = 0;
        let mut groups: [PermissionGroupValues; GROUP_LIMIT as usize] = Default::default();

        for values in groups.iter_mut() {
            *values = self.groups[i].join(&left.groups[i]);
            i += 1;
        }

        PermissionSet { groups }
    }

    /// Combines permission sets vertically.
    /// No permissions override Yes permissions.
    fn stack(&self, below: &PermissionSet) -> PermissionSet {
        let mut i: usize = 0;
        let mut groups: [PermissionGroupValues; GROUP_LIMIT as usize] = Default::default();

        for values in groups.iter_mut() {
            *values = self.groups[i].stack(&below.groups[i]);
            i += 1;
        }

        PermissionSet { groups }
    }
}

/// A single Instruction output by the tokenizer.
pub enum PermissionValue {
    /// Grants permission
    YES = 1,
    /// Unset permission
    DEFAULT = 0,
    /// Removes any existing YES permission
    NO = -1,
    /// Never permitted, cannot be re-permitted
    NEVER = -2,
}

bitflags::bitflags! {
    /// 8 byte / 64 bit binary mask.
    #[derive(Default)]
    pub struct PermissionValues: u64 { }
}