use crate::error::AppError;
use std::process::{Command, Stdio};

/// Returns the executable name reported for every process visible to the current user.
///
/// Windows exposes image names through `tasklist`, while Unix-like systems expose the command
/// path through `ps`. The adapter layer normalizes these platform-specific values and decides
/// which supported Agent, if any, each process belongs to.
pub(crate) fn running_process_names() -> Result<Vec<String>, AppError> {
    #[cfg(target_os = "windows")]
    let output = Command::new("tasklist")
        .args(["/fo", "csv", "/nh"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| AppError::ProcessProbeFailed)?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("ps")
        .args(["-axo", "comm="])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| AppError::ProcessProbeFailed)?;

    if !output.status.success() {
        return Err(AppError::ProcessProbeFailed);
    }

    let stdout = String::from_utf8(output.stdout).map_err(|_| AppError::ProcessProbeFailed)?;

    #[cfg(target_os = "windows")]
    return Ok(stdout
        .lines()
        .filter_map(|line| {
            // CSV output starts with the quoted image name; the remaining fields are irrelevant
            // to Agent detection and may contain localized values.
            line.strip_prefix('"')
                .and_then(|line| line.split("\",").next())
                .map(str::to_string)
        })
        .collect());

    #[cfg(not(target_os = "windows"))]
    Ok(stdout.lines().map(str::to_string).collect())
}
