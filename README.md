# 右键菜单管理器 (Right-Click Menu Manager)

一个使用 Rust 和 Windows API 开发的右键菜单管理工具，可以方便地管理文件、文件夹、驱动器和桌面背景的右键菜单项。

## 功能特性

- 📂 **多上下文支持**：支持文件、文件夹、驱动器和桌面背景四种右键菜单类型
- 👁️ **可视化界面**：直观的图形界面，显示当前右键菜单项
- ➕ **添加菜单项**：轻松添加新的右键菜单项
- ✏️ **编辑菜单项**：修改现有菜单项的显示名称、命令和图标
- 🗑️ **删除菜单项**：移除不需要的右键菜单项
- ⚡ **启用/禁用**：快速启用或禁用右键菜单项
- 🔄 **实时刷新**：实时查看注册表变化
- 🔒 **管理员权限**：自动检测并提示管理员权限

## 系统要求

- **操作系统**：Windows 10 或更高版本
- **运行环境**：无需额外依赖，可直接运行
- **权限要求**：需要管理员权限才能修改注册表

## 快速开始

### 下载使用

1. 从 [Releases](https://github.com/yourusername/right-click-manager/releases) 页面下载最新版本的 `right-click-manager-native.exe`
2. 右键选择"以管理员身份运行"
3. 选择要管理的右键菜单类型（文件、文件夹、驱动器、桌面背景）
4. 点击"刷新"按钮查看当前菜单项
5. 使用添加、编辑、删除、启用/禁用按钮进行管理

### 从源码编译

如果你想要从源码编译：

1. 确保已安装 [Rust](https://www.rust-lang.org/tools/install)
2. 克隆项目：
   ```bash
   git clone https://github.com/yourusername/right-click-manager.git
   cd right-click-manager
   ```
3. 编译项目：
   ```bash
   cargo build --release
   ```
4. 运行程序：
   ```bash
   .\target\release\right-click-manager-native.exe
   ```

## 使用说明

### 界面布局

- **顶部下拉框**：选择要管理的右键菜单类型
- **列表视图**：显示当前右键菜单项，包含显示名称、键名、命令和状态
- **功能按钮**：刷新、添加、编辑、删除、启用/禁用

### 操作指南

1. **查看菜单项**：
   - 选择菜单类型
   - 点击"刷新"按钮

2. **添加新菜单项**：
   - 点击"添加"按钮
   - 填写键名、显示名称、命令和图标路径
   - 点击"确定"保存

3. **编辑菜单项**：
   - 在列表中选择要编辑的项
   - 点击"编辑"按钮
   - 修改信息后点击"确定"

4. **删除菜单项**：
   - 在列表中选择要删除的项
   - 点击"删除"按钮确认

5. **启用/禁用菜单项**：
   - 在列表中选择要操作的项
   - 点击"启用/禁用"按钮切换状态

## 技术细节

### 技术栈

- **编程语言**：Rust
- **GUI框架**：原生 Windows API (windows-rs)
- **注册表操作**：Windows Registry API
- **构建工具**：Cargo

### 项目结构

```
right-click-manager/
├── src/
│   └── main.rs          # 主程序文件
├── Cargo.toml           # 项目配置和依赖
├── README.md            # 项目说明文档
├── LICENSE              # 许可证文件
└── .gitignore           # Git忽略文件
```

### 主要依赖

- `windows` - Windows API 绑定
- 其他标准库依赖

## 注意事项

⚠️ **重要提示**：

- 修改注册表可能会影响系统稳定性，请谨慎操作
- 建议在操作前备份重要数据
- 程序需要管理员权限才能正常工作
- 某些杀毒软件可能会误报，请添加信任

## 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 更新日志

### v1.0.0 (2024-03-24)
- 初始版本发布
- 支持四种右键菜单类型管理
- 完整的 CRUD 操作功能
- 图形化用户界面

## 联系方式

- 项目主页：https://github.com/yourusername/right-click-manager
- 问题反馈：https://github.com/yourusername/right-click-manager/issues

---

⭐ 如果这个项目对你有帮助，请给个星标支持！