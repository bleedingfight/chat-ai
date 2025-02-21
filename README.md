# 构建项目

本项目是一个跨平台的聊天助手，主要是通过API调用实现和AI智能体沟通。主要是为了解决聚合所有平台的API服务在一起，这样就可以在一个地方调用所有的服务。如果需要免费的API服务，可以到下面的地址中注册：

- [KIMI API](https://platform.moonshot.cn/console/account)

# 编译和安装

## 本地编译安装

```bash
cargo install tauri-cli # 安装 tauri环境
cargo tauri dev # 本地构建
```

## 打包

- `cargo tauri build`:构建软件包
