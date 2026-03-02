use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

struct ZedSheetsExtension;

impl zed::Extension for ZedSheetsExtension {
    fn new() -> Self {
        ZedSheetsExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Command> {
        Ok(Command {
            command: "zed-sheets-lsp".to_string(),
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(ZedSheetsExtension);
