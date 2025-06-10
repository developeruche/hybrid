fn main() {
    // Tell Cargo to re-run this build script if the contracts directory changes
    println!("cargo:rerun-if-changed=../../contracts");
}