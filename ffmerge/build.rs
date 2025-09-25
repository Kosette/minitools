fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=ffmerge.rc");
        embed_resource::compile("./ffmerge.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
