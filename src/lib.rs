use zed_extension_api::{self as zed, LanguageServerId, Result};

struct PackageSwiftLSPExtension {}

impl PackageSwiftLSPExtension {
    fn get_path_to_language_server_executable(&self) -> Result<String> {
        Ok("/Users/kattouf/Development/package-swift-lsp/.build/arm64-apple-macosx/debug/package-swift-lsp".to_string())
    }

    fn get_args_for_language_server(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

impl zed::Extension for PackageSwiftLSPExtension {
    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.get_path_to_language_server_executable()?,
            args: self.get_args_for_language_server()?,
            env: Default::default(),
        })
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }
}

zed::register_extension!(PackageSwiftLSPExtension);
