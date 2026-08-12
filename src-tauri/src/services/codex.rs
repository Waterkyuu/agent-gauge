use crate::adapters::codex::CodexAdapter;
use crate::domain::codex_run::CodexRunOutput;
use crate::dto::codex::CodexLoginStatus;
use crate::error::AppError;

/// Resolves a stable UI contract from the adapter's local authentication probe.
pub(crate) fn check_codex_login(adapter: &impl CodexAdapter) -> Result<CodexLoginStatus, AppError> {
    let authentication = adapter.check_authentication()?;

    Ok(CodexLoginStatus {
        installed: authentication.installed,
        logged_in: authentication.logged_in,
        authentication_method: authentication.authentication_method,
        model: authentication.model,
        reasoning_effort: authentication.reasoning_effort,
    })
}

/// Runs one Codex task after enforcing the bounded MVP query contract.
pub(crate) fn run_codex_task(
    adapter: &impl CodexAdapter,
    query: &str,
) -> Result<CodexRunOutput, AppError> {
    if query.trim().is_empty() || query.len() > 16_000 {
        return Err(AppError::InvalidQuery);
    }

    adapter.run_task(query)
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

        fn run_task(
            &self,
            _query: &str,
        ) -> Result<crate::domain::codex_run::CodexRunOutput, AppError> {
            Err(AppError::CodexTaskFailed)
        }
    }

    #[test]
    fn reports_the_local_codex_authentication_state() {
        let adapter = FakeCodexAdapter {
            authentication: CodexAuthentication {
                installed: true,
                logged_in: true,
                authentication_method: Some("ChatGPT".to_string()),
                model: Some("gpt-5.6-sol".to_string()),
                reasoning_effort: Some("high".to_string()),
            },
        };

        let status = super::check_codex_login(&adapter).expect("authentication probe should pass");

        assert!(status.installed);
        assert!(status.logged_in);
        assert_eq!(status.authentication_method.as_deref(), Some("ChatGPT"));
        assert_eq!(status.model.as_deref(), Some("gpt-5.6-sol"));
        assert_eq!(status.reasoning_effort.as_deref(), Some("high"));
    }
}
