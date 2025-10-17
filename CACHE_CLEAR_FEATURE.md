# 清除所有缓存数据功能文档

## 功能概述

在设置页面新增"清除所有缓存数据"按钮,允许用户一键清除应用中的所有本地缓存数据,但保留设置和 API 密钥。

## 功能位置

**设置页面** → **侧边栏底部** → **数据管理卡片**

## 清除范围

此功能将清除以下数据:

| 数据类型   | 说明               | 数据库表                                |
| ---------- | ------------------ | --------------------------------------- |
| 任务       | 所有任务记录       | `tasks`                                 |
| 规划会话   | 智能规划生成的会话 | `planning_sessions`, `planning_options` |
| 推荐记录   | AI 生成的推荐      | `recommendations`                       |
| 分析快照   | 每日分析快照       | `analytics_daily_snapshots`             |
| 效率评分   | 生产力评分历史     | `productivity_scores`                   |
| 健康提醒   | 健康休息提醒       | `wellness_nudges`                       |
| 工作量预测 | 工作量预测数据     | `workload_forecasts`                    |
| AI 反馈    | AI 功能反馈记录    | `ai_feedback`                           |
| 社区导出   | 社区导出日志       | `community_export_log`                  |

## 保留数据

以下数据**不会**被清除:

- ✅ 应用设置 (工作时间段、主题偏好等)
- ✅ DeepSeek API 密钥 (加密存储)
- ✅ AI 反馈选项设置

## 实现细节

### 后端实现

#### 1. 新增 Tauri 命令 (`src-tauri/src/commands/cache.rs`)

```rust
#[tauri::command]
pub async fn cache_clear_all(state: State<'_, AppState>) -> CommandResult<CacheClearResult> {
    let app_state = state.inner().clone();
    run_blocking(move || app_state.clear_all_cache()).await
}
```

#### 2. AppState 方法 (`src-tauri/src/commands/mod.rs`)

```rust
pub fn clear_all_cache(&self) -> AppResult<CacheClearResult> {
    let mut result = CacheClearResult::default();

    self.db_pool.with_connection(|conn| {
        // Count before clearing
        result.tasks_cleared = conn.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
        // ... 其他计数

        // Delete data (keep settings and ai_settings)
        conn.execute("DELETE FROM tasks", [])?;
        // ... 其他删除操作

        Ok(())
    })?;

    Ok(result)
}
```

#### 3. 返回类型

```rust
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheClearResult {
    pub tasks_cleared: i64,
    pub planning_sessions_cleared: i64,
    pub recommendations_cleared: i64,
    pub analytics_snapshots_cleared: i64,
    pub productivity_scores_cleared: i64,
    pub wellness_nudges_cleared: i64,
    pub workload_forecasts_cleared: i64,
    pub ai_feedback_cleared: i64,
    pub community_exports_cleared: i64,
}
```

### 前端实现

#### 1. 类型定义 (`src/types/settings.ts`)

```typescript
export interface CacheClearResult {
  tasksCleared: number;
  planningSessionsCleared: number;
  recommendationsCleared: number;
  analyticsSnapshotsCleared: number;
  productivityScoresCleared: number;
  wellnessNudgesCleared: number;
  workloadForecastsCleared: number;
  aiFeedbackCleared: number;
  communityExportsCleared: number;
}
```

#### 2. API 函数 (`src/services/tauriApi.ts`)

```typescript
export const clearAllCache = async (): Promise<CacheClearResult> => {
  if (isTauriAvailable()) {
    try {
      const result = await invoke<CacheClearResult>(COMMANDS.CACHE_CLEAR_ALL);
      return result;
    } catch (error) {
      throw mapUnknownError(error);
    }
  }

  warnMockUsage();

  // Mock result in development
  return {
    tasksCleared: 0,
    // ... 其他字段
  };
};
```

#### 3. UI 组件 (`src/pages/Settings.tsx`)

