fn main() {
    let mut config = prost_build::Config::new();
    config.bytes(&["."]); // 生成的Bytes类型，而不是默认Vec<u8>
    config.type_attribute(".", "#[derive(PartialOrd)]"); // 为所有类型加入 PartialOrd派生宏。
    config
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap();
}