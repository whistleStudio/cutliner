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

异步await前调用Mutex::lock的话，可能会导致守卫MutexGuard在await结束后被异步运行时分配到其他线程，违背系统要求【获取锁的线程必须与释放锁的线程相同】，所以会报错; 最简单方法就是clone或者改变锁的位置（放在tokio::task::spawn_blocking回调里或者await后面，如果逻辑允许）

多线程实际也会遇到同样问题，只不过表现为所有权的移交，所以也要clone

关于【获取锁的线程必须与释放锁的线程相同】

-------------------------------------------------------------

如果允许别的线程解锁会怎样？

举个具体例子：

1️⃣ 线程 A 上锁，然后正在访问共享资源；

2️⃣ 线程 B（或某个 async 任务）错误地释放了这个锁；

3️⃣ 此时 A 仍认为自己安全地持有锁，在修改数据；

4️⃣ 但实际上锁已经被别人释放，别的线程 C 又拿到了锁；

5️⃣ A 和 C 同时修改数据 → 数据竞态 + 崩溃 + 难以发现的 bug。

--------------------------------------------------------------

当一个线程在 pthread_mutex_lock() 阻塞时：

- 该线程被放进内核的等待队列；

- 这个等待记录是挂在**该线程控制块（TCB）**上的；

- 唤醒时由解锁线程唤醒等待的具体线程。

如果允许“其他线程解锁”，内核得处理如下复杂问题：

> “谁来唤醒等待者？原本拥有锁的线程？新解锁线程？系统统一唤醒？”

这导致：

- 解锁操作需要跨线程修改等待队列；

- 增加调度竞争；

- 导致唤醒顺序不确定；

- 甚至可能出现“锁释放了，但没人被唤醒”的悬挂状态。

5 svg to dxf 

用python-svgpathtools ezdxf生成exe; 做tauri sidecar

6 img.copy_to_masked(&mut mat, &mask) 通道数和img有关