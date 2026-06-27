# HTML 探索报告

用于把探索结果整理成一个可离线打开、适合嵌入内网 Wiki 的 HTML 报告。报告必须适合内网环境，不依赖任何 CDN、在线图片、在线字体、PlantUML server、Mermaid CDN 或其他外部资源。

强约束：

1. 不允许输出 `<script>` 标签。
2. 不允许使用任何 JavaScript。
3. 不允许引用任何外部 CSS、字体、图片、图标或渲染服务。
4. 页面结构尽量使用原生 `div`、`span`、`a`、`style` 等基础标签组织。
5. 所有动效只能使用 CSS transition、animation、keyframes、hover、focus、target 等能力实现。

输出要求：

1. 只输出一个完整 HTML 片段或完整 HTML 文档。
2. CSS 必须内联在 `<style>` 中。
3. 内容结构要包括：项目定位、模块地图、核心链路、关键文件、风险点、下一步建议。
4. 每个模块/文件/风险点使用清晰的卡片或 section，方便浏览器 Ctrl+F 和 Wiki 自带搜索定位。
5. 如果需要展示图，不要调用在线渲染引擎；可以用纯 HTML/CSS/SVG 或 ASCII 图。
6. 如果无法确认某个结论，必须标注“推测”或“不确定”。

Wiki 适配建议：

- 不做依赖 JavaScript 的搜索框。
- 搜索能力交给浏览器 Ctrl+F 或 Wiki 自带全文检索。
- 可以用目录锚点、卡片标题、data-search 属性提高可搜索性。
- 页面必须在断网环境下可用。
