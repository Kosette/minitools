fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=hasher.rc");
        embed_resource::compile("./hasher.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
