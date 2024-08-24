# 实验文档
## 1. 增加的内容
- 完成了 `spawn` 系统调用
- 合并了 lab4 中的 `get_time, mmap, munmap` 系统调用改动
- 合并了 `logging` 模块以完成编程题 2 （显示进程切换过程）

## 2. 完成问答题
### 1. 实践练习 1 的问答作业
- 1. 可以使用 Copy-On-Write 策略
- 2. fork 在现代操作系统中表现出其不正交，不够模块化，线程不安全，信息不安全，效率低下，适用面窄，容易引发内存过载等问题。
    fork 不仅在开发新功能时容易引发问题，还使得系统其他部分的实现更复杂。同时现在已经有一些 fork 的替代方案如 spawn, vfork 等。
- 3. 主程序先运行：`01342`, child 先运行：`03412`. 
- 4. $1+3*1+3*2*1+3*2*2=22$.   
    ```rust
    fn countA(q: Vec<String>) -> i32 {
        let div = q.split(|s| *s == "||".to_string());
        let mut base = 1;
        let mut ans = 0;
        div.for_each(|c| {
            ans += base;
            base *= (c.len() + 1);
        });
        (ans + base) as i32
    }
    ```

### 2. 实践练习 2 的问答作业
- stride 已经在 `ch3-lab` 中实现过，这里只完成问答题。
- 1. 不是，因为整型溢出，`p2.stride = 4`. 
- 2. 因为根据算法，每次调度到的进程 stride 最小，优先级 $\ge 2$ 的情况下，$pass \le BigStride/2$,   
    用归纳法，归纳假设: $STRIDE\_MAX - STRIDE\_MIN\le pass = BigStride/2$. 
    如果在一次调度前满足归纳假设，调度选择的进程 stride $=STRIDE\_MIN$, 
    调度后有 $STRIDE\_MAX^*=STRIDE\_MIN+pass,STRIDE\_MIN^*\ge STRIDE\_MIN$, 
    所以仍然有 $STRIDE\_MAX^* - STRIDE\_MIN^*\le pass$ 成立。  
    归纳假设对于第一个运行的进程显然成立。命题得证。
- 3. 
```rust
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(((self.0 - other.0) as i64).cmp(&0))
    }
}
```

