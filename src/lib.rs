use std::fs;
use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result};

// Constants for the Package.swift LSP
const PACKAGE_SWIFT_LSP_GITHUB_REPO: &str = "kattouf/package-swift-lsp";
const EXECUTABLE_NAME: &str = "package-swift-lsp";

struct PackageSwiftBinary {
    path: String,
    args: Option<Vec<String>>,
}

struct PackageSwiftLSPExtension {
    cached_binary_path: Option<String>,
}

impl PackageSwiftLSPExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<PackageSwiftBinary> {
        let binary_settings = LspSettings::for_worktree("package-swift-lsp", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        // If the user has specified a custom path to the language server binary, use that
        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(PackageSwiftBinary {
                path,
                args: binary_args,
            });
        }

        // If the binary is available in PATH, use that
        if let Some(path) = worktree.which(EXECUTABLE_NAME) {
            return Ok(PackageSwiftBinary {
                path,
                args: binary_args,
            });
        }

        // If we've already downloaded the binary, use that
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(PackageSwiftBinary {
                    path: path.clone(),
                    args: binary_args,
                });
            }
        }

        // Download the binary from GitHub releases
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            PACKAGE_SWIFT_LSP_GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        // Asset name pattern should match your GitHub release assets
        // package-swift-lsp-1.0.0-arm64-apple-macosx.zip
        // package-swift-lsp-1.0.0-x86_64-apple-macosx.zip
        let asset_name = format!(
            "{}-{}-{}-{}.zip",
            EXECUTABLE_NAME,
            release.version,
            match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            match platform {
                zed::Os::Mac => "apple-macosx",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("{}-{}", EXECUTABLE_NAME, release.version);
        fs::create_dir_all(&version_dir)
            .map_err(|err| format!("failed to create directory '{version_dir}': {err}"))?;

        // We've already confirmed this is macOS, so no need to check again
        let binary_path = format!("{version_dir}/{bin_name}", bin_name = EXECUTABLE_NAME);

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            // Adjust this based on how your binary is packaged
            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|err| format!("failed to download file: {err}"))?;

            zed::make_file_executable(&binary_path)?;

            // Clean up old versions
            let entries = fs::read_dir(".")
                .map_err(|err| format!("failed to list working directory {err}"))?;
            for entry in entries {
                let entry = entry.map_err(|err| format!("failed to load directory entry {err}"))?;
                if entry.file_name().to_str() != Some(&version_dir)
                    && entry.file_name().to_str().map_or(false, |name| {
                        name.starts_with(&format!("{}-", EXECUTABLE_NAME))
                    })
                {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(PackageSwiftBinary {
            path: binary_path,
            args: binary_args,
        })
    }
}

impl zed::Extension for PackageSwiftLSPExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Check platform compatibility early
        let (platform, arch) = zed::current_platform();
        if platform != zed::Os::Mac {
            return Err(format!(
                "Package.swift LSP is currently only supported on macOS, not on {:?}",
                platform
            )
            .into());
        }
        if arch != zed::Architecture::Aarch64 && arch != zed::Architecture::X8664 {
            return Err(format!("Package.swift LSP is currently only supported on x86_64 and arm64 architectures, not on {:?}", arch).into());
        }

        let binary = self.language_server_binary(language_server_id, worktree)?;
        Ok(zed::Command {
            command: binary.path,
            args: binary.args.unwrap_or_else(|| vec![]),
            env: Default::default(),
        })
    }
}

zed::register_extension!(PackageSwiftLSPExtension);
