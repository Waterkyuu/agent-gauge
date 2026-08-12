use crate::adapters::codex::CodexAdapter;
use crate::dto::codex::CodexLoginStatus;
use crate::error::AppError;

/// Resolves a stable UI contract from the adapter's local authentication probe.
pub(crate) fn check_codex_login(adapter: &impl CodexAdapter) -> Result<CodexLoginStatus, AppError> {
    let authentication = adapter.check_authentication()?;

    Ok(CodexLoginStatus {
        installed: authentication.installed,
        logged_in: authentication.logged_in,
        authentication_method: authentication.authentication_method,
    })
}

#[cfg(test)]
mod tests {
    use crate::adapters::codex::{CodexAdapter, CodexAuthentication};
    use crate::error::AppError;

    struct FakeCodexAdapter {
        authentication: CodexAuthentication,
    }

    impl CodexAdapter for FakeCodexAdapter {
        fn check_authentication(&self) -> Result<CodexAuthentication, AppError> {
            Ok(self.authentication.clone())
        }
    }

    #[test]
    fn reports_the_local_codex_authentication_state() {
        let adapter = FakeCodexAdapter {
            authentication: CodexAuthentication {
                installed: true,
                logged_in: true,
                authentication_method: Some("ChatGPT".to_string()),
            },
        };

        let status = super::check_codex_login(&adapter).expect("authentication probe should pass");

        assert!(status.installed);
        assert!(status.logged_in);
        assert_eq!(status.authentication_method.as_deref(), Some("ChatGPT"));
    }
}
