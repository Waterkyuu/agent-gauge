---
name: uiux-checker
description: Use this skill when reviewing front-end UI quality, especially when the user says the interface looks messy, inconsistent, amateurish, 草台, UI 混乱, 需要 UI 自检, or asks to check whether a page/app has consistent visual design without a designer. This skill audits UI consistency, spacing, typography, colors, component states, responsiveness, and basic interaction polish.
---

# UI Checker Skill

## Purpose

This skill helps review a front-end page or app when there is no designer involved. The goal is not to make the UI visually stunning. The goal is to make it feel consistent, clean, usable, and not amateurish.

Core principle:

> 统一、留白、对齐、反馈

Expanded principle:

- 统一: typography, colors, buttons, icons, border radius, shadows, borders, component structure.
- 留白: page padding, module spacing, item gap, card padding, density.
- 对齐: text, icons, buttons, forms, lists, tables, headers.
- 反馈: hover, active, focus-visible, disabled, loading, empty, error, success, permission states.

## When to Use This Skill

Use this skill when the user asks to:

- Check a UI for inconsistency or visual messiness.
- Review whether a front-end page looks 草台.
- Improve UI quality in a vibe coding / AI-generated front-end project.
- Create or enforce UI consistency rules without a designer.
- Review screenshots, DOM/code, Tailwind classes, component files, or page implementations for UI quality.
- Generate a prompt/checklist for a coding agent to fix UI inconsistency.

Trigger examples:

- “帮我检查这个 UI 为什么看起来很草台”
- “这个页面 UI 有点乱，帮我找问题”
- “没有设计师，怎么让前端 UI 统一一点”
- “检查页面字体、按钮、图标、背景色是否统一”
- “帮我写一个 UI 自检 prompt 给 agent”

## Inputs

Prefer using any concrete materials the user provides:

- Screenshot or screen recording.
- Page/component source code.
- Tailwind / CSS / design token config.
- Component library usage.
- Existing UI rules from the user.
- Product type: admin dashboard, mobile app, landing page, form app, SaaS console, etc.

If materials are incomplete, still provide a best-effort checklist and ask for a screenshot or code only if necessary.

## Review Method

When reviewing a UI, inspect in this order:

1. Visual consistency
2. Spacing and layout
3. Component states
4. Page states
5. Responsiveness
6. Text/content robustness
7. Code/component reuse
8. Priority fixes

Do not only say “looks good” or “needs improvement.” Always identify concrete issues and give concrete fixes.

## UI Audit Checklist

### 1. Typography Consistency

Check whether same-level text uses consistent size, weight, and color.

Recommended baseline:

- Page title: `text-2xl font-semibold text-gray-900`
- Section title: `text-lg font-medium text-gray-900`
- Body text: `text-sm text-gray-700`
- Secondary text: `text-xs text-gray-500`
- Error text: `text-sm text-red-600`
- Link text: `text-sm text-primary`

Flag problems:

- Same-level titles use different sizes.
- Body text mixes `text-gray-600`, `text-gray-700`, `text-black` without reason.
- Helper text and secondary text use inconsistent grays.
- Random font weights appear across similar elements.

### 2. Button Consistency

Check whether same-level buttons share color, size, radius, hover, active, disabled, and loading states.

Recommended button levels:

- Primary: `bg-primary text-white hover:bg-primary/90`
- Secondary/outline: `border border-gray-300 bg-white text-gray-700 hover:bg-gray-50`
- Ghost/text: `text-gray-700 hover:bg-gray-100`
- Danger: `bg-red-600 text-white hover:bg-red-700`

Required states:

- Default
- Hover
- Active
- Focus-visible
- Disabled
- Loading

Flag problems:

- Primary buttons use different blues/purples across pages.
- Buttons have inconsistent heights or border radius.
- Disabled buttons have no visual distinction.
- Hover color changes are inconsistent.

### 3. Icon Color Consistency

Check icon color across the app.

Recommended baseline:

- Normal icon: `text-gray-500`
- Hover icon: `text-gray-700`
- Active icon: `text-primary`
- Danger icon: `text-red-500`
- Disabled icon: `text-gray-300`

Flag problems:

- Some icons are pure black while others are gray.
- Icons inherit random text colors.
- Same action uses different icon colors in different places.

### 4. Icon Size Consistency

Recommended baseline:

