{
  typos,
  callPackage,
  shader-slang,
  llvmPackages_20,
}:
let
  inherit (callPackage ./devshell-tools.nix { }) mkFragment;
in
mkFragment {
  name = "extra";
  buildInputs = [
    shader-slang
    typos
  ];

  nativeBuildInputs = [
    llvmPackages_20.libllvm
  ];
}
