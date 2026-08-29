# 发布与自动更新

## 发布流程

1. 在本地更新 `package.json` 和 `src-tauri/tauri.conf.json` 的版本号，并提交代码。
2. 创建并推送版本标签，例如 `git tag v0.1.1 && git push origin v0.1.1`。
3. GitHub Actions 的 `Release` 工作流在 Windows runner 上安装依赖、构建 Tauri NSIS 安装包，并使用 minisign 生成 updater 签名。
4. 工作流创建 GitHub Release，上传安装包及签名，然后生成并上传 `latest.json`。
5. 如果配置了 MinIO Secrets，工作流会将同一安装包、签名和 `latest.json` 同步到自部署的 MinIO bucket；没有配置时 GitHub Release 仍然可用。

## 更新源

客户端按 endpoint 顺序检查更新，前一个源不可用时继续尝试后一个源：

1. MinIO（可选）：`https://<你的公开下载地址>/latest.json`，由 Release 工作流在构建时根据 Secrets 注入；仓库内的 `src-tauri/tauri.conf.json` 默认只保留 GitHub 兜底。
2. GitHub Release（兜底）：`https://github.com/zxbdzh/QuickPane/releases/latest/download/latest.json`

两个源必须使用同一版本和同一签名；工作流会分别为 GitHub 和 MinIO 生成各自下载 URL 的 `latest.json`。

## MinIO 配置

MinIO 与 AWS S3 兼容，工作流用 AWS CLI 的 `--endpoint-url` 上传并启用 path-style 寻址。需要区分两个地址：

- **S3 API 地址**（`MINIO_ENDPOINT`）：CI 上传使用的接口，例如 `https://minio.example.com`。不是浏览器控制台端口。
- **公开下载地址**（`MINIO_PUBLIC_BASE_URL`）：QuickPane 客户端实际 GET `latest.json` 和安装包的 URL 前缀，例如 `https://minio.example.com/quickpane-releases`。可以与 API 地址同域（path-style），也可以是反代后的独立域名。

两者必须在仓库 Secrets 中显式配置，代码和 workflow 中没有任何硬编码的 MinIO 地址或 bucket。

## GitHub Secrets

必需：

- `TAURI_SIGNING_PRIVATE_KEY`：本机 `%USERPROFILE%/.tauri/quickpane.key` 的完整内容。
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：生成密钥时设置的密码；无密码密钥可设为空字符串。

可选，用于启用 MinIO 更新源（全部配置才会启用）：

- `MINIO_ENDPOINT`：S3 API 地址，例如 `https://minio.example.com`。
- `MINIO_PUBLIC_BASE_URL`：客户端可公开 GET 的 bucket URL 前缀。
- `MINIO_ACCESS_KEY`、`MINIO_SECRET_KEY`：上传凭据，建议使用只允许写入该 bucket 的 MinIO policy。
- `MINIO_BUCKET`：bucket 名称，默认 `quickpane-releases`。
- `MINIO_REGION`：默认 `us-east-1`。

bucket 需要允许客户端匿名只读以下对象：`latest.json`、Windows 安装包、`.sig` 签名文件。如果通过反向代理暴露下载地址，需保证 HTTPS 有效（updater 默认校验 TLS）并透传 `Content-Length`。

updater 私钥只能保存为 GitHub Secret 或本机安全文件，不能提交到仓库。

## 本地生成清单

`scripts/make-update-manifest.mjs` 需要以下环境变量：

- `UPDATE_VERSION`
- `UPDATE_URL`
- `UPDATE_SIGNATURE`

可选变量：`UPDATE_NOTES`、`UPDATE_PUB_DATE`、`UPDATE_MANIFEST`。平台目标固定为 Windows x64：`windows-x86_64`。
