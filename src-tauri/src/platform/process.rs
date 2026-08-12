use crate::error::AppError;
use std::process::{Command, Stdio};

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
            line.strip_prefix('"')
                .and_then(|line| line.split("\",").next())
                .map(str::to_string)
        })
        .collect());

    #[cfg(not(target_os = "windows"))]
    Ok(stdout.lines().map(str::to_string).collect())
}
