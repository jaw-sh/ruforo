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

mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_mask_can()
    {
        let mut mask: PermissionMask = Default::default();
        mask.groups[0] = 0b0101u64;
        
        assert_eq!(mask.can(0, 0), true);
        assert_eq!(mask.can(0, 1), false);
        assert_eq!(mask.can(0, 2), true);
        assert_eq!(mask.can(0, 3), false);
        assert_eq!(mask.can(0, 4), false);
        
        assert_eq!(mask.can(1, 0), false);
        assert_eq!(mask.can(2, 1), false);
        assert_eq!(mask.can(3, 2), false);
        assert_eq!(mask.can(4, 3), false);
    }

    #[test]
    fn test_permission_add()
    {
        let mut cat = PermissionCategory::default();

        for i in 0..PERM_LIMIT {
            match cat.add_permission(rand::thread_rng().gen_range(1..999) as i32) {
                Ok(p) => assert_eq!(p.position as u32, i),
                Err(_) => assert!(false, "Unexpected overflowing permission category"),
            }
            
        }

        let pr = cat.add_permission((PERM_LIMIT + 1) as i32);
        assert!(pr.is_err(), "Did not contain an error.");
    }

    #[test]
    fn test_set_join()
    {
        let group1a = PermissionGroupValues {
            yes:   0b0011u64,
            no:    0b0000u64,
            never: 0b0000u64,
        };
        let group1b = PermissionGroupValues {
            yes:   0b0000u64,
            no:    0b1000u64,
            never: 0b0100u64,
        };
        let group2a = PermissionGroupValues {
            yes:   0b0000u64,
            no:    0b0010u64,
            never: 0b0001u64,
        };
        let group2b = PermissionGroupValues {
            yes:   0b1100u64,
            no:    0b0000u64,
            never: 0b0000u64,
        };

        let mut set1: PermissionSet = Default::default();
        set1.groups[0] = group1a;
        set1.groups[1] = group1b;

        let mut set2: PermissionSet = Default::default();
        set2.groups[0] = group2a;
        set2.groups[1] = group2b;

        let set3 = set1.join(&set2);

        assert_eq!(set1.groups[0].yes, 0b0011u64);
        assert_eq!(set2.groups[1].yes, 0b1100u64);
        assert_eq!(set3.groups[0].yes, 0b0010u64);
        assert_eq!(set3.groups[1].yes, 0b1000u64);
    }

    #[test]
    fn test_values_join_combines_never()
    {
        let group1 = PermissionGroupValues {
            yes:   0b100u64,
            no:    0b000u64,
            never: 0b010u64,
        };
        let group2 = PermissionGroupValues {
            yes:   0b011u64,
            no:    0b000u64,
            never: 0b100u64,
        };
        let group3 = group1.join(&group2);

        assert_eq!(group3.yes,   0b0001u64);
        assert_eq!(group3.no,    0b0000u64);
        assert_eq!(group3.never, 0b0110u64);
    }

    #[test]
    fn test_values_join_overwrites_no()
    {
        let group1 = PermissionGroupValues {
            yes:   0b111u64,
            no:    0b000u64,
            never: 0b000u64,
        };
        let group2 = PermissionGroupValues {
            yes:   0b0000u64,
            no:    0b0010u64,
            never: 0b1000u64,
        };
        let group3 = group1.join(&group2);

        assert_eq!(group3.yes,   0b0111u64);
        assert_eq!(group3.no,    0b0000u64);
        assert_eq!(group3.never, 0b1000u64);
    }

    #[test]
    fn test_values_stack_negatives()
    {
        let group1 = PermissionGroupValues {
            yes:   0b01100u64,
            no:    0b00010u64,
            never: 0b00001u64,
        };
        let group2 = PermissionGroupValues {
            yes:   0b10011u64,
            no:    0b01111u64,
            never: 0b01000u64,
        };
        let group3 = group1.stack(&group2);

        assert_eq!(group3.yes,   0b10100u64);
        assert_eq!(group3.no,    0b00010u64);
        assert_eq!(group3.never, 0b01001u64);
    }
}