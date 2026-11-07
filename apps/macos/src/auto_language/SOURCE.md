# InputSourcePro 参考与移植记录

- 上游：[runjuu/InputSourcePro](https://github.com/runjuu/InputSourcePro)
- 参考提交：`c2ab1751104646f82b9a57c3ca6d0f759dd5b1a2`
- 许可：GPL-3.0；本目录代码随青简以 GPL-3.0-or-later 发布。上游原许可全文保存在 `INPUTSOURCEPRO-LICENSE`。

本包提取了以下处理方式，并以 Rust / objc2 重新实现：

| 上游文件 | 保留的处理 | 青简中的位置与差异 |
|---|---|---|
| `Input Source Pro/Models/ApplicationVM.swift` | 从 `NSWorkspace` 监听应用激活，以应用标识关联后续规则 | `tracker.rs`；直接读通知携带的应用对象，按到达顺序缓存，避免前台应用快速变化丢失中间事件 |
| `Input Source Pro/Models/PreferencesVM.swift` 的 `filterApps` | 普通模式只选择 `.regular` 应用 | `application.rs` / `filter.rs`；不增加浮窗增强模式或 AX 权限 |
| `Input Source Pro/Persistence/AppRule.swift` | 规则使用稳定 bundle ID，名称与图标只是展示信息 | `application.rs`；传递 bundle ID、显示名称与 bundle 路径，MRU 和规则由平台无关模块负责 |
| `Input Source Pro/Utilities/InputSource/InputSourceSwitcher.swift` 的临时输入窗口激活过滤 | 自身临时窗口不构成真实应用切换 | `filter.rs`；青简输入法与设置程序一律透明，另过滤系统输入面板，不移植上游的系统输入源修复与临时文本窗口流程 |

本目录没有直接复制 Swift 源码，也没有引入 Combine、Core Data、AX 窗口观察、网站规则或输入源切换。新增的 helper 过滤、通知事件队列、observer 释放及设置程序识别是青简的适配逻辑。
