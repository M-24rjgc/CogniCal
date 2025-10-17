# DeepSeek 密钥清除功能修复文档

## 问题描述

用户报告清除 DeepSeek API 密钥后,虽然 UI 暂时显示密钥已清除,但实际上:

1. 点击"保存设置"后密钥又出现
2. 测试连接时密钥又恢复
3. 重新加载页面密钥又回来
4. 在线模式判断也有问题

## 根本原因

前后端数据格式不匹配:

### 前端发送的数据

```typescript
// 清除密钥时
{
  removeDeepseekKey: true;
}

// 设置密钥时
{
  deepseekApiKey: 'sk-xxxx';
}
```

### 后端期望的数据格式

Rust 的 `SettingsUpdateInput` 结构使用 `Option<Option<String>>`:

- `None` = 不修改密钥
- `Some(None)` = 清除密钥
- `Some(Some(value))` = 设置新密钥

**问题**: 后端的 `SettingsUpdatePayload` 没有 `remove_deepseek_key` 字段,前端发送 `removeDeepseekKey: true` 时,后端接收到的 `deepseek_api_key` 是 `None`,被解释为"不修改",所以密钥没有被清除。

## 解决方案

修改后端 `SettingsUpdatePayload` 结构,添加 `remove_deepseek_key` 字段并在转换时正确处理:

### 修改文件: `src-tauri/src/commands/settings.rs`

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdatePayload {
    #[serde(default)]
    deepseek_api_key: Option<String>,          // 改为 Option<String>
    #[serde(default)]
    remove_deepseek_key: Option<bool>,         // 新增字段
    #[serde(default)]
    workday_start_minute: Option<i16>,
    #[serde(default)]
    workday_end_minute: Option<i16>,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    ai_feedback_opt_out: Option<bool>,
}

impl SettingsUpdatePayload {
    fn into_input(self) -> SettingsUpdateInput {
        // 转换逻辑:
        // - 如果 remove_deepseek_key == true => Some(None) (清除)
        // - 否则如果 deepseek_api_key 有值 => Some(Some(value)) (设置)
        // - 否则 => None (不修改)
        let deepseek_api_key = if self.remove_deepseek_key == Some(true) {
            Some(None)
        } else {
            self.deepseek_api_key.map(Some)
        };

        SettingsUpdateInput {
            deepseek_api_key,
            workday_start_minute: self.workday_start_minute,
            workday_end_minute: self.workday_end_minute,
            theme: self.theme,
            ai_feedback_opt_out: self.ai_feedback_opt_out,
        }
    }
}
```

## 测试验证

### 单元测试

在 `src-tauri/src/commands/settings.rs` 中添加了测试模块来验证转换逻辑:

1. ✅ `remove_deepseek_key: true` => `Some(None)`
2. ✅ `deepseek_api_key: "value"` => `Some(Some("value"))`
3. ✅ 两者都不提供 => `None`
4. ✅ 同时提供时 remove 优先 => `Some(None)`

### Rust 后端测试

运行 `cargo test --lib commands::settings::tests` - 所有 4 个测试通过 ✅

```
running 4 tests
test commands::settings::tests::test_no_change_deepseek_key ... ok
test commands::settings::tests::test_remove_takes_precedence ... ok
test commands::settings::tests::test_remove_deepseek_key_flag ... ok
test commands::settings::tests::test_set_deepseek_key ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
```

### 前端测试

运行 `pnpm test` - 所有 25 个测试通过 ✅

```
Test Files  5 passed (5)
     Tests  25 passed (25)
```

## 测试步骤

### 手动测试清除密钥功能

1. **设置密钥**
   - 进入设置页面
   - 填写 DeepSeek API 密钥
   - 点击"保存设置"
   - ✅ 验证: 显示"已配置"徽章和掩码密钥

2. **清除密钥**
   - 点击"清除 DeepSeek 密钥"按钮
   - ✅ 验证: 显示"已清除"通知
   - ✅ 验证: 徽章变为"未配置"
   - ✅ 验证: 掩码密钥消失

3. **持久化验证**
   - 点击"保存设置"(修改其他设置)
   - ✅ 验证: 密钥仍然保持清除状态
   - 点击"测试连接"
   - ✅ 验证: 密钥仍然保持清除状态
   - 刷新页面
   - ✅ 验证: 密钥仍然保持清除状态

4. **在线模式验证**
   - 清除密钥后
   - ✅ 验证: AI 状态显示"离线模式"
   - 重新设置密钥
   - 点击"测试连接"
   - ✅ 验证: AI 状态更新为"在线模式"或相应模式

## 数据流程图

### 清除密钥流程

```
Settings.tsx (handleRemoveKey)
    ↓
    { removeDeepseekKey: true }
    ↓
Tauri IPC (settings_update)
    ↓
SettingsUpdatePayload::into_input()
    remove_deepseek_key == Some(true) => Some(None)
    ↓
SettingsService::update()
    ApiKeyAction::Clear
    ↓
Database: DELETE FROM ai_settings WHERE key = 'deepseek_api_key'
    ↓
Cache: settings.deepseek_api_key = None
    ↓
Response: { hasDeepseekKey: false, maskedDeepseekKey: null }
    ↓
Settings UI 更新
```

### 设置密钥流程

```
Settings.tsx (handleSubmit)
    ↓
    { deepseekApiKey: "sk-xxxx" }
    ↓
Tauri IPC (settings_update)
    ↓
SettingsUpdatePayload::into_input()
    deepseek_api_key.map(Some) => Some(Some("sk-xxxx"))
    ↓
SettingsService::update()
    ApiKeyAction::Set(encrypted, masked)
    ↓
Database: INSERT/UPDATE ai_settings (encrypted key)
    ↓
Cache: settings.deepseek_api_key = Some("sk-****")
    ↓
Response: { hasDeepseekKey: true, maskedDeepseekKey: "sk-****" }
    ↓
Settings UI 更新
```

## 影响范围

### 修改的文件

- ✅ `src-tauri/src/commands/settings.rs` - 添加 remove_deepseek_key 字段处理和单元测试模块

### 不需要修改的文件

- ❌ `src/services/tauriApi.ts` - 前端模拟已正确处理 removeDeepseekKey
- ❌ `src/pages/Settings.tsx` - UI 逻辑正确
- ❌ `src/utils/validators.ts` - 验证规则正确
- ❌ `src-tauri/src/services/settings_service.rs` - 服务层逻辑正确

## 兼容性说明

此修复向后兼容:

- ✅ 旧的前端代码(只发送 deepseekApiKey)仍然有效
- ✅ 新的前端代码(发送 removeDeepseekKey)现在能正确工作
- ✅ 不影响其他设置项的更新逻辑

## 部署检查清单

- [x] 后端代码修改
- [x] 后端编译成功
- [x] 后端单元测试通过 (4/4 tests)
- [x] 前端测试通过 (25/25 tests)
- [ ] 手动集成测试(需要运行完整应用)
- [ ] 生产环境部署前验证

## 相关问题

此修复解决了以下相关问题:

1. 密钥清除后立即恢复的问题
2. 在线/离线模式状态不准确的问题
3. 密钥持久化不一致的问题
