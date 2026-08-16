use crate::adapters::workbuddy::WorkBuddyAdapter;
use crate::dto::workbuddy::WorkBuddyLoginStatus;
use crate::error::AppError;

/// Resolves a stable UI contract from the WorkBuddy ACP authentication probe.
pub(crate) fn check_workbuddy_login(
    adapter: &impl WorkBuddyAdapter,
) -> Result<WorkBuddyLoginStatus, AppError> {
    let authentication = adapter.check_authentication()?;

    Ok(WorkBuddyLoginStatus {
        installed: authentication.installed,
        logged_in: authentication.logged_in,
        authentication_method: authentication.authentication_method,
        model: None,
        reasoning_effort: None,
    })
}

#[cfg(test)]
mod tests {
    use super::check_workbuddy_login;
    use crate::adapters::workbuddy::{WorkBuddyAdapter, WorkBuddyAuthentication};
    use crate::error::AppError;

    struct FakeWorkBuddyAdapter;

    impl WorkBuddyAdapter for FakeWorkBuddyAdapter {
        fn check_authentication(&self) -> Result<WorkBuddyAuthentication, AppError> {
            Ok(WorkBuddyAuthentication {
                installed: true,
                logged_in: true,
                authentication_method: Some("WorkBuddy account".to_string()),
            })
        }
    }

    #[test]
    fn reports_authentication_without_polling_runtime_config() {
        let status =
            check_workbuddy_login(&FakeWorkBuddyAdapter).expect("authentication probe should pass");

        assert!(status.installed);
        assert!(status.logged_in);
        assert_eq!(status.model, None);
        assert_eq!(status.reasoning_effort, None);
    }
}
