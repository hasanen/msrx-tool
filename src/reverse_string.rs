pub trait ReverseString {
    fn reverse(&self) -> String;
}
impl<T: AsRef<str>> ReverseString for T {
    fn reverse(&self) -> String {
        self.as_ref().chars().rev().collect::<String>()
    }
}
