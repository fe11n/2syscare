use std::ffi::OsString;
use std::path::PathBuf;

use common::util::ext_cmd::{ExternCommand, ExternCommandArgs, ExternCommandEnvs};

use crate::cli::{CliWorkDir, CliArguments};
use crate::package::RpmHelper;
use crate::patch::{PatchInfo, PatchBuilder, PatchBuilderArguments};

use super::kpatch_helper::{KernelPatchHelper, VMLINUX_FILE_NAME};
use super::kpatch_builder_args::KernelPatchBuilderArguments;

pub struct KernelPatchBuilder<'a> {
    workdir: &'a CliWorkDir
}

impl<'a> KernelPatchBuilder<'a> {
    pub fn new(workdir: &'a CliWorkDir) -> Self {
        Self { workdir }
    }
}

impl KernelPatchBuilder<'_> {
    fn parse_cmd_args(&self, args: &KernelPatchBuilderArguments) -> ExternCommandArgs {
        let mut cmd_args = ExternCommandArgs::new()
            .arg("--name")
            .arg(&args.patch_name)
            .arg("--sourcedir")
            .arg(&args.source_dir)
            .arg("--config")
            .arg(&args.config)
            .arg("--vmlinux")
            .arg(&args.vmlinux)
            .arg("--jobs")
            .arg(args.jobs.to_string())
            .arg("--output")
            .arg(&args.output_dir)
            .arg("--skip-cleanup");

        if args.skip_compiler_check {
            cmd_args = cmd_args.arg("--skip-compiler-check");
        }
        cmd_args = cmd_args.args(args.patch_list.iter().map(|patch| &patch.path));

        cmd_args
    }

    fn parse_cmd_envs(&self, args: &KernelPatchBuilderArguments) -> ExternCommandEnvs {
        ExternCommandEnvs::new()
            .env("CACHEDIR",           &args.build_root)
            .env("NO_PROFILING_CALLS", &args.build_root)
            .env("DISABLE_AFTER_LOAD", &args.build_root)
            .env("KEEP_JUMP_LABEL",    &args.build_root)
    }
}

impl PatchBuilder for KernelPatchBuilder<'_> {
    fn parse_builder_args(&self, patch_info: &PatchInfo, args: &CliArguments) -> std::io::Result<PatchBuilderArguments> {
        let patch_build_root = self.workdir.patch.build.as_path();
        let patch_output_dir = self.workdir.patch.output.as_path();

        let source_pkg_dir = self.workdir.package.source.as_path();
        let debug_pkg_dir  = self.workdir.package.debug.as_path();

        let source_pkg_build_root = RpmHelper::find_build_root(source_pkg_dir)?;
        let source_pkg_build_dir  = source_pkg_build_root.build.as_path();
        let kernel_source_dir = RpmHelper::find_build_source(source_pkg_build_dir, patch_info)?;

        KernelPatchHelper::generate_defconfig(&kernel_source_dir)?;
        let kernel_config_file = KernelPatchHelper::find_kernel_config(&kernel_source_dir)?;
        let vmlinux_file = KernelPatchHelper::find_vmlinux(debug_pkg_dir)?;

        let builder_args = KernelPatchBuilderArguments {
            build_root:          patch_build_root.to_owned(),
            patch_name:          patch_info.name.to_owned(),
            source_dir:          kernel_source_dir,
            config:              kernel_config_file,
            vmlinux:             vmlinux_file,
            jobs:                args.kjobs,
            output_dir:          patch_output_dir.to_owned(),
            skip_compiler_check: args.skip_compiler_check,
            patch_list:          patch_info.patches.to_owned(),
        };

        Ok(PatchBuilderArguments::KernelPatch(builder_args))
    }

    fn build_patch(&self, args: &PatchBuilderArguments) -> std::io::Result<()> {
        const KPATCH_BUILD: ExternCommand = ExternCommand::new("kpatch-build");

        match args {
            PatchBuilderArguments::KernelPatch(kargs) => {
                KPATCH_BUILD.execve(
                    self.parse_cmd_args(kargs),
                    self.parse_cmd_envs(kargs)
                )?.check_exit_code()
            },
            PatchBuilderArguments::UserPatch(_) => unreachable!(),
        }
    }

    fn write_patch_info(&self, patch_info: &mut PatchInfo, _: &PatchBuilderArguments) -> std::io::Result<()> {
        /*
         * Kernel patch does not use target_elf for patch operation,
         * so we just add it for display purpose.
         */
        patch_info.target_elfs.insert(
            OsString::from(VMLINUX_FILE_NAME),
            PathBuf::from(VMLINUX_FILE_NAME),
        );

        Ok(())
    }
}