```tsx
function DataManagementCard() {
  const [isClearing, setIsClearing] = useState(false);
  const { notify } = useToast();

  const handleClearAllCache = async () => {
    if (!confirm('⚠️ 确定要清除所有缓存数据吗？...')) {
      return;
    }

    setIsClearing(true);
    try {
      const { clearAllCache } = await import('../services/tauriApi');
      const result = await clearAllCache();

      // Calculate total and show notification
      const total = /* sum all cleared counts */;

      notify({
        title: '缓存已清除',
        description: `已删除 ${total} 条记录...`,
        variant: 'success',
      });
    } catch (error) {
      // Error handling
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <Card className="rounded-3xl border-destructive/40 bg-destructive/5">
      {/* UI content */}
    </Card>
  );
}
```

## 用户流程

### 正常流程

1. 用户进入**设置页面**
2. 滚动到侧边栏底部的"数据管理"卡片
3. 点击"清除所有缓存数据"按钮
4. 系统弹出确认对话框,列出将被删除的数据类型
5. 用户确认后,系统执行清除操作
6. 清除完成后显示成功通知,包含删除的记录数量统计

### 确认对话框内容

```
⚠️ 确定要清除所有缓存数据吗?

这将删除:
• 所有任务
• 规划会话
• 推荐记录
• 分析快照
• 效率评分
• 健康提醒
• 工作量预测
• AI 反馈
• 社区导出记录

⚠️ 此操作不可撤销!设置和 API 密钥将保留。
```

### 成功通知示例

```
标题: 缓存已清除
描述: 已删除 42 条记录 (任务: 15, 规划: 3, 其他: 24)
```

## 使用场景

### 适用场景

1. **测试环境重置**: 开发或测试时需要清空所有测试数据
2. **性能优化**: 数据过多导致性能下降时
3. **故障排除**: 数据损坏或不一致时重新开始
4. **隐私清理**: 不想保留历史记录时
5. **重新开始**: 想从零开始使用应用时

### 注意事项

⚠️ **重要提醒**:

- 此操作**不可撤销**
- 建议在清除前手动备份重要数据
- 清除后需要重新创建任务和规划
- 分析仪表盘将显示为空状态
- 不影响设置和 API 密钥

## 安全考虑

1. **二次确认**: 使用 `confirm()` 对话框防止误操作
2. **保留关键数据**: 不删除设置和 API 密钥,避免用户需要重新配置
3. **事务安全**: 数据库操作在事务中执行,保证原子性
4. **错误处理**: 完善的错误捕获和用户友好的错误提示

## 测试验证

### 手动测试步骤

1. **准备数据**

   ```bash
   # 启动应用
   pnpm tauri dev

   # 创建一些测试任务和规划
   ```

2. **执行清除**
   - 进入设置页面
   - 点击"清除所有缓存数据"
   - 确认对话框
   - 验证成功通知

3. **验证结果**
   - 检查任务列表为空
   - 检查分析仪表盘显示零状态
   - 验证设置页面配置仍然保留
   - 验证 API 密钥未被清除

### 自动化测试

- ✅ 前端测试: `pnpm test` (25/25 通过)
- ✅ 后端编译: `cargo build` (成功)
- ✅ 前端构建: `pnpm run build` (成功)

## 修改的文件

### 后端文件

1. `src-tauri/src/commands/mod.rs`
   - 添加 `cache` 模块声明
   - 添加 `clear_all_cache()` 方法到 `AppState`
   - 添加 `CacheClearResult` 结构体

2. `src-tauri/src/commands/cache.rs` (新建)
   - 实现 `cache_clear_all` Tauri 命令

3. `src-tauri/src/lib.rs`
   - 注册 `cache_clear_all` 命令到 invoke_handler

### 前端文件

1. `src/types/settings.ts`
   - 添加 `CacheClearResult` 接口定义

2. `src/services/tauriApi.ts`
   - 添加 `CACHE_CLEAR_ALL` 命令常量
   - 导出 `CacheClearResult` 类型
   - 实现 `clearAllCache()` 函数

3. `src/pages/Settings.tsx`
   - 添加 `DataManagementCard` 组件
   - 在设置页面侧边栏中引用

## 后续优化建议

1. **备份功能**: 在清除前提供数据导出/备份选项
2. **选择性清除**: 允许用户选择清除特定类型的数据
3. **清除历史**: 记录清除操作日志
4. **恢复功能**: 提供短期内的数据恢复机制
5. **清除统计**: 展示更详细的清除前后对比

## 相关文档

- [设置页面功能说明](./Settings.md)
- [数据存储架构](./Database.md)
- [API 文档](./API.md)
