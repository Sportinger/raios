fn main() {
    println!("cargo:rerun-if-env-changed=RAISOS_DEFAULT_OPENAI_API_KEY");
    println!("cargo:rerun-if-env-changed=RAISOS_OPENAI_CERT_SHA256");
    println!("cargo:rerun-if-env-changed=RAISOS_ALLOW_UNVERIFIED_OPENAI_TLS");
}
