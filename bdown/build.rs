fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=bdown.rc");
        embed_resource::compile("./bdown.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
