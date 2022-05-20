/// Data struct.
/// Yes, no, and never masks for a single Category.
#[derive(Default)]
pub struct CategoryValues {
    pub yes: u64,
    pub no: u64,
    pub never: u64,
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
}
