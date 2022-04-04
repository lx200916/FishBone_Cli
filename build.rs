fn main() {
    prost_build::Config::new()
        // .type_attribute(".", "#[derive(Debug)]")
    .out_dir("src/pb")
        .compile_protos(&["message.proto"], &["."])

    .unwrap();
}