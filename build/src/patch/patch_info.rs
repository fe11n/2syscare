use std::path::Path;

use crate::statics::*;
use crate::util::fs;

use crate::cli::CliArguments;

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
pub struct PatchName {
    name:    String,
    version: String,
    release: String,
}

impl PatchName {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_release(&self) -> &str {
        &self.release
    }
}

impl std::fmt::Display for PatchName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}-{}-{}", self.get_name(), self.get_version(), self.get_release()))
    }
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PatchType {
    UserPatch,
    KernelPatch,
}

impl std::fmt::Display for PatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub struct PatchFile {
    name: String,
    path: String,
    hash: String,
}

impl PatchFile {
    pub fn new<P: AsRef<Path>>(file: P) -> std::io::Result<Self> {
        fs::check_file(&file)?;

        let file_path = file.as_ref().canonicalize()?;
        let name = fs::stringtify_path(file_path.file_name().expect("Get patch name failed"));
        let path = fs::stringtify_path(file_path.as_path());
        let hash = fs::sha256_digest_file(file_path)?[..PATCH_VERSION_DIGITS].to_owned();

        Ok(Self { name, path, hash })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn set_path(&mut self, value: String) {
        self.path = value;
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn set_hash(&mut self, value: String) {
        self.hash = value
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub struct PatchInfo {
    patch_name:  PatchName,
    patch_type:  PatchType,
    target_name: Option<PatchName>,
    license:     String,
    summary:     String,
    file_list:   Vec<PatchFile>,
}

impl PatchInfo {
    pub fn get_patch_name(&self) -> &PatchName {
        &self.patch_name
    }

    pub fn get_patch_type(&self) -> PatchType {
        self.patch_type
    }

    pub fn get_target_name(&self) -> Option<&PatchName> {
        self.target_name.as_ref()
    }

    pub fn get_file_list(&self) -> &[PatchFile] {
        self.file_list.as_slice()
    }

    pub fn get_license(&self) -> &str {
        &self.license
    }

    pub fn get_summary(&self) -> &str {
        &self.summary
    }
}

impl std::fmt::Display for PatchInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let patch_target = self.get_target_name()
            .map(PatchName::to_string)
            .unwrap_or(PATCH_UNDEFINED_VALUE.to_string());

        f.write_fmt(format_args!("{}\n\n",        self.get_summary()))?;
        f.write_fmt(format_args!("name:    {}\n", self.get_patch_name().get_name()))?;
        f.write_fmt(format_args!("type:    {}\n", self.get_patch_type()))?;
        f.write_fmt(format_args!("version: {}\n", self.get_patch_name().get_version()))?;
        f.write_fmt(format_args!("release: {}\n", self.get_patch_name().get_release()))?;
        f.write_fmt(format_args!("license: {}\n", self.get_license()))?;
        f.write_fmt(format_args!("target:  {}\n", patch_target))?;
        f.write_str("\npatch list:")?;
        for patch_file in self.get_file_list() {
            f.write_fmt(format_args!("\n{} {}", patch_file.get_name(), patch_file.get_hash()))?;
        }

        Ok(())
    }
}

impl TryFrom<&CliArguments> for PatchInfo {
    type Error = std::io::Error;

    fn try_from(args: &CliArguments) -> Result<Self, Self::Error> {
        #[inline(always)]
        fn parse_patch_name(args: &CliArguments) -> std::io::Result<PatchName> {
            Ok(PatchName {
                name:    args.patch_name.to_owned(),
                version: args.patch_version.to_owned(),
                release: fs::sha256_digest_file_list(&args.patches)?[..PATCH_VERSION_DIGITS].to_string(),
            })
        }

        #[inline(always)]
        fn parse_patch_type(args: &CliArguments) -> PatchType {
            let find_result = fs::find_file(
                args.source.to_string(),
                KERNEL_SOURCE_DIR_FLAG,
                false,
                false,
            );

            match find_result.is_ok() {
                true  => PatchType::KernelPatch,
                false => PatchType::UserPatch,
            }
        }

        #[inline(always)]
        fn parse_target_name(args: &CliArguments) -> Option<PatchName> {
            match (args.target_name.clone(), args.target_version.clone(), args.target_release.clone()) {
                (Some(name), Some(version), Some(release)) => {
                    Some(PatchName { name, version, release })
                },
                _ => None
            }
        }

        #[inline(always)]
        fn parse_license(args: &CliArguments) -> String {
            let license: Option<&str> = args.target_license.as_deref();
            license.unwrap_or(PATCH_UNDEFINED_VALUE).to_owned()
        }

        #[inline(always)]
        fn parse_summary(args: &CliArguments) -> String {
            args.patch_summary.to_owned()
        }

        #[inline(always)]
        fn parse_file_list(args: &CliArguments) -> std::io::Result<Vec<PatchFile>> {
            let mut file_list = Vec::new();
            for file in &args.patches {
                file_list.push(PatchFile::new(file)?);
            }

            Ok(file_list)
        }

        Ok(PatchInfo {
            patch_name:  parse_patch_name(args)?,
            patch_type:  parse_patch_type(args),
            target_name: parse_target_name(args),
            license:     parse_license(args),
            summary:     parse_summary(args),
            file_list:   parse_file_list(args)?
        })
    }
}
