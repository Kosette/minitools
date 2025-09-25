fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=oxipng-manifest.rc");
        embed_resource::compile("./oxipng-manifest.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
