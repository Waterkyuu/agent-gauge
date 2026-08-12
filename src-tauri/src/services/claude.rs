use crate::adapters::claude::ClaudeAdapter;
use crate::dto::claude::ClaudeLoginStatus;
use crate::error::AppError;

/// Resolves a stable UI contract from the Claude Code authentication probe.
pub(crate) fn check_claude_login(
    adapter: &impl ClaudeAdapter,
) -> Result<ClaudeLoginStatus, AppError> {
    let authentication = adapter.check_authentication()?;

    Ok(ClaudeLoginStatus {
        installed: authentication.installed,
        logged_in: authentication.logged_in,
        authentication_method: authentication.authentication_method,
        model: authentication.model,
        reasoning_effort: authentication.reasoning_effort,
    })
}

#[cfg(test)]
mod tests {
    use super::check_claude_login;
    use crate::adapters::claude::{ClaudeAdapter, ClaudeAuthentication};
    use crate::error::AppError;

    struct FakeClaudeAdapter;

    impl ClaudeAdapter for FakeClaudeAdapter {
        fn check_authentication(&self) -> Result<ClaudeAuthentication, AppError> {
            Ok(ClaudeAuthentication {
                installed: true,
                logged_in: true,
                authentication_method: Some("Claude account".to_string()),
                model: None,
                reasoning_effort: None,
            })
        }
    }

    #[test]
    fn reports_the_local_claude_authentication_state() {
        let status =
            check_claude_login(&FakeClaudeAdapter).expect("authentication probe should pass");

        assert!(status.installed);
        assert!(status.logged_in);
        assert_eq!(
            status.authentication_method.as_deref(),
            Some("Claude account")
        );
        assert_eq!(status.model, None);
        assert_eq!(status.reasoning_effort, None);
    }
}
