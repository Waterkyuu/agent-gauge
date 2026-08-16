use crate::dto::comparison::{
    ComparisonDetailResponse, GetComparisonRequest, ListComparisonsRequest,
    ListComparisonsResponse, SaveComparisonRequest, SaveComparisonResponse,
};
use crate::error::IpcError;
use crate::services::comparison::ComparisonService;

/// Persists one completed comparison and returns its history identifier.
#[tauri::command]
pub(crate) async fn save_comparison_history(
    request: SaveComparisonRequest,
    service: tauri::State<'_, ComparisonService>,
) -> Result<SaveComparisonResponse, IpcError> {
    let id = service.save(request).await.map_err(IpcError::from)?;
    Ok(SaveComparisonResponse { id })
}

/// Returns one bounded newest-first page of comparison history.
#[tauri::command]
pub(crate) async fn list_comparison_history(
    request: ListComparisonsRequest,
    service: tauri::State<'_, ComparisonService>,
) -> Result<ListComparisonsResponse, IpcError> {
    service
        .list(request)
        .await
        .map(Into::into)
        .map_err(Into::into)
}

/// Returns one complete comparison for the history detail view.
#[tauri::command]
pub(crate) async fn get_comparison_history(
    request: GetComparisonRequest,
    service: tauri::State<'_, ComparisonService>,
) -> Result<ComparisonDetailResponse, IpcError> {
    service
        .find(request.id)
        .await
        .map(Into::into)
        .map_err(Into::into)
}