- Inline icon beside text: `h-4 w-4`
- Button icon: `h-4 w-4`
- Navigation icon: `h-5 w-5`
- Empty state icon: `h-10 w-10` to `h-12 w-12`
- Feature entry icon: `h-6 w-6` to `h-8 w-8`

Flag problems:

- Similar menu items use 16px, 20px, and 24px icons together.
- Icon button click target is only the icon itself.

### 5. Background Consistency

Recommended baseline:

- App/page background: `bg-gray-50`
- Card/content surface: `bg-white`
- Modal/dropdown: `bg-white`
- Input: `bg-white`
- Disabled area: `bg-gray-100`

Flag problems:

- One page uses white background, another uses gray, without intent.
- Random mixing of `gray`, `slate`, `zinc`, `neutral` color families.

### 6. Spacing and Gap

Every continuous horizontal or vertical group must define spacing.

Recommended baseline:

- Mobile page padding: `px-4`
- Tablet page padding: `px-6`
- Desktop page padding: `px-8`
- Module spacing: `gap-6` / `space-y-6`
- Card padding: `p-4` or `p-6`
- Icon + text: `gap-1.5` / `gap-2`
- Button group: `gap-2` / `gap-3`
- List items: `gap-3` / `gap-4`
- Card grid: `gap-4` / `gap-6`
- Form fields: `space-y-4` / `space-y-5`

Flag problems:

- Icons and text are stuck together.
- Vertically stacked items have no `gap` or `space-y`.
- Card content is too close to the card edge.
- One page is spacious while another is cramped.

### 7. Transitions

All color changes caused by hover, active, selected, or focus states should use:

`transition-colors duration-200`

For shadows/transforms, use:

- `transition-shadow duration-200`
- `transition-transform duration-200`

Flag problems:

- Hover colors change abruptly.
- Different components use inconsistent animation duration.
- Excessive bounce/scale/rotation effects.

### 8. Border Radius

Recommended baseline:

- Button: `rounded-lg`
- Input/select/textarea: `rounded-lg`
- Card: `rounded-xl`
- Modal: `rounded-2xl`
- Avatar: `rounded-full`
- Badge/tag: `rounded-full` or `rounded-md`

Flag problems:

- Same-level buttons use `rounded-md`, `rounded-xl`, and `rounded-full` randomly.
- Cards and modals have inconsistent corner radius.

### 9. Shadow

Recommended baseline:

- Normal card: `shadow-sm` or border-only
- Hover card: `hover:shadow-md`
- Dropdown: `shadow-lg`
- Modal: `shadow-xl`
- Header: `shadow-sm` or `border-b`

Flag problems:

- Similar cards have very different shadow weights.
- Dropdown or modal shadow is too heavy or too weak compared with the rest of the app.

### 10. Border Color

Recommended baseline:

- Normal border: `border-gray-200`
- Input border: `border-gray-300`
- Hover border: `border-gray-400`
- Focus border: `border-primary`
- Error border: `border-red-500`

Flag problems:

- Mixing `border-gray`, `border-slate`, `border-zinc`, `border-neutral` without a token system.

### 11. Primary Color Discipline

The app should have one primary color. Use it consistently for:

- Primary button
- Link
- Active navigation
- Focus ring/border
- Tabs active state
- Checkbox/radio selected state
- Switch active state
- Progress bar
- Important active icon

Flag problems:

- Primary button uses blue, link uses indigo, tab uses sky, focus uses cyan.

### 12. Input and Form States

Inputs must support:

- Default
- Hover
- Focus
- Disabled
- Error
- Placeholder

Recommended baseline:

- Default: `border-gray-300 bg-white text-gray-900`
- Placeholder: `placeholder:text-gray-400`
- Focus: `focus:border-primary focus:ring-2 focus:ring-primary/20`
- Disabled: `disabled:bg-gray-100 disabled:text-gray-400 disabled:cursor-not-allowed`
- Error: `border-red-500 text-red-600`

Form field structure:

- Label
- Input/select/textarea
- Helper text or error text

Flag problems:

- Bare inputs with no label or focus state.
- Error messages appear in inconsistent positions.
- Helper text colors vary randomly.

### 13. Clickable Feedback and Accessibility

Every clickable element needs:

- Hover
- Active
- Focus-visible
- Disabled, when applicable
- Selected, when applicable

Recommended focus baseline:

- `focus-visible:outline-none`
- `focus-visible:ring-2`
- `focus-visible:ring-primary/30`

