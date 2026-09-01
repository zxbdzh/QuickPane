# Issue 跟踪器：GitHub

本仓库的 Issue 与规格统一记录在 GitHub Issues 中。所有操作均使用 `gh` CLI。

## 操作约定

- **创建 Issue**：`gh issue create --title "..." --body "..."`。正文包含多行内容时使用 heredoc。
- **读取 Issue**：`gh issue view <编号> --comments`，同时获取标签，并根据需要使用 `jq` 过滤评论。
- **列出 Issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，按需添加 `--label` 和 `--state` 筛选条件。
- **评论 Issue**：`gh issue comment <编号> --body "..."`
- **添加或移除标签**：`gh issue edit <编号> --add-label "..."` / `gh issue edit <编号> --remove-label "..."`
- **关闭 Issue**：`gh issue close <编号> --comment "..."`

通过 `git remote -v` 推断仓库；在当前克隆目录中执行时，`gh` 会自动完成此操作。

## 是否将 Pull Request 作为分流入口

**将 PR 作为请求入口：否。**（如果本仓库以后将外部 PR 视为功能请求，可将此项改为“是”；`/triage` 会读取此设置。）

设置为“是”后，PR 使用与 Issue 相同的标签和状态，并改用对应的 `gh pr` 命令：

- **读取 PR**：使用 `gh pr view <编号> --comments`，并用 `gh pr diff <编号>` 查看差异。
- **列出待分流的外部 PR**：运行 `gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments`，仅保留 `authorAssociation` 为 `CONTRIBUTOR`、`FIRST_TIME_CONTRIBUTOR` 或 `NONE` 的项目，排除 `OWNER`、`MEMBER` 和 `COLLABORATOR`。
- **评论、添加标签或关闭**：使用 `gh pr comment`、`gh pr edit --add-label` / `--remove-label`、`gh pr close`。

GitHub 的 Issue 与 PR 共用编号空间，因此 `#42` 可能指向任意一种资源。先运行 `gh pr view 42`，失败后再运行 `gh issue view 42`。

## 当技能要求“发布到 Issue 跟踪器”时

创建一个 GitHub Issue。

## 当技能要求“获取相关工单”时

运行 `gh issue view <编号> --comments`。
