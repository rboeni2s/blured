use cargo_gpu_install::install::Install;
use std::path::PathBuf;


fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let shader_crate = PathBuf::from("./shaders");
    let backend = Install::from_shader_crate(shader_crate.clone()).run()?;

    let mut builder = backend.to_spirv_builder(shader_crate, "spirv-unknown-vulkan1.2");
    builder.build_script.defaults = true;
    builder.build_script.env_shader_spv_path = Some(true);
    let spv_result = builder.build()?;
    let spv_path = spv_result.module.unwrap_single();

    println!("cargo::rustc-env=BLURED_SHADER_PATH={}", spv_path.display());

    Ok(())
}
