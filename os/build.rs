static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={TARGET_PATH}");

    // 编译汇编代码
    let asm_files = ["src/trap/trap.S", "src/trap/kpthread_trap.S"];

    for asm_file in asm_files {
        // 告诉 cargo 重新构建的条件
        println!("cargo:rerun-if-changed={asm_file}");
    }
}
