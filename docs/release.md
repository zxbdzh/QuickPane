# 发布与自动更新

## 发布流程

1. 在本地更新 `package.json` 和 `src-tauri/tauri.conf.json` 的版本号，并提交代码。
2. 创建并推送版本标签，例如 `git tag v0.1.1 && git push origin v0.1.1`。
3. GitHub Actions 的 `Release` 工作流在 Windows runner 上安装依赖、构建 Tauri NSIS 安装包，并使用 minisign 生成 updater 签名。
4. 工作流创建 GitHub Release，上传安装包及签名，然后生成并上传 `latest.json`。
5. 如果配置了 AWS 凭据，工作流会将同一安装包、签名和 `latest.json` 同步到 `quickpane-releases` S3 bucket；没有 AWS 凭据时 GitHub Release 仍然可用。

## 更新源

`src-tauri/tauri.conf.json` 按顺序配置了两个 endpoint：

1. S3：`https://quickpane-releases.s3.amazonaws.com/latest.json`
2. GitHub Release：`https://github.com/zxbdzh/QuickPane/releases/latest/download/latest.json`

Tauri updater 会按 endpoint 顺序检查，前一个源不可用时继续尝试后一个源。两个源必须使用同一版本和同一签名。

## GitHub Secrets

必需：

- `TAURI_SIGNING_PRIVATE_KEY`：本机 `%USERPROFILE%/.tauri/quickpane.key` 的完整内容。
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：生成密钥时设置的密码；无密码密钥可设为空字符串。

可选，用于启用 S3 同步：

- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_REGION`，默认 `us-east-1`

建议使用只允许写入该 bucket 的 IAM 用户或 OIDC 短期凭据，并开启 S3 公共读取策略或 CloudFront 读取权限。updater 私钥只能保存为 GitHub Secret 或本机安全文件，不能提交到仓库。

## 本地生成清单

`scripts/make-update-manifest.mjs` 需要以下环境变量：

- `UPDATE_VERSION`
- `UPDATE_URL`
- `UPDATE_SIGNATURE`

可选变量：`UPDATE_NOTES`、`UPDATE_PUB_DATE`、`UPDATE_MANIFEST`。平台目标固定为 Windows x64：`windows-x86_64`。
