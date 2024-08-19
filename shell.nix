with import <nixpkgs> { }; 

runCommand "dummy" {
    buildInputs = [ rustup ];
} ""
