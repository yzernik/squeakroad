use std::path::{Path, PathBuf};

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-env-changed=LND_REPO_DIR");
    let lnd_rpc_dir_owned;
    let dir = match std::env::var_os("LND_REPO_DIR") {
        Some(lnd_repo_path) => {
            let mut lnd_rpc_dir = PathBuf::from(lnd_repo_path);
            lnd_rpc_dir.push("lnrpc");
            lnd_rpc_dir_owned = lnd_rpc_dir;
            &*lnd_rpc_dir_owned
        }
        None => Path::new("vendor"),
    };

    let lnd_rpc_proto_file = dir.join("lightning.proto");
    println!("cargo:rerun-if-changed={}", lnd_rpc_proto_file.display());

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(&[&*lnd_rpc_proto_file], &[dir])
}
