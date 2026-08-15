# AGENTS.md

## Git 提交规则

- 每次任务完成后主动提交代码。
- Git 提交信息必须使用英文，并采用 `feat`、`fix`、`docs`、`chore`、`style`、`ci` 等规范前缀。
- 每个提交只包含一个业务逻辑，最多提交 6 个文件，不得混入无关修改。
- 提交信息使用以下格式：

```text
feat(scope): summary
- change one
- change two
- change three
```

- 任何被 `.gitignore` 匹配的文件或目录都禁止擅自加入 Git 或提交。
- 禁止使用 `git add -f`、`git add --force` 或其他方式绕过 `.gitignore`。
- 只有用户明确指定某个被忽略的路径需要提交时，才允许强制加入该路径。
- 提交前必须检查待提交文件，确保没有误提交被忽略的文件、用户的其他修改或与当前任务无关的内容。
- 用户要求“撤销提交”或“删除提交”时，必须从当前分支的 Git 历史中直接移除对应提交，同时保留其他无关提交；禁止用新增 `git revert` 提交代替删除历史。
- 对应历史已经推送到远端时，完成本地历史重写后必须使用 `git push --force-with-lease` 更新远端；禁止使用无租约保护的 `git push --force`。
- 只有用户明确要求创建 revert 提交时，才允许使用 `git revert`。
