pub struct Permission {
    id: i32,
    pos: u8,
}

const GROUP_LIMIT: usize = 16;

// Identifier for a group of permissions.
// Bitmasks are limited to 64 per group.
pub enum PermissionGroup {
    ACCOUNT,
    FORUM,
}

// Per-value permission masks.
// Organized this way for complex binary operations that intersect each other.
// 24 byte struct
#[derive(Default)]
pub struct PermissionGroupValues {
    yes: u64,
    no: u64,
    never: u64,
}

impl PermissionGroupValues {
    // Combines group values laterally.
    // Explicit YES permissions override explicit NO permissions.
    fn join(&self, left: &PermissionGroupValues) -> PermissionGroupValues {
        // Combine NEVER
        let never = self.never | left.never;
        // Combine NO, remove explicit YES
        let no = (self.no | left.no) & !(self.yes | left.yes);
        // Combine YES, remove NO and NEVER
        let yes = (self.yes | left.yes) & !(no | never);
        PermissionGroupValues { yes, no, never, }
    }
}

bitflags::bitflags! {
    // 8 byte / 64 bit binary mask.
    #[derive(Default)]
    pub struct PermissionValues: u64 { }
}

// Set of permission values organized by permission group.
pub struct PermissionSet {
    // Group ID -> PermissionGroupValues
    groups: [PermissionGroupValues; GROUP_LIMIT]
}

impl PermissionSet {
    // vertical join (NO overrides YES)
    // sideways join (YES overrides NO)

    // Combines permission sets at the same depth.
    // Explicit YES permissions override explicit NO permissions.
    fn join(self, left: PermissionSet) -> PermissionSet {
        let mut i: usize = 0;
        let mut groups: [PermissionGroupValues; GROUP_LIMIT] = Default::default();

        for values in groups.iter_mut() {
            *values = self.groups[i].join(&left.groups[i]);
            i += 1;
        }

        PermissionSet { groups }
    }
}

// A single Instruction output by the tokenizer.
pub enum PermissionValue {
    YES = 1,     // Grants permission
    DEFAULT = 0, // Unset permission
    NO = -1,     // Removes any existing YES permission
    NEVER = -2,  // Never permitted, cannot be re-permitted.
}

mod test {
    use super::*;

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

        assert_eq!(group3.yes, 0b111u64);
        assert_eq!(group3.no, 0b0u64);
        assert_eq!(group3.never, 0b1000u64);
    }
}