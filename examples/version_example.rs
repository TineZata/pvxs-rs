fn main() {
    let version: String = pvxs::get_version_str();
    println!("PVXS Version: {}", version);

    let version_int: u32 = pvxs::get_version_int();
    println!("PVXS Version: {}", version_int);

    let version_abi_int: u32 = pvxs::get_version_abi_int();
    println!("PVXS ABI Version: {}", version_abi_int);
}
