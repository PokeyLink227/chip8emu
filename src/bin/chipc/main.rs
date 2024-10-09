use crate::assembler::assemble;

mod assembler;

fn main() {
    let prog = "v0 v155";
    let _ = assemble(prog);
    //println!("compiled! :D");
}
