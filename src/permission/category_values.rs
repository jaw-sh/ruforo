use super::flag::Flag;

/// Data struct.
/// Yes, no, and never masks for a single Category.
#[derive(Clone, Copy, Debug, Default)]
pub struct CategoryValues {
    pub yes: u64,
    pub no: u64,
    pub never: u64,
}

impl From<CategoryValues> for u64 {
    fn from(item: CategoryValues) -> Self {
        item.yes & !(item.no | item.never)
    }
}

impl CategoryValues {
    /// Combines values laterally.
    /// Explicit YES permissions override explicit NO permissions.
    pub fn join(&self, left: &Self) -> Self {
        // Combine NEVER
        let never = self.never | left.never;
        // Combine NO, remove explicit YES
        let no = (self.no | left.no) & !(self.yes | left.yes);
        // Combine YES, remove NO and NEVER
        let yes = (self.yes | left.yes) & !(no | never);
        Self { yes, no, never }
    }

    /// Combines values vertically.
    /// NO permissions override YES permissions.
    pub fn stack(&self, below: &Self) -> Self {
        // Combine NEVER
        let never = self.never | below.never;
        // Replace NO
        let no = self.no;
        // Combine YES, remove NO and NEVER
        let yes = (self.yes | below.yes) & !(no | never);
        Self { yes, no, never }
    }

    pub fn set_flag(&mut self, item: &u8, flag: &Flag) {
        let bit: u64 = 1 << item; // 0b0001
        let not: u64 = !bit; // 0b1110

        match flag {
            Flag::YES => {
                self.yes |= bit;
                self.no &= not;
                self.never &= not;
            }
            Flag::DEFAULT => {
                self.yes &= not;
                self.no &= not;
                self.never &= not;
            }
            Flag::NO => {
                self.yes &= not;
                self.no |= bit;
                self.never &= not;
            }
            Flag::NEVER => {
                self.yes &= not;
                self.no &= not;
                self.never |= not;
            }
        }
    }
}
