use zed_extension_api::{
    self as zed, Architecture, Command, DownloadedFileType, GithubReleaseAsset,
    GithubReleaseOptions, LanguageServerId, LanguageServerInstallationStatus, Os, Result, Worktree,
    current_platform, download_file, latest_github_release, make_file_executable,
    set_language_server_installation_status,
};

const GITHUB_REPO: &str = "crombo/zed-sheets-lsp";
const LSP_BIN_NAME: &str = "zed-sheets-lsp";

struct ZedSheetsExtension {
    cached_binary_path: Option<String>,
}

impl ZedSheetsExtension {
    fn platform_asset_name() -> String {
        let (os, arch) = current_platform();

        let os_name = match os {
            Os::Mac => "macos",
            Os::Linux => "linux",
            Os::Windows => "windows",
        };

        let arch_name = match arch {
            Architecture::Aarch64 => "aarch64",
            Architecture::X8664 => "x86_64",
            Architecture::X86 => "x86",
        };

        format!("{LSP_BIN_NAME}-{os_name}-{arch_name}.tar.gz")
    }

    fn select_asset<'a>(
        assets: &'a [GithubReleaseAsset],
        name: &str,
    ) -> Option<&'a GithubReleaseAsset> {
        assets.iter().find(|asset| asset.name == name)
    }

    fn get_or_download_lsp_binary(
        &mut self,
        language_server_id: &LanguageServerId,
    ) -> Result<String> {
        if let Some(path) = self.cached_binary_path.clone() {
            return Ok(path);
        }

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = latest_github_release(
            GITHUB_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = Self::platform_asset_name();
        let asset = Self::select_asset(&release.assets, &asset_name).ok_or_else(|| {
            format!(
                "No asset named '{asset_name}' found for release {} in {}",
                release.version, GITHUB_REPO
            )
        })?;

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );

        let install_dir = format!("lsp/{}", release.version);
        download_file(
            &asset.download_url,
            &install_dir,
            DownloadedFileType::GzipTar,
        )?;

        let mut binary_path = format!("{install_dir}/{LSP_BIN_NAME}");
        if matches!(current_platform().0, Os::Windows) {
            binary_path.push_str(".exe");
        }

        make_file_executable(&binary_path)?;
        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::None,
        );

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for ZedSheetsExtension {
    fn new() -> Self {
        ZedSheetsExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        if let Some(path) = worktree.which(LSP_BIN_NAME) {
            return Ok(Command {
                command: path,
                args: vec![],
                env: worktree.shell_env(),
            });
        }

        let lsp_path = match self.get_or_download_lsp_binary(language_server_id) {
            Ok(path) => path,
            Err(err) => {
                set_language_server_installation_status(
                    language_server_id,
                    &LanguageServerInstallationStatus::Failed(err.clone()),
                );
                return Err(err);
            }
        };

        Ok(Command {
            command: lsp_path,
            args: vec![],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(ZedSheetsExtension);
