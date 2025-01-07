#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvxs_version() {
        let version = get_pvxs_version();
        println!("PVXS Version: {}", version);
        assert!(!version.is_empty(), "Version string should not be empty");
    }
}