Flag problems:

- Clickable cards look static.
- Icon-only buttons have no focus-visible style.
- Disabled state is only functionally disabled but visually identical.

### 14. Click Target Size

Recommended baseline:

- Mobile minimum click target: 44px
- Desktop icon button: 32px to 40px
- Normal button height: 36px to 44px

Prefer:

`<button className="flex h-9 w-9 items-center justify-center rounded-lg hover:bg-gray-100">...</button>`

Avoid binding click directly to a small icon.

### 15. Alignment

Check:

- Page title alignment
- Form label alignment
- Button group alignment
- Icon + text vertical centering
- List row alignment
- Table column alignment
- Card header/content alignment

Recommended baseline:

- Icon + text: `flex items-center gap-2`
- Button group: `flex justify-end gap-2`
- Form: `space-y-4`
- List: `divide-y` or `gap-3`

### 16. Lists and Tables

Recommended baseline:

- Compact list item: `h-10` or `py-2`
- Normal list item: `h-12` or `py-3`
- Complex list item: `py-4`
- Table row: `h-12`
- Table header: `h-10 text-xs font-medium text-gray-500`

Flag problems:

- Row heights are inconsistent.
- Operation buttons are misaligned.
- Long text breaks the row layout.

### 17. Text Overflow

Always handle long:

- Project names
- Usernames
- Emails
- URLs
- File names
- Table cell content

Use:

- `truncate`
- `line-clamp-2`
- `break-words`
- `max-w-*`
- `min-w-0` inside flex containers

Flag problems:

- Long content breaks cards/tables.
- Horizontal overflow appears unintentionally.

### 18. Empty State

Empty states should include:

- Icon
- Title
- Explanation
- Optional action button

Recommended layout:

- Container: `flex flex-col items-center justify-center py-16`
- Icon: `text-gray-300`
- Title: `text-sm font-medium text-gray-900`
- Description: `text-sm text-gray-500`
- Button: `mt-4`

### 19. Loading State

Required loading states:

- Button loading
- Page loading
- List/table skeleton
- Local partial loading

Button loading behavior:

- Show “保存中...” / “提交中...” / “上传中...”
- Disable the button
- Prevent repeated submits

### 20. Error State

API or page errors need:

- Toast or inline error
- Clear message
- Retry entry, when useful

Recommended copy:

- “加载失败，请稍后重试”
- “保存失败，请重试”
- “网络异常，请稍后再试”
- “暂无操作权限”

### 21. Toast Consistency

Use one consistent feedback pattern for success, error, warning, and info.

Recommended copy:

- Success: “保存成功”
- Failure: “保存失败，请重试”
- Delete: “删除成功”
- Network: “网络异常，请稍后再试”
- Permission: “暂无操作权限”

Avoid mixing `alert`, toast, console-only errors, and inline red text randomly.

### 22. Dangerous Operations

Dangerous operations must use confirmation:

- Delete
- Clear
- Reset
- Revoke
- Unpublish

Recommended confirmation structure:

- Title: “确认删除该项目？”
- Description: “删除后无法恢复，请谨慎操作。”
- Buttons: “取消” and “删除”

Danger style:

- Button: `bg-red-600 text-white hover:bg-red-700`
- Text/menu item: `text-red-600`

### 23. Dropdown/Menu

Recommended structure:

- Normal action
- Normal action
- Divider
- Danger action

Example:

- 重命名
- 下载
- Divider
- 删除

Recommended baseline:

- Container: `rounded-lg border border-gray-200 bg-white shadow-lg`
- Item: `h-9 px-3 text-sm hover:bg-gray-100`
- Danger item: `text-red-600 hover:bg-red-50`

### 24. Modal

Every modal should have:

- Title
- Description
- Main content
- Footer actions

Recommended baseline:

- Width: `max-w-md` or `max-w-lg`
- Radius: `rounded-2xl`
- Padding: `p-6`
- Footer: `flex justify-end gap-2`

Check:

- Close button exists.
- ESC behavior is considered.
- Overlay behavior is considered.
- Mobile height does not exceed viewport.

### 25. Card

Recommended structure:

- Card Header
- Card Content
- Card Footer, when needed

Recommended baseline:

- `bg-white`
- `border border-gray-200`
- `rounded-xl`
- `p-4` or `p-6`
- Optional `shadow-sm`

Flag problems:

