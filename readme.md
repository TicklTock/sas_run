# sas_run

用于windows下读取注册表中的SAS路径来执行指定的SAS程序文件

# 要求

需要安装SAS Base用于运行SAS程序

# 示例

```rust
let mut sas_dm = Sas::new(Encoding::UTF8, r"sample.sas", None);
sas_dm.run()?;
```