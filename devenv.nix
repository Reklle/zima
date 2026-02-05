{ pkgs, ... }: {
  languages.rust = {
    channel = "nightly";
    components = [ "rustc" "cargo" "rust-src" "clippy" ];
    enable = true;
  };

 packages = with pkgs; [
    fontconfig
    freetype
    pkg-config
    libpng
  ];

  env.CARGO_TARGET_DIR = "./target";
}