- Similar cards have different borders, padding, radius, or shadows.

### 26. Navigation and Sidebar

Navigation items need:

- Default
- Hover
- Active
- Disabled

Recommended baseline:

- Default: `text-gray-600`
- Hover: `bg-gray-100 text-gray-900`
- Active: `bg-primary/10 text-primary`
- Disabled: `text-gray-300 cursor-not-allowed`

Sidebar item baseline:

- `h-10 px-3 rounded-lg flex items-center gap-2`

### 27. Tabs

Tabs need clear selected state.

Recommended baseline:

- Default: `text-gray-500`
- Hover: `text-gray-700`
- Active: `text-primary border-primary`

Check:

- Tab spacing is consistent.
- Selected state is obvious.
- Mobile tabs can scroll horizontally if needed.

### 28. Badge/Tag

Recommended semantic colors:

- Normal: `bg-gray-100 text-gray-700`
- Success: `bg-green-50 text-green-700`
- Warning: `bg-yellow-50 text-yellow-700`
- Error: `bg-red-50 text-red-700`
- Info: `bg-blue-50 text-blue-700`

Check:

- Padding is consistent.
- Radius is consistent.
- Color semantics are not reversed or random.

### 29. Copywriting Consistency

Use consistent action copy:

- 新增
- 编辑
- 删除
- 保存
- 取消
- 确认
- 重试
- 下载
- 上传
- 重命名

Avoid mixing:

- OK
- 确定
- 确认
- 提交
- 确认提交

Use consistent loading copy:

- 加载中...
- 保存中...
- 提交中...
- 上传中...

### 30. Page Max Width

Recommended baseline:

- Form page: `max-w-2xl`
- Detail page: `max-w-3xl`
- Document page: `max-w-4xl`
- Admin list page: `max-w-7xl`
- Full-width console page: `w-full`

Common layout:

`<main className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">...</main>`

### 31. Responsive Checks

Always check these widths:

- 375px mobile
- 768px tablet
- 1440px desktop

Check:

- No unintended horizontal overflow.
- Buttons do not squeeze together.
- Tables can scroll horizontally if needed.
- Modals do not exceed screen width/height.
- Sidebar adapts.
- Card grid wraps.
- Text remains readable.

### 32. Mobile Experience

Check:

- Click target size
- Button height
- Bottom action area
- Keyboard covering inputs
- Modal height
- Menu ease of tapping
- Horizontal overflow

Recommended mobile baseline:

- Primary button height: 44px
- Icon button hit area: 40px to 44px
- Page horizontal padding: 16px
- Bottom actions: `sticky bottom-0` when useful

### 33. Dark Mode

If dark mode is supported, fully check:

- Page background
- Card background
- Text
- Border
- Input
- Button
- Hover
- Active
- Modal
- Dropdown
- Toast

If dark mode is not supported, avoid partial random `dark:` classes.

### 34. Image and Avatar Fallback

Required fallbacks:

- Empty avatar: initials or placeholder
- Image failed: placeholder image/area
- Empty cover: default background

Check avatar:

- Size
- Radius
- Fallback color
- Fallback text size

### 35. Skeleton

Recommended baseline:

- Title skeleton: `h-6 w-48`
- Text skeleton: `h-4 w-full`
- Card skeleton: `rounded-xl border p-4`
- Avatar skeleton: `h-10 w-10 rounded-full`

Avoid showing blank pages, `undefined`, or layout jumps during loading.

### 36. Never Show Invalid Data

The UI must never show:

- `undefined`
- `null`
- `NaN`
- `Invalid Date`
- `[object Object]`

Use fallbacks:

- `user.name || "未命名用户"`
- `project.description || "暂无描述"`
- `price ?? "-"`

### 37. Date and Number Formatting

Use consistent formats:

- Date: `2026-06-26`
- DateTime: `2026-06-26 14:30`
- Relative time: `3 分钟前`
- Money: `¥1,299.00`
- Count: `1,234`
- Percent: `23.5%`

Do not mix many date formats in one app.

### 38. Permission State

When users lack permission:

- Disable or hide unavailable actions.
- Explain why when useful.
- Use tooltip or inline message.

Recommended copy:

- “暂无权限操作该项目”
- “请联系管理员开通权限”

### 39. Table Action Column

Recommended action pattern:

- 查看
- 编辑
- 更多

Put dangerous operations under “更多” when appropriate:

- 重命名
- 下载
- Divider
- 删除

