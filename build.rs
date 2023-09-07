use std::process::Command;

fn main() {
    let git_commit = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("HEAD")
        .output()
        .unwrap();

    let mut toolchain = None;

    let content = String::from_utf8(
        Command::new("rustup")
            .arg("toolchain")
            .arg("list")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let content = content.split('\n');

    for s in content {
        let vec = s.split(' ').collect::<Vec<&str>>();
        if vec.len() > 1 {
            let rhs = Some(vec[0]);
            for tag in vec {
                match tag {
                    "(override)" => {
                        toolchain = rhs;
                    }
                    "(default)" if toolchain.is_none() => {
                        toolchain = rhs;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
    }

    let table = include_str!("Cargo.toml").parse::<toml::Table>().unwrap();
    let package = table.get("package").unwrap();

    let edition = package["edition"].as_str().unwrap();
    let version = package["version"].as_str().unwrap();

    println!(
        "cargo:rustc-env=GIT_HASH={}",
        String::from_utf8(git_commit.stdout).unwrap().as_str()
    );
    println!("cargo:rustc-env=RUST_EDITION={}", edition);
    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rustc-env=BUILD_TOOLCHAIN={}", toolchain.unwrap());
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now());

    println!("cargo:rerun-if-changed=build.rs");
}
