fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=rnmd.rc");
        embed_resource::compile("./rnmd.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
