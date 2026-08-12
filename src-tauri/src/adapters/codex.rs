use crate::error::AppError;
use std::ffi::OsString;
use std::io;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexAuthentication {
    pub(crate) installed: bool,
    pub(crate) logged_in: bool,
    pub(crate) authentication_method: Option<String>,
}

pub(crate) trait CodexAdapter {
    fn check_authentication(&self) -> Result<CodexAuthentication, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemCodexAdapter;

impl CodexAdapter for SystemCodexAdapter {
    fn check_authentication(&self) -> Result<CodexAuthentication, AppError> {
        for executable in codex_executable_candidates() {
            let output = Command::new(&executable)
                .args(["login", "status"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output();

            match output {
                Ok(output) => {
                    let authentication_method = output.status.success().then(|| {
                        String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .strip_prefix("Logged in using ")
                            .unwrap_or("authenticated credentials")
                            .to_string()
                    });

                    return Ok(CodexAuthentication {
                        installed: true,
                        logged_in: output.status.success(),
                        authentication_method,
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(_) => return Err(AppError::CodexProbeFailed),
            }
        }

        Ok(CodexAuthentication {
            installed: false,
            logged_in: false,
            authentication_method: None,
        })
    }
}

fn codex_executable_candidates() -> Vec<OsString> {
    let mut candidates = vec![OsString::from("codex")];

    #[cfg(target_os = "macos")]
    candidates.push(OsString::from(
        "/Applications/Codex.app/Contents/Resources/codex",
    ));

    candidates
}