### 40. Complete Page States

Every page should consider:

- Initial loading
- Normal with data
- Empty data
- Search no result
- API error
- No permission
- Submitting
- Submit success
- Submit failure

### 41. Search and Filter

Check:

- Search input style
- Search loading state
- Clear button
- Reset filter button
- No result empty state

Recommended copy:

- “未找到相关结果”
- “请尝试调整关键词或筛选条件”
- “重置筛选”

### 42. Page Header

Recommended structure:

- Left: page title + description
- Right: primary action button

Example:

- 项目管理
- 管理你的所有项目和配置
- 创建项目

Mobile can stack vertically.

### 43. Content Density

Make density consistent across the app.

Check:

- Card padding
- List item height
- Form spacing
- Module spacing
- Table row height

Avoid one page being very loose and another very cramped.

### 44. z-index

Recommended baseline:

- Header: `z-40`
- Dropdown: `z-50`
- Modal overlay/content: `z-50`
- Toast: `z-[60]`
- Tooltip: `z-[70]`

Check:

- Dropdown not hidden behind modal.
- Toast not hidden behind header.
- Tooltip not clipped by parent overflow.

### 45. Overflow

Check:

- Horizontal scrollbar
- Long text breaking layout
- Table overflow
- Modal overflow
- Dropdown clipped by parent container

Useful classes:

- `overflow-hidden`
- `overflow-auto`
- `overflow-x-auto`
- `min-w-0`
- `truncate`

Inside flex layouts, long text usually needs `min-w-0` on the container.

### 46. Component Reuse

Similar UI should be extracted and reused.

Recommended reusable components:

- Button
- IconButton
- Input
- Textarea
- Select
- Checkbox
- Radio
- Switch
- Modal
- Dropdown
- Card
- Badge
- Toast
- Empty
- Skeleton
- PageHeader

Flag problems:

- Every page writes its own button styles.
- Every modal has different layout.
- Form fields are manually styled everywhere.

### 47. Tailwind Token Discipline

Prefer semantic tokens:

- `text-primary`
- `bg-primary`
- `border-border`
- `text-muted-foreground`
- `bg-background`
- `bg-card`

Avoid random one-off classes:

- `text-blue-500`
- `text-indigo-500`
- `text-sky-600`
- `bg-gray-50`
- `bg-slate-50`
- `border-zinc-200`

If the project does not have tokens, recommend creating them.

## Output Format

When auditing a UI, respond in this structure:

### UI 质量结论

Give a short assessment:

- `不草台，只有少量一致性问题`
- `中等混乱，主要问题是 spacing / typography / states`
- `比较草台，需要先统一 token 和组件`

### 主要问题

List the most important problems by priority. For each issue include:

- Problem
- Why it hurts UI quality
- Concrete fix

Example:

1. 字体层级不统一
   - 问题: 页面标题和模块标题都在使用不同的 text size。
   - 影响: 用户无法快速判断信息层级。
   - 建议: 固定页面标题为 `text-2xl font-semibold`，模块标题为 `text-lg font-medium`。

### 修复优先级

Group fixes into:

- P0: Must fix now, obvious visual breakage or usability issue.
- P1: Should fix, consistency and polish issue.
- P2: Nice to improve, refinement issue.

### 可直接给 Agent 的修复 Prompt

When useful, provide a copyable prompt for a coding agent. The prompt should be concrete and project-aware.

Example:

```txt
请检查并修复当前页面 UI 一致性问题：
1. 统一同级标题、正文、辅助文字的字号、字重、颜色。
2. 统一 Button 的高度、圆角、hover、disabled、loading 状态。
3. 所有 flex 横向/纵向排列都必须显式设置 gap。
4. 所有 hover 颜色变化都加 transition-colors duration-200。
5. 统一图标颜色为普通 text-gray-500、hover text-gray-700、active text-primary、danger text-red-500。
6. 检查 375px、768px、1440px 响应式，不允许横向溢出。
不要改业务逻辑，只修复 UI 样式、组件复用和页面状态。
```

## Final Review Gate

Before finishing, ensure the response includes at least:

- The top 3 to 8 UI issues.
- Concrete class-level or component-level fixes.
- Priority order.
- A short copyable agent prompt if the user is using vibe coding / coding agent.

Do not over-focus on subjective aesthetics. Prioritize consistency, spacing, alignment, feedback, and complete states.
