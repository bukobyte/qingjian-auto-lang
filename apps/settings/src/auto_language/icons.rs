//! macOS 应用图标只按最近列表的已知路径读取，不枚举应用或查询窗口。

use std::sync::Arc;

use bamboo_slip::Pixmap;
use qingjian_platform::AppInfo;

#[cfg(target_os = "macos")]
pub(super) fn load(app: &AppInfo) -> Option<Arc<Pixmap>> {
    use objc2::AnyThread;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
    use objc2_foundation::{NSDictionary, NSSize, NSString};

    let path = app.bundle_path.as_deref()?;
    let image = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(path));
    image.setSize(NSSize::new(32.0, 32.0));
    let tiff = image.TIFFRepresentation()?;
    let bitmap = NSBitmapImageRep::initWithData(NSBitmapImageRep::alloc(), &tiff)?;
    // SAFETY: 空属性字典不包含类型不匹配的编码参数，位图和返回数据均由 AppKit 持有。
    let data = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    Pixmap::decode_png(&data.to_vec()).ok().map(Arc::new)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn load(_app: &AppInfo) -> Option<Arc<Pixmap>> {
    None
}
