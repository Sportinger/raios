fn main() {
    println!("cargo:rerun-if-env-changed=SEEDOS_DEFAULT_OPENAI_API_KEY");
}
