fn main() {
    println!("cargo:rerun-if-env-changed=RAIOS_DEFAULT_OPENAI_API_KEY");
    println!("cargo:rerun-if-env-changed=RAIOS_OPENAI_CERT_SHA256");
    println!("cargo:rerun-if-env-changed=RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");
}
