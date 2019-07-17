/// Options and flags which can be used to configure how a file is opened.
pub trait OpenOptions {
    /// Sets the option for read access.
    fn read(self, read: bool) -> Self;

    /// Sets the option for write access.
    fn write(self, write: bool) -> Self;

    /// Sets the option for the append mode.
    fn append(self, append: bool) -> Self;

    /// Sets the option for truncating a previous file.
    fn truncate(self, truncate: bool) -> Self;

    /// Sets the option for creating a new file.
    fn create(self, create: bool) -> Self;

    /// Sets the option to always create a new file.
    fn create_new(self, create_new: bool) -> Self;
}
