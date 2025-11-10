CfgMenu

【输入框】二值化阈值

【选项卡】去背 / 外轮廓 / 全部轮廓  （影响可导出格式）

去背:

【checkbox】保留内部

【输入框】出血

外轮廓:

【输入框】平滑

【输入框】偏移

【输入框】简化

全部轮廓：

【输入框】孔洞填充

【输入框】平滑

【输入框】偏移

【输入框】简化

预览

1 not allowed to load local resources  - convertFileSrc 修改tauri.config.json assetProtocol

2 后端静态资源的动态路径设置

tauri.config.json - bundle.resources 

BaseDirectory::Resource 指向的是应用的“资源目录”（由打包配置决定）；

- 在 dev 模式下，Tauri 会自动将资源复制到
src-tauri/target/debug/assets/；

- 在 release 模式（打包后）它会变为应用安装目录下的 resources

3 invoke传参过程

参数对象会经过一次 JS → Rust 的 Serde 映射, 默认会把 JS 的 key（驼峰命名）转换成 Rust 端参数（蛇形命名）

4 mutex锁在异步调用时要注意 锁的位置

异步await调用前锁的话，可能会将数据锁死在异步任务里，所以会报错; 最简单方法就是clone或者改变锁的位置（放在tokio::task::spawn_blocking回调里或者await后面，如果逻辑允许）

多线程实际也会遇到同样问题，只不过表现为所有权的移交，所以也要clone