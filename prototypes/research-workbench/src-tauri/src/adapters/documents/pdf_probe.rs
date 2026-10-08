use crate::transport::error::{AppError, ErrorCode};
use std::collections::HashSet;

fn effective_media_box(page: &pdf::object::Page) -> Result<pdf::object::Rectangle, AppError> {
    if let Some(rect) = page.media_box {
        return Ok(rect);
    }
    let mut parent = Some(&page.parent);
    let mut visited = HashSet::new();
    while let Some(tree) = parent {
        // PagesRc keeps a strong reference to its parsed object. The pointer remains stable
        // while that reference is retained, so it detects actual object cycles without a
        // policy limit on valid inheritance depth.
        let identity = std::ptr::from_ref(&**tree);
        if !visited.insert(identity) {
            return Err(AppError::new(ErrorCode::InvalidPdf));
        }
        if let Some(rect) = tree.media_box {
            return Ok(rect);
        }
        parent = tree.parent.as_ref();
    }
    Err(AppError::new(ErrorCode::InvalidPdf))
}

pub fn validate_pdf(bytes: Vec<u8>) -> Result<(), AppError> {
    use pdf::file::FileOptions;
    let file = FileOptions::uncached()
        .load(bytes)
        .map_err(|_| AppError::new(ErrorCode::InvalidPdf))?;
    if file.trailer.encrypt_dict.is_some() || file.num_pages() == 0 {
        return Err(AppError::new(ErrorCode::InvalidPdf));
    }
    let page = file
        .get_page(0)
        .map_err(|_| AppError::new(ErrorCode::InvalidPdf))?;
    let rect = effective_media_box(&page)?;
    let width = rect.right - rect.left;
    let height = rect.top - rect.bottom;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(AppError::new(ErrorCode::InvalidPdf));
    }
    Ok(())
}
